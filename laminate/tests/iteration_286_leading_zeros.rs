//! Iteration 286 — Coerce-LeadingZeros
//! Leading zeros in numeric strings are decimal, not octal (PASS)

use laminate::FlexValue;

#[test]
fn leading_zeros_decimal_not_octal() {
    let fv = FlexValue::from_json(r#""007""#).unwrap();
    assert_eq!(fv.extract_root::<i64>().unwrap(), 7);

    // "042" is 42 decimal, NOT 34 octal
    let fv = FlexValue::from_json(r#""042""#).unwrap();
    assert_eq!(fv.extract_root::<i64>().unwrap(), 42);
}

#[test]
fn leading_zeros_float() {
    let fv = FlexValue::from_json(r#""007.5""#).unwrap();
    assert_eq!(fv.extract_root::<f64>().unwrap(), 7.5);
}

#[test]
fn edge_cases() {
    let fv = FlexValue::from_json(r#""0""#).unwrap();
    assert_eq!(fv.extract_root::<i64>().unwrap(), 0);

    let fv = FlexValue::from_json(r#""00""#).unwrap();
    assert_eq!(fv.extract_root::<i64>().unwrap(), 0);
}
