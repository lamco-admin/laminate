//! Iteration 73 — Domain Pack: Japanese yen + Indian lakhs currency
//!
//! GAP: Full-width Japanese digits (U+FF10-FF19) and comma (U+FF0C)
//! were not recognized. Fixed with normalize_fullwidth() helper.

use laminate::packs::currency::parse_currency;

// --- Japanese yen ---

#[test]
fn yen_comma_thousands() {
    let (amount, code) = parse_currency("¥1,234").unwrap();
    assert_eq!(amount, 1234.0);
    assert_eq!(code.as_deref(), Some("JPY"));
}

#[test]
fn yen_no_comma() {
    let (amount, code) = parse_currency("¥500").unwrap();
    assert_eq!(amount, 500.0);
    assert_eq!(code.as_deref(), Some("JPY"));
}

#[test]
fn yen_millions() {
    let (amount, _) = parse_currency("¥1,234,567").unwrap();
    assert_eq!(amount, 1234567.0);
}

// --- Indian rupee lakhs ---

#[test]
fn rupee_lakhs_grouping() {
    // Indian grouping: first group of 3, then groups of 2
    let (amount, code) = parse_currency("₹1,23,456.78").unwrap();
    assert_eq!(amount, 123456.78);
    assert_eq!(code.as_deref(), Some("INR"));
}

#[test]
fn rupee_crores() {
    let (amount, _) = parse_currency("₹12,34,56,789").unwrap();
    assert_eq!(amount, 123456789.0);
}

#[test]
fn inr_code_suffix_lakhs() {
    let (amount, code) = parse_currency("1,23,456.78 INR").unwrap();
    assert_eq!(amount, 123456.78);
    assert_eq!(code.as_deref(), Some("INR"));
}

// --- Full-width Japanese characters (the gap) ---

#[test]
fn yen_fullwidth_digits_and_comma() {
    // ¥１，２３４ uses full-width 1234 (U+FF11 etc.) and full-width comma (U+FF0C)
    let (amount, code) = parse_currency("¥１，２３４").unwrap();
    assert_eq!(amount, 1234.0);
    assert_eq!(code.as_deref(), Some("JPY"));
}

#[test]
fn yen_fullwidth_with_decimal() {
    // ¥１２３．４５ with full-width period (U+FF0E)
    let (amount, _) = parse_currency("¥１２３．４５").unwrap();
    assert_eq!(amount, 123.45);
}
