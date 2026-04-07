/// Iteration 223: #[laminate(parse_json_string)] on stringified JSON
///
/// Target #223: When a JSON value is a string containing valid JSON, the
/// parse_json_string attribute should parse it before deserialization.
/// Test: "{\"a\":1}" → parsed into a struct with field `a: i64`.
use laminate::Laminate;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Wrapper {
    name: String,
    #[laminate(parse_json_string)]
    config: HashMap<String, serde_json::Value>,
}

#[test]
fn parse_json_string_on_stringified_object() {
    let json = r#"{"name": "app", "config": "{\"port\":8080,\"debug\":true}"}"#;
    let (wrapper, diagnostics) = Wrapper::from_json(json).unwrap();

    println!("wrapper = {:?}", wrapper);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(wrapper.name, "app");
    assert_eq!(wrapper.config["port"], serde_json::json!(8080));
    assert_eq!(wrapper.config["debug"], serde_json::json!(true));
}

/// When the value is already a proper JSON object (not stringified), it should work too
#[test]
fn parse_json_string_on_already_parsed_object() {
    let json = r#"{"name": "app", "config": {"port": 8080, "debug": true}}"#;
    let (wrapper, diagnostics) = Wrapper::from_json(json).unwrap();

    println!("wrapper = {:?}", wrapper);

    assert_eq!(wrapper.config["port"], serde_json::json!(8080));
    assert!(diagnostics.is_empty());
}

/// Nested struct target
#[derive(Debug, Deserialize, serde::Serialize)]
struct Inner {
    x: i64,
    y: i64,
}

#[derive(Debug, Laminate)]
struct WithInner {
    label: String,
    #[laminate(parse_json_string)]
    point: Inner,
}

#[test]
fn parse_json_string_into_struct() {
    let json = r#"{"label": "origin", "point": "{\"x\":0,\"y\":0}"}"#;
    let (w, diagnostics) = WithInner::from_json(json).unwrap();

    println!("w = {:?}", w);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(w.point.x, 0);
    assert_eq!(w.point.y, 0);
}
