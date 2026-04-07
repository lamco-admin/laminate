//! Iteration 285 — Coerce-UnicodeDigits
//! Non-ASCII digits don't coerce to i64, but must not panic (BUG fixed)

use laminate::{CoercionLevel, FlexValue};

#[test]
fn arabic_indic_digits_no_panic() {
    // ٤٢ = Arabic-Indic digits for 42 (2-byte chars, happened to not panic before)
    let fv = FlexValue::from_json(r#""٤٢""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert!(fv.extract::<i64>("").is_err());
}

#[test]
fn devanagari_digits_no_panic() {
    // ४२ = Devanagari digits for 42 (3-byte chars, panicked before fix)
    let fv = FlexValue::from_json(r#""४२""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert!(fv.extract::<i64>("").is_err());
}

#[test]
fn fullwidth_digits_no_panic() {
    // ４２ = Fullwidth digits for 42 (3-byte chars, panicked before fix)
    let fv = FlexValue::from_json(r#""４２""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    assert!(fv.extract::<i64>("").is_err());
}
