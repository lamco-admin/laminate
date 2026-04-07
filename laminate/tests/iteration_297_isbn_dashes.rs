//! Iteration 297 — ID-ISBNDashes
//! validate("978-0-306-40615-7", Isbn13) — dashed ISBN-13

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn isbn13_with_dashes() {
    let result = validate("978-0-306-40615-7", IdentifierType::Isbn13);
    eprintln!("ISBN-13 with dashes: {:?}", result);
    assert!(result.is_valid, "dashed ISBN-13 should validate");
    assert_eq!(result.normalized, "9780306406157");
}
