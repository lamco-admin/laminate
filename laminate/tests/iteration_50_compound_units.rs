use laminate::packs::units::{parse_unit_value, UnitCategory};

/// Iteration 50 — Pack Probe: Compound weight "120 lbs 4 oz".
/// Result: GAP — compound units completely unrecognized.
/// Fix: added parse_compound() fallback that decomposes and converts.

#[test]
fn iter50_compound_lbs_oz() {
    let uv = parse_unit_value("120 lbs 4 oz").unwrap();
    assert!((uv.amount - 120.25).abs() < 0.001); // 4 oz = 0.25 lb
    assert_eq!(uv.unit, "lb");
    assert_eq!(uv.category, UnitCategory::Weight);
}

#[test]
fn iter50_compound_ft_in() {
    let uv = parse_unit_value("5 ft 11 in").unwrap();
    assert!((uv.amount - 5.9167).abs() < 0.01); // 11 in ≈ 0.917 ft
    assert_eq!(uv.unit, "ft");
    assert_eq!(uv.category, UnitCategory::Length);
}

#[test]
fn iter50_single_unit_still_works() {
    let uv = parse_unit_value("120 lbs").unwrap();
    assert!((uv.amount - 120.0).abs() < f64::EPSILON);
    assert_eq!(uv.unit, "lb");
}

#[test]
fn iter50_compound_no_conversion_available() {
    // Mixing incompatible categories should fail (no lb→ft conversion)
    assert!(parse_unit_value("5 lbs 3 ft").is_none());
}
