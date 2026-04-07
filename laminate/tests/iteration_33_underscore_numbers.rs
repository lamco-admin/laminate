//! Iteration 33: Format — Rust/Python-style underscore numeric separators
//!
//! Tests coercion of underscore-formatted numeric strings like "1_000" to
//! integer and float types. Common in Rust, Python 3, and config formats.

use laminate::{CoercionLevel, FlexValue};

fn fv(s: &str) -> FlexValue {
    FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, s))
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort)
}

#[test]
fn underscore_integer_coercion() {
    // Basic thousands
    assert_eq!(fv("1_000").extract::<i64>("v").unwrap(), 1000);
    // Millions
    assert_eq!(fv("1_000_000").extract::<i64>("v").unwrap(), 1_000_000);
    // Negative
    assert_eq!(fv("-1_000").extract::<i64>("v").unwrap(), -1000);
    // Arbitrary grouping (Rust allows any placement)
    assert_eq!(fv("1_0").extract::<i64>("v").unwrap(), 10);
}

#[test]
fn underscore_float_coercion() {
    let v: f64 = fv("3.14_15").extract("v").unwrap();
    assert!((v - 3.1415).abs() < f64::EPSILON);

    let v: f64 = fv("1_000.5").extract("v").unwrap();
    assert!((v - 1000.5).abs() < f64::EPSILON);
}

#[test]
fn underscore_hex_coercion() {
    // Hex with underscores → strip underscores then parse radix
    assert_eq!(fv("0xFF_FF").extract::<i64>("v").unwrap(), 65535);
}

#[test]
fn invalid_underscore_formats_rejected() {
    // Leading underscore (looks like identifier)
    assert!(fv("_100").extract::<i64>("v").is_err());
    // Trailing underscore
    assert!(fv("100_").extract::<i64>("v").is_err());
    // Double underscore
    assert!(fv("1__000").extract::<i64>("v").is_err());
}

#[test]
fn underscore_as_string_preserves_original() {
    // Extracting as String should NOT strip underscores
    let s: String = fv("1_000").extract("v").unwrap();
    assert_eq!(s, "1_000");
}
