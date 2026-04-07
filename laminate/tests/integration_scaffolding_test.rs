/// Integration scaffolding test: simulate a user with Rig/SDK who
/// gets a raw serde_json::Value and uses laminate to process it.
use laminate::{FlexValue, Laminate, ToolDefinition};

// === Step 4: Tool definition for sending TO the LLM ===

/// Get current weather for a city
#[derive(Debug, Laminate, ToolDefinition)]
struct WeatherArgs {
    /// The city to look up
    #[laminate(coerce)]
    city: String,
    /// Temperature units (celsius or fahrenheit)
    #[laminate(coerce, default)]
    units: Option<String>,
}

#[test]
fn full_integration_openai_format() {
    // Step 1: User gets raw JSON from Rig/OpenAI SDK
    let raw_response = serde_json::json!({
        "id": "chatcmpl-abc",
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"city\": \"London\", \"units\": \"celsius\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 50, "completion_tokens": 20}
    });

    // Step 2: Wrap in FlexValue and navigate
    let fv = FlexValue::new(raw_response);
    let tool_name: String = fv
        .extract("choices[0].message.tool_calls[0].function.name")
        .unwrap();
    assert_eq!(tool_name, "get_weather");

    // Step 2b: Get the stringified arguments
    let args_raw = fv
        .at("choices[0].message.tool_calls[0].function.arguments")
        .unwrap();
    println!("args_raw is_string: {}", args_raw.is_string());

    // Step 2c: Parse the stringified JSON into a navigable FlexValue
    // The args are a String containing JSON — need to parse them
    let args_str: String = args_raw.extract_root().unwrap();
    let args_fv = FlexValue::from_json(&args_str).unwrap();
    let city: String = args_fv.extract("city").unwrap();
    assert_eq!(city, "London");

    // Step 3: Shape into typed struct
    let (weather_args, diags) = WeatherArgs::from_flex_value(args_fv.raw()).unwrap();
    assert_eq!(weather_args.city, "London");
    assert_eq!(weather_args.units, Some("celsius".to_string()));
    println!("diagnostics: {:?}", diags);

    // Step 4: Generate tool definition to send TO the LLM
    let tool_def = WeatherArgs::tool_definition();
    assert_eq!(tool_def["name"], "weather_args");
    assert_eq!(
        tool_def["input_schema"]["properties"]["city"]["type"],
        "string"
    );
    println!(
        "tool_def: {}",
        serde_json::to_string_pretty(&tool_def).unwrap()
    );
}

#[test]
fn full_integration_anthropic_format() {
    // Anthropic returns tool args as a JSON OBJECT (not stringified)
    let raw_response = serde_json::json!({
        "id": "msg_abc",
        "model": "claude-opus-4-6-20260301",
        "content": [
            {"type": "text", "text": "I'll check the weather."},
            {
                "type": "tool_use",
                "id": "toolu_123",
                "name": "get_weather",
                "input": {"city": "Paris", "units": "celsius"}
            }
        ],
        "stop_reason": "tool_use",
        "usage": {"input_tokens": 50, "output_tokens": 30}
    });

    let fv = FlexValue::new(raw_response);

    // Navigate to the tool input — it's already an object (no parsing needed)
    let city: String = fv.extract("content[1].input.city").unwrap();
    assert_eq!(city, "Paris");

    // Shape into struct
    let input_val = fv.at("content[1].input").unwrap();
    let (args, _) = WeatherArgs::from_flex_value(input_val.raw()).unwrap();
    assert_eq!(args.city, "Paris");
}

#[test]
fn transparent_stringified_json_navigation() {
    // at() now transparently crosses stringified-JSON boundaries
    let fv = FlexValue::from(serde_json::json!({
        "args": "{\"city\": \"London\", \"temp\": 22}"
    }));

    // Navigate INTO the stringified JSON — at() auto-parses the string
    let city: String = fv.extract("args.city").unwrap();
    assert_eq!(city, "London");

    let temp: i64 = fv.extract("args.temp").unwrap();
    assert_eq!(temp, 22);

    // Extracting as String still gives the raw string when no further navigation
    let raw: String = fv.extract("args").unwrap();
    assert_eq!(raw, "{\"city\": \"London\", \"temp\": 22}");
}

#[test]
fn openai_tool_args_one_step() {
    // The whole point: OpenAI stringified args accessible in one extract() call
    let raw = serde_json::json!({
        "choices": [{
            "message": {
                "tool_calls": [{
                    "function": {
                        "name": "search",
                        "arguments": "{\"query\": \"rust laminate\", \"limit\": 10}"
                    }
                }]
            }
        }]
    });

    let fv = FlexValue::new(raw);

    // One call — navigates through the stringified JSON boundary transparently
    let query: String = fv
        .extract("choices[0].message.tool_calls[0].function.arguments.query")
        .unwrap();
    assert_eq!(query, "rust laminate");

    let limit: i64 = fv
        .extract("choices[0].message.tool_calls[0].function.arguments.limit")
        .unwrap();
    assert_eq!(limit, 10);
}
