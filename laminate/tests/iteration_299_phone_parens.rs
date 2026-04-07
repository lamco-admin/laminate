//! Iteration 299 — ID-PhoneParens
//! US phone with parentheses "(555) 123-4567" now validates (GAP fixed)

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn phone_with_parentheses() {
    let result = validate("(555) 123-4567", IdentifierType::Phone);
    assert!(result.is_valid, "parenthesized area code should be valid");
    assert_eq!(result.normalized, "5551234567");
}

#[test]
fn phone_with_dots() {
    let result = validate("555.123.4567", IdentifierType::Phone);
    assert!(result.is_valid);
}

#[test]
fn phone_plain_digits() {
    let result = validate("5551234567", IdentifierType::Phone);
    assert!(result.is_valid);
}
