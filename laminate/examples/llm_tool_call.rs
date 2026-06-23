//! End-to-end: parse a real (and slightly drifting) LLM tool-call response, then shape
//! the arguments into a typed Rust struct with a record of every adjustment.
//!
//! The same extraction and shaping code runs against an OpenAI-compatible response and
//! an Anthropic response, even though the two put the arguments in different shapes.
//!
//! Run: cargo run -p laminate --example llm_tool_call

use std::collections::HashMap;

use laminate::provider::anthropic::parse_anthropic_response;
use laminate::provider::openai::parse_openai_response;
use laminate::{FlexValue, Laminate};

/// The arguments your tool actually takes.
#[derive(Debug, Laminate)]
#[allow(dead_code)] // fields are shown via Debug; this is a demonstration
struct WeatherArgs {
    city: String,
    #[laminate(coerce)]
    days: u8, // the model sometimes sends "3"
    #[laminate(default)]
    units: String, // sometimes absent
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>, // anything the provider added
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // An OpenAI-compatible response from a gateway or local model: an extra envelope
    // field, a non-standard finish_reason, stringified arguments, a numeric argument
    // sent as a string, and an argument the struct does not name.
    let openai_body = FlexValue::from_json(
        r#"{
            "id": "chatcmpl-x",
            "model": "some-openai-compatible-model",
            "x_provider_note": "openai-compatible gateway",
            "choices": [{
                "message": {
                    "tool_calls": [{
                        "id": "call_1", "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"London\",\"days\":\"3\",\"region\":\"EU\"}"
                        }
                    }]
                },
                "finish_reason": "eos_token"
            }]
        }"#,
    )?;

    // The same call from Anthropic: arguments as an object, under different keys.
    let anthropic_body = FlexValue::from_json(
        r#"{
            "id": "msg_1",
            "model": "claude-sonnet-4-6",
            "content": [{
                "type": "tool_use",
                "id": "tu_1",
                "name": "get_weather",
                "input": {"city": "London", "days": "3", "region": "EU"}
            }],
            "stop_reason": "tool_use"
        }"#,
    )?;

    let responses = [
        ("openai", parse_openai_response(&openai_body)?),
        ("anthropic", parse_anthropic_response(&anthropic_body)?),
    ];

    for (label, resp) in &responses {
        println!("== {label} (stop_reason = {}) ==", resp.stop_reason);
        for block in &resp.content {
            if let Some((_id, name, input)) = block.as_tool_use() {
                // The same shaping call, whichever provider answered.
                let (args, diags) = WeatherArgs::from_flex_value(input.raw())?;
                println!("  tool: {name}");
                println!("  args: {args:?}");
                for d in &diags {
                    println!("    {d}");
                }
            }
        }
        println!();
    }

    Ok(())
}
