//! Iteration 74 — Domain Pack: European currency with code prefix/suffix
//!
//! BUG: "1.234,56 EUR" (European locale + code suffix) produced 1.23456
//! instead of 1234.56 — silent 1000x data corruption.
//! GAP: "EUR 1.234,56" (code prefix) returned None — no code-prefix variant.
//! Fix: Added CodePrefix variant and parse_amount_str() with auto-detection
//! of European vs US locale based on comma/dot position.

use laminate::packs::currency::parse_currency;

#[test]
fn eur_code_prefix_european() {
    // "EUR 1.234,56" — code before number, European locale
    let (amount, code) = parse_currency("EUR 1.234,56").unwrap();
    assert_eq!(amount, 1234.56);
    assert_eq!(code.as_deref(), Some("EUR"));
}

#[test]
fn eur_code_suffix_european() {
    // "1.234,56 EUR" — European locale with code suffix
    // Previously returned 1.23456 (BUG — 1000x error)
    let (amount, code) = parse_currency("1.234,56 EUR").unwrap();
    assert_eq!(amount, 1234.56);
    assert_eq!(code.as_deref(), Some("EUR"));
}

#[test]
fn eur_code_prefix_us_format() {
    // "EUR 12.99" — code prefix, US decimal
    let (amount, code) = parse_currency("EUR 12.99").unwrap();
    assert_eq!(amount, 12.99);
    assert_eq!(code.as_deref(), Some("EUR"));
}

#[test]
fn usd_code_prefix() {
    // "USD 1,234.56" — code prefix, US thousands
    let (amount, code) = parse_currency("USD 1,234.56").unwrap();
    assert_eq!(amount, 1234.56);
    assert_eq!(code.as_deref(), Some("USD"));
}

#[test]
fn eur_negative_code_prefix() {
    // "-EUR 1.234,56" — negative European with code prefix
    let (amount, code) = parse_currency("-EUR 1.234,56").unwrap();
    assert_eq!(amount, -1234.56);
    assert_eq!(code.as_deref(), Some("EUR"));
}
