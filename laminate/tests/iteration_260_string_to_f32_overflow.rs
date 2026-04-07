//! Iteration 260: String→f32 overflow via coercion
//!
//! BUG found: Same f32 overflow issue as iter 259, but on the String→Float
//! coercion path. String "1e308" parsed as f64, then serde silently
//! produced f32::INFINITY.
//!
//! Fixed: f32 range check added to String→Float arm in coerce_value().

use laminate::FlexValue;

#[test]
fn string_to_f32_overflow_errors() {
    let val = FlexValue::from_json(r#"{"big": "1e308"}"#).unwrap();
    let result = val.extract::<f32>("big");
    assert!(
        result.is_err(),
        "string \"1e308\" should not silently become f32::INFINITY"
    );
}

#[test]
fn string_to_f32_underflow_errors() {
    let val = FlexValue::from_json(r#"{"tiny": "1e-300"}"#).unwrap();
    let result = val.extract::<f32>("tiny");
    assert!(
        result.is_err(),
        "string \"1e-300\" should not silently become 0.0"
    );
}

#[test]
fn string_to_f32_negative_overflow_errors() {
    let val = FlexValue::from_json(r#"{"neg": "-1e39"}"#).unwrap();
    let result = val.extract::<f32>("neg");
    assert!(
        result.is_err(),
        "string \"-1e39\" should not silently become -f32::INFINITY"
    );
}

#[test]
fn string_to_f64_large_still_ok() {
    let val = FlexValue::from_json(r#"{"big": "1e308"}"#).unwrap();
    let result = val.extract::<f64>("big");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1e308);
}

#[test]
fn string_to_f32_normal_ok() {
    let val = FlexValue::from_json(r#"{"price": "12.99"}"#).unwrap();
    let result = val.extract::<f32>("price");
    assert!(result.is_ok());
    let price = result.unwrap();
    assert!((price - 12.99_f32).abs() < 0.001);
}
