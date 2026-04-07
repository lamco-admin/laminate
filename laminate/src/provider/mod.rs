//! Provider normalization — map provider-specific response shapes to a common envelope.
//!
//! Downstream code doesn't care which API it's talking to. Anthropic, OpenAI,
//! and Ollama responses all normalize to the same `NormalizedResponse`.

/// Anthropic Claude API response normalization.
pub mod anthropic;
/// Ollama API response normalization.
pub mod ollama;
/// OpenAI ChatGPT API response normalization.
pub mod openai;

use std::collections::HashMap;

use crate::diagnostic::StopReason;
use crate::error::Result;
#[cfg(feature = "streaming")]
use crate::streaming::FlexStream;
use crate::value::FlexValue;

/// Normalized response envelope.
///
/// The common shape that all provider responses map to. Downstream code
/// works with this type regardless of which API produced it.
#[derive(Debug, Clone)]
pub struct NormalizedResponse {
    /// Provider-specific response ID (e.g., "msg_xxx", "chatcmpl-xxx").
    pub id: String,
    /// Model that generated the response.
    pub model: String,
    /// Content blocks in order.
    pub content: Vec<ContentBlock>,
    /// Why the response ended.
    pub stop_reason: StopReason,
    /// Token usage information.
    pub usage: Usage,
    /// The original raw response, always accessible.
    pub raw: FlexValue,
}

/// Normalized content block.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// Plain text content.
    Text {
        /// The text content.
        text: String,
    },
    /// A tool/function call with structured input.
    ToolUse {
        /// Provider-assigned tool call ID.
        id: String,
        /// Tool/function name.
        name: String,
        /// Structured input as a FlexValue (navigable, coercible).
        input: FlexValue,
    },
    /// Forward-compatible: new block types don't break existing code.
    Unknown {
        /// The unrecognized block type string.
        block_type: String,
        /// The block payload.
        data: FlexValue,
    },
}

/// Token usage information.
#[derive(Debug, Clone, Default)]
pub struct Usage {
    /// Number of input/prompt tokens consumed.
    pub input_tokens: u64,
    /// Number of output/completion tokens generated.
    pub output_tokens: u64,
    /// Tokens read from cache (Anthropic prompt caching).
    pub cache_read_tokens: Option<u64>,
    /// Tokens written to cache (Anthropic prompt caching).
    pub cache_creation_tokens: Option<u64>,
    /// Provider-specific extra usage fields.
    pub extra: HashMap<String, serde_json::Value>,
}

/// Implement this trait to add new providers.
pub trait ProviderAdapter: Send + Sync {
    /// Parse a non-streaming API response body.
    fn parse_response(&self, body: &FlexValue) -> Result<NormalizedResponse>;

    /// Emit a NormalizedResponse in this provider's format.
    fn emit_response(&self, response: &NormalizedResponse) -> serde_json::Value;

    /// Create a streaming parser configured for this provider.
    #[cfg(feature = "streaming")]
    fn stream_parser(&self) -> FlexStream;
}

impl ContentBlock {
    /// Returns `true` if this is a text block.
    pub fn is_text(&self) -> bool {
        matches!(self, ContentBlock::Text { .. })
    }

    /// Returns `true` if this is a tool use block.
    pub fn is_tool_use(&self) -> bool {
        matches!(self, ContentBlock::ToolUse { .. })
    }

    /// Extract text content, if this is a text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Extract tool use details, if this is a tool use block.
    pub fn as_tool_use(&self) -> Option<(&str, &str, &FlexValue)> {
        match self {
            ContentBlock::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }
}

impl NormalizedResponse {
    /// Get all text content concatenated.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| b.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get all tool use blocks.
    pub fn tool_uses(&self) -> Vec<&ContentBlock> {
        self.content.iter().filter(|b| b.is_tool_use()).collect()
    }

    /// Returns `true` if the response contains any tool use blocks.
    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|b| b.is_tool_use())
    }
}
