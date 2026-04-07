//! Iteration 40: Boundary — FlexValue::from_json error quality for invalid JSON
//!
//! Verifies that common JSON mistakes produce clear, specific error messages
//! rather than opaque parse failures.

use laminate::FlexValue;

#[test]
fn trailing_comma_produces_clear_error() {
    let err = FlexValue::from_json(r#"{"a": 1,}"#).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("trailing comma"),
        "expected 'trailing comma', got: {msg}"
    );
}

#[test]
fn unquoted_key_produces_clear_error() {
    let err = FlexValue::from_json(r#"{a: 1}"#).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("key must be a string"), "got: {msg}");
}

#[test]
fn empty_string_produces_clear_error() {
    let err = FlexValue::from_json("").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("EOF"), "got: {msg}");
}

#[test]
fn trailing_text_produces_clear_error() {
    let err = FlexValue::from_json(r#"{"a": 1} extra"#).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("trailing characters"), "got: {msg}");
}

#[test]
fn all_errors_include_path_context() {
    // All from_json errors should be wrapped with "(root)" path
    let err = FlexValue::from_json("invalid").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("(root)"),
        "error should include path context, got: {msg}"
    );
}
