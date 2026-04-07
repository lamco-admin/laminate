/// Iteration 111: Apostrophe and space thousands in core coercion
///
/// Swiss format (apostrophe: "1'234'567") and French/SI format
/// (space: "1 234 567") are now handled by try_strip_alt_thousands.
use laminate::coerce::CoercionLevel;
use laminate::FlexValue;

#[test]
fn apostrophe_thousands_to_i64() {
    let val =
        FlexValue::from(serde_json::json!("1'234'567")).with_coercion(CoercionLevel::BestEffort);
    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 1234567);
}

#[test]
fn apostrophe_thousands_with_decimal_to_f64() {
    let val =
        FlexValue::from(serde_json::json!("1'234.56")).with_coercion(CoercionLevel::BestEffort);
    let result: f64 = val.extract_root().unwrap();
    assert!((result - 1234.56).abs() < 0.01);
}

#[test]
fn space_thousands_to_i64() {
    let val =
        FlexValue::from(serde_json::json!("1 234 567")).with_coercion(CoercionLevel::BestEffort);
    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 1234567);
}

#[test]
fn plain_number_unaffected() {
    let val = FlexValue::from(serde_json::json!("42")).with_coercion(CoercionLevel::BestEffort);
    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 42);
}
