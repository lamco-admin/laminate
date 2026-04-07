//! Iteration 20: Precision — String-encoded numeric limits
//! Gap: String→Numeric coercion lacked overflow detection. "9223372036854775808"
//! (i64::MAX+1) parsed as u64 and coerced without range check, producing opaque
//! serde error instead of flagged_no_coerce diagnostic.
//! Fix: Added integer_fits_target check to String→Numeric i64 path, and explicit
//! u64 range check for unsigned-only values.

use laminate::FlexValue;

#[test]
fn iter20_i64_max_string_coerces() {
    let flex = FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, i64::MAX)).unwrap();
    let result: i64 = flex.extract("v").unwrap();
    assert_eq!(result, i64::MAX);
}

#[test]
fn iter20_i64_max_plus_one_string_overflow_detected() {
    let val = (i64::MAX as u64) + 1;
    let flex = FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, val)).unwrap();
    // flagged_no_coerce returns original string value → serde errors on string-as-i64
    assert!(flex.extract::<i64>("v").is_err());
}

#[test]
fn iter20_u64_max_string_to_u64_works() {
    let flex = FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, u64::MAX)).unwrap();
    let result: u64 = flex.extract("v").unwrap();
    assert_eq!(result, u64::MAX);
}

#[test]
fn iter20_u64_max_string_to_i64_overflow_detected() {
    let flex = FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, u64::MAX)).unwrap();
    assert!(flex.extract::<i64>("v").is_err());
}

#[test]
fn iter20_small_string_to_u8_overflow_detected() {
    // "300" fits i64 but overflows u8
    let flex = FlexValue::from_json(r#"{"v": "300"}"#).unwrap();
    assert!(flex.extract::<u8>("v").is_err());
}

#[test]
fn iter20_f64_max_string_coerces() {
    let flex = FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, f64::MAX)).unwrap();
    let result: f64 = flex.extract("v").unwrap();
    assert_eq!(result, f64::MAX);
}
