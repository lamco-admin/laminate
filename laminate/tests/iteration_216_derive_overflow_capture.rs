#![allow(dead_code)]
//! Iteration 216: #[laminate(overflow)] captures unknown fields.
//!
//! shape_lenient drops unknown fields with diagnostics.
//! #[laminate(overflow)] should capture them instead.
//! Question: does overflow preserve the original values? And what diagnostics?

use laminate::Laminate;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct UserProfile {
    name: String,
    age: i64,
    #[laminate(overflow)]
    extra: HashMap<String, Value>,
}

#[test]
fn overflow_captures_unknown_fields() {
    let json = json!({
        "name": "Alice",
        "age": 30,
        "role": "admin",
        "department": "engineering"
    });
    let result = UserProfile::from_flex_value(&json);
    println!("Result: {:?}", result);

    assert!(result.is_ok());
    let (profile, diags) = result.unwrap();
    assert_eq!(profile.name, "Alice");
    assert_eq!(profile.age, 30);
    assert_eq!(profile.extra.len(), 2);
    assert_eq!(profile.extra.get("role"), Some(&json!("admin")));
    assert_eq!(profile.extra.get("department"), Some(&json!("engineering")));

    println!("Diagnostics: {:?}", diags);
}

#[test]
fn no_overflow_empty_map() {
    let json = json!({"name": "Bob", "age": 25});
    let result = UserProfile::from_flex_value(&json);
    assert!(result.is_ok());
    let (profile, _) = result.unwrap();
    assert!(
        profile.extra.is_empty(),
        "No unknown fields → empty overflow"
    );
}

#[test]
fn overflow_preserves_complex_values() {
    let json = json!({
        "name": "Charlie",
        "age": 35,
        "metadata": {"nested": true, "count": 42},
        "tags": ["a", "b", "c"]
    });
    let result = UserProfile::from_flex_value(&json);
    assert!(result.is_ok());
    let (profile, _) = result.unwrap();

    assert_eq!(
        profile.extra.get("metadata"),
        Some(&json!({"nested": true, "count": 42}))
    );
    assert_eq!(profile.extra.get("tags"), Some(&json!(["a", "b", "c"])));
}

#[test]
fn overflow_roundtrip_via_to_value() {
    let json = json!({
        "name": "Dana",
        "age": 28,
        "custom_field": "preserved"
    });
    let result = UserProfile::from_flex_value(&json);
    assert!(result.is_ok());
    let (profile, _) = result.unwrap();

    // Round-trip: to_value should include overflow fields
    let output = profile.to_value();
    println!(
        "Round-trip output: {}",
        serde_json::to_string_pretty(&output).unwrap()
    );
    assert_eq!(
        output.get("custom_field"),
        Some(&json!("preserved")),
        "Overflow fields should survive round-trip"
    );
}
