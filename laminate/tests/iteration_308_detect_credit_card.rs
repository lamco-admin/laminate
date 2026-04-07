//! Iteration 308 — ID-Detect-CreditCard
//! CreditCard now ranks above Integer for Luhn-valid digit strings

use laminate::detect::{guess_type, GuessedType};

#[test]
fn credit_card_ranks_above_integer() {
    let guesses = guess_type("4111111111111111");
    assert_eq!(guesses[0].kind, GuessedType::CreditCard);
    // Integer should be demoted
    let int_guess = guesses
        .iter()
        .find(|g| g.kind == GuessedType::Integer)
        .unwrap();
    assert!(
        int_guess.confidence < 0.9,
        "Integer should be demoted below CreditCard"
    );
}

#[test]
fn amex_ranks_above_integer() {
    let guesses = guess_type("378282246310005");
    assert_eq!(guesses[0].kind, GuessedType::CreditCard);
}

#[test]
fn spaced_credit_card_detected() {
    let guesses = guess_type("4111 1111 1111 1111");
    assert_eq!(guesses[0].kind, GuessedType::CreditCard);
}

#[test]
fn regular_integer_not_affected() {
    // A normal integer that doesn't pass Luhn should remain Integer
    let guesses = guess_type("12345");
    assert_eq!(guesses[0].kind, GuessedType::Integer);
}
