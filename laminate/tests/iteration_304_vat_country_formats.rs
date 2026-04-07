//! Iteration 304: EU VAT validation for countries with letter suffixes.
//! Spain: X12345678A (letter + 7 digits + letter), France: XX + 9 digits, Ireland: mixed.

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn es_vat_digit_plus_letter() {
    // Standard Spanish format: 8 digits + letter
    let result = validate("ES12345678A", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "ES 8digit+letter should be valid: {:?}",
        result
    );
}

#[test]
fn es_vat_letter_prefix() {
    // Spanish NIE format: letter + 7 digits + letter
    let result = validate("ESX1234567A", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "ES letter+7digit+letter should be valid: {:?}",
        result
    );
}

#[test]
fn fr_vat_with_letters() {
    // French VAT can have 2 letters + 9 digits (e.g. FR AB 123456789)
    let result = validate("FRAB123456789", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "FR letter-prefix VAT should be valid: {:?}",
        result
    );
}

#[test]
fn ie_vat_7_digits_plus_letter() {
    // Irish old format: 7 digits + 1 letter = 8 chars
    let result = validate("IE1234567A", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "IE 7digit+letter should be valid: {:?}",
        result
    );
}

#[test]
fn ie_vat_new_format() {
    // Irish new format: digit + letter + 5 digits + letter = 8 chars
    let result = validate("IE1A23456B", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "IE new format should be valid: {:?}",
        result
    );
}

#[test]
fn at_vat_u_prefix() {
    // Austrian: U + 8 digits = 9 chars
    let result = validate("ATU12345678", IdentifierType::EuVat);
    assert!(result.is_valid, "AT U-prefix should be valid: {:?}", result);
}
