use crate::diagnostic::StopReason;
use crate::value::FlexValue;

use super::sse::SseEvent;
use super::{BlockAccumulator, StreamEvent, StreamHandler};

/// OpenAI streaming event handler.
pub struct OpenAiStreamHandler;

impl StreamHandler for OpenAiStreamHandler {
    fn process_event(&self, sse: &SseEvent) -> Vec<StreamEvent> {
        process_event(sse, &std::collections::HashMap::new())
    }
}

/// Process an OpenAI SSE event into StreamEvents.
#[allow(private_interfaces)]
///
/// OpenAI format:
/// - Each `data:` line contains a JSON chunk with `choices[0].delta`
/// - Text content arrives as `delta.content`
/// - Tool calls arrive as `delta.tool_calls[i]` with `index`, `id`, `function.name`, `function.arguments`
/// - `finish_reason` signals end
/// - `data: [DONE]` is the final sentinel
pub(crate) fn process_event(
    sse: &SseEvent,
    _accumulators: &std::collections::HashMap<usize, BlockAccumulator>,
) -> Vec<StreamEvent> {
    // OpenAI doesn't use event types — just bare data lines
    let data_str = &sse.data;

    // Handle [DONE] sentinel
    if data_str.trim() == "[DONE]" {
        return vec![StreamEvent::Stop(StopReason::EndTurn)];
    }

    let data = match serde_json::from_str::<serde_json::Value>(data_str) {
        Ok(v) => v,
        Err(e) => {
            return vec![StreamEvent::ParseError {
                event_type: sse.event_type.clone(),
                raw_data: data_str.to_string(),
                error: e.to_string(),
            }];
        }
    };

    let mut events = Vec::new();

    // Extract from choices[0]
    let choice = match data.get("choices").and_then(|c| c.get(0)) {
        Some(c) => c,
        None => {
            // Might be a usage/metadata event (OpenAI sends these at the end)
            if data.get("usage").is_some() {
                events.push(StreamEvent::Metadata(FlexValue::new(data)));
            }
            return events;
        }
    };

    let delta = match choice.get("delta") {
        Some(d) => d,
        None => return events,
    };

    // Text content
    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
        if !content.is_empty() {
            events.push(StreamEvent::TextDelta(content.to_string()));
        }
    }

    // Tool calls
    if let Some(tool_calls) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
        for tc in tool_calls {
            let index = tc["index"].as_u64().unwrap_or(0) as usize;

            // If this chunk has an id, it's a new tool call starting
            if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                let name = tc
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());

                events.push(StreamEvent::BlockStart {
                    index,
                    id: id.to_string(),
                    block_type: "function".to_string(),
                    name,
                });
            }

            // Argument fragments
            if let Some(args) = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
            {
                if !args.is_empty() {
                    events.push(StreamEvent::BlockDelta {
                        index,
                        fragment: args.to_string(),
                    });
                }
            }
        }
    }

    // Finish reason
    if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
        events.push(StreamEvent::Stop(parse_stop_reason(reason)));
    }

    events
}

fn parse_stop_reason(s: &str) -> StopReason {
    match s {
        "stop" => StopReason::EndTurn,
        "tool_calls" => StopReason::ToolUse,
        "length" => StopReason::MaxTokens,
        "stop_sequence" => StopReason::StopSequence,
        other => StopReason::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_event(data: &str) -> SseEvent {
        SseEvent {
            event_type: None,
            data: data.to_string(),
        }
    }

    #[test]
    fn done_sentinel() {
        let events = process_event(&make_event("[DONE]"), &HashMap::new());
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], StreamEvent::Stop(StopReason::EndTurn)));
    }

    #[test]
    fn text_content_delta() {
        let events = process_event(
            &make_event(r#"{"choices":[{"index":0,"delta":{"content":"Hello"}}]}"#),
            &HashMap::new(),
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::TextDelta(text) => assert_eq!(text, "Hello"),
            _ => panic!("expected TextDelta"),
        }
    }

    #[test]
    fn tool_call_start() {
        let events = process_event(
            &make_event(
                r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"search","arguments":""}}]}}]}"#,
            ),
            &HashMap::new(),
        );
        // Should have BlockStart (and possibly empty BlockDelta)
        assert!(events
            .iter()
            .any(|e| matches!(e, StreamEvent::BlockStart { .. })));
        match &events[0] {
            StreamEvent::BlockStart {
                index,
                id,
                name,
                block_type,
            } => {
                assert_eq!(*index, 0);
                assert_eq!(id, "call_1");
                assert_eq!(name, &Some("search".into()));
                assert_eq!(block_type, "function");
            }
            _ => panic!("expected BlockStart"),
        }
    }

    #[test]
    fn tool_call_argument_delta() {
        let events = process_event(
            &make_event(
                r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"city\":"}}]}}]}"#,
            ),
            &HashMap::new(),
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::BlockDelta { index, fragment } => {
                assert_eq!(*index, 0);
                assert_eq!(fragment, "{\"city\":");
            }
            _ => panic!("expected BlockDelta"),
        }
    }

    #[test]
    fn finish_reason_stop() {
        let events = process_event(
            &make_event(r#"{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#),
            &HashMap::new(),
        );
        assert!(events
            .iter()
            .any(|e| matches!(e, StreamEvent::Stop(StopReason::EndTurn))));
    }

    #[test]
    fn finish_reason_tool_calls() {
        let events = process_event(
            &make_event(r#"{"choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#),
            &HashMap::new(),
        );
        assert!(events
            .iter()
            .any(|e| matches!(e, StreamEvent::Stop(StopReason::ToolUse))));
    }
}
