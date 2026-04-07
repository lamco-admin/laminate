//! Streaming SSE parser with pluggable event handlers.
//!
//! The SSE parser is provider-agnostic. Provider-specific interpretation
//! is handled through the [`StreamHandler`] trait.

/// Server-Sent Events (SSE) parser — protocol-level byte stream to event frames.
pub mod sse;

/// Anthropic streaming event handler.
pub mod anthropic;
/// OpenAI streaming event handler.
pub mod openai;

use crate::diagnostic::StopReason;
use crate::value::FlexValue;
use sse::{SseEvent, SseParser};

/// Which provider's SSE format to parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Anthropic's streaming format (content_block_start, content_block_delta, etc.).
    Anthropic,
    /// OpenAI's streaming format (chat.completion.chunk).
    OpenAI,
}

/// Configuration for a streaming parser.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Which provider's SSE format to parse.
    pub provider: Provider,
    /// Maximum buffer size for incomplete SSE events (default: 1 MB).
    pub max_buffer_bytes: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Anthropic,
            max_buffer_bytes: 1_048_576, // 1MB
        }
    }
}

/// Events emitted by the stream parser.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Incremental text content.
    TextDelta(String),

    /// A structured block has started.
    BlockStart {
        /// Block index within the response.
        index: usize,
        /// Provider-assigned block ID.
        id: String,
        /// Block type (e.g., "tool_use", "text").
        block_type: String,
        /// Tool name (for tool_use blocks).
        name: Option<String>,
    },

    /// A fragment of a block's content (e.g., streaming tool arguments).
    BlockDelta {
        /// Block index this fragment belongs to.
        index: usize,
        /// The content fragment.
        fragment: String,
    },

    /// A block is fully accumulated and parseable.
    BlockComplete {
        /// Block index within the response.
        index: usize,
        /// Provider-assigned block ID.
        id: String,
        /// Block type (e.g., "tool_use", "text").
        block_type: String,
        /// Tool name (for tool_use blocks).
        name: Option<String>,
        /// Assembled content as a FlexValue.
        content: FlexValue,
    },

    /// Usage / metadata information.
    Metadata(FlexValue),

    /// Stop/end signal.
    Stop(StopReason),

    /// Unrecognized event — forward-compatible.
    Unknown {
        /// The SSE event type string.
        event_type: String,
        /// The event payload.
        data: FlexValue,
    },

    /// An SSE event that failed to parse. Captured for diagnostics
    /// instead of being silently dropped.
    ParseError {
        /// The SSE event type, if available.
        event_type: Option<String>,
        /// The raw unparseable data.
        raw_data: String,
        /// What went wrong.
        error: String,
    },
}

/// Trait for converting SSE events into stream events.
///
/// Implement this to add support for new streaming providers.
/// The streaming module provides built-in handlers for Anthropic and OpenAI.
pub trait StreamHandler: Send {
    /// Process a single SSE event into zero or more stream events.
    fn process_event(&self, sse: &SseEvent) -> Vec<StreamEvent>;
}

/// Running snapshot of the message being streamed.
///
/// Evolves with each event — text accumulates, tool calls assemble,
/// usage updates. Access at any point during streaming via
/// [`FlexStream::current_message()`].
#[derive(Debug, Clone)]
pub struct MessageSnapshot {
    /// Accumulated text content so far.
    pub text: String,
    /// Completed tool calls (fully assembled).
    pub tool_calls: Vec<(String, String, FlexValue)>, // (id, name, input)
    /// Stop reason (set when Stop event arrives).
    pub stop_reason: Option<StopReason>,
    /// Whether the stream has ended.
    pub done: bool,
}

impl MessageSnapshot {
    fn new() -> Self {
        Self {
            text: String::new(),
            tool_calls: Vec::new(),
            stop_reason: None,
            done: false,
        }
    }

    fn apply_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::TextDelta(text) => {
                self.text.push_str(text);
            }
            StreamEvent::BlockComplete {
                id,
                name,
                content,
                block_type,
                ..
            } => {
                if block_type == "tool_use" || block_type == "function" {
                    self.tool_calls.push((
                        id.clone(),
                        name.clone().unwrap_or_default(),
                        content.clone(),
                    ));
                }
            }
            StreamEvent::Stop(reason) => {
                self.stop_reason = Some(reason.clone());
                self.done = true;
            }
            _ => {}
        }
    }
}

/// The main streaming parser. Feeds SSE bytes and emits normalized events.
pub struct FlexStream {
    _config: StreamConfig,
    sse_parser: SseParser,
    handler: Box<dyn StreamHandler>,
    /// Per-block accumulators for tool call arguments (keyed by index).
    block_accumulators: std::collections::HashMap<usize, BlockAccumulator>,
    /// Running message snapshot — evolves with each event.
    snapshot: MessageSnapshot,
}

