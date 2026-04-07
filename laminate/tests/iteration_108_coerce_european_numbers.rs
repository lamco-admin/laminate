/// Iteration 108: Coercion system — European number format support
///
/// BestEffort coercion now handles European format:
/// "1.234,56" → 1234.56 (dot=thousands, comma=decimal)
/// "1.234.567" → 1234567 (multiple dots=thousands)
use laminate::coerce::CoercionLevel;
use laminate::FlexValue;

#[test]
fn european_comma_decimal_to_f64() {
    let val =
        FlexValue::from(serde_json::json!("1.234,56")).with_coercion(CoercionLevel::BestEffort);
    let result: f64 = val.extract_root().unwrap();
    assert!(
        (result - 1234.56).abs() < 0.01,
        "expected 1234.56, got {}",
        result
    );
}

#[test]
fn european_multiple_dot_thousands_to_i64() {
    let val =
        FlexValue::from(serde_json::json!("1.234.567")).with_coercion(CoercionLevel::BestEffort);
    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 1234567);
}

#[test]
fn us_comma_thousands_still_works() {
    let val =
        FlexValue::from(serde_json::json!("1,234.56")).with_coercion(CoercionLevel::BestEffort);
    let result: f64 = val.extract_root().unwrap();
    assert!((result - 1234.56).abs() < 0.01);
}

#[test]
fn european_not_applied_in_string_coercion() {
    // StringCoercion level should only do basic string→number, not European parsing
    let val =
        FlexValue::from(serde_json::json!("1.234,56")).with_coercion(CoercionLevel::StringCoercion);
    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_err(),
        "European parsing should require BestEffort"
    );
}
