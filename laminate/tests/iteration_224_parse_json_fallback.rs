/// Iteration 224: #[laminate(parse_json_string)] on non-JSON string
///
/// Target #224: When parse_json_string gets a string that isn't valid JSON,
/// it falls back to the original string value. What happens when the target
/// type can't deserialize a plain string?
use laminate::Laminate;
use std::collections::HashMap;

/// Case 1: Target is String — non-JSON string should work fine
#[derive(Debug, Laminate)]
struct WithString {
    name: String,
    #[laminate(parse_json_string)]
    data: String,
}

#[test]
fn parse_json_string_fallback_to_plain_string() {
    let json = r#"{"name": "test", "data": "just a plain string"}"#;
    let (w, diagnostics) = WithString::from_json(json).unwrap();

    println!("w = {:?}", w);
    assert_eq!(w.data, "just a plain string");
    assert!(diagnostics.is_empty());
}

/// Case 2: Target is HashMap — non-JSON string can't be deserialized as HashMap
#[derive(Debug, Laminate)]
struct WithMap {
    name: String,
    #[laminate(parse_json_string)]
    config: HashMap<String, serde_json::Value>,
}

#[test]
fn parse_json_string_non_json_into_map_fails() {
    let json = r#"{"name": "test", "config": "not-json-at-all"}"#;
    let result = WithMap::from_json(json);

    println!("result = {:?}", result);

    // parse_json_string tries from_str("not-json-at-all") → fails
    // falls back to original Value::String("not-json-at-all")
    // then serde tries from_value::<HashMap>(String("not-json-at-all")) → fails
    assert!(result.is_err(), "non-JSON string into HashMap should fail");
}

/// Case 3: Target is serde_json::Value — anything goes
#[derive(Debug, Laminate)]
struct WithValue {
    name: String,
    #[laminate(parse_json_string)]
    blob: serde_json::Value,
}

#[test]
fn parse_json_string_non_json_into_value_keeps_string() {
    let json = r#"{"name": "test", "blob": "not-json"}"#;
    let (w, diagnostics) = WithValue::from_json(json).unwrap();

    println!("w = {:?}", w);
    // parse_json_string fails on "not-json", falls back to Value::String
    // from_value::<Value>(String("not-json")) succeeds
    assert_eq!(w.blob, serde_json::json!("not-json"));
    assert!(diagnostics.is_empty());
}

#[test]
fn parse_json_string_valid_json_into_value() {
    let json = r#"{"name": "test", "blob": "[1,2,3]"}"#;
    let (w, _) = WithValue::from_json(json).unwrap();

    // parse_json_string succeeds: "[1,2,3]" → Value::Array
    assert_eq!(w.blob, serde_json::json!([1, 2, 3]));
}
