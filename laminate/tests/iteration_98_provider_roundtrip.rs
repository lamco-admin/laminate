#![cfg(feature = "providers")]
/// Iteration 98: Anthropic provider parse→emit→parse round-trip
///
/// emit_anthropic_response was dropping cache usage tokens. Fixed to
/// preserve cache_read_input_tokens and cache_creation_input_tokens
/// when present in the Usage struct.
use laminate::provider::anthropic::{emit_anthropic_response, parse_anthropic_response};
use laminate::FlexValue;

#[test]
fn anthropic_roundtrip_preserves_cache_usage() {
    let original_json = r#"{
        "id": "msg_abc",
        "type": "message",
        "model": "claude-opus-4-6-20260301",
        "content": [
            {"type": "text", "text": "Hello"},
            {"type": "tool_use", "id": "tu_1", "name": "search", "input": {"q": "test"}}
        ],
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_read_input_tokens": 80,
            "cache_creation_input_tokens": 20
        }
    }"#;

    let body = FlexValue::from_json(original_json).unwrap();
    let parsed = parse_anthropic_response(&body).unwrap();

    // Emit and re-parse
    let emitted = emit_anthropic_response(&parsed);
    let re_parsed = parse_anthropic_response(&FlexValue::new(emitted)).unwrap();

    // All usage fields should survive the round-trip
    assert_eq!(re_parsed.usage.input_tokens, 100);
    assert_eq!(re_parsed.usage.output_tokens, 50);
    assert_eq!(
        re_parsed.usage.cache_read_tokens,
        Some(80),
        "cache_read should survive round-trip"
    );
    assert_eq!(
        re_parsed.usage.cache_creation_tokens,
        Some(20),
        "cache_creation should survive round-trip"
    );

    // Content should survive
    assert_eq!(re_parsed.text(), "Hello");
    assert!(re_parsed.has_tool_use());
}

#[test]
fn anthropic_roundtrip_without_cache_tokens() {
    // When cache tokens aren't present, they shouldn't appear in output
    let json = r#"{
        "id": "msg_nocache",
        "model": "claude-opus-4-6-20260301",
        "content": [{"type": "text", "text": "No cache"}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 5}
    }"#;

    let body = FlexValue::from_json(json).unwrap();
    let parsed = parse_anthropic_response(&body).unwrap();
    let emitted = emit_anthropic_response(&parsed);

    // cache fields should not appear in emitted JSON
    assert!(emitted["usage"].get("cache_read_input_tokens").is_none());
    assert!(emitted["usage"]
        .get("cache_creation_input_tokens")
        .is_none());
}
