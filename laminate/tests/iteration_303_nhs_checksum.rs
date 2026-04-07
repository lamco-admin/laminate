//! Iteration 303: NHS number MOD-11 check digit edge cases.
//! Tests check digit = 10 (invalid per NHS rules) and check digit = 0 (remainder=0).

use laminate::packs::identifiers::{validate, IdentifierType};

#[test]
fn nhs_check_digit_10_is_invalid() {
    // Prefix 100000001: sum=12, rem=1, check=11-1=10 → invalid
    let result = validate("1000000010", IdentifierType::UkNhs);
    assert!(
        !result.is_valid,
        "check digit 10 should be invalid: {:?}",
        result
    );
    assert!(result
        .error
        .as_deref()
        .unwrap_or("")
        .contains("check digit would be 10"));
}

#[test]
fn nhs_check_digit_0_from_remainder_0() {
    // Prefix 100000006: sum=22, rem=0, check=0 → valid, last digit must be 0
    let result = validate("1000000060", IdentifierType::UkNhs);
    assert!(result.is_valid, "check=0 should be valid: {:?}", result);
}

#[test]
fn nhs_check_digit_0_wrong_last_digit() {
    // Same prefix but wrong check digit (1 instead of 0)
    let result = validate("1000000061", IdentifierType::UkNhs);
    assert!(
        !result.is_valid,
        "wrong check digit should fail: {:?}",
        result
    );
}
