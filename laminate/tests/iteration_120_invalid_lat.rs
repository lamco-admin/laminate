/// Iteration 120: parse_coordinate rejects geometrically impossible coordinates
///
/// BUG: parse_coordinate("91, 91") returned Some with lat=91 — impossible.
/// FIX: validate lat ∈ [-90,90] and lng ∈ [-180,180] AFTER swap heuristic.
use laminate::packs::geo::parse_coordinate;

#[test]
fn both_over_90_rejected() {
    let result = parse_coordinate("91, 91");
    assert!(
        result.is_none(),
        "(91,91) should be rejected — no valid lat/lng assignment"
    );
}

#[test]
fn first_over_90_rejected() {
    // 91 > 90 in first position (latitude) — reject rather than silently swap.
    // Use detect_coordinate_order() for batch analysis when order is unknown.
    let result = parse_coordinate("91, 0");
    assert!(
        result.is_none(),
        "(91,0) should be rejected — first value exceeds ±90"
    );
}

#[test]
fn over_180_rejected() {
    assert!(
        parse_coordinate("0, 181").is_none(),
        "longitude > 180 should be rejected"
    );
    assert!(
        parse_coordinate("181, 0").is_none(),
        "value > 180 should be rejected"
    );
}
