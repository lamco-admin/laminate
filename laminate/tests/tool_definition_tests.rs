#![allow(dead_code, unused_imports, unused_must_use)]
/// ToolDefinition derive macro tests — generates JSON tool schemas for LLM APIs.
use laminate::ToolDefinition;

/// Get the current weather for a location
#[derive(ToolDefinition)]
struct GetWeather {
    /// The city to look up weather for
    city: String,
    /// Temperature units
    units: Option<String>,
}

#[test]
fn basic_tool_definition() {
    let def = GetWeather::tool_definition();
    println!("{}", serde_json::to_string_pretty(&def).unwrap());

    assert_eq!(def["name"], "get_weather");
    assert_eq!(def["description"], "Get the current weather for a location");
    assert_eq!(def["input_schema"]["type"], "object");

    // city is required (not Option)
    let required = def["input_schema"]["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("city")));
    assert!(!required.contains(&serde_json::json!("units")));

    // Properties have correct types
    assert_eq!(def["input_schema"]["properties"]["city"]["type"], "string");
    assert_eq!(def["input_schema"]["properties"]["units"]["type"], "string");

    // Doc comments become descriptions
    assert_eq!(
        def["input_schema"]["properties"]["city"]["description"],
        "The city to look up weather for"
    );
}

/// Search for documents matching a query
#[derive(ToolDefinition)]
struct SearchDocs {
    /// The search query string
    query: String,
    /// Maximum number of results to return
    max_results: i32,
    /// Whether to include archived documents
    include_archived: bool,
}

#[test]
fn all_required_fields() {
    let def = SearchDocs::tool_definition();
    let required = def["input_schema"]["required"].as_array().unwrap();
    assert_eq!(required.len(), 3);
    assert_eq!(
        def["input_schema"]["properties"]["max_results"]["type"],
        "integer"
    );
    assert_eq!(
        def["input_schema"]["properties"]["include_archived"]["type"],
        "boolean"
    );
}

#[derive(ToolDefinition)]
#[tool(name = "custom_name", description = "A custom description")]
struct CustomNamed {
    value: f64,
}

#[test]
fn custom_name_and_description() {
    let def = CustomNamed::tool_definition();
    assert_eq!(def["name"], "custom_name");
    assert_eq!(def["description"], "A custom description");
}
