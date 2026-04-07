//! Iteration 77 probe: degree symbol temperatures
//! Target: "72°F", "22°C", and edge cases including Unicode confusables

use laminate::packs::units::{parse_unit_value, UnitCategory};

// --- Standard degree symbol (° U+00B0) ---

#[test]
fn temp_fahrenheit_no_space() {
    // "72°F" — most common format
    let uv = parse_unit_value("72°F").unwrap();
    assert!((uv.amount - 72.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°F");
    assert_eq!(uv.category, UnitCategory::Temperature);
}

#[test]
fn temp_celsius_no_space() {
    // "22°C"
    let uv = parse_unit_value("22°C").unwrap();
    assert!((uv.amount - 22.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°C");
    assert_eq!(uv.category, UnitCategory::Temperature);
}

#[test]
fn temp_negative() {
    // "-40°C" — negative temperature
    let uv = parse_unit_value("-40°C").unwrap();
    assert!((uv.amount - -40.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°C");
}

#[test]
fn temp_decimal() {
    // "98.6°F" — body temperature
    let uv = parse_unit_value("98.6°F").unwrap();
    assert!((uv.amount - 98.6).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°F");
}

#[test]
fn temp_with_space_before_degree() {
    // "350 °F" — oven temperature with space
    let uv = parse_unit_value("350 °F").unwrap();
    assert!((uv.amount - 350.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°F");
}

#[test]
fn temp_zero() {
    // "0°C" — freezing point
    let uv = parse_unit_value("0°C").unwrap();
    assert!((uv.amount - 0.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°C");
}

// --- Unicode confusable: masculine ordinal indicator (º U+00BA) ---
// Looks identical to degree sign in most fonts but is a different code point.
// Common in OCR output, copy-paste from some locales, and web data.

#[test]
fn temp_ordinal_indicator_fahrenheit() {
    // "72ºF" using U+00BA instead of U+00B0
    let uv = parse_unit_value("72ºF");
    assert!(
        uv.is_some(),
        "masculine ordinal indicator (U+00BA) should be treated as degree sign"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 72.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°F");
}

#[test]
fn temp_ordinal_indicator_celsius() {
    // "22ºC" using U+00BA instead of U+00B0
    let uv = parse_unit_value("22ºC");
    assert!(
        uv.is_some(),
        "masculine ordinal indicator (U+00BA) should be treated as degree sign"
    );
    let uv = uv.unwrap();
    assert!((uv.amount - 22.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°C");
}

// --- Spelled-out units ---

#[test]
fn temp_spelled_out_celsius() {
    // "22 celsius" — full word
    let uv = parse_unit_value("22 celsius").unwrap();
    assert!((uv.amount - 22.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°C");
}

#[test]
fn temp_spelled_out_fahrenheit() {
    // "72 fahrenheit" — full word
    let uv = parse_unit_value("72 fahrenheit").unwrap();
    assert!((uv.amount - 72.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "°F");
}
