//! Iteration 305: Email validation edge cases.
//! Tests: consecutive dots in local part, leading dot, very long local part.

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn email_consecutive_dots_in_local() {
    // RFC 5321 forbids consecutive dots in local part
    let result = validate("user..name@example.com", IdentifierType::Email);
    // Observe: does validation catch consecutive dots in local part?
    println!(
        "consecutive dots local: is_valid={}, error={:?}",
        result.is_valid, result.error
    );
    // This SHOULD be invalid per RFC, but the code only checks domain
    assert!(
        !result.is_valid,
        "consecutive dots in local part should be invalid"
    );
}

#[test]
fn email_leading_dot_in_local() {
    // RFC 5321 forbids leading dot in local part
    let result = validate(".user@example.com", IdentifierType::Email);
    println!(
        "leading dot local: is_valid={}, error={:?}",
        result.is_valid, result.error
    );
    assert!(
        !result.is_valid,
        "leading dot in local part should be invalid"
    );
}

#[test]
fn email_trailing_dot_in_local() {
    let result = validate("user.@example.com", IdentifierType::Email);
    println!(
        "trailing dot local: is_valid={}, error={:?}",
        result.is_valid, result.error
    );
    assert!(
        !result.is_valid,
        "trailing dot in local part should be invalid"
    );
}

#[test]
fn email_very_long_local() {
    // RFC 5321 says local part max is 64 chars
    let local = "a".repeat(65);
    let email = format!("{}@example.com", local);
    let result = validate(&email, IdentifierType::Email);
    println!(
        "65-char local: is_valid={}, error={:?}",
        result.is_valid, result.error
    );
    assert!(
        !result.is_valid,
        "65-char local part should be invalid per RFC 5321"
    );
}
