//! Iteration 294 — ID-CreditCardSpaces
//! validate("4111 1111 1111 1111", CreditCard) — space-separated card numbers

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn credit_card_with_spaces() {
    let result = validate("4111 1111 1111 1111", IdentifierType::CreditCard);
    eprintln!("CC with spaces: {:?}", result);
    assert!(result.is_valid, "space-separated CC should validate");
}
