// Iteration 175: parse_currency with crypto symbols
// Fresh target — BTC/ETH codes exist but ₿/Ξ symbols don't.
// Does "₿1.5" parse? Does "BTC 1.5" parse?

use laminate::packs::currency::parse_currency;

#[test]
fn btc_code_prefix() {
    // "BTC 1.5" — code prefix format
    let result = parse_currency("BTC 1.5");
    assert!(result.is_some(), "BTC code prefix should parse");
    let (amount, code) = result.unwrap();
    assert!((amount - 1.5).abs() < 0.001);
    assert_eq!(code.as_deref(), Some("BTC"));
}

#[test]
fn btc_symbol_prefix() {
    // "₿1.5" — Bitcoin symbol prefix
    let result = parse_currency("₿1.5");
    println!("₿1.5 → {:?}", result);
    // ₿ is not in CURRENCY_SYMBOLS, so this won't parse as currency
    // This is a GAP if we want to support crypto symbols
}

#[test]
fn eth_code_prefix() {
    // "ETH 2.0" — code prefix
    let result = parse_currency("ETH 2.0");
    assert!(result.is_some(), "ETH code prefix should parse");
    let (amount, code) = result.unwrap();
    assert!((amount - 2.0).abs() < 0.001);
    assert_eq!(code.as_deref(), Some("ETH"));
}

#[test]
fn btc_code_suffix() {
    // "0.5 BTC" — code suffix
    let result = parse_currency("0.5 BTC");
    assert!(result.is_some(), "BTC code suffix should parse");
    let (amount, code) = result.unwrap();
    assert!((amount - 0.5).abs() < 0.001);
    assert_eq!(code.as_deref(), Some("BTC"));
}

#[test]
fn btc_high_precision() {
    // "BTC 0.00000001" — 1 satoshi
    let result = parse_currency("BTC 0.00000001");
    assert!(result.is_some(), "BTC satoshi precision should parse");
    let (amount, _code) = result.unwrap();
    assert!((amount - 0.00000001).abs() < 1e-12);
}
