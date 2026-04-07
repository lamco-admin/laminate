//! Iteration 256: SourceHint behavior for all 6 variants
//!
//! Tests that each SourceHint correctly adjusts coercion level and pack coercion.
//! - Csv, Env, FormData → BestEffort + PackCoercion::All
//! - Json, Database, Unknown → no change
//!
//! Adversarial: explicit coercion level should NOT be overridden by hint.

use laminate::value::{PackCoercion, SourceHint};
use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn csv_hint_enables_string_coercion() {
    let val = FlexValue::from(serde_json::json!({"age": "30"})).with_source_hint(SourceHint::Csv);

    let age: i64 = val.extract("age").unwrap();
    assert_eq!(age, 30, "CSV hint should enable string→int coercion");
}

#[test]
fn env_hint_enables_string_coercion() {
    let val =
        FlexValue::from(serde_json::json!({"port": "8080"})).with_source_hint(SourceHint::Env);

    let port: i64 = val.extract("port").unwrap();
    assert_eq!(port, 8080, "Env hint should enable string→int coercion");
}

#[test]
fn formdata_hint_enables_string_coercion() {
    let val =
        FlexValue::from(serde_json::json!({"count": "5"})).with_source_hint(SourceHint::FormData);

    let count: i64 = val.extract("count").unwrap();
    assert_eq!(count, 5, "FormData hint should enable string→int coercion");
}

#[test]
fn json_hint_preserves_default_besteffort() {
    // Default coercion is BestEffort. JSON hint does NOT change it.
    // So string→int still works because BestEffort allows it.
    let val = FlexValue::from(serde_json::json!({"age": "30"})).with_source_hint(SourceHint::Json);

    let result: i64 = val.extract("age").unwrap();
    assert_eq!(result, 30, "JSON hint preserves default BestEffort");
}

#[test]
fn database_hint_preserves_default_besteffort() {
    let val =
        FlexValue::from(serde_json::json!({"age": "30"})).with_source_hint(SourceHint::Database);

    let result: i64 = val.extract("age").unwrap();
    assert_eq!(result, 30, "Database hint preserves default BestEffort");
}

#[test]
fn json_hint_with_explicit_exact_stays_exact() {
    // Explicit Exact + JSON hint → should stay Exact (no coercion)
    let val = FlexValue::from(serde_json::json!({"age": "30"}))
        .with_coercion(CoercionLevel::Exact)
        .with_source_hint(SourceHint::Json);

    let result: Result<i64, _> = val.extract("age");
    assert!(result.is_err(), "Exact + JSON hint should stay Exact");
}

#[test]
fn json_hint_does_not_enable_packs() {
    // JSON hint should NOT enable pack coercion
    let val =
        FlexValue::from(serde_json::json!({"price": "$12.99"})).with_source_hint(SourceHint::Json);

    let result: Result<f64, _> = val.extract("price");
    // Default PackCoercion is None, Json hint doesn't change it
    assert!(result.is_err(), "JSON hint should NOT enable pack coercion");
}

#[test]
fn explicit_coercion_overrides_csv_hint() {
    // Explicit Exact + CSV hint → Exact should win
    let val = FlexValue::from(serde_json::json!({"age": "30"}))
        .with_coercion(CoercionLevel::Exact)
        .with_source_hint(SourceHint::Csv);

    let result: Result<i64, _> = val.extract("age");
    assert!(result.is_err(), "explicit Exact should override CSV hint");
}

#[test]
fn csv_hint_enables_pack_coercion() {
    // CSV hint should enable PackCoercion::All
    let val =
        FlexValue::from(serde_json::json!({"price": "$12.99"})).with_source_hint(SourceHint::Csv);

    let price: f64 = val.extract("price").unwrap();
    assert!(
        (price - 12.99).abs() < 0.01,
        "CSV hint should enable pack coercion"
    );
}

#[test]
fn explicit_pack_none_overrides_csv_hint() {
    // Explicit PackCoercion::None + CSV hint → None should win
    let val = FlexValue::from(serde_json::json!({"price": "$12.99"}))
        .with_pack_coercion(PackCoercion::None)
        .with_source_hint(SourceHint::Csv);

    let result: Result<f64, _> = val.extract("price");
    assert!(
        result.is_err(),
        "explicit None pack coercion should override CSV hint"
    );
}

#[test]
fn formdata_enables_packs() {
    let val = FlexValue::from(serde_json::json!({"weight": "5 kg"}))
        .with_source_hint(SourceHint::FormData);

    let weight: f64 = val.extract("weight").unwrap();
    assert!(
        (weight - 5.0).abs() < 0.01,
        "FormData hint should enable pack coercion"
    );
}
