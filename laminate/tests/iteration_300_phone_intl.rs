//! Iteration 300 — ID-PhoneIntl
//! validate("+44 20 7946 0958", Phone) — international format with spaces

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn phone_international_uk() {
    let result = validate("+44 20 7946 0958", IdentifierType::Phone);
    eprintln!("Phone '+44 20 7946 0958': {:?}", result);
    assert!(result.is_valid, "international phone should validate");
}

#[test]
fn phone_e164() {
    let result = validate("+442079460958", IdentifierType::Phone);
    eprintln!("Phone E.164: {:?}", result);
    assert!(result.is_valid);
}
