/// Iteration 125: guess_type now detects identifiers (credit cards, IBAN, etc.)
///
/// GAP: guess_type didn't use identifiers::detect(). Now wired in.
use laminate::detect::{guess_type, GuessedType};

#[test]
fn guess_type_detects_credit_card() {
    let guesses = guess_type("4111111111111111");
    assert!(
        guesses.iter().any(|g| g.kind == GuessedType::CreditCard),
        "should detect credit card among candidates"
    );
}
