#![allow(dead_code)]
//! Iteration 208: is_coercible_value() on Object→String.
//!
//! Schema audit: Object observed, String expected. is_coercible_value returns false.
//! But BestEffort coercion CAN JSON-serialize objects to strings.
//! Is this an intentional gap between audit and coercion? Or a bug?

use laminate::schema::{InferredSchema, JsonType};
use laminate::CoercionLevel;
use laminate::FlexValue;
use serde_json::json;

#[test]
fn audit_classifies_object_to_string_as_violation() {
    // Train on strings
    let rows = vec![json!({"data": "hello"}), json!({"data": "world"})];
    let schema = InferredSchema::from_values(&rows);
    assert_eq!(
        schema.fields.get("data").unwrap().dominant_type,
        Some(JsonType::String)
    );

    // Audit with an object value
    let test = vec![json!({"data": {"nested": true}})];
    let report = schema.audit(&test);
    println!("Violations: {:?}", report.violations);
    println!("Field stats: {:?}", report.field_stats);

    // is_coercible_value(Object, String) returns false — should be TypeMismatch
    let violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "data")
        .collect();
    assert!(
        !violations.is_empty(),
        "Object→String should be flagged as TypeMismatch in audit"
    );
}

#[test]
fn coercion_engine_can_serialize_object_to_string() {
    // BestEffort coercion: can it JSON-serialize an object to a string?
    let fv = FlexValue::new(json!({"nested": true})).with_coercion(CoercionLevel::BestEffort);
    let result: Result<String, _> = fv.extract_root();
    println!("BestEffort extract::<String> on object: {:?}", result);

    // Observe: does BestEffort coercion produce a JSON string representation?
    // This would reveal a gap between audit (says not coercible) and runtime (coerces fine)
}

#[test]
fn exact_coercion_rejects_object_to_string() {
    // FlexValue::new() defaults to BestEffort. Use explicit Exact.
    let fv = FlexValue::new(json!({"nested": true})).with_coercion(CoercionLevel::Exact);
    let result: Result<String, _> = fv.extract_root();
    println!("Exact extract::<String> on object: {:?}", result);

    // At Exact, Object→String coercion should NOT fire
    assert!(result.is_err(), "Exact should reject Object→String");
}

#[test]
fn audit_and_best_effort_intentionally_differ() {
    // Audit says Object→String is NOT coercible (TypeMismatch)
    // BestEffort CAN do it via JSON serialization
    // This is intentional: audit is conservative, reports it as a real issue
    let fv = FlexValue::new(json!({"nested": true})).with_coercion(CoercionLevel::BestEffort);
    let result: Result<String, _> = fv.extract_root();
    assert!(
        result.is_ok(),
        "BestEffort should serialize object to JSON string"
    );
    assert_eq!(result.unwrap(), r#"{"nested":true}"#);
    // This gap is by design: audit flags it as a violation because
    // Object→String is a drastic, lossy conversion even though it's technically possible
}
