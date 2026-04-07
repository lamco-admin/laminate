//! Iteration 240: E2E Roundtrip — derive struct → to_json() → from_json() → preservation
//!
//! Tests whether coerced values survive a round-trip through serialization
//! and re-deserialization. Also tests overflow field preservation.
//!
//! Adversarial angles:
//! - Coerced field: "30" (string) → 30 (i64) → serialized as 30 → re-extracted as 30
//! - Overflow field: unknown key "extra" → preserved in HashMap → re-serialized
//! - Renamed field: #[laminate(rename)] → serialized with rename key
//! - Default field: missing in input → default value → serialized → present in output

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct RoundTrip {
    name: String,
    #[laminate(coerce)]
    age: i64,
    #[laminate(coerce, default)]
    active: bool,
    #[laminate(overflow)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[test]
fn coerced_values_survive_roundtrip() {
    // Input has coercible values
    let json = serde_json::json!({
        "name": "Alice",
        "age": "30",        // string → will be coerced to i64
        "active": "yes",    // string → will be coerced to bool
    });

    let (original, diags) = RoundTrip::from_flex_value(&json).unwrap();
    println!("Original: {:?}, diags: {:?}", original, diags);

    assert_eq!(original.age, 30);
    assert!(original.active);

    // Round-trip: serialize and re-deserialize
    let serialized = original.to_value();
    println!(
        "Serialized: {}",
        serde_json::to_string_pretty(&serialized).unwrap()
    );

    let (restored, diags2) = RoundTrip::from_flex_value(&serialized).unwrap();
    println!("Restored: {:?}, diags: {:?}", restored, diags2);

    assert_eq!(original.name, restored.name);
    assert_eq!(original.age, restored.age);
    assert_eq!(original.active, restored.active);

    // Second round-trip should have no coercion diagnostics (types now match)
    let coercion_diags: Vec<_> = diags2
        .iter()
        .filter(|d| matches!(d.kind, laminate::DiagnosticKind::Coerced { .. }))
        .collect();
    assert!(
        coercion_diags.is_empty(),
        "round-tripped data should not need coercion, got {:?}",
        coercion_diags
    );
}

#[test]
fn overflow_preserved_through_roundtrip() {
    let json = serde_json::json!({
        "name": "Bob",
        "age": 25,
        "active": true,
        "role": "admin",
        "department": "engineering"
    });

    let (original, _) = RoundTrip::from_flex_value(&json).unwrap();
    println!("Overflow: {:?}", original.extra);

    // Overflow should contain the unknown fields
    assert!(
        original.extra.contains_key("role"),
        "overflow should capture 'role'"
    );
    assert!(
        original.extra.contains_key("department"),
        "overflow should capture 'department'"
    );

    // Serialize and check the unknown fields are in the output
    let serialized = original.to_value();
    let obj = serialized.as_object().unwrap();
    println!("Serialized keys: {:?}", obj.keys().collect::<Vec<_>>());

    assert_eq!(
        obj.get("role").and_then(|v| v.as_str()),
        Some("admin"),
        "overflow field 'role' should be preserved in serialized output"
    );
    assert_eq!(
        obj.get("department").and_then(|v| v.as_str()),
        Some("engineering"),
        "overflow field 'department' should be preserved in serialized output"
    );

    // Full round-trip
    let (restored, _) = RoundTrip::from_flex_value(&serialized).unwrap();
    assert_eq!(
        original.extra, restored.extra,
        "overflow should survive round-trip"
    );
}

#[test]
fn default_field_serialized_in_output() {
    // Missing "active" field → default (false) → serialized → present in output
    let json = serde_json::json!({"name": "Charlie", "age": 35});

    let (original, _) = RoundTrip::from_flex_value(&json).unwrap();
    assert_eq!(original.active, false, "default should be false");

    let serialized = original.to_value();
    let obj = serialized.as_object().unwrap();

    println!(
        "Serialized: {}",
        serde_json::to_string_pretty(&serialized).unwrap()
    );

    // "active" should be in the output as false
    assert_eq!(
        obj.get("active").and_then(|v| v.as_bool()),
        Some(false),
        "default field should appear in serialized output"
    );
}

#[test]
fn empty_overflow_no_extra_keys() {
    // No unknown fields → overflow is empty HashMap → no extra keys in output
    let json = serde_json::json!({"name": "Diana", "age": 28, "active": true});

    let (original, _) = RoundTrip::from_flex_value(&json).unwrap();
    assert!(original.extra.is_empty());

    let serialized = original.to_value();
    let obj = serialized.as_object().unwrap();

    // Should have exactly 3 known fields + 0 overflow
    let keys: Vec<_> = obj.keys().collect();
    println!("Keys: {:?}", keys);
    assert_eq!(
        keys.len(),
        3,
        "should have exactly 3 fields, got {:?}",
        keys
    );
}
