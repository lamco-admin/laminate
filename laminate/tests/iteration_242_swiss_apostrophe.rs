//! Iteration 242: Swiss apostrophe "1'234.56" at BestEffort
//!
//! Swiss number format uses apostrophe as thousands separator: "1'234.56"
//! The coercion engine has try_strip_alt_thousands() which handles apostrophe
//! and space separators. Let's see if it works for:
//! - Integer target: "1'234" → 1234
//! - Float target: "1'234.56" → 1234.56
//! - Negative: "-1'234.56" → -1234.56
//! - Mixed with decimal: "1'234'567.89" → 1234567.89

use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn swiss_apostrophe_integer() {
    let val = FlexValue::from(serde_json::json!("1'234")).with_coercion(CoercionLevel::BestEffort);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 1234, "Swiss '1'234' should parse to 1234");
}

#[test]
fn swiss_apostrophe_float() {
    let val =
        FlexValue::from(serde_json::json!("1'234.56")).with_coercion(CoercionLevel::BestEffort);

    let result: f64 = val.extract_root().unwrap();
    println!("1'234.56 → {}", result);
    assert!(
        (result - 1234.56).abs() < 0.001,
        "Swiss '1'234.56' should parse to 1234.56"
    );
}

#[test]
fn swiss_apostrophe_large_number() {
    let val =
        FlexValue::from(serde_json::json!("1'234'567")).with_coercion(CoercionLevel::BestEffort);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 1234567, "Swiss '1'234'567' should parse to 1234567");
}

#[test]
fn swiss_apostrophe_large_float() {
    let val =
        FlexValue::from(serde_json::json!("1'234'567.89")).with_coercion(CoercionLevel::BestEffort);

    let result: f64 = val.extract_root().unwrap();
    assert!((result - 1234567.89).abs() < 0.01);
}

#[test]
fn swiss_apostrophe_negative() {
    let val =
        FlexValue::from(serde_json::json!("-1'234.56")).with_coercion(CoercionLevel::BestEffort);

    let result: f64 = val.extract_root().unwrap();
    println!("-1'234.56 → {}", result);
    assert!((result - (-1234.56)).abs() < 0.001);
}

#[test]
fn swiss_apostrophe_at_string_coercion_level() {
    // Apostrophe thousands should only fire at BestEffort, not StringCoercion
    let val =
        FlexValue::from(serde_json::json!("1'234")).with_coercion(CoercionLevel::StringCoercion);

    let result: Result<i64, _> = val.extract_root();
    println!("1'234 at StringCoercion: {:?}", result);
    // StringCoercion only does direct parse — "1'234" isn't a valid integer string
    assert!(
        result.is_err(),
        "apostrophe thousands should NOT parse at StringCoercion"
    );
}

#[test]
fn french_space_thousands() {
    // French/SI format uses space: "1 234 567"
    let val =
        FlexValue::from(serde_json::json!("1 234 567")).with_coercion(CoercionLevel::BestEffort);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(
        result, 1234567,
        "French space thousands '1 234 567' should parse"
    );
}

#[test]
fn invalid_apostrophe_grouping_rejected() {
    // Invalid grouping: "12'34" is not valid thousands (groups after first must be 3 digits)
    let val = FlexValue::from(serde_json::json!("12'34")).with_coercion(CoercionLevel::BestEffort);

    let result: Result<i64, _> = val.extract_root();
    println!("12'34 (invalid grouping): {:?}", result);
    // Should be rejected because "34" is only 2 digits, not 3
    assert!(
        result.is_err(),
        "invalid grouping '12'34' should be rejected"
    );
}
