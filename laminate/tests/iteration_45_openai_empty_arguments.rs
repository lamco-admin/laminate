use laminate::provider::openai::{emit_openai_response, parse_openai_response};
use laminate::FlexValue;

/// Iteration 45 — Provider Probe: OpenAI emit with empty/edge-case tool arguments.
/// Result: GAP — empty string arguments were double-stringified during emit.
/// Fix: detect string-valued input and pass through directly.

#[test]
fn iter45_empty_arguments_round_trip() {
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-empty-args",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "no_args_tool", "arguments": "{}"}
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }"#,
    )
    .unwrap();

    let resp = parse_openai_response(&raw).unwrap();
    let (_, name, input) = resp.content[0].as_tool_use().unwrap();
    assert_eq!(name, "no_args_tool");
    assert!(input.is_object());

    // Emit and verify arguments is "{}"
    let emitted = emit_openai_response(&resp);
    let args = emitted["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .unwrap();
    assert_eq!(args, "{}");

    // Full round-trip: re-parse the emitted response
    let re_raw = FlexValue::from_json(&serde_json::to_string(&emitted).unwrap()).unwrap();
    let re_resp = parse_openai_response(&re_raw).unwrap();
    let (_, _, re_input) = re_resp.content[0].as_tool_use().unwrap();
    assert!(re_input.is_object());
}

#[test]
fn iter45_empty_string_arguments_no_double_stringify() {
    // OpenAI sends "" during streaming while building arguments
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-streaming",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_2",
                    "type": "function",
                    "function": {"name": "building_tool", "arguments": ""}
                }]
            },
            "finish_reason": null
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 1}
    }"#,
    )
    .unwrap();

    let resp = parse_openai_response(&raw).unwrap();
    let (_, _, input) = resp.content[0].as_tool_use().unwrap();
    // Unparseable "" kept as string
    assert!(input.is_string());

    // After fix: emit should produce "" not "\"\""
    let emitted = emit_openai_response(&resp);
    let args = emitted["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .unwrap();
    assert_eq!(args, "", "empty string should not be double-stringified");
}

#[test]
fn iter45_pre_parsed_object_arguments() {
    // Some callers pre-parse arguments before passing to laminate
    let raw = FlexValue::from_json(
        r#"{
        "id": "chatcmpl-preparsed",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_3",
                    "type": "function",
                    "function": {"name": "search", "arguments": {"query": "rust", "limit": 10}}
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }"#,
    )
    .unwrap();

    let resp = parse_openai_response(&raw).unwrap();
    let (_, name, input) = resp.content[0].as_tool_use().unwrap();
    assert_eq!(name, "search");
    assert!(input.is_object());
    let query: String = input.extract("query").unwrap();
    assert_eq!(query, "rust");

    // Emit should stringify the pre-parsed object
    let emitted = emit_openai_response(&resp);
    let args = emitted["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .unwrap();
    // Re-parse the stringified arguments to verify
    let parsed: serde_json::Value = serde_json::from_str(args).unwrap();
    assert_eq!(parsed["query"], "rust");
    assert_eq!(parsed["limit"], 10);
}
