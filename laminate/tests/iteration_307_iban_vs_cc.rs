//! Iteration 307: Can a string pass both IBAN MOD-97 and Luhn CC validation?
//! IBAN requires 2-letter country prefix; CC is all-digits. Should not overlap.

use laminate::packs::identifiers::{detect, validate, IdentifierType};

#[test]
fn iban_and_cc_cannot_overlap() {
    // A valid IBAN (GB format): starts with letters — can't be CC
    let iban = "GB29NWBK60161331926819";
    let results = detect(iban);
    println!("IBAN detect: {:?}", results);

    let has_iban = results.iter().any(|(t, _)| *t == IdentifierType::Iban);
    let has_cc = results
        .iter()
        .any(|(t, _)| *t == IdentifierType::CreditCard);
    assert!(has_iban, "should detect as IBAN");
    assert!(!has_cc, "IBAN should not also match CC (has letters)");
}

#[test]
fn cc_cannot_be_iban() {
    // Valid Visa card: all digits — can't have letter country prefix
    let cc = "4111111111111111";
    let results = detect(cc);
    println!("CC detect: {:?}", results);

    let has_cc = results
        .iter()
        .any(|(t, _)| *t == IdentifierType::CreditCard);
    let has_iban = results.iter().any(|(t, _)| *t == IdentifierType::Iban);
    assert!(has_cc, "should detect as CC");
    assert!(!has_iban, "CC should not match IBAN (no country code)");
}

#[test]
fn long_numeric_iban_attempt() {
    // Some countries have long IBANs (e.g., NO = 15 chars). But still has country prefix.
    // Try a numeric-only string that might match IBAN length
    let fake = "1234567890123456"; // 16 digits, like a CC
    let iban_result = validate(fake, IdentifierType::Iban);
    println!("Numeric IBAN attempt: {:?}", iban_result);
    assert!(!iban_result.is_valid, "all-digit string should not be IBAN");
}
