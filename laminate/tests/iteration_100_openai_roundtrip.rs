#![cfg(feature = "providers")]
/// Iteration 100: OpenAI provider parse→emit→parse round-trip for tool calls
///
/// OpenAI stringifies tool arguments in the API response. The parse path
/// de-stringifies them, the emit path re-stringifies. Does a round-trip
/// preserve arguments with special characters, nested objects, and unicode?
use laminate::provider::openai::{emit_openai_response, parse_openai_response};
use laminate::FlexValue;

#[test]
fn openai_tool_call_roundtrip_complex_arguments() {
    let original_json = r#"{
        "id": "chatcmpl-rt100",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "I'll search for that.",
                "tool_calls": [{
                    "id": "call_abc",
                    "type": "function",
                    "function": {
                        "name": "search",
                        "arguments": "{\"query\":\"laminate \\\"quoted\\\"\",\"filters\":{\"lang\":\"en\",\"year\":2026},\"tags\":[\"rust\",\"serde\"],\"unicode\":\"café ☕\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 100, "completion_tokens": 50}
    }"#;

    // Parse original
    let body = FlexValue::from_json(original_json).unwrap();
    let parsed = parse_openai_response(&body).unwrap();

    // Verify arguments were parsed into navigable FlexValue
    let (_, name, input) = parsed.content[1].as_tool_use().unwrap();
    assert_eq!(name, "search");
    let query: String = input.extract("query").unwrap();
    assert_eq!(
        query, "laminate \"quoted\"",
        "escaped quotes should be preserved"
    );
    let lang: String = input.extract("filters.lang").unwrap();
    assert_eq!(lang, "en");
    let unicode: String = input.extract("unicode").unwrap();
    assert_eq!(unicode, "café ☕", "unicode should survive parse");

    // Emit back to OpenAI format
    let emitted = emit_openai_response(&parsed);
    println!(
        "emitted: {}",
        serde_json::to_string_pretty(&emitted).unwrap()
    );

    // The arguments should be re-stringified
    let emitted_args = emitted["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .expect("arguments should be a string");
    println!("emitted args string: {}", emitted_args);

    // Re-parse the emitted JSON
    let re_body = FlexValue::new(emitted);
    let re_parsed = parse_openai_response(&re_body).unwrap();

    // All arguments should survive the full round-trip
    let (_, re_name, re_input) = re_parsed.content[1].as_tool_use().unwrap();
    assert_eq!(re_name, "search");

    let re_query: String = re_input.extract("query").unwrap();
    assert_eq!(
        re_query, "laminate \"quoted\"",
        "query should survive round-trip"
    );

    let re_lang: String = re_input.extract("filters.lang").unwrap();
    assert_eq!(re_lang, "en", "nested object should survive round-trip");

    let re_year: i64 = re_input.extract("filters.year").unwrap();
    assert_eq!(re_year, 2026, "nested number should survive round-trip");

    let re_unicode: String = re_input.extract("unicode").unwrap();
    assert_eq!(re_unicode, "café ☕", "unicode should survive round-trip");

    // Text should also survive
    assert_eq!(re_parsed.text(), "I'll search for that.");
}
