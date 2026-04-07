use laminate::diagnostic::StopReason;
use laminate::provider::anthropic::{emit_anthropic_response, parse_anthropic_response};
use laminate::FlexValue;

/// Iteration 43 — Provider Probe: Anthropic response with empty content array.
/// Result: PASS — empty, missing, and null content all parse gracefully.

#[test]
fn iter43_empty_content_array() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "msg_empty",
        "model": "claude-opus-4-6-20260301",
        "content": [],
        "stop_reason": "max_tokens",
        "usage": {"input_tokens": 100, "output_tokens": 0}
    }"#,
    )
    .unwrap();

    let resp = parse_anthropic_response(&raw).unwrap();
    assert_eq!(resp.id, "msg_empty");
    assert_eq!(resp.content.len(), 0);
    assert_eq!(resp.text(), "");
    assert!(!resp.has_tool_use());
    assert_eq!(resp.stop_reason, StopReason::MaxTokens);
    assert_eq!(resp.usage.output_tokens, 0);

    // Round-trip preserves empty content
    let emitted = emit_anthropic_response(&resp);
    assert_eq!(emitted["content"].as_array().unwrap().len(), 0);
}

#[test]
fn iter43_missing_content_key() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "msg_nocontent",
        "model": "claude-opus-4-6-20260301",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 5, "output_tokens": 0}
    }"#,
    )
    .unwrap();

    let resp = parse_anthropic_response(&raw).unwrap();
    assert_eq!(resp.content.len(), 0);
    assert_eq!(resp.text(), "");
}

#[test]
fn iter43_null_content() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "msg_nullcontent",
        "model": "claude-opus-4-6-20260301",
        "content": null,
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 5, "output_tokens": 0}
    }"#,
    )
    .unwrap();

    let resp = parse_anthropic_response(&raw).unwrap();
    assert_eq!(resp.content.len(), 0);
    assert_eq!(resp.text(), "");
}
