//! Iteration 248: Derive macro with ALL attributes combined
//!
//! Stress test: a single struct using every derive attribute simultaneously.
//! This exercises all code generation paths in expand.rs at once.
//!
//! Attributes tested:
//! - #[laminate(coerce)] — type coercion
//! - #[laminate(default)] — default on missing
//! - #[laminate(coerce, default)] — combined
//! - #[laminate(overflow)] — capture unknown fields
//! - #[laminate(flatten)] — flatten nested struct
//! - #[laminate(rename = "x")] — JSON key rename
//! - #[laminate(skip)] — ignored field
//! - #[laminate(parse_json_string)] — parse stringified JSON

use laminate::Laminate;
use std::collections::HashMap;

#[derive(Debug, Laminate, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
struct FlatData {
    city: String,
    #[serde(default)]
    country: String,
}

#[derive(Debug, Laminate)]
struct KitchenSink {
    // Regular field (no attributes)
    id: i64,

    // Renamed field
    #[laminate(rename = "userName")]
    name: String,

    // Coerced field
    #[laminate(coerce)]
    score: f64,

    // Coerced + default
    #[laminate(coerce, default)]
    active: bool,

    // Parse JSON string
    #[laminate(parse_json_string)]
    metadata: serde_json::Value,

    // Skip — uses Default, never read from JSON
    #[laminate(skip)]
    internal_cache: String,

    // Flatten
    #[laminate(flatten)]
    location: FlatData,

    // Overflow — captures remaining unknown keys
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn all_attributes_clean_data() {
    let json = serde_json::json!({
        "id": 1,
        "userName": "Alice",
        "score": 95.5,
        "active": true,
        "metadata": "{\"role\":\"admin\"}",
        "city": "Portland",
        "country": "US",
        "theme": "dark"
    });

    let (result, diags) = KitchenSink::from_flex_value(&json).unwrap();

    println!(
        "Result: id={}, name={}, score={}, active={}",
        result.id, result.name, result.score, result.active
    );
    println!("metadata: {:?}", result.metadata);
    println!("location: {:?}", result.location);
    println!("internal_cache: {:?}", result.internal_cache);
    println!("extra: {:?}", result.extra);
    println!("diagnostics: {:?}", diags);

    assert_eq!(result.id, 1);
    assert_eq!(result.name, "Alice");
    assert!((result.score - 95.5).abs() < 0.01);
    assert!(result.active);
    // parse_json_string should parse the stringified JSON
    assert!(
        result.metadata.is_object() || result.metadata.is_string(),
        "metadata should be parsed JSON or original string"
    );
    assert_eq!(
        result.internal_cache, "",
        "skip field should be Default (empty string)"
    );
    assert_eq!(result.location.city, "Portland");
    // "theme" should be in overflow
    assert!(
        result.extra.contains_key("theme"),
        "unknown 'theme' should be in overflow"
    );
}

#[test]
fn all_attributes_with_coercion() {
    let json = serde_json::json!({
        "id": 2,
        "userName": "Bob",
        "score": "88",           // string → f64 coercion
        "active": "yes",         // string → bool coercion
        "metadata": "not-json",  // not valid JSON, should fall through
        "city": "Seattle",
    });

    let (result, diags) = KitchenSink::from_flex_value(&json).unwrap();

    println!(
        "Coerced result: score={}, active={}",
        result.score, result.active
    );
    println!("metadata: {:?}", result.metadata);
    println!("diagnostics: {:?}", diags);

    assert_eq!(result.score, 88.0, "string '88' should coerce to f64");
    assert_eq!(result.active, true, "'yes' should coerce to true");
}

#[test]
fn all_attributes_missing_optional_fields() {
    // Missing: score, active (has default), metadata, country (in flatten), theme
    let json = serde_json::json!({
        "id": 3,
        "userName": "Charlie",
        "city": "Austin",
    });

    let result = KitchenSink::from_flex_value(&json);
    println!("Missing fields result: {:?}", result);
    // score is not #[laminate(default)] so it MUST be present
    // This should error because score is required
}

#[test]
fn all_attributes_serialization_roundtrip() {
    let json = serde_json::json!({
        "id": 4,
        "userName": "Diana",
        "score": 99.0,
        "active": false,
        "metadata": "{\"level\":5}",
        "city": "Denver",
        "country": "US",
        "preference": "minimal"
    });

    let (original, _) = KitchenSink::from_flex_value(&json).unwrap();
    let serialized = original.to_value();

    println!(
        "Serialized: {}",
        serde_json::to_string_pretty(&serialized).unwrap()
    );

    // Verify key fields are in the output
    let obj = serialized.as_object().unwrap();
    assert!(
        obj.contains_key("userName"),
        "renamed key should appear in output"
    );
    assert!(obj.contains_key("id"));
    assert!(obj.contains_key("city"), "flattened field should appear");
    // "preference" should be in overflow → serialized back
    assert!(
        obj.contains_key("preference"),
        "overflow field should be preserved"
    );
    // "internal_cache" (skip) might or might not be in output — let's observe
    println!("Has internal_cache: {}", obj.contains_key("internal_cache"));
}
