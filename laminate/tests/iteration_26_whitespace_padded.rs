//! Iteration 26: GAP — whitespace-padded numeric strings failed to coerce
//! "  42  ", "\t7\t", "  3.14  " all produced opaque serde errors.
//! Fix: Added .trim() at top of String→Numeric, String→Float, and String→Bool arms.

use laminate::FlexValue;

#[test]
fn iter26_whitespace_padded_integer() {
    let json = serde_json::json!({"val": "  42  "});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<i64>("val").unwrap(), 42);
}

#[test]
fn iter26_leading_trailing_tabs_newlines() {
    let json = serde_json::json!({"tab": "\t7\t", "nl": "  55\n", "lead": "  100"});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<i64>("tab").unwrap(), 7);
    assert_eq!(flex.extract::<i64>("nl").unwrap(), 55);
    assert_eq!(flex.extract::<i64>("lead").unwrap(), 100);
}

#[test]
fn iter26_whitespace_padded_float() {
    let json = serde_json::json!({"val": "  3.14  "});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<f64>("val").unwrap(), 3.14);
}

#[test]
fn iter26_whitespace_padded_bool() {
    let json = serde_json::json!({"val": "  true  "});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<bool>("val").unwrap(), true);
}

#[test]
fn iter26_whitespace_padded_as_string_preserved() {
    // Extracting as String should preserve the whitespace
    let json = serde_json::json!({"val": "  42  "});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<String>("val").unwrap(), "  42  ");
}
