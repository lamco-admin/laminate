/// Iteration 106: Currency — European thousands with no decimal part
///
/// "€1.234.567" uses dots as thousands separators with no decimal.
/// parse_amount_str now recognizes multiple dots as thousands when
/// there's no comma present.
use laminate::packs::currency::parse_currency;

#[test]
fn european_thousands_no_decimal() {
    let result = parse_currency("€1.234.567");
    assert_eq!(result, Some((1234567.0, Some("EUR".to_string()))));
}

#[test]
fn us_thousands_no_decimal() {
    let result = parse_currency("$1,234,567");
    assert_eq!(result, Some((1234567.0, Some("USD".to_string()))));
}

#[test]
fn single_dot_still_decimal() {
    // Single dot is always decimal: "$12.99" → 12.99, not 1299
    let result = parse_currency("$12.99");
    assert_eq!(result, Some((12.99, Some("USD".to_string()))));
}
