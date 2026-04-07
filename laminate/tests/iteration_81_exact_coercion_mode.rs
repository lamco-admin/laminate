/// Iteration 81: FlexValue::with_coercion(Exact) — enforce strict numeric sub-type matching
///
/// In Exact coercion mode, serde_json silently widens integer→float (42 → 42.0).
/// This violates the documented "types must match exactly" contract. Laminate's
/// coercion layer now intercepts this case and rejects it in Exact mode.
use laminate::coerce::CoercionLevel;
use laminate::FlexValue;

#[test]
fn exact_mode_i64_from_integer() {
    // Number(42) → i64: exact match, should succeed
    let val = FlexValue::from(serde_json::json!(42)).with_coercion(CoercionLevel::Exact);
    let result: Result<i64, _> = val.extract_root();
    assert!(result.is_ok(), "integer → i64 should work in Exact mode");
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn exact_mode_rejects_integer_to_f64() {
    // Number(42) → f64: this is int→float widening, should be rejected in Exact mode
    let val = FlexValue::from(serde_json::json!(42)).with_coercion(CoercionLevel::Exact);
    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_err(),
        "integer → f64 should be rejected in Exact mode (requires SafeWidening)"
    );
}

#[test]
fn exact_mode_f64_from_float() {
    // Number(42.5) → f64: exact match, should succeed
    let val = FlexValue::from(serde_json::json!(42.5)).with_coercion(CoercionLevel::Exact);
    let result: Result<f64, _> = val.extract_root();
    assert!(result.is_ok(), "float → f64 should work in Exact mode");
    assert!((result.unwrap() - 42.5).abs() < f64::EPSILON);
}

#[test]
fn safe_widening_allows_integer_to_f64() {
    // Number(42) → f64 in SafeWidening: should succeed (this is the designed behavior)
    let val = FlexValue::from(serde_json::json!(42)).with_coercion(CoercionLevel::SafeWidening);
    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_ok(),
        "integer → f64 should succeed in SafeWidening mode"
    );
    assert!((result.unwrap() - 42.0).abs() < f64::EPSILON);
}
