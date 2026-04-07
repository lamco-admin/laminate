//! Iteration 78 probe: no-space units and formatted numbers in unit values
//! Target: "1.5L", "500ml", and comma-thousands edge cases

use laminate::packs::units::{parse_unit_value, UnitCategory};

// --- No-space basics (expected to work) ---

#[test]
fn nospace_volume_liters() {
    // "1.5L" — no space between number and unit
    let uv = parse_unit_value("1.5L").unwrap();
    assert!((uv.amount - 1.5).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "L");
    assert_eq!(uv.category, UnitCategory::Volume);
}

#[test]
fn nospace_volume_ml() {
    // "500ml"
    let uv = parse_unit_value("500ml").unwrap();
    assert!((uv.amount - 500.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "mL");
}

#[test]
fn nospace_weight_kg() {
    // "2.5kg"
    let uv = parse_unit_value("2.5kg").unwrap();
    assert!((uv.amount - 2.5).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "kg");
}

#[test]
fn nospace_length_cm() {
    // "180cm"
    let uv = parse_unit_value("180cm").unwrap();
    assert!((uv.amount - 180.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "cm");
}

#[test]
fn nospace_data_gb() {
    // "16GB"
    let uv = parse_unit_value("16GB").unwrap();
    assert!((uv.amount - 16.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "GB");
}

#[test]
fn nospace_negative() {
    // "-3.5kg" — negative with no space
    let uv = parse_unit_value("-3.5kg").unwrap();
    assert!((uv.amount - -3.5).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "kg");
}

// --- Comma-thousands in unit values ---
// Real-world data often has "1,500ml" or "2,500 kg"

#[test]
fn comma_thousands_nospace() {
    // "1,500ml" — comma thousands with no space
    let uv = parse_unit_value("1,500ml");
    assert!(
        uv.is_some(),
        "\"1,500ml\" should parse — comma thousands are common in real data"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 1500.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "mL");
}

#[test]
fn comma_thousands_with_space() {
    // "2,500 kg" — comma thousands with space
    let uv = parse_unit_value("2,500 kg");
    assert!(
        uv.is_some(),
        "\"2,500 kg\" should parse — comma thousands are common"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 2500.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "kg");
}

#[test]
fn comma_thousands_decimal() {
    // "1,234.5 lb" — comma thousands with decimal
    let uv = parse_unit_value("1,234.5 lb");
    assert!(uv.is_some(), "\"1,234.5 lb\" should parse");
    let uv = uv.unwrap();
    assert!((uv.amount - 1234.5).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "lb");
}

#[test]
fn underscore_thousands() {
    // "1_500ml" — Rust/Python style (already handled for bare numbers by coercion)
    let uv = parse_unit_value("1_500ml");
    assert!(
        uv.is_some(),
        "\"1_500ml\" should parse — underscore separators in numeric context"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 1500.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "mL");
}

#[test]
fn apostrophe_thousands() {
    // "1'500 km" — Swiss style (already handled for currency by strip_amount_formatting)
    let uv = parse_unit_value("1'500 km");
    assert!(
        uv.is_some(),
        "\"1'500 km\" should parse — Swiss apostrophe thousands"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 1500.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "km");
}
