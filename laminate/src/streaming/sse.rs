/// A parsed SSE event frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    /// The event type (from `event:` line). None if not specified.
    pub event_type: Option<String>,
    /// The data payload (from `data:` lines, joined with newlines for multi-line).
    pub data: String,
}

/// Incremental SSE parser.
///
/// Feeds raw bytes/text and emits complete `SseEvent` frames.
/// Handles:
/// - `data: {...}\n\n` — standard events
/// - `event: type\ndata: {...}\n\n` — typed events (Anthropic style)
/// - `data: [DONE]\n\n` — OpenAI end-of-stream sentinel
/// - Multi-line data fields (rare but spec-legal)
/// - `: heartbeat\n` — comment lines (ignored)
#[derive(Debug, Default)]
pub struct SseParser {
    /// Buffer for incomplete lines.
    line_buffer: String,
    /// Current event type being accumulated.
    current_event_type: Option<String>,
    /// Current data lines being accumulated.
    current_data: Vec<String>,
}

impl SseParser {
    /// Create a new SSE parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a chunk of text into the parser. Returns zero or more complete events.
    pub fn feed(&mut self, chunk: &str) -> Vec<SseEvent> {
        let mut events = Vec::new();

        for ch in chunk.chars() {
            if ch == '\n' {
                let line = std::mem::take(&mut self.line_buffer);
                self.process_line(&line, &mut events);
            } else if ch != '\r' {
                self.line_buffer.push(ch);
            }
        }

        events
    }

    /// Feed raw bytes (UTF-8) into the parser.
    ///
    /// Invalid UTF-8 sequences are replaced with U+FFFD (replacement character)
    /// per the SSE specification's UTF-8 decode requirement. This prevents a single
    /// corrupted byte from silently dropping an entire chunk of events.
    pub fn feed_bytes(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        let text = String::from_utf8_lossy(bytes);
        self.feed(&text)
    }

    /// Signal end of stream. Flushes any remaining buffered event.
    pub fn finish(mut self) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // Process any remaining data in line buffer
        if !self.line_buffer.is_empty() {
            let line = std::mem::take(&mut self.line_buffer);
            self.process_line(&line, &mut events);
        }

        // Flush any accumulated event
        self.flush_event(&mut events);

        events
    }

    fn process_line(&mut self, line: &str, events: &mut Vec<SseEvent>) {
        if line.is_empty() {
            // Empty line = event boundary
            self.flush_event(events);
        } else if let Some(comment) = line.strip_prefix(':') {
            // Comment line — ignore (heartbeats)
            let _ = comment;
        } else if let Some(value) = line.strip_prefix("event:") {
            self.current_event_type = Some(strip_one_leading_space(value).to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            self.current_data
                .push(strip_one_leading_space(value).to_string());
        } else if let Some(value) = line.strip_prefix("id:") {
            // Event ID — ignore for now (could track for reconnection)
            let _ = value;
        } else if let Some(value) = line.strip_prefix("retry:") {
            // Retry interval — ignore for now
            let _ = value;
        }
        // Unknown field names are ignored per SSE spec
    }

    fn flush_event(&mut self, events: &mut Vec<SseEvent>) {
        if self.current_data.is_empty() {
            // No data accumulated — reset event type and skip
            self.current_event_type = None;
            return;
        }

        let data = self.current_data.join("\n");
        let event_type = self.current_event_type.take();
        self.current_data.clear();

        events.push(SseEvent { event_type, data });
    }
}

/// Strip at most one leading U+0020 SPACE character, per SSE spec §9.2.6.
/// Only ASCII space (0x20) counts — tabs and other whitespace are preserved.
fn strip_one_leading_space(s: &str) -> &str {
    s.strip_prefix(' ').unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_data_event() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        assert_eq!(events[0].event_type, None);
    }

    #[test]
    fn typed_event() {
        let mut parser = SseParser::new();
        let events = parser.feed("event: message_start\ndata: {\"type\":\"message\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, Some("message_start".into()));
        assert_eq!(events[0].data, "{\"type\":\"message\"}");
    }

    #[test]
    fn multiple_events() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: first\n\ndata: second\n\n");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "first");
        assert_eq!(events[1].data, "second");
    }

    #[test]
    fn chunked_delivery() {
        let mut parser = SseParser::new();
        let e1 = parser.feed("data: hel");
        assert!(e1.is_empty());
        let e2 = parser.feed("lo\n\n");
        assert_eq!(e2.len(), 1);
        assert_eq!(e2[0].data, "hello");
    }

    #[test]
    fn multiline_data() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: line1\ndata: line2\ndata: line3\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2\nline3");
    }

    #[test]
    fn heartbeat_comment_ignored() {
        let mut parser = SseParser::new();
        let events = parser.feed(": heartbeat\ndata: actual\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual");
    }

    #[test]
    fn done_sentinel() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: [DONE]\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "[DONE]");
    }

    #[test]
    fn carriage_return_handling() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: hello\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn finish_flushes_remaining() {
        let mut parser = SseParser::new();
        let e1 = parser.feed("data: partial");
        assert!(e1.is_empty());
        let e2 = parser.finish();
        assert_eq!(e2.len(), 1);
        assert_eq!(e2[0].data, "partial");
    }

    #[test]
    fn empty_data_between_events_ignored() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: first\n\n\n\ndata: second\n\n");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn json_data() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: {\"id\":\"msg_123\",\"type\":\"message_start\"}\n\n");
        assert_eq!(events.len(), 1);
        let parsed: serde_json::Value = serde_json::from_str(&events[0].data).unwrap();
        assert_eq!(parsed["id"], "msg_123");
    }

    #[test]
    fn anthropic_stream_sequence() {
        let mut parser = SseParser::new();
        let stream = "\
event: message_start\n\
data: {\"type\":\"message_start\"}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

        let events = parser.feed(stream);
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].event_type, Some("message_start".into()));
        assert_eq!(events[1].event_type, Some("content_block_start".into()));
        assert_eq!(events[2].event_type, Some("content_block_delta".into()));
        assert_eq!(events[3].event_type, Some("content_block_stop".into()));
        assert_eq!(events[4].event_type, Some("message_stop".into()));
    }
}
