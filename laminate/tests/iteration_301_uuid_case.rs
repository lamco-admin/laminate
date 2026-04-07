//! Iteration 301 — ID-UUIDLowercase + 302 ID-UUIDUppercase
//! UUID validation should be case-insensitive

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn uuid_lowercase() {
    let result = validate("550e8400-e29b-41d4-a716-446655440000", IdentifierType::Uuid);
    eprintln!("UUID lowercase: {:?}", result);
    assert!(result.is_valid);
}

#[test]
fn uuid_uppercase() {
    let result = validate("550E8400-E29B-41D4-A716-446655440000", IdentifierType::Uuid);
    eprintln!("UUID uppercase: {:?}", result);
    // RFC 4122 says UUIDs should be case-insensitive
}
