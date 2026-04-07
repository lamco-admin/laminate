/// Iteration 160: normalize_pharma_unit("µg") — Unicode mu (target #121)
use laminate::packs::medical::normalize_pharma_unit;

#[test]
fn unicode_mu_vs_micro_sign() {
    // µ (U+00B5 MICRO SIGN) vs μ (U+03BC GREEK SMALL LETTER MU)
    let micro_sign = normalize_pharma_unit("\u{00B5}g");
    let greek_mu = normalize_pharma_unit("\u{03BC}g");
    println!("micro sign µg: {:?}", micro_sign);
    println!("greek mu μg: {:?}", greek_mu);
    // Both should normalize to the same thing
    assert_eq!(
        micro_sign, greek_mu,
        "both Unicode mu variants should normalize identically"
    );
}

#[test]
fn standard_units_normalize() {
    let mg = normalize_pharma_unit("mg");
    println!("mg: {:?}", mg);
    let mcg = normalize_pharma_unit("mcg");
    println!("mcg: {:?}", mcg);
}
