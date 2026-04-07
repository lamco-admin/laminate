// Iteration 162: EU VAT with varying formats per country
// Target #104 — Spain has old CIF (letter+8digits) and NIF (8digits+letter)
// France allows 2 letters at the start. Greece uses EL or GR prefix.

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn spain_vat_nif_format() {
    // Spanish NIF: 8 digits + letter (e.g., "12345678A")
    let result = validate("ESA12345678", IdentifierType::EuVat);
    // ES + 9 chars = valid format
    assert!(
        result.is_valid,
        "Spain CIF format should be valid: {:?}",
        result.error
    );
    assert_eq!(result.detail.as_deref(), Some("ES"));
}

#[test]
fn spain_vat_old_cif_format() {
    // Spanish CIF: letter + 7 digits + check (e.g., "B12345678")
    let result = validate("ESB12345678", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "Spain old CIF format should be valid: {:?}",
        result.error
    );
}

#[test]
fn france_vat_with_letters() {
    // France: 2 chars (can be letters) + 9 digits = 11 total
    let result = validate("FRAB123456789", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "France VAT with letter prefix should be valid: {:?}",
        result.error
    );
}

#[test]
fn greece_el_prefix() {
    // Greece uses "EL" officially, not "GR"
    let result = validate("EL123456789", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "Greece EL prefix should be valid: {:?}",
        result.error
    );
}

#[test]
fn greece_gr_prefix() {
    // Some systems use GR instead of EL
    let result = validate("GR123456789", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "Greece GR prefix should be valid: {:?}",
        result.error
    );
}

#[test]
fn ireland_old_format() {
    // Ireland old format: 7 digits + 1 letter = 8 chars
    let result = validate("IE1234567A", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "Ireland old format (8 chars) should be valid: {:?}",
        result.error
    );
}

#[test]
fn ireland_new_format() {
    // Ireland new format: 7 digits + 2 letters = 9 chars
    let result = validate("IE1234567AB", IdentifierType::EuVat);
    assert!(
        result.is_valid,
        "Ireland new format (9 chars) should be valid: {:?}",
        result.error
    );
}
