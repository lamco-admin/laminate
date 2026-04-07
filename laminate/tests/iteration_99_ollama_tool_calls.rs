#![cfg(feature = "providers")]
use laminate::diagnostic::StopReason;
/// Iteration 99: Ollama adapter — tool call support
///
/// Ollama 0.5+ supports tool calls with arguments as objects (not
/// stringified like OpenAI). The adapter now parses and emits tool calls.
use laminate::provider::ollama::{parse_ollama_response, OllamaAdapter};
use laminate::provider::ProviderAdapter;
use laminate::provider::{ContentBlock, NormalizedResponse, Usage};
use laminate::FlexValue;

#[test]
fn ollama_parses_tool_calls() {
    let raw = FlexValue::from_json(
        r#"{
            "model": "llama3.2:latest",
            "created_at": "2026-04-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": [{
                    "function": {
                        "name": "get_weather",
                        "arguments": {"city": "London"}
                    }
                }]
            },
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 50,
            "eval_count": 10
        }"#,
    )
    .unwrap();

    let resp = parse_ollama_response(&raw).unwrap();
    assert!(resp.has_tool_use(), "should detect tool calls");
    assert_eq!(resp.tool_uses().len(), 1);

    let (_, name, input) = resp.content[0].as_tool_use().unwrap();
    assert_eq!(name, "get_weather");
    let city: String = input.extract("city").unwrap();
    assert_eq!(city, "London");
}

#[test]
fn ollama_emits_tool_calls() {
    let response = NormalizedResponse {
        id: "test".into(),
        model: "llama3.2:latest".into(),
        content: vec![ContentBlock::ToolUse {
            id: "tu_1".into(),
            name: "search".into(),
            input: FlexValue::from_json(r#"{"q": "test"}"#).unwrap(),
        }],
        stop_reason: StopReason::ToolUse,
        usage: Usage::default(),
        raw: FlexValue::new(serde_json::json!({})),
    };

    let adapter = OllamaAdapter;
    let emitted = adapter.emit_response(&response);

    assert!(
        emitted["message"]["tool_calls"].is_array(),
        "emitted should contain tool_calls"
    );
    assert_eq!(
        emitted["message"]["tool_calls"][0]["function"]["name"],
        "search"
    );
}
