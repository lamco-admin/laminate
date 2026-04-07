use crate::diagnostic::StopReason;
use crate::error::Result;
use crate::value::FlexValue;

use super::{ContentBlock, NormalizedResponse, ProviderAdapter, Usage};

#[cfg(feature = "streaming")]
use crate::streaming::{FlexStream, Provider, StreamConfig};

/// Ollama API response adapter.
///
/// Parses responses from the Ollama chat API into `NormalizedResponse`.
///
/// Expected input shape:
/// ```json
/// {
///   "model": "llama3.2:latest",
///   "message": {"role": "assistant", "content": "Hello!"},
///   "done": true,
///   "done_reason": "stop",
///   "total_duration": 1234567890,
///   "eval_count": 42,
///   "prompt_eval_count": 26
/// }
/// ```
pub struct OllamaAdapter;

impl ProviderAdapter for OllamaAdapter {
    fn parse_response(&self, body: &FlexValue) -> Result<NormalizedResponse> {
        parse_ollama_response(body)
    }

    fn emit_response(&self, response: &NormalizedResponse) -> serde_json::Value {
        let mut message = serde_json::json!({
            "role": "assistant",
            "content": response.text(),
        });

        // Tool calls (Ollama format — arguments as objects, not strings)
        let tool_calls: Vec<serde_json::Value> = response
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::ToolUse { name, input, .. } = block {
                    Some(serde_json::json!({
                        "function": {
                            "name": name,
                            "arguments": input.raw(),
                        }
                    }))
                } else {
                    None
                }
            })
            .collect();
        if !tool_calls.is_empty() {
            message["tool_calls"] = serde_json::json!(tool_calls);
        }

        serde_json::json!({
            "model": response.model,
            "message": message,
            "done": true,
            "done_reason": match &response.stop_reason {
                StopReason::EndTurn => "stop",
                StopReason::MaxTokens => "length",
                _ => "stop",
            },
            "prompt_eval_count": response.usage.input_tokens,
            "eval_count": response.usage.output_tokens,
        })
    }

    #[cfg(feature = "streaming")]
    fn stream_parser(&self) -> FlexStream {
        // Ollama uses newline-delimited JSON, not SSE.
        // For now, return an Anthropic-style parser as a placeholder.
        FlexStream::new(StreamConfig {
            provider: Provider::Anthropic,
            ..Default::default()
        })
    }
}

/// Parse an Ollama API response body into a `NormalizedResponse`.
pub fn parse_ollama_response(body: &FlexValue) -> Result<NormalizedResponse> {
    let model: String = body.extract("model")?;

    // Ollama doesn't have a response ID — generate from model + timestamp
    let id = body
        .maybe::<String>("created_at")?
        .unwrap_or_else(|| "ollama".into());

    let mut content = Vec::new();

    // Message content
    if let Ok(text) = body.extract::<String>("message.content") {
        if !text.is_empty() {
            content.push(ContentBlock::Text { text });
        }
    }

    // Tool calls (Ollama 0.5+ — arguments are objects, not stringified JSON)
    for tc in body.each("message.tool_calls") {
        let name: String = tc.extract("function.name")?;
        let input = tc
            .at("function.arguments")
            .unwrap_or_else(|_| FlexValue::new(serde_json::json!({})));
        // Ollama doesn't provide tool call IDs — generate one from name
        let tool_id = format!("ollama_{}", name);
        content.push(ContentBlock::ToolUse {
            id: tool_id,
            name,
            input,
        });
    }

    // Stop reason
    let stop_reason = body
        .maybe::<String>("done_reason")?
        .map(|s| match s.as_str() {
            "stop" => StopReason::EndTurn,
            "length" => StopReason::MaxTokens,
            other => StopReason::Unknown(other.to_string()),
        })
        .unwrap_or(StopReason::Unknown("unknown".into()));

    // Usage — Ollama uses different field names
    let mut usage = Usage::default();
    if let Ok(prompt_count) = body.extract::<u64>("prompt_eval_count") {
        usage.input_tokens = prompt_count;
    }
    if let Ok(eval_count) = body.extract::<u64>("eval_count") {
        usage.output_tokens = eval_count;
    }

    Ok(NormalizedResponse {
        id,
        model,
        content,
        stop_reason,
        usage,
        raw: body.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_response() {
        let raw = FlexValue::from_json(
            r#"{
                "model": "llama3.2:latest",
                "created_at": "2026-03-31T15:00:00Z",
                "message": {"role": "assistant", "content": "Hello, world!"},
                "done": true,
                "done_reason": "stop",
                "total_duration": 1234567890,
                "prompt_eval_count": 26,
                "eval_count": 42
            }"#,
        )
        .unwrap();

        let resp = parse_ollama_response(&raw).unwrap();
        assert_eq!(resp.model, "llama3.2:latest");
        assert_eq!(resp.text(), "Hello, world!");
        assert_eq!(resp.stop_reason, StopReason::EndTurn);
        assert_eq!(resp.usage.input_tokens, 26);
        assert_eq!(resp.usage.output_tokens, 42);
    }

    #[test]
    fn parse_max_tokens_response() {
        let raw = FlexValue::from_json(
            r#"{
                "model": "llama3.2:latest",
                "message": {"role": "assistant", "content": "Truncated..."},
                "done": true,
                "done_reason": "length",
                "prompt_eval_count": 100,
                "eval_count": 500
            }"#,
        )
        .unwrap();

        let resp = parse_ollama_response(&raw).unwrap();
        assert_eq!(resp.stop_reason, StopReason::MaxTokens);
    }

    #[test]
    fn parse_from_fixture() {
        let json = std::fs::read_to_string(format!(
            "{}/testdata/api-responses/ollama_response.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let raw = FlexValue::from_json(&json).unwrap();

        let resp = parse_ollama_response(&raw).unwrap();
        assert_eq!(resp.model, "llama3.2:latest");
        assert!(resp.text().contains("Hello, world!"));
        assert_eq!(resp.usage.input_tokens, 26);
        assert_eq!(resp.usage.output_tokens, 42);
    }
}
