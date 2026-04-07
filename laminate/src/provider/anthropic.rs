use crate::diagnostic::StopReason;
use crate::error::Result;
use crate::value::FlexValue;

#[cfg(feature = "streaming")]
use crate::streaming::{FlexStream, Provider, StreamConfig};

use super::{ContentBlock, NormalizedResponse, ProviderAdapter, Usage};

/// Anthropic API response adapter.
///
/// Parses responses from the Anthropic Messages API into `NormalizedResponse`.
///
/// Expected input shape:
/// ```json
/// {
///   "id": "msg_xxx",
///   "model": "claude-4-opus-20260301",
///   "content": [
///     {"type": "text", "text": "Hello"},
///     {"type": "tool_use", "id": "tu_1", "name": "search", "input": {"q": "rust"}}
///   ],
///   "stop_reason": "end_turn",
///   "usage": {"input_tokens": 50, "output_tokens": 30}
/// }
/// ```
pub struct AnthropicAdapter;

impl ProviderAdapter for AnthropicAdapter {
    fn parse_response(&self, body: &FlexValue) -> Result<NormalizedResponse> {
        parse_anthropic_response(body)
    }

    fn emit_response(&self, response: &NormalizedResponse) -> serde_json::Value {
        emit_anthropic_response(response)
    }

    #[cfg(feature = "streaming")]
    fn stream_parser(&self) -> FlexStream {
        FlexStream::new(StreamConfig {
            provider: Provider::Anthropic,
            ..Default::default()
        })
    }
}

/// Emit a NormalizedResponse as Anthropic API format.
pub fn emit_anthropic_response(response: &NormalizedResponse) -> serde_json::Value {
    use serde_json::json;

    let content: Vec<serde_json::Value> = response
        .content
        .iter()
        .map(|block| match block {
            ContentBlock::Text { text } => json!({"type": "text", "text": text}),
            ContentBlock::ToolUse { id, name, input } => {
                json!({"type": "tool_use", "id": id, "name": name, "input": input.raw()})
            }
            ContentBlock::Unknown { block_type, data } => {
                json!({"type": block_type, "data": data.raw()})
            }
        })
        .collect();

    let stop_reason = match &response.stop_reason {
        StopReason::EndTurn => "end_turn",
        StopReason::ToolUse => "tool_use",
        StopReason::MaxTokens => "max_tokens",
        StopReason::StopSequence => "stop_sequence",
        StopReason::Unknown(s) => s.as_str(),
    };

    let mut usage = serde_json::json!({
        "input_tokens": response.usage.input_tokens,
        "output_tokens": response.usage.output_tokens,
    });
    if let Some(cache_read) = response.usage.cache_read_tokens {
        usage["cache_read_input_tokens"] = serde_json::json!(cache_read);
    }
    if let Some(cache_create) = response.usage.cache_creation_tokens {
        usage["cache_creation_input_tokens"] = serde_json::json!(cache_create);
    }

    json!({
        "id": response.id,
        "type": "message",
        "model": response.model,
        "content": content,
        "stop_reason": stop_reason,
        "usage": usage,
    })
}

/// Parse an Anthropic Messages API response body into a `NormalizedResponse`.
pub fn parse_anthropic_response(body: &FlexValue) -> Result<NormalizedResponse> {
    let id: String = body.extract("id")?;
    let model: String = body.extract("model")?;

    // Parse content blocks
    let content_blocks = body.each("content");
    let mut content = Vec::with_capacity(content_blocks.len());

    for block in &content_blocks {
        let block_type: String = block.extract("type")?;
        match block_type.as_str() {
            "text" => {
                let text: String = block.extract("text")?;
                content.push(ContentBlock::Text { text });
            }
            "tool_use" => {
                let tool_id: String = block.extract("id")?;
                let name: String = block.extract("name")?;
                let input = block.at("input")?;
                content.push(ContentBlock::ToolUse {
                    id: tool_id,
                    name,
                    input,
                });
            }
            other => {
                content.push(ContentBlock::Unknown {
                    block_type: other.to_string(),
                    data: block.clone(),
                });
            }
        }
    }

    // Parse stop reason — check for null first (can be null in incomplete responses)
    let stop_reason = match body.at("stop_reason") {
        Ok(v) if v.is_null() => StopReason::Unknown("null".into()),
        _ => body
            .extract::<String>("stop_reason")
            .map(|s| match s.as_str() {
                "end_turn" => StopReason::EndTurn,
                "tool_use" => StopReason::ToolUse,
                "max_tokens" => StopReason::MaxTokens,
                "stop_sequence" => StopReason::StopSequence,
                other => StopReason::Unknown(other.to_string()),
            })
            .unwrap_or(StopReason::Unknown("missing".into())),
    };

    // Parse usage
    let usage = parse_anthropic_usage(body);

    Ok(NormalizedResponse {
        id,
        model,
        content,
        stop_reason,
        usage,
        raw: body.clone(),
    })
}

