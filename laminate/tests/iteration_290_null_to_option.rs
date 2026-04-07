//! Iteration 290 — Coerce-NullToOption
//! null → Option::None works at Exact; absent → Err (PASS)

use laminate::{CoercionLevel, FlexValue};

#[test]
fn null_to_none_at_exact() {
    let fv = FlexValue::from_json("null")
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    let result: Option<i64> = fv.extract("").unwrap();
    assert_eq!(result, None);
}

#[test]
fn null_field_to_none() {
    let fv = FlexValue::from_json(r#"{"value": null}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    let result: Option<i64> = fv.extract("value").unwrap();
    assert_eq!(result, None);
}

#[test]
fn absent_field_is_error_not_none() {
    let fv = FlexValue::from_json(r#"{"other": 42}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    assert!(fv.extract::<Option<i64>>("missing").is_err());
}

#[test]
fn present_value_is_some() {
    let fv = FlexValue::from_json(r#"42"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    let result: Option<i64> = fv.extract("").unwrap();
    assert_eq!(result, Some(42));
}
