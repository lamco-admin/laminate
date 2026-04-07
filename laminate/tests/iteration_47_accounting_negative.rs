use laminate::packs::currency::{detect_currency_format, parse_currency, CurrencyFormat};

/// Iteration 47 — Pack Probe: Accounting negative format with parentheses.
/// Result: GAP — parenthesized negatives completely unsupported.
/// Fix: detect balanced parens as accounting negatives, use s_stripped consistently.

#[test]
fn iter47_accounting_symbol_inside_parens() {
    match detect_currency_format("($12.99)") {
        CurrencyFormat::SymbolPrefix { symbol, amount } => {
            assert_eq!(symbol, "$");
            assert_eq!(amount, "-12.99");
        }
        other => panic!("expected SymbolPrefix, got {other:?}"),
    }
    let (amount, code) = parse_currency("($12.99)").unwrap();
    assert!((amount - (-12.99)).abs() < f64::EPSILON);
    assert_eq!(code, Some("USD".into()));
}

#[test]
fn iter47_accounting_code_inside_parens() {
    match detect_currency_format("(1,234.56 USD)") {
        CurrencyFormat::CodeSuffix { code, amount } => {
            assert_eq!(code, "USD");
            assert_eq!(amount, "-1,234.56");
        }
        other => panic!("expected CodeSuffix, got {other:?}"),
    }
    let (amount, code) = parse_currency("(1,234.56 USD)").unwrap();
    assert!((amount - (-1234.56)).abs() < f64::EPSILON);
    assert_eq!(code, Some("USD".into()));
}

#[test]
fn iter47_plain_parens_not_currency() {
    // No symbol → not detectable as currency (defensible)
    assert_eq!(
        detect_currency_format("(12.99)"),
        CurrencyFormat::NotCurrency
    );
}

#[test]
fn iter47_minus_prefix_still_works() {
    // Existing behavior preserved
    let (amount, code) = parse_currency("-$12.99").unwrap();
    assert!((amount - (-12.99)).abs() < f64::EPSILON);
    assert_eq!(code, Some("USD".into()));
}
