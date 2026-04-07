use laminate_derive::Laminate;
use std::collections::HashMap;

// ── Basic struct with no attributes ──

#[derive(Debug, Laminate)]
struct Simple {
    name: String,
    age: u32,
}

#[test]
fn simple_struct() {
    let (s, diags) = Simple::from_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    assert_eq!(s.name, "Alice");
    assert_eq!(s.age, 30);
    assert!(diags.is_empty());
}

#[test]
fn simple_struct_missing_field() {
    let result = Simple::from_json(r#"{"name": "Alice"}"#);
    assert!(result.is_err());
}

// ── Struct with overflow ──

#[derive(Debug, Laminate)]
struct WithOverflow {
    id: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn overflow_captures_unknown_fields() {
    let (s, _) = WithOverflow::from_json(r#"{"id": "abc", "foo": 1, "bar": "hello"}"#).unwrap();
    assert_eq!(s.id, "abc");
    assert_eq!(s.extra.len(), 2);
    assert_eq!(s.extra["foo"], serde_json::json!(1));
    assert_eq!(s.extra["bar"], serde_json::json!("hello"));
}

#[test]
fn overflow_empty_when_no_unknowns() {
    let (s, _) = WithOverflow::from_json(r#"{"id": "abc"}"#).unwrap();
    assert_eq!(s.id, "abc");
    assert!(s.extra.is_empty());
}

// ── Struct with rename ──

#[derive(Debug, Laminate)]
struct WithRename {
    #[laminate(rename = "type")]
    response_type: String,
    id: u64,
}

#[test]
fn rename_field() {
    let (s, _) = WithRename::from_json(r#"{"type": "message", "id": 42}"#).unwrap();
    assert_eq!(s.response_type, "message");
    assert_eq!(s.id, 42);
}

// ── Struct with default ──

#[derive(Debug, Laminate)]
struct WithDefault {
    name: String,
    #[laminate(default)]
    verified: bool,
    #[laminate(default)]
    score: u32,
}

#[test]
fn default_fills_missing_fields() {
    let (s, _) = WithDefault::from_json(r#"{"name": "Alice"}"#).unwrap();
    assert_eq!(s.name, "Alice");
    assert!(!s.verified); // bool default = false
    assert_eq!(s.score, 0); // u32 default = 0
}

#[test]
fn default_fills_null_fields() {
    let (s, _) =
        WithDefault::from_json(r#"{"name": "Alice", "verified": null, "score": null}"#).unwrap();
    assert_eq!(s.name, "Alice");
    assert!(!s.verified);
    assert_eq!(s.score, 0);
}

#[test]
fn default_respects_present_values() {
    let (s, _) =
        WithDefault::from_json(r#"{"name": "Alice", "verified": true, "score": 100}"#).unwrap();
    assert_eq!(s.name, "Alice");
    assert!(s.verified);
    assert_eq!(s.score, 100);
}

// ── Struct with coerce ──

#[derive(Debug, Laminate)]
struct WithCoerce {
    #[laminate(coerce)]
    port: u16,
    #[laminate(coerce)]
    debug: bool,
    #[laminate(coerce)]
    workers: u32,
}

#[test]
fn coerce_string_values() {
    let (s, diags) =
        WithCoerce::from_json(r#"{"port": "8080", "debug": "true", "workers": "4"}"#).unwrap();
    assert_eq!(s.port, 8080);
    assert!(s.debug);
    assert_eq!(s.workers, 4);
    // Should have diagnostics for each coercion
    assert!(!diags.is_empty());
}

#[test]
fn coerce_already_correct_types() {
    let (s, _) = WithCoerce::from_json(r#"{"port": 8080, "debug": true, "workers": 4}"#).unwrap();
    assert_eq!(s.port, 8080);
    assert!(s.debug);
    assert_eq!(s.workers, 4);
}

// ── Combined attributes ──

#[derive(Debug, Laminate)]
struct ApiResponse {
    id: String,
    #[laminate(rename = "type")]
    response_type: String,
    #[laminate(coerce)]
    status_code: u16,
    #[laminate(default)]
    deprecated: bool,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn combined_attributes_real_world() {
    let (resp, diags) = ApiResponse::from_json(
        r#"{
            "id": "msg_abc123",
            "type": "message",
            "status_code": "200",
            "metadata": {"model": "claude-4"},
            "version": "2.0"
        }"#,
    )
    .unwrap();

    assert_eq!(resp.id, "msg_abc123");
    assert_eq!(resp.response_type, "message");
    assert_eq!(resp.status_code, 200);
    assert!(!resp.deprecated); // defaulted
    assert_eq!(resp.extra.len(), 2); // metadata + version captured
    assert!(resp.extra.contains_key("metadata"));
    assert!(resp.extra.contains_key("version"));
    // status_code coercion should produce a diagnostic
    assert!(!diags.is_empty());
}

// ── Error cases ──

#[test]
fn non_object_input_fails() {
    let result = Simple::from_json(r#"[1, 2, 3]"#);
    assert!(result.is_err());
}

#[test]
fn invalid_json_fails() {
    let result = Simple::from_json(r#"not json"#);
    assert!(result.is_err());
}

// ── from_flex_value ──

#[test]
fn from_flex_value_works() {
    let value = serde_json::json!({"name": "Bob", "age": 25});
    let (s, _) = Simple::from_flex_value(&value).unwrap();
    assert_eq!(s.name, "Bob");
    assert_eq!(s.age, 25);
}
