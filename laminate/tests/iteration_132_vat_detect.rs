use laminate::detect::{guess_type, GuessedType};
/// Iteration 132: detect() missing EuVat — GAP
///
/// Off-plan discovery: EuVat not in detect()'s type array,
/// so guess_type() can never detect VAT numbers.
use laminate::packs::identifiers::{detect, validate, IdentifierType};

#[test]
fn vat_validates_directly() {
    let result = validate("DE123456789", IdentifierType::EuVat);
    assert!(result.is_valid, "DE VAT should validate directly");
}

#[test]
fn vat_detected_by_detect() {
    let candidates = detect("DE123456789");
    assert!(
        candidates.iter().any(|(t, _)| *t == IdentifierType::EuVat),
        "detect() should find EU VAT, got: {:?}",
        candidates.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );
}

#[test]
fn vat_detected_by_guess_type() {
    let guesses = guess_type("DE123456789");
    assert!(
        guesses.iter().any(|g| g.kind == GuessedType::VatNumber),
        "guess_type() should find VatNumber, got: {:?}",
        guesses.iter().map(|g| &g.kind).collect::<Vec<_>>()
    );
}
