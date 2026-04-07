//! Iteration 295 — ID-CreditCardDashes
//! validate("4111-1111-1111-1111", CreditCard) — dash-separated card numbers

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn credit_card_with_dashes() {
    let result = validate("4111-1111-1111-1111", IdentifierType::CreditCard);
    eprintln!("CC with dashes: {:?}", result);
    assert!(result.is_valid, "dash-separated CC should validate");
    assert_eq!(result.normalized, "4111111111111111");
}
