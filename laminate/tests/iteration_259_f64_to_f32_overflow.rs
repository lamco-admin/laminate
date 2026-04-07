//! Iteration 259: f64→f32 overflow during extract
//!
//! BUG found: extract::<f32>() silently produced f32::INFINITY for values
//! exceeding f32 range. The coercion engine had integer overflow checks but
//! no float narrowing checks.
//!
//! Fixed: added f32 range validation in the Integer→Float coercion arm.

use laminate::FlexValue;

#[test]
fn extract_f32_from_large_f64_errors() {
    // 1e308 fits in f64 but overflows f32 (max ~3.4e38)
    let val = FlexValue::from_json(r#"{"big": 1e308}"#).unwrap();
    let result = val.extract::<f32>("big");
    assert!(
        result.is_err(),
        "1e308 should not silently become f32::INFINITY"
    );
}

#[test]
fn extract_f32_from_borderline_overflow_errors() {
    // 1e39 also exceeds f32::MAX
    let val = FlexValue::from_json(r#"{"n": 1e39}"#).unwrap();
    let result = val.extract::<f32>("n");
    assert!(
        result.is_err(),
        "1e39 should not silently become f32::INFINITY"
    );
}

#[test]
fn extract_f32_from_tiny_f64_errors() {
    // 1e-300 underflows f32 to 0.0 — not the same as actual zero
    let val = FlexValue::from_json(r#"{"tiny": 1e-300}"#).unwrap();
    let result = val.extract::<f32>("tiny");
    assert!(result.is_err(), "1e-300 should not silently become 0.0");
}

#[test]
fn extract_f32_from_normal_value_ok() {
    // Values within f32 range should work fine
    let val = FlexValue::from_json(r#"{"price": 12.99}"#).unwrap();
    let result = val.extract::<f32>("price");
    assert!(result.is_ok());
    let price = result.unwrap();
    assert!((price - 12.99_f32).abs() < 0.001);
}

#[test]
fn extract_f64_from_large_value_still_ok() {
    // f64 itself can handle 1e308 — no overflow
    let val = FlexValue::from_json(r#"{"big": 1e308}"#).unwrap();
    let result = val.extract::<f64>("big");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1e308);
}

#[test]
fn extract_f32_negative_overflow_errors() {
    // Negative overflow should also be caught
    let val = FlexValue::from_json(r#"{"neg": -1e39}"#).unwrap();
    let result = val.extract::<f32>("neg");
    assert!(
        result.is_err(),
        "-1e39 should not silently become -f32::INFINITY"
    );
}
