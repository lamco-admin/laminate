//! Tests for issue #3: `from_llm_response` locates a JSON payload inside an LLM
//! text response (fenced or prose-wrapped) without loosening `from_json`.

use laminate::FlexValue;

#[test]
fn parses_fenced_json_with_lang_tag() {
    let text = "Here you go:\n```json\n{\"port\": 8080, \"ok\": true}\n```\nthanks!";
    let v = FlexValue::from_llm_response(text).unwrap();
    assert_eq!(v.extract::<u16>("port").unwrap(), 8080);
    assert!(v.extract::<bool>("ok").unwrap());
}

#[test]
fn parses_bare_fence() {
    let text = "```\n{\"a\": 1}\n```";
    let v = FlexValue::from_llm_response(text).unwrap();
    assert_eq!(v.extract::<i64>("a").unwrap(), 1);
}

#[test]
fn parses_json_embedded_in_prose() {
    let text =
        "Sure! Here's the JSON: {\"name\": \"Alice\", \"nested\": {\"x\": 1}} -- hope that helps.";
    let v = FlexValue::from_llm_response(text).unwrap();
    assert_eq!(v.extract::<String>("name").unwrap(), "Alice");
    assert_eq!(v.extract::<i64>("nested.x").unwrap(), 1);
}

#[test]
fn ignores_braces_inside_strings() {
    let text = "prefix {\"msg\": \"a } brace and ] bracket inside\"} suffix";
    let v = FlexValue::from_llm_response(text).unwrap();
    assert_eq!(
        v.extract::<String>("msg").unwrap(),
        "a } brace and ] bracket inside"
    );
}

#[test]
fn parses_array_payload() {
    let v = FlexValue::from_llm_response("result: [1, 2, 3]").unwrap();
    assert!(v.is_array());
}

#[test]
fn clean_json_still_parses() {
    let v = FlexValue::from_llm_response(r#"{"a": 1}"#).unwrap();
    assert_eq!(v.extract::<i64>("a").unwrap(), 1);
}

#[test]
fn from_json_remains_strict() {
    // The strict entry point must NOT accept fenced/prose input.
    assert!(FlexValue::from_json("```json\n{\"a\":1}\n```").is_err());
    assert!(FlexValue::from_json("here: {\"a\":1}").is_err());
}
