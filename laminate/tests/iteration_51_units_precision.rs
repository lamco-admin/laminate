//! Iteration 51: Very small unit values — precision regression tests.
//!
//! Confirms that very small amounts (0.001 mg, 1e-6 kg, 1 nanogram)
//! survive parse → coerce → extract pipeline without precision loss.

use laminate::packs::units::{coerce_unit_value, convert, parse_unit_value};
use serde_json::Value;

#[test]
fn small_unit_values_parse_correctly() {
    // Sub-unit precision
    let uv = parse_unit_value("0.001 mg").unwrap();
    assert!((uv.amount - 0.001).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "mg");

    let uv2 = parse_unit_value("0.0001 kg").unwrap();
    assert!((uv2.amount - 0.0001).abs() < f64::EPSILON);

    // Very small: 1 nanogram in grams
    let nano = parse_unit_value("0.000000001 g").unwrap();
    assert!((nano.amount - 1e-9).abs() < 1e-20);
    assert_eq!(nano.unit, "g");
}

#[test]
fn scientific_notation_in_unit_values() {
    let uv = parse_unit_value("1e-6 kg").unwrap();
    assert!((uv.amount - 1e-6).abs() < 1e-18);
    assert_eq!(uv.unit, "kg");

    let uv2 = parse_unit_value("1.5e-3 mg").unwrap();
    assert!((uv2.amount - 0.0015).abs() < f64::EPSILON);
    assert_eq!(uv2.unit, "mg");
}

#[test]
fn small_value_conversion_precision() {
    // 0.001 mg → kg = 1e-9 kg
    let result = convert(0.001, "mg", "kg").unwrap();
    assert!((result - 1e-9).abs() < 1e-20);

    // 0.001 mg → g = 1e-6 g
    let result2 = convert(0.001, "mg", "g").unwrap();
    assert!((result2 - 1e-6).abs() < 1e-18);
}

#[test]
fn small_value_coerce_pipeline() {
    let coerced = coerce_unit_value(&Value::String("0.001 mg".into()), "dose");
    assert!(coerced.coerced);
    let num = coerced.value.as_f64().unwrap();
    assert!((num - 0.001).abs() < f64::EPSILON);

    // Extreme small value survives coerce
    let coerced2 = coerce_unit_value(&Value::String("0.000000001 g".into()), "mass");
    assert!(coerced2.coerced);
    let num2 = coerced2.value.as_f64().unwrap();
    assert!((num2 - 1e-9).abs() < 1e-20);
}

#[test]
fn zero_and_negative_small_values() {
    let zero = parse_unit_value("0 kg").unwrap();
    assert!((zero.amount - 0.0).abs() < f64::EPSILON);

    let neg = parse_unit_value("-0.001 mg").unwrap();
    assert!((neg.amount - (-0.001)).abs() < f64::EPSILON);
    assert_eq!(neg.unit, "mg");
}
