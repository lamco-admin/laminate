use crate::diagnostic::StopReason;
use crate::error::Result;
use crate::value::FlexValue;

#[cfg(feature = "streaming")]
use crate::streaming::{FlexStream, Provider, StreamConfig};

use super::{ContentBlock, NormalizedResponse, ProviderAdapter, Usage};

/// OpenAI API response adapter.
///
/// Parses responses from the OpenAI Chat Completions API into `NormalizedResponse`.
/// Handles the critical stringified `arguments` field in tool calls — FlexValue's
/// coercion system parses the embedded JSON automatically.
///
/// Expected input shape:
/// ```json
/// {
///   "id": "chatcmpl-xxx",
///   "model": "gpt-4o",
///   "choices": [{
///     "message": {
///       "content": "Hello",
///       "tool_calls": [{
///         "id": "tc_1",
///         "type": "function",
///         "function": {"name": "search", "arguments": "{\"q\":\"rust\"}"}
///       }]
///     },
///     "finish_reason": "stop"
///   }],
///   "usage": {"prompt_tokens": 50, "completion_tokens": 30}
/// }
/// ```
pub struct OpenAiAdapter;

impl ProviderAdapter for OpenAiAdapter {
    fn parse_response(&self, body: &FlexValue) -> Result<NormalizedResponse> {
        parse_openai_response(body)
    }

    fn emit_response(&self, response: &NormalizedResponse) -> serde_json::Value {
        emit_openai_response(response)
    }

    #[cfg(feature = "streaming")]
    fn stream_parser(&self) -> FlexStream {
        FlexStream::new(StreamConfig {
            provider: Provider::OpenAI,
            ..Default::default()
        })
    }
}

/// Emit a NormalizedResponse as OpenAI API format.
pub fn emit_openai_response(response: &NormalizedResponse) -> serde_json::Value {
    use serde_json::json;

    let mut message = serde_json::Map::new();
    message.insert("role".into(), json!("assistant"));

    // Text content
    let text = response.text();
    if !text.is_empty() {
        message.insert("content".into(), json!(text));
    } else {
        message.insert("content".into(), json!(null));
    }

    // Tool calls
    let tool_calls: Vec<serde_json::Value> = response
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::ToolUse { id, name, input } = block {
                // OpenAI expects arguments as a stringified JSON string.
                // If input is already a string (e.g., partial streaming args that
                // couldn't be parsed), pass through as-is to avoid double-stringifying.
                let arguments = if input.is_string() {
                    input.raw().as_str().unwrap_or("{}").to_string()
                } else {
                    serde_json::to_string(input.raw()).unwrap_or_default()
                };
                Some(json!({
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments
                    }
                }))
            } else {
                None
            }
        })
        .collect();

    if !tool_calls.is_empty() {
        message.insert("tool_calls".into(), json!(tool_calls));
    }

    let finish_reason = match &response.stop_reason {
        StopReason::EndTurn => "stop",
        StopReason::ToolUse => "tool_calls",
        StopReason::MaxTokens => "length",
        StopReason::StopSequence => "stop_sequence",
        StopReason::Unknown(s) => s.as_str(),
    };

    json!({
        "id": response.id,
        "object": "chat.completion",
        "model": response.model,
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish_reason
        }],
        "usage": {
            "prompt_tokens": response.usage.input_tokens,
            "completion_tokens": response.usage.output_tokens,
            "total_tokens": response.usage.input_tokens + response.usage.output_tokens
        }
    })
}

/// Parse an OpenAI Chat Completions API response body into a `NormalizedResponse`.
pub fn parse_openai_response(body: &FlexValue) -> Result<NormalizedResponse> {
    let id: String = body.extract("id")?;
    let model: String = body.extract("model")?;

    // Parse first choice (OpenAI always returns choices array)
    let message = body.at("choices[0].message")?;

    let mut content = Vec::new();

    // Text content (may be null for tool-only responses)
    if let Ok(text) = message.extract::<String>("content") {
        if !text.is_empty() {
            content.push(ContentBlock::Text { text });
        }
    }

    // Tool calls
    for tc in message.each("tool_calls") {
        let tool_id: String = tc.extract("id")?;
        let name: String = tc.extract("function.name")?;

        // OpenAI stringifies arguments — the value at function.arguments
        // may be a string (raw API) or already parsed (if BestEffort coercion fired).
        let args_raw = tc.at("function.arguments")?;
        let input = if args_raw.is_object() || args_raw.is_array() {
            // Already parsed (coercion handled it, or caller pre-parsed)
            args_raw
        } else if args_raw.is_string() {
            // Still a string — parse it manually
            let args_str = args_raw.raw().as_str().unwrap_or("{}");
            match FlexValue::from_json(args_str) {
                Ok(parsed) => parsed,
                Err(_) => args_raw,
            }
        } else {
            args_raw
        };

        content.push(ContentBlock::ToolUse {
            id: tool_id,
            name,
            input,
        });
    }

    // Parse finish reason — check for null first (OpenAI sends null during streaming)
    let fr_path = "choices[0].finish_reason";
    let stop_reason = match body.at(fr_path) {
        Ok(v) if v.is_null() => StopReason::Unknown("null".into()),
        _ => body
            .extract::<String>(fr_path)
            .map(|s| match s.as_str() {
                "stop" => StopReason::EndTurn,
                "tool_calls" => StopReason::ToolUse,
                "length" => StopReason::MaxTokens,
                "stop_sequence" => StopReason::StopSequence,
                other => StopReason::Unknown(other.to_string()),
            })
            .unwrap_or(StopReason::Unknown("missing".into())),
    };

    // Parse usage
    let usage = parse_openai_usage(body);

    Ok(NormalizedResponse {
        id,
        model,
        content,
        stop_reason,
        usage,
        raw: body.clone(),
    })
}

