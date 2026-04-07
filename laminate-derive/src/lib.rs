//! Derive macros for the laminate data shaping library.
//!
//! Provides `#[derive(Laminate)]` for progressive deserialization and
//! `#[derive(ToolDefinition)]` for JSON schema generation from Rust structs.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod tool_def;

/// Derive macro for progressive data shaping.
///
/// Generates deserialization logic that operates on an intermediate
/// `HashMap<String, serde_json::Value>`, enabling coercion, overflow capture,
/// and mode-dependent behavior.
///
/// # Attributes
///
/// - `#[laminate(overflow)]` — Captures unrecognized fields. Field must be `HashMap<String, serde_json::Value>`.
/// - `#[laminate(rename = "x")]` — Deserialize from a different JSON key.
/// - `#[laminate(default)]` — Use `Default::default()` if missing or null.
/// - `#[laminate(coerce)]` — Apply type coercion rules from the coercion table.
///
/// # Example
///
/// ```ignore
/// use laminate::Laminate;
///
/// #[derive(Debug, Laminate)]
/// pub struct Config {
///     #[laminate(coerce)]
///     pub port: u16,
///
///     #[laminate(default)]
///     pub debug: bool,
///
///     #[laminate(overflow)]
///     pub extra: HashMap<String, serde_json::Value>,
/// }
/// ```
#[proc_macro_derive(Laminate, attributes(laminate))]
pub fn derive_laminate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::expand_laminate(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive macro for generating LLM tool definition JSON schemas.
///
/// Generates a `tool_definition()` method that returns a `serde_json::Value`
/// matching the tool definition format expected by Anthropic, OpenAI, and
/// other LLM APIs.
///
/// # Attributes
///
/// - `#[tool(name = "x")]` — Override the tool name (defaults to snake_case struct name)
/// - `#[tool(description = "x")]` — Tool description sent to the LLM
/// - `#[tool(rename = "x")]` — Use a different parameter name in the schema
///
/// # Example
///
/// ```ignore
/// use laminate::ToolDefinition;
///
/// /// Get current weather for a city
/// #[derive(ToolDefinition)]
/// struct GetWeather {
///     /// The city to look up
///     city: String,
///     /// Temperature units (celsius or fahrenheit)
///     units: Option<String>,
/// }
///
/// let schema = GetWeather::tool_definition();
/// // Returns: {"name": "get_weather", "description": "Get current weather for a city",
/// //   "input_schema": {"type": "object", "properties": {"city": {"type": "string",
/// //   "description": "The city to look up"}, ...}, "required": ["city"]}}
/// ```
#[proc_macro_derive(ToolDefinition, attributes(tool))]
pub fn derive_tool_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    tool_def::expand_tool_definition(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
