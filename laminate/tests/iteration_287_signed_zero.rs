//! Iteration 287 — Coerce-SignedZero
//! "-0" correctly parses to f64 -0.0, and -0.0 == 0.0 per IEEE 754 (PASS)

use laminate::{CoercionLevel, FlexValue};

#[test]
fn string_negative_zero_parses() {
    let fv = FlexValue::from_json(r#""-0""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let val: f64 = fv.extract("").unwrap();
    assert!(val.is_sign_negative(), "should be negative zero");
    assert_eq!(val, 0.0, "negative zero equals positive zero");
}

#[test]
fn json_negative_zero_parses() {
    let fv = FlexValue::from_json(r#"-0"#).unwrap();
    let val: f64 = fv.extract("").unwrap();
    assert!(val.is_sign_negative());
    assert_eq!(val, 0.0);
}
