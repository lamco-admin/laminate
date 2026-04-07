/// Iteration 107: Unit parser — European dot-thousands and comma-decimal
///
/// parse_numeric now handles European format: "2.500,5" → 2500.5
/// and multiple-dot thousands: "1.234.567" → 1234567.
use laminate::packs::units::parse_unit_value;

#[test]
fn european_dot_thousands_comma_decimal() {
    // "2.500,5 kg" → 2500.5 kg
    let result = parse_unit_value("2.500,5 kg").unwrap();
    assert!(
        (result.amount - 2500.5).abs() < f64::EPSILON,
        "expected 2500.5, got {}",
        result.amount
    );
    assert_eq!(result.unit, "kg");
}

#[test]
fn european_multiple_dot_thousands() {
    // "1.234.567 kg" → 1234567 kg
    let result = parse_unit_value("1.234.567 kg").unwrap();
    assert!(
        (result.amount - 1234567.0).abs() < f64::EPSILON,
        "expected 1234567, got {}",
        result.amount
    );
}

#[test]
fn us_decimal_unaffected() {
    // "2.5 kg" — single dot, no comma → US decimal unchanged
    let result = parse_unit_value("2.5 kg").unwrap();
    assert!((result.amount - 2.5).abs() < f64::EPSILON);
}

#[test]
fn us_comma_thousands_unaffected() {
    // "1,234 kg" — US comma thousands still work
    let result = parse_unit_value("1,234 kg").unwrap();
    assert!((result.amount - 1234.0).abs() < f64::EPSILON);
}
