use laminate::diagnostic::StopReason;
use laminate::provider::openai::parse_openai_response;
use laminate::FlexValue;

/// Iteration 44 — Provider Probe: OpenAI null/missing finish_reason.
/// Result: GAP — null finish_reason silently coerced to Unknown("").
/// Fix: explicit null check before extract in both OpenAI and Anthropic providers.

#[test]
fn iter44_null_finish_reason() {
    // OpenAI sends finish_reason: null during streaming (incomplete response)
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-streaming",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "partial respo"},
            "finish_reason": null
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }"#,
    )
    .unwrap();

    let resp = parse_openai_response(&raw).unwrap();
    assert_eq!(resp.text(), "partial respo");
    // After fix: null produces Unknown("null"), not Unknown("")
    assert_eq!(resp.stop_reason, StopReason::Unknown("null".into()));
}

#[test]
fn iter44_missing_finish_reason() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-nofr",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "response"}
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }"#,
    )
    .unwrap();

    let resp = parse_openai_response(&raw).unwrap();
    // Missing key produces Unknown("missing")
    assert_eq!(resp.stop_reason, StopReason::Unknown("missing".into()));
}

#[test]
fn iter44_empty_choices_error() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-empty",
        "model": "gpt-4o",
        "choices": [],
        "usage": {"prompt_tokens": 10, "completion_tokens": 0}
    }"#,
    )
    .unwrap();

    // Empty choices correctly produces an IndexOutOfBounds error
    let result = parse_openai_response(&raw);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("choices[0]"),
        "error should mention choices[0]: {err}"
    );
}
