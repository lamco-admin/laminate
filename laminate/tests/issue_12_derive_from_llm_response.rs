//! Test for issue #12: derive-generated `from_llm_response` on structs and
//! enums — one call from LLM text (fenced/prose) to a typed value.

use laminate_derive::Laminate;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Laminate)]
struct ToolArgs {
    #[laminate(coerce)]
    count: u16,
    query: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, PartialEq, Laminate)]
enum Status {
    Active,
    #[laminate(unknown)]
    Unknown(String),
}

#[test]
fn struct_from_llm_response_fenced() {
    let text = "Result:\n```json\n{\"count\": \"3\", \"query\": \"hi\"}\n```";
    let (a, _diags) = ToolArgs::from_llm_response(text).unwrap();
    assert_eq!(a.count, 3); // coerced from "3"
    assert_eq!(a.query, "hi");
}

#[test]
fn struct_from_llm_response_prose() {
    let (a, _diags) =
        ToolArgs::from_llm_response("here you go: {\"count\": 7, \"query\": \"x\"} done").unwrap();
    assert_eq!(a.count, 7);
    assert_eq!(a.query, "x");
}

#[test]
fn enum_from_llm_response_unknown_capture() {
    let (s, _diags) = Status::from_llm_response("```\n\"paused\"\n```").unwrap();
    assert_eq!(s, Status::Unknown("paused".to_string()));
}

#[test]
fn struct_from_llm_response_matches_clean_json() {
    let (a, _diags) = ToolArgs::from_llm_response(r#"{"count": 1, "query": "q"}"#).unwrap();
    assert_eq!(a.count, 1);
}
