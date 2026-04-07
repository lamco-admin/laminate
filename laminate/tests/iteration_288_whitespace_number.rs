//! Iteration 288 — Coerce-WhitespaceInNumber
//! Whitespace around numbers is trimmed before parsing (PASS)

use laminate::{CoercionLevel, FlexValue};

#[test]
fn leading_whitespace_trimmed() {
    let fv = FlexValue::from_json(r#"" 42""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert_eq!(fv.extract::<i64>("").unwrap(), 42);
}

#[test]
fn trailing_whitespace_trimmed() {
    let fv = FlexValue::from_json(r#""42 ""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert_eq!(fv.extract::<i64>("").unwrap(), 42);
}

#[test]
fn both_whitespace_f64_trimmed() {
    let fv = FlexValue::from_json(r#"" 3.14 ""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let val: f64 = fv.extract("").unwrap();
    assert!((val - 3.14).abs() < 1e-10);
}

#[test]
fn tab_whitespace_trimmed() {
    let fv = FlexValue::from_json(r#""\t42\t""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert_eq!(fv.extract::<i64>("").unwrap(), 42);
}
