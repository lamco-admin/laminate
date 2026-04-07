//! Iteration 296 — ID-ISBNSpaces
//! validate("978 0 306 40615 7", Isbn13) — space-separated ISBN

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn isbn13_with_spaces() {
    let result = validate("978 0 306 40615 7", IdentifierType::Isbn13);
    eprintln!("ISBN-13 with spaces: {:?}", result);
    assert!(result.is_valid, "space-separated ISBN-13 should validate");
}
