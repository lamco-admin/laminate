use laminate::packs::currency::{detect_currency_format, parse_currency, CurrencyFormat};

/// Iteration 46 — Pack Probe: Swiss apostrophe thousands separator.
/// Result: GAP — apostrophe completely blocked currency detection.
/// Fix: added apostrophe to is_numeric_string strip list and strip_amount_formatting.

#[test]
fn iter46_swiss_prefix_format() {
    match detect_currency_format("CHF 1'234.56") {
        CurrencyFormat::SymbolPrefix { symbol, amount } => {
            assert_eq!(symbol, "CHF");
            assert_eq!(amount, "1'234.56");
        }
        other => panic!("expected SymbolPrefix, got {other:?}"),
    }
    let (amount, code) = parse_currency("CHF 1'234.56").unwrap();
    assert!((amount - 1234.56).abs() < f64::EPSILON);
    assert_eq!(code, Some("CHF".into()));
}

#[test]
fn iter46_swiss_suffix_format() {
    match detect_currency_format("1'000.00 CHF") {
        CurrencyFormat::CodeSuffix { code, amount } => {
            assert_eq!(code, "CHF");
            assert_eq!(amount, "1'000.00");
        }
        other => panic!("expected CodeSuffix, got {other:?}"),
    }
    let (amount, code) = parse_currency("1'000.00 CHF").unwrap();
    assert!((amount - 1000.0).abs() < f64::EPSILON);
    assert_eq!(code, Some("CHF".into()));
}

#[test]
fn iter46_swiss_no_space() {
    let (amount, code) = parse_currency("CHF1'234.56").unwrap();
    assert!((amount - 1234.56).abs() < f64::EPSILON);
    assert_eq!(code, Some("CHF".into()));
}

#[test]
fn iter46_plain_apostrophe_not_currency() {
    // Plain number with apostrophes but no currency symbol → not currency
    assert_eq!(
        detect_currency_format("1'234'567.89"),
        CurrencyFormat::NotCurrency
    );
    assert!(parse_currency("1'234'567.89").is_none());
}
