// Iteration 167: parse_currency with accounting negative formats
// Fresh target — various negative currency representations

use laminate::packs::currency::parse_currency;

#[test]
fn accounting_parens_with_dollar() {
    // "($12.99)" — accounting negative
    let result = parse_currency("($12.99)");
    assert!(result.is_some(), "Accounting format should parse");
    let (amount, code) = result.unwrap();
    assert!(
        (amount - (-12.99)).abs() < 0.001,
        "Expected -12.99, got {}",
        amount
    );
    assert_eq!(code.as_deref(), Some("USD"));
}

#[test]
fn accounting_parens_without_symbol() {
    // "(12.99)" — parens without currency symbol
    let result = parse_currency("(12.99)");
    println!("(12.99) → {:?}", result);
    // This is ambiguous — could be negative number or math grouping
    // parse_currency should return None since there's no currency indicator
}

#[test]
fn negative_with_euro_symbol() {
    // "-€42.50" — negative with euro
    let result = parse_currency("-€42.50");
    assert!(result.is_some(), "Negative euro should parse");
    let (amount, code) = result.unwrap();
    assert!(
        (amount - (-42.50)).abs() < 0.001,
        "Expected -42.50, got {}",
        amount
    );
    assert_eq!(code.as_deref(), Some("EUR"));
}

#[test]
fn accounting_parens_with_code() {
    // "(1,234.56 USD)" — parens with currency code suffix
    let result = parse_currency("(1,234.56 USD)");
    assert!(result.is_some(), "Accounting format with code should parse");
    let (amount, code) = result.unwrap();
    assert!(
        (amount - (-1234.56)).abs() < 0.01,
        "Expected -1234.56, got {}",
        amount
    );
    assert_eq!(code.as_deref(), Some("USD"));
}

#[test]
fn negative_zero_currency() {
    // "-$0.00" — negative zero
    let result = parse_currency("-$0.00");
    assert!(result.is_some(), "Negative zero should parse");
    let (amount, _code) = result.unwrap();
    assert!(amount.abs() < 0.001, "Expected ~0, got {}", amount);
}
