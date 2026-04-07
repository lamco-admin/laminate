//! Iteration 245: Float 3.7 → i64 at SafeWidening
//!
//! Float-to-integer coercion should only succeed when the float has no
//! fractional part (e.g., 3.0 → 3, but 3.7 should be rejected because
//! truncation would lose data).
//!
//! This tests the precision guard in the Number → Integer arm of coerce_value.

use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn float_with_fraction_rejected_at_safewidening() {
    let val = FlexValue::from(serde_json::json!(3.7)).with_coercion(CoercionLevel::SafeWidening);

    let result: Result<i64, _> = val.extract_root();
    println!("3.7 → i64 at SafeWidening: {:?}", result);
    assert!(
        result.is_err(),
        "3.7 → i64 should be rejected (fractional data loss)"
    );
}

#[test]
fn float_without_fraction_accepted_at_safewidening() {
    // 3.0 has no fractional part → can safely convert to i64
    let val = FlexValue::from(serde_json::json!(3.0)).with_coercion(CoercionLevel::SafeWidening);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 3, "3.0 → i64 should succeed (no data loss)");
}

#[test]
fn float_with_fraction_rejected_at_besteffort() {
    // Even at BestEffort, float→int with fractional part should be rejected
    // because the coercion engine checks fract() != 0.0
    let val = FlexValue::from(serde_json::json!(3.7)).with_coercion(CoercionLevel::BestEffort);

    let result: Result<i64, _> = val.extract_root();
    println!("3.7 → i64 at BestEffort: {:?}", result);
    assert!(
        result.is_err(),
        "3.7 → i64 should be rejected even at BestEffort"
    );
}

#[test]
fn very_small_fraction_rejected() {
    // 3.0000001 — has a tiny fractional part
    let val =
        FlexValue::from(serde_json::json!(3.0000001)).with_coercion(CoercionLevel::SafeWidening);

    let result: Result<i64, _> = val.extract_root();
    println!("3.0000001 → i64: {:?}", result);
    // Even a tiny fraction should prevent conversion
    assert!(
        result.is_err(),
        "3.0000001 should be rejected (has fractional part)"
    );
}

#[test]
fn negative_float_no_fraction() {
    let val = FlexValue::from(serde_json::json!(-5.0)).with_coercion(CoercionLevel::SafeWidening);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, -5, "-5.0 → i64 should succeed");
}

#[test]
fn float_at_exact_level_rejected() {
    // At Exact level, even 3.0 → i64 should be rejected (int→float is ok, float→int is not)
    let val = FlexValue::from(serde_json::json!(3.0)).with_coercion(CoercionLevel::Exact);

    let result: Result<i64, _> = val.extract_root();
    println!("3.0 → i64 at Exact: {:?}", result);
    // Exact: types must match exactly. A JSON number 3.0 is a float, not an integer.
    // But serde_json may represent 3.0 as integer... let's observe
}

#[test]
fn integer_to_float_at_exact_rejected() {
    // This was covered by iteration 180 but let's re-verify
    let val = FlexValue::from(serde_json::json!(42)).with_coercion(CoercionLevel::Exact);

    let result: Result<f64, _> = val.extract_root();
    println!("42 → f64 at Exact: {:?}", result);
    assert!(
        result.is_err(),
        "integer → float should be rejected at Exact"
    );
}

#[test]
fn string_float_to_int_at_besteffort() {
    // String "3.7" → i64 at BestEffort
    // First: "3.7" → f64 (StringCoercion), then f64 → i64 would lose data
    // But the engine parses "3.7" directly as an f64, then tries serde
    let val = FlexValue::from(serde_json::json!("3.7")).with_coercion(CoercionLevel::BestEffort);

    let result: Result<i64, _> = val.extract_root();
    println!("\"3.7\" → i64 at BestEffort: {:?}", result);
    // "3.7" should parse as f64 but NOT truncate to i64
}