fn parse_openai_usage(body: &FlexValue) -> Usage {
    let mut usage = Usage::default();

    if let Ok(input) = body.extract::<u64>("usage.prompt_tokens") {
        usage.input_tokens = input;
    }
    if let Ok(output) = body.extract::<u64>("usage.completion_tokens") {
        usage.output_tokens = output;
    }
    // OpenAI doesn't have cache tokens in the standard API (yet)

    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_response() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "chatcmpl-abc123",
                "model": "gpt-4o-2024-08-06",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": "Hello!"},
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 10, "completion_tokens": 3}
            }"#,
        )
        .unwrap();

        let resp = parse_openai_response(&raw).unwrap();
        assert_eq!(resp.id, "chatcmpl-abc123");
        assert_eq!(resp.model, "gpt-4o-2024-08-06");
        assert_eq!(resp.text(), "Hello!");
        assert!(!resp.has_tool_use());
        assert_eq!(resp.stop_reason, StopReason::EndTurn);
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 3);
    }

    #[test]
    fn parse_tool_call_with_stringified_arguments() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "chatcmpl-def456",
                "model": "gpt-4o",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": "call_abc",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"city\":\"London\",\"units\":\"celsius\"}"
                            }
                        }]
                    },
                    "finish_reason": "tool_calls"
                }],
                "usage": {"prompt_tokens": 50, "completion_tokens": 20}
            }"#,
        )
        .unwrap();

        let resp = parse_openai_response(&raw).unwrap();
        assert!(resp.has_tool_use());
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
        assert_eq!(resp.content.len(), 1); // null content is skipped

        let (id, name, input) = resp.content[0].as_tool_use().unwrap();
        assert_eq!(id, "call_abc");
        assert_eq!(name, "get_weather");

        // The stringified JSON was parsed into a navigable FlexValue
        let city: String = input.extract("city").unwrap();
        let units: String = input.extract("units").unwrap();
        assert_eq!(city, "London");
        assert_eq!(units, "celsius");
    }

    #[test]
    fn parse_text_and_tool_calls() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "chatcmpl-mixed",
                "model": "gpt-4o",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "I'll search for that.",
                        "tool_calls": [{
                            "id": "call_1",
                            "type": "function",
                            "function": {"name": "search", "arguments": "{\"q\":\"rust\"}"}
                        }]
                    },
                    "finish_reason": "tool_calls"
                }],
                "usage": {"prompt_tokens": 30, "completion_tokens": 15}
            }"#,
        )
        .unwrap();

        let resp = parse_openai_response(&raw).unwrap();
        assert_eq!(resp.content.len(), 2);
        assert!(resp.content[0].is_text());
        assert!(resp.content[1].is_tool_use());
        assert_eq!(resp.text(), "I'll search for that.");
    }

    #[test]
    fn parse_multiple_tool_calls() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "chatcmpl-multi",
                "model": "gpt-4o",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [
                            {"id": "call_1", "type": "function", "function": {"name": "search", "arguments": "{\"q\":\"rust\"}"}},
                            {"id": "call_2", "type": "function", "function": {"name": "lookup", "arguments": "{\"key\":\"docs\"}"}}
                        ]
                    },
                    "finish_reason": "tool_calls"
                }],
                "usage": {"prompt_tokens": 40, "completion_tokens": 25}
            }"#,
        )
        .unwrap();

        let resp = parse_openai_response(&raw).unwrap();
        assert_eq!(resp.tool_uses().len(), 2);

        let (_, name1, _) = resp.content[0].as_tool_use().unwrap();
        let (_, name2, _) = resp.content[1].as_tool_use().unwrap();
        assert_eq!(name1, "search");
        assert_eq!(name2, "lookup");
    }

    #[test]
    fn max_tokens_stop_reason() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "chatcmpl-len",
                "model": "gpt-4o",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": "Truncated..."},
                    "finish_reason": "length"
                }],
                "usage": {"prompt_tokens": 10, "completion_tokens": 100}
            }"#,
        )
        .unwrap();

        let resp = parse_openai_response(&raw).unwrap();
        assert_eq!(resp.stop_reason, StopReason::MaxTokens);
    }
}