/// Accumulates fragments for a single content block.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BlockAccumulator {
    id: String,
    block_type: String,
    name: Option<String>,
    content_fragments: Vec<String>,
}

/// Create the appropriate handler for a provider.
fn handler_for_provider(provider: Provider) -> Box<dyn StreamHandler> {
    match provider {
        Provider::Anthropic => Box::new(anthropic::AnthropicStreamHandler),
        Provider::OpenAI => Box::new(openai::OpenAiStreamHandler),
    }
}

impl FlexStream {
    /// Create a new streaming parser with the given configuration.
    pub fn new(config: StreamConfig) -> Self {
        let handler = handler_for_provider(config.provider);
        Self {
            _config: config,
            sse_parser: SseParser::new(),
            handler,
            block_accumulators: std::collections::HashMap::new(),
            snapshot: MessageSnapshot::new(),
        }
    }

    /// Create with a custom stream handler.
    pub fn with_handler(handler: Box<dyn StreamHandler>) -> Self {
        Self {
            _config: StreamConfig::default(),
            sse_parser: SseParser::new(),
            handler,
            block_accumulators: std::collections::HashMap::new(),
            snapshot: MessageSnapshot::new(),
        }
    }

    /// Access the current message snapshot.
    ///
    /// Returns the running state of the message being streamed:
    /// accumulated text, completed tool calls, and stop reason.
    /// The snapshot evolves with each call to `feed()` or `feed_str()`.
    pub fn current_message(&self) -> &MessageSnapshot {
        &self.snapshot
    }

    /// Feed raw bytes into the parser. Returns zero or more events.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<StreamEvent> {
        let sse_events = self.sse_parser.feed_bytes(chunk);
        self.process_sse_events(sse_events)
    }

    /// Feed a text chunk into the parser.
    pub fn feed_str(&mut self, chunk: &str) -> Vec<StreamEvent> {
        let sse_events = self.sse_parser.feed(chunk);
        self.process_sse_events(sse_events)
    }

    /// Signal end-of-stream. Flushes remaining content.
    pub fn finish(self) -> Vec<StreamEvent> {
        let sse_events = self.sse_parser.finish();
        let mut stream_events = Vec::new();
        for sse_event in sse_events {
            stream_events.extend(self.handler.process_event(&sse_event));
        }
        stream_events
    }

    fn process_sse_events(&mut self, sse_events: Vec<SseEvent>) -> Vec<StreamEvent> {
        let mut stream_events = Vec::new();

        for sse_event in sse_events {
            let events = self.handler.process_event(&sse_event);

            for event in events {
                match event {
                    StreamEvent::BlockStart {
                        index,
                        ref id,
                        ref block_type,
                        ref name,
                    } => {
                        self.block_accumulators.insert(
                            index,
                            BlockAccumulator {
                                id: id.clone(),
                                block_type: block_type.clone(),
                                name: name.clone(),
                                content_fragments: Vec::new(),
                            },
                        );
                        stream_events.push(event);
                    }
                    StreamEvent::BlockDelta {
                        index,
                        ref fragment,
                    } => {
                        if let Some(acc) = self.block_accumulators.get_mut(&index) {
                            acc.content_fragments.push(fragment.clone());
                        }
                        stream_events.push(event);
                    }
                    StreamEvent::BlockComplete { index, .. } => {
                        // Assemble accumulated fragments into the BlockComplete
                        if let Some(acc) = self.block_accumulators.remove(&index) {
                            let assembled_json = acc.content_fragments.join("");
                            let content = if assembled_json.is_empty() {
                                FlexValue::new(serde_json::Value::Null)
                            } else {
                                match serde_json::from_str::<serde_json::Value>(&assembled_json) {
                                    Ok(v) => FlexValue::new(v),
                                    // If fragments don't form valid JSON, preserve as string
                                    Err(_) => {
                                        FlexValue::new(serde_json::Value::String(assembled_json))
                                    }
                                }
                            };
                            stream_events.push(StreamEvent::BlockComplete {
                                index,
                                id: acc.id,
                                block_type: acc.block_type,
                                name: acc.name,
                                content,
                            });
                        } else {
                            // No accumulator — pass through as-is
                            stream_events.push(event);
                        }
                    }
                    _ => {
                        stream_events.push(event);
                    }
                }
            }
        }

        // Update running snapshot with all events
        for event in &stream_events {
            self.snapshot.apply_event(event);
        }

        stream_events
    }
}
