#![allow(dead_code)]
//! Iteration 189: shape on nested path with Absorbing mode.
//!
//! Question: Does shape::<f64, Absorbing>("deeply.nested.value") correctly
//! propagate the coercion level through the path navigation?
//! Absorbing uses SafeWidening — which allows int→float but NOT string→float.

use laminate::{Absorbing, FlexValue, Lenient};

#[test]
fn shape_absorbing_nested_int_to_float() {
    // SafeWidening should allow int → float
    let val = FlexValue::from_json(r#"{"a": {"b": {"c": 42}}}"#).unwrap();
    let result = val.shape::<f64, Absorbing>("a.b.c");
    println!("int→f64 Absorbing: {result:?}");
    assert!(result.is_ok(), "SafeWidening should allow int→float");
    let (v, _) = result.unwrap();
    assert!((v - 42.0).abs() < f64::EPSILON);
}

#[test]
fn shape_absorbing_nested_string_to_float() {
    // SafeWidening should NOT allow string → float
    let val = FlexValue::from_json(r#"{"a": {"b": {"c": "3.14"}}}"#).unwrap();
    let result = val.shape::<f64, Absorbing>("a.b.c");
    println!("string→f64 Absorbing: {result:?}");
    // This should fail — SafeWidening doesn't parse strings
    assert!(result.is_err(), "SafeWidening should reject string→float");
}

#[test]
fn shape_lenient_nested_string_to_float() {
    // BestEffort (Lenient) should allow string → float
    let val = FlexValue::from_json(r#"{"a": {"b": {"c": "3.14"}}}"#).unwrap();
    let result = val.shape::<f64, Lenient>("a.b.c");
    println!("string→f64 Lenient: {result:?}");
    assert!(result.is_ok(), "BestEffort should parse string→float");
    let (v, _) = result.unwrap();
    assert!((v - 3.14).abs() < f64::EPSILON);
}
