//! Iteration 298 — ID-EINOverlap
//! validate("000-12-3456", UsEin) — EIN format with area 000

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn ein_with_ssn_area_000() {
    // "000-12-3456" has 9 digits — passes EIN format (any 9 digits)
    // But area 000 is invalid for SSN. Does EIN accept it?
    let result = validate("000-12-3456", IdentifierType::UsEin);
    eprintln!("EIN '000-12-3456': {:?}", result);
    // EIN has no area validation — it's format-only (9 digits)
}

#[test]
fn ein_normal_format() {
    let result = validate("12-3456789", IdentifierType::UsEin);
    eprintln!("EIN '12-3456789': {:?}", result);
}
