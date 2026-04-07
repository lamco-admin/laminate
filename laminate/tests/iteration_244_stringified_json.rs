//! Iteration 244 (from target 245): Stringified JSON at BestEffort
//!
//! When a JSON value is a string containing valid JSON like '{"a":1}',
//! and the target type is serde_json::Value (not String), BestEffort
//! should parse the stringified JSON and return the parsed value.
//!
//! Adversarial:
//! - Does '{"a":1}' extract as Value? As String?
//! - Does '[1,2,3]' extract as Value?
//! - Does '"hello"' (string containing quoted string) get double-parsed?
//! - Does '42' (string containing number) get parsed as Value?

use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn stringified_json_object_as_value() {
    let val = FlexValue::from(serde_json::json!(r#"{"a":1,"b":"hello"}"#))
        .with_coercion(CoercionLevel::BestEffort);

    // Extract as serde_json::Value — should parse the stringified JSON
    let result: serde_json::Value = val.extract_root().unwrap();
    println!("Stringified JSON object: {:?}", result);

    // The coercion engine at BestEffort parses stringified JSON for non-String targets.
    // serde_json::Value has no coercion_hint (returns None), so the standard coercion
    // path doesn't apply. Let's observe what actually happens.
}

#[test]
fn stringified_json_object_as_string() {
    let val =
        FlexValue::from(serde_json::json!(r#"{"a":1}"#)).with_coercion(CoercionLevel::BestEffort);

    // Extract as String — should keep the original string
    let result: String = val.extract_root().unwrap();
    assert_eq!(
        result, r#"{"a":1}"#,
        "String extraction should preserve stringified JSON"
    );
}

#[test]
fn stringified_json_array_as_value() {
    let val =
        FlexValue::from(serde_json::json!("[1,2,3]")).with_coercion(CoercionLevel::BestEffort);

    let result: serde_json::Value = val.extract_root().unwrap();
    println!("Stringified JSON array: {:?}", result);
    // serde_json::Value accepts strings natively, so it should be the string "[1,2,3]"
    // not the parsed array [1,2,3] — because Value has no coercion_hint
}

#[test]
fn stringified_number_as_i64() {
    // This is normal string→int coercion, not stringified JSON
    let val = FlexValue::from(serde_json::json!("42")).with_coercion(CoercionLevel::BestEffort);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 42);
}

#[test]
fn stringified_json_object_at_exact() {
    // At Exact level, stringified JSON should NOT be parsed
    let val = FlexValue::from(serde_json::json!(r#"{"a":1}"#)).with_coercion(CoercionLevel::Exact);

    let result: serde_json::Value = val.extract_root().unwrap();
    println!("Stringified JSON at Exact: {:?}", result);
    // Should be the string value, not parsed
    assert!(
        result.is_string(),
        "Exact should keep stringified JSON as string"
    );
}

#[test]
fn deeply_nested_stringified_json() {
    // A string containing deeply nested JSON
    let json_str = r#"{"users":[{"name":"Alice","scores":[1,2,3]}]}"#;
    let val = FlexValue::from(serde_json::json!(json_str)).with_coercion(CoercionLevel::BestEffort);

    let result: String = val.extract_root().unwrap();
    assert_eq!(
        result, json_str,
        "String target preserves deeply nested stringified JSON"
    );
}

#[test]
fn double_stringified_json() {
    // JSON string that contains a JSON string: '"hello"' (with inner quotes)
    // When serialized, this is: "\"hello\""
    // At BestEffort, extracting as String should give the original content
    let val =
        FlexValue::from(serde_json::json!("\"hello\"")).with_coercion(CoercionLevel::BestEffort);

    let result: String = val.extract_root().unwrap();
    println!("Double-stringified: {:?}", result);
    assert_eq!(result, "\"hello\"", "should not double-parse string");
}