fn parse_anthropic_usage(body: &FlexValue) -> Usage {
    let mut usage = Usage::default();

    if let Ok(input) = body.extract::<u64>("usage.input_tokens") {
        usage.input_tokens = input;
    }
    if let Ok(output) = body.extract::<u64>("usage.output_tokens") {
        usage.output_tokens = output;
    }
    if let Ok(cache_read) = body.extract::<u64>("usage.cache_read_input_tokens") {
        usage.cache_read_tokens = Some(cache_read);
    }
    if let Ok(cache_create) = body.extract::<u64>("usage.cache_creation_input_tokens") {
        usage.cache_creation_tokens = Some(cache_create);
    }

    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_response() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "msg_abc123",
                "type": "message",
                "model": "claude-opus-4-6-20260301",
                "content": [{"type": "text", "text": "Hello, world!"}],
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 10, "output_tokens": 5}
            }"#,
        )
        .unwrap();

        let resp = parse_anthropic_response(&raw).unwrap();
        assert_eq!(resp.id, "msg_abc123");
        assert_eq!(resp.model, "claude-opus-4-6-20260301");
        assert_eq!(resp.text(), "Hello, world!");
        assert!(!resp.has_tool_use());
        assert_eq!(resp.stop_reason, StopReason::EndTurn);
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
    }

    #[test]
    fn parse_tool_use_response() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "msg_def456",
                "model": "claude-opus-4-6-20260301",
                "content": [
                    {"type": "text", "text": "I'll search for that."},
                    {"type": "tool_use", "id": "tu_1", "name": "search", "input": {"query": "rust laminate"}}
                ],
                "stop_reason": "tool_use",
                "usage": {"input_tokens": 50, "output_tokens": 30}
            }"#,
        )
        .unwrap();

        let resp = parse_anthropic_response(&raw).unwrap();
        assert_eq!(resp.content.len(), 2);
        assert!(resp.content[0].is_text());
        assert!(resp.content[1].is_tool_use());
        assert!(resp.has_tool_use());
        assert_eq!(resp.stop_reason, StopReason::ToolUse);

        let (id, name, input) = resp.content[1].as_tool_use().unwrap();
        assert_eq!(id, "tu_1");
        assert_eq!(name, "search");
        let query: String = input.extract("query").unwrap();
        assert_eq!(query, "rust laminate");
    }

    #[test]
    fn parse_with_cache_usage() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "msg_cached",
                "model": "claude-opus-4-6-20260301",
                "content": [{"type": "text", "text": "Cached response"}],
                "stop_reason": "end_turn",
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 20,
                    "cache_read_input_tokens": 80,
                    "cache_creation_input_tokens": 0
                }
            }"#,
        )
        .unwrap();

        let resp = parse_anthropic_response(&raw).unwrap();
        assert_eq!(resp.usage.cache_read_tokens, Some(80));
        assert_eq!(resp.usage.cache_creation_tokens, Some(0));
    }

    #[test]
    fn unknown_block_type_preserved() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "msg_future",
                "model": "claude-opus-4-6-20260301",
                "content": [{"type": "thinking", "text": "Let me think..."}],
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 10, "output_tokens": 5}
            }"#,
        )
        .unwrap();

        let resp = parse_anthropic_response(&raw).unwrap();
        assert_eq!(resp.content.len(), 1);
        match &resp.content[0] {
            ContentBlock::Unknown { block_type, .. } => {
                assert_eq!(block_type, "thinking");
            }
            _ => panic!("expected Unknown block"),
        }
    }

    #[test]
    fn multiple_text_blocks_concatenate() {
        let raw = FlexValue::from_json(
            r#"{
                "id": "msg_multi",
                "model": "claude-opus-4-6-20260301",
                "content": [
                    {"type": "text", "text": "Hello "},
                    {"type": "text", "text": "world!"}
                ],
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 10, "output_tokens": 5}
            }"#,
        )
        .unwrap();

        let resp = parse_anthropic_response(&raw).unwrap();
        assert_eq!(resp.text(), "Hello world!");
    }
}
