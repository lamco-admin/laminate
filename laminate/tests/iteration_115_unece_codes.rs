/// Iteration 115: UNECE Recommendation 20 and X12 EDI code recognition
///
/// parse_unit_value now recognizes standard codes like "500 KGM" (UNECE),
/// "100 LB" (X12), and "37 CEL" (UNECE temperature).
use laminate::packs::units::{convert, parse_unit_value, resolve_standard_code};

#[test]
fn unece_weight_codes() {
    let uv = parse_unit_value("500 KGM").unwrap();
    assert_eq!(uv.unit, "kg");
    assert!((uv.amount - 500.0).abs() < f64::EPSILON);

    let uv = parse_unit_value("100 LBR").unwrap();
    assert_eq!(uv.unit, "lb");

    let uv = parse_unit_value("250 GRM").unwrap();
    assert_eq!(uv.unit, "g");
}

#[test]
fn unece_length_codes() {
    let uv = parse_unit_value("10 MTR").unwrap();
    assert_eq!(uv.unit, "m");

    let uv = parse_unit_value("30 CMT").unwrap();
    assert_eq!(uv.unit, "cm");

    let uv = parse_unit_value("5 KMT").unwrap();
    assert_eq!(uv.unit, "km");
}

#[test]
fn unece_temperature() {
    let uv = parse_unit_value("37 CEL").unwrap();
    assert_eq!(uv.unit, "°C");
    assert!((uv.amount - 37.0).abs() < f64::EPSILON);

    // Now convert to Fahrenheit — convert uses normalized unit names
    let f = convert(uv.amount, "°C", "°F").unwrap_or_else(|| {
        // Fall back to old single-char names for conversion table compatibility
        convert(uv.amount, "c", "f").unwrap()
    });
    assert!(
        (f - 98.6).abs() < 0.1,
        "37 CEL should be ~98.6°F, got {}",
        f
    );
}

#[test]
fn x12_edi_codes() {
    let uv = parse_unit_value("100 LB").unwrap();
    assert_eq!(uv.unit, "lb");

    let uv = parse_unit_value("5 GA").unwrap();
    assert_eq!(uv.unit, "gal");
}

#[test]
fn unece_no_space() {
    // "500KGM" — code attached to number
    let uv = parse_unit_value("500KGM").unwrap();
    assert_eq!(uv.unit, "kg");
    assert!((uv.amount - 500.0).abs() < f64::EPSILON);
}

#[test]
fn resolve_code_function() {
    let (name, _cat) = resolve_standard_code("KGM").unwrap();
    assert_eq!(name, "kg");

    let (name, _cat) = resolve_standard_code("CEL").unwrap();
    assert_eq!(name, "°C");

    assert!(resolve_standard_code("INVALID").is_none());
}

#[test]
fn regular_units_still_work() {
    // Existing unit patterns should still work
    let uv = parse_unit_value("2.5 kg").unwrap();
    assert_eq!(uv.unit, "kg");

    let uv = parse_unit_value("37.2°C").unwrap();
    assert_eq!(uv.unit, "°C");
}
