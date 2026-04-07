//! Iteration 243: Null sentinel case sensitivity and chain behavior
//!
//! The is_null_sentinel() function uses to_lowercase() matching, so
//! "UNDEFINED", "Undefined", "undefined" should all match.
//!
//! Full chain at BestEffort: "undefined" → null-sentinel → Null → Default(0).
//!
//! Adversarial: What about edge cases like "NONE", "NULL", mixed case "nUlL"?
//! What about non-i64 targets like Option<i64>?

use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn null_sentinel_case_insensitive() {
    let cases = vec![
        "undefined",
        "UNDEFINED",
        "Undefined",
        "UnDeFiNeD",
        "null",
        "NULL",
        "Null",
        "nUlL",
        "none",
        "NONE",
        "None",
        "n/a",
        "N/A",
        "N/a",
        "na",
        "NA",
        "Na",
        "nil",
        "NIL",
        "Nil",
        "nan",
        "NaN",
        "NAN",
        "unknown",
        "UNKNOWN",
        "Unknown",
        "-",
    ];

    for sentinel in &cases {
        let val =
            FlexValue::from(serde_json::json!(sentinel)).with_coercion(CoercionLevel::BestEffort);

        let result: i64 = val.extract_root().unwrap();
        println!("{:?} → i64: {}", sentinel, result);
        // All null sentinels should chain to default: 0 for i64
        assert_eq!(result, 0, "'{}' should be null sentinel → 0", sentinel);
    }
}

#[test]
fn null_sentinel_option_i64_returns_none() {
    // For Option<i64>, null sentinel should produce None, not Some(0)
    let val = FlexValue::from(serde_json::json!("N/A")).with_coercion(CoercionLevel::BestEffort);

    let result: Option<i64> = val.extract_root().unwrap();
    println!("N/A as Option<i64>: {:?}", result);
    assert_eq!(
        result, None,
        "null sentinel on Option<i64> should produce None"
    );
}

#[test]
fn null_sentinel_string_target_not_triggered() {
    // When extracting as String, "N/A" should stay as "N/A" (no sentinel)
    let val = FlexValue::from(serde_json::json!("N/A")).with_coercion(CoercionLevel::BestEffort);

    let result: String = val.extract_root().unwrap();
    assert_eq!(
        result, "N/A",
        "String extraction should preserve null sentinel text"
    );
}

#[test]
fn null_sentinel_at_string_coercion_level() {
    // Null sentinels only fire at BestEffort, not StringCoercion
    let val =
        FlexValue::from(serde_json::json!("N/A")).with_coercion(CoercionLevel::StringCoercion);

    let result: Result<i64, _> = val.extract_root();
    println!("N/A at StringCoercion: {:?}", result);
    // StringCoercion tries to parse "N/A" as i64 → fails. No sentinel.
    assert!(
        result.is_err(),
        "null sentinel should NOT fire at StringCoercion"
    );
}

#[test]
fn null_sentinel_f64_default() {
    let val =
        FlexValue::from(serde_json::json!("unknown")).with_coercion(CoercionLevel::BestEffort);

    let result: f64 = val.extract_root().unwrap();
    assert_eq!(result, 0.0, "'unknown' → null → 0.0 for f64");
}

#[test]
fn null_sentinel_bool_default() {
    let val = FlexValue::from(serde_json::json!("null")).with_coercion(CoercionLevel::BestEffort);

    let result: bool = val.extract_root().unwrap();
    assert_eq!(result, false, "'null' → null → false for bool");
}

#[test]
fn dash_as_null_sentinel() {
    // Single dash "-" is a common CSV null indicator
    let val = FlexValue::from(serde_json::json!("-")).with_coercion(CoercionLevel::BestEffort);

    let result: i64 = val.extract_root().unwrap();
    assert_eq!(result, 0, "'-' should be null sentinel → 0 for i64");

    // But "--" (double dash) should NOT be a null sentinel
    let val2 = FlexValue::from(serde_json::json!("--")).with_coercion(CoercionLevel::BestEffort);

    let result2: Result<i64, _> = val2.extract_root();
    println!("'--' as i64: {:?}", result2);
    // "--" is not in the sentinel list, so it should fail
    assert!(result2.is_err(), "'--' should NOT be a null sentinel");
}

#[test]
fn empty_string_not_null_sentinel() {
    // Empty string "" is intentionally NOT a null sentinel (per iter 249)
    let val = FlexValue::from(serde_json::json!("")).with_coercion(CoercionLevel::BestEffort);

    let result: Result<i64, _> = val.extract_root();
    println!("'' as i64: {:?}", result);
    // Empty string should fail for i64 (not treated as null sentinel)
    assert!(
        result.is_err(),
        "empty string should NOT be a null sentinel for i64"
    );
}
