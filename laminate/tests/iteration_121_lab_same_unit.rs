/// Iteration 121: convert_lab_value same unit returns identity — GAP fixed
use laminate::packs::medical::convert_lab_value;

#[test]
fn same_unit_returns_identity() {
    let result = convert_lab_value(7.0, "glucose", "mmol/L", "mmol/L").unwrap();
    assert!(
        (result - 7.0).abs() < f64::EPSILON,
        "same unit should return identity"
    );

    let result = convert_lab_value(126.0, "glucose", "mg/dL", "mg/dL").unwrap();
    assert!((result - 126.0).abs() < f64::EPSILON);
}
