use crate::diagnostic::StopReason;
use crate::value::FlexValue;

use super::sse::SseEvent;
use super::{StreamEvent, StreamHandler};

/// Anthropic streaming event handler.
pub struct AnthropicStreamHandler;

impl StreamHandler for AnthropicStreamHandler {
    fn process_event(&self, sse: &SseEvent) -> Vec<StreamEvent> {
        process_event(sse)
    }
}

/// Process an Anthropic SSE event into StreamEvents.
///
/// Anthropic event types:
/// - `message_start` → envelope with model, usage
/// - `content_block_start` → signals text or tool_use block
/// - `content_block_delta` → text_delta or input_json_delta
/// - `content_block_stop` → block complete
/// - `message_delta` → stop_reason, final usage
/// - `message_stop` → end
/// - `ping` → heartbeat (ignored)
pub(crate) fn process_event(sse: &SseEvent) -> Vec<StreamEvent> {
    let event_type = match &sse.event_type {
        Some(t) => t.as_str(),
        None => return vec![],
    };

    let data = match serde_json::from_str::<serde_json::Value>(&sse.data) {
        Ok(v) => v,
        Err(e) => {
            return vec![StreamEvent::ParseError {
                event_type: sse.event_type.clone(),
                raw_data: sse.data.clone(),
                error: e.to_string(),
            }];
        }
    };

    match event_type {
        "message_start" => {
            vec![StreamEvent::Metadata(FlexValue::new(data))]
        }

        "content_block_start" => {
            let index = data["index"].as_u64().unwrap_or(0) as usize;
            let block = &data["content_block"];
            let block_type = block["type"].as_str().unwrap_or("text").to_string();
            let id = block["id"].as_str().unwrap_or("").to_string();
            let name = block["name"].as_str().map(|s| s.to_string());

            vec![StreamEvent::BlockStart {
                index,
                id,
                block_type,
                name,
            }]
        }

        "content_block_delta" => {
            let index = data["index"].as_u64().unwrap_or(0) as usize;
            let delta = &data["delta"];
            let delta_type = delta["type"].as_str().unwrap_or("");

            match delta_type {
                "text_delta" => {
                    let text = delta["text"].as_str().unwrap_or("").to_string();
                    vec![StreamEvent::TextDelta(text)]
                }
                "input_json_delta" => {
                    let fragment = delta["partial_json"].as_str().unwrap_or("").to_string();
                    vec![StreamEvent::BlockDelta { index, fragment }]
                }
                _ => vec![],
            }
        }

        "content_block_stop" => {
            let index = data["index"].as_u64().unwrap_or(0) as usize;
            // Block complete — the FlexStream will assemble the full content
            // from accumulated fragments. For text blocks, content was already
            // streamed via TextDelta. For tool_use blocks, fragments were
            // accumulated via BlockDelta.
            //
            // We emit a BlockComplete with the assembled content.
            // Note: the actual assembly happens in FlexStream, not here.
            // For now, emit with empty content — FlexStream will enrich.
            vec![StreamEvent::BlockComplete {
                index,
                id: String::new(),
                block_type: String::new(),
                name: None,
                content: FlexValue::new(serde_json::Value::Null),
            }]
        }

        "message_delta" => {
            let stop_reason = data["delta"]["stop_reason"].as_str().map(parse_stop_reason);

            let mut events = Vec::new();
            if let Some(usage) = data.get("usage") {
                events.push(StreamEvent::Metadata(FlexValue::new(usage.clone())));
            }
            if let Some(reason) = stop_reason {
                events.push(StreamEvent::Stop(reason));
            }
            events
        }

        "message_stop" => {
            // Final signal — if we haven't already emitted Stop from message_delta
            vec![]
        }

        "ping" => vec![],

        _ => {
            vec![StreamEvent::Unknown {
                event_type: event_type.to_string(),
                data: FlexValue::new(data),
            }]
        }
    }
}

fn parse_stop_reason(s: &str) -> StopReason {
    match s {
        "end_turn" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        "stop_sequence" => StopReason::StopSequence,
        other => StopReason::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(event_type: &str, data: &str) -> SseEvent {
        SseEvent {
            event_type: Some(event_type.to_string()),
            data: data.to_string(),
        }
    }

    #[test]
    fn message_start_produces_metadata() {
        let events = process_event(&make_event(
            "message_start",
            r#"{"type":"message_start","message":{"model":"claude-4"}}"#,
        ));
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], StreamEvent::Metadata(_)));
    }

    #[test]
    fn content_block_start_text() {
        let events = process_event(&make_event(
            "content_block_start",
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        ));
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::BlockStart {
                index, block_type, ..
            } => {
                assert_eq!(*index, 0);
                assert_eq!(block_type, "text");
            }
            _ => panic!("expected BlockStart"),
        }
    }

    #[test]
    fn content_block_start_tool_use() {
        let events = process_event(&make_event(
            "content_block_start",
            r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"tu_1","name":"search"}}"#,
        ));
        match &events[0] {
            StreamEvent::BlockStart {
                index,
                id,
                block_type,
                name,
            } => {
                assert_eq!(*index, 1);
                assert_eq!(id, "tu_1");
                assert_eq!(block_type, "tool_use");
                assert_eq!(name, &Some("search".into()));
            }
            _ => panic!("expected BlockStart"),
        }
    }

    #[test]
    fn text_delta() {
        let events = process_event(&make_event(
            "content_block_delta",
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#,
        ));
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::TextDelta(text) => assert_eq!(text, "Hello"),
            _ => panic!("expected TextDelta"),
        }
    }

    #[test]
    fn input_json_delta() {
        let events = process_event(&make_event(
            "content_block_delta",
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"city\":"}}"#,
        ));
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::BlockDelta { index, fragment } => {
                assert_eq!(*index, 1);
                assert_eq!(fragment, "{\"city\":");
            }
            _ => panic!("expected BlockDelta"),
        }
    }

    #[test]
    fn message_delta_with_stop() {
        let events = process_event(&make_event(
            "message_delta",
            r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":30}}"#,
        ));
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], StreamEvent::Metadata(_)));
        assert!(matches!(&events[1], StreamEvent::Stop(StopReason::EndTurn)));
    }

    #[test]
    fn ping_ignored() {
        let events = process_event(&make_event("ping", r#"{"type":"ping"}"#));
        assert!(events.is_empty());
    }
}
