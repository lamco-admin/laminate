//! Currency coercion pack.
//!
//! Handles currency symbol stripping, locale-aware decimal parsing,
//! and currency format detection.

use serde_json::Value;

use crate::coerce::CoercionResult;
use crate::diagnostic::{Diagnostic, DiagnosticKind, RiskLevel};

/// Recognized currency format patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyFormat {
    /// Symbol prefix: "$12.99", "€24.50", "£3.49"
    SymbolPrefix {
        /// The currency symbol (e.g., "$", "€").
        symbol: String,
        /// The numeric amount string.
        amount: String,
    },
    /// Code prefix: "EUR 12.99", "USD 1,234.56"
    CodePrefix {
        /// The ISO 4217 currency code.
        code: String,
        /// The numeric amount string.
        amount: String,
    },
    /// Code suffix: "12.99 USD", "24.50 EUR"
    CodeSuffix {
        /// The ISO 4217 currency code.
        code: String,
        /// The numeric amount string.
        amount: String,
    },
    /// European locale: "2.450,75" (dot for thousands, comma for decimal)
    EuropeanLocale {
        /// The normalized amount string (European separators converted).
        amount: String,
    },
    /// Plain number that could be currency: "12.99"
    PlainNumber,
    /// Not a currency value
    NotCurrency,
}

/// Known currency symbols.
// Sorted longest-first so "A$" matches before "$", "HK$" before "$", etc.
const CURRENCY_SYMBOLS: &[(&str, &str)] = &[
    ("HK$", "HKD"),
    ("NT$", "TWD"),
    ("NZ$", "NZD"),
    ("A$", "AUD"),
    ("C$", "CAD"),
    ("R$", "BRL"),
    ("S$", "SGD"),
    ("Z$", "ZWD"),
    ("CHF", "CHF"),
    ("kr", "SEK"), // Also DKK, NOK
    ("zł", "PLN"),
    ("$", "USD"),
    ("€", "EUR"),
    ("£", "GBP"),
    ("¥", "JPY"),
    ("₹", "INR"),
    ("₩", "KRW"),
    ("₽", "RUB"),
    ("₺", "TRY"),
];

/// Known currency codes.
const CURRENCY_CODES: &[&str] = &[
    "USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "INR", "KRW", "BRL", "MXN", "SGD",
    "HKD", "NOK", "SEK", "DKK", "NZD", "ZAR", "RUB", "TRY", "PLN", "THB", "TWD", "ILS", "AED",
    "SAR", "BTC", "ETH",
];

/// Detect the currency format of a string value.
pub fn detect_currency_format(s: &str) -> CurrencyFormat {
    let s = s.trim();
    if s.is_empty() {
        return CurrencyFormat::NotCurrency;
    }

    // Check for negative: "-$12.99" or accounting parentheses "($12.99)"
    let (is_negative, s_stripped) = if s.starts_with('(') && s.ends_with(')') {
        // Accounting negative format: (12.99), ($12.99), (12.99 USD)
        (true, &s[1..s.len() - 1])
    } else if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s)
    };

    // Check for symbol prefix (including after minus sign)
    for (symbol, _code) in CURRENCY_SYMBOLS {
        if let Some(rest) = s_stripped.strip_prefix(symbol) {
            let amount_str = rest.trim();
            if is_numeric_string(amount_str) {
                let amount = if is_negative {
                    format!("-{amount_str}")
                } else {
                    amount_str.to_string()
                };
                return CurrencyFormat::SymbolPrefix {
                    symbol: symbol.to_string(),
                    amount,
                };
            }
        }
    }

    // Check for code prefix: "EUR 12.99", "USD 1,234.56", "EUR 1.234,56"
    let prefix_parts: Vec<&str> = s_stripped.splitn(2, ' ').collect();
    if prefix_parts.len() == 2 {
        let potential_code = prefix_parts[0].to_uppercase();
        if CURRENCY_CODES.contains(&potential_code.as_str()) && is_numeric_string(prefix_parts[1]) {
            let amount = if is_negative {
                format!("-{}", prefix_parts[1])
            } else {
                prefix_parts[1].to_string()
            };
            return CurrencyFormat::CodePrefix {
                code: potential_code,
                amount,
            };
        }
    }

    // Check for code suffix: "12.99 USD", "(1,234.56 USD)"
    let parts: Vec<&str> = s_stripped.rsplitn(2, ' ').collect();
    if parts.len() == 2 {
        let potential_code = parts[0].to_uppercase();
        if CURRENCY_CODES.contains(&potential_code.as_str()) && is_numeric_string(parts[1]) {
            let amount = if is_negative {
                format!("-{}", parts[1])
            } else {
                parts[1].to_string()
            };
            return CurrencyFormat::CodeSuffix {
                code: potential_code,
                amount,
            };
        }
    }

    // Check for European locale format: "2.450,75"
    if s_stripped.contains(',') && s_stripped.contains('.') {
        let dot_pos = s_stripped
            .rfind('.')
            .expect("guarded by contains check above");
        let comma_pos = s_stripped
            .rfind(',')
            .expect("guarded by contains check above");
        if comma_pos > dot_pos {
            // Comma is the decimal separator (European)
            let sign = if is_negative { "-" } else { "" };
            let normalized = format!("{sign}{}", s_stripped.replace('.', "").replace(',', "."));
            if normalized.parse::<f64>().is_ok() {
                return CurrencyFormat::EuropeanLocale { amount: normalized };
            }
        }
    }

    CurrencyFormat::NotCurrency
}

/// Strip currency symbols/codes and parse to a numeric value.
///
/// Returns the numeric amount and the detected currency code (if any).
pub fn parse_currency(s: &str) -> Option<(f64, Option<String>)> {
    let format = detect_currency_format(s);
    match format {
        CurrencyFormat::SymbolPrefix { symbol, amount } => {
            let code = CURRENCY_SYMBOLS
                .iter()
                .find(|(sym, _)| *sym == symbol)
                .map(|(_, code)| code.to_string());
            parse_amount_str(&amount).map(|v| (v, code))
        }
        CurrencyFormat::CodePrefix { code, amount } => {
            parse_amount_str(&amount).map(|v| (v, Some(code)))
        }
        CurrencyFormat::CodeSuffix { code, amount } => {
            parse_amount_str(&amount).map(|v| (v, Some(code)))
        }
        CurrencyFormat::EuropeanLocale { amount } => amount.parse::<f64>().ok().map(|v| (v, None)),
        CurrencyFormat::PlainNumber | CurrencyFormat::NotCurrency => None,
    }
}

/// Coerce a currency string to a numeric value with diagnostics.
pub fn coerce_currency(value: &Value, path: &str) -> CoercionResult {
    match value {
        Value::String(s) => {
            if let Some((amount, currency_code)) = parse_currency(s) {
                let code_info = currency_code.as_deref().unwrap_or("unknown");

                let new_value = serde_json::Number::from_f64(amount)
                    .map(Value::Number)
                    .unwrap_or_else(|| value.clone());

                CoercionResult {
                    value: new_value,
                    coerced: true,
                    diagnostic: Some(Diagnostic {
                        path: path.to_string(),
                        kind: DiagnosticKind::Coerced {
                            from: format!("currency string ({code_info})"),
                            to: "f64".into(),
                        },
                        risk: RiskLevel::Warning,
                        suggestion: Some(format!(
                            "currency symbol stripped from '{s}'; consider using a Decimal type \
                             for financial precision and preserving the currency code ({code_info})"
                        )),
                    }),
                }
            } else {
                CoercionResult {
                    value: value.clone(),
                    coerced: false,
                    diagnostic: None,
                }
            }
        }
        _ => CoercionResult {
            value: value.clone(),
            coerced: false,
            diagnostic: None,
        },
    }
}

/// Parse an amount string, auto-detecting European locale (dot-thousands, comma-decimal).
///
/// "1,234.56" → 1234.56 (US format)
/// "1.234,56" → 1234.56 (European format)
/// "1234"     → 1234.0  (no separators)
fn parse_amount_str(s: &str) -> Option<f64> {
    let s = normalize_fullwidth(s);
    let has_comma = s.contains(',');
    let has_dot = s.contains('.');

    if has_comma && has_dot {
        let last_comma = s.rfind(',').expect("guarded by has_comma check above");
        let last_dot = s.rfind('.').expect("guarded by has_dot check above");
        if last_comma > last_dot {
            // European: dot is thousands, comma is decimal → "1.234,56"
            let normalized = s.replace('.', "").replace(',', ".");
            return normalized.parse::<f64>().ok();
        }
        // US: comma is thousands, dot is decimal → "1,234.56"
    }

    // Check for multiple dots with no comma — European thousands-only format
    // e.g., "1.234.567" → 1234567
    if !has_comma && has_dot && s.matches('.').count() > 1 {
        let stripped = s.replace('.', "");
        return stripped.parse::<f64>().ok();
    }

    // Strip formatting (commas, spaces, apostrophes) and parse
    strip_amount_formatting(&s).parse::<f64>().ok()
}

fn is_numeric_string(s: &str) -> bool {
    let s = normalize_fullwidth(s.trim());
    let s = s.replace([',', ' ', '\''], "");
    // Handle multiple dots as thousands separators (European: "1.234.567")
    if s.matches('.').count() > 1 {
        return s.replace('.', "").parse::<f64>().is_ok();
    }
    s.parse::<f64>().is_ok()
}

/// Strip formatting characters (commas, spaces, apostrophes) from an amount string.
fn strip_amount_formatting(s: &str) -> String {
    let s = normalize_fullwidth(s);
    s.replace([',', ' ', '\''], "")
}

/// Normalize full-width digits and punctuation to ASCII equivalents.
///
/// Japanese text frequently uses full-width characters:
/// - ０-９ (U+FF10-FF19) → 0-9
/// - ，  (U+FF0C) → ,
/// - ．  (U+FF0E) → .
fn normalize_fullwidth(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{FF10}'..='\u{FF19}' => (c as u32 - 0xFF10 + b'0' as u32) as u8 as char,
            '\u{FF0C}' => ',',
            '\u{FF0E}' => '.',
            _ => c,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Built-in reference exchange rates
// ---------------------------------------------------------------------------

/// Static reference exchange rates for development and testing.
///
/// These are approximate USD-based rates (as of early 2026) suitable for:
/// - Unit tests and integration testing
/// - Development and prototyping
/// - Rough estimates where precision isn't critical
///
/// **Not suitable for financial transactions.** For production use, implement
/// [`CoercionDataSource`](crate::coerce::CoercionDataSource) with a live rate provider (e.g., ECB, Open Exchange Rates).
///
/// # Usage
///
/// ```
/// use laminate::FlexValue;
/// use laminate::packs::currency::BuiltinRates;
///
/// let rates = BuiltinRates::new();
/// assert!(rates.rate("USD", "EUR").is_some());
///
/// // Use with FlexValue for currency-aware extraction
/// let val = FlexValue::from_json(r#"{"price": "$12.99"}"#).unwrap()
///     .with_data_source(rates);
/// ```
#[derive(Debug, Clone)]
pub struct BuiltinRates {
    /// When these rates were last updated (informational only).
    pub as_of: &'static str,
}

impl BuiltinRates {
    /// Create a new `BuiltinRates` instance.
    pub fn new() -> Self {
        Self { as_of: "2026-01" }
    }

    /// Get the exchange rate from one currency to another.
    ///
    /// Returns `None` if either currency is unknown.
    pub fn rate(&self, from: &str, to: &str) -> Option<f64> {
        if from == to {
            return Some(1.0);
        }
        let from_usd = usd_rate(from)?;
        let to_usd = usd_rate(to)?;
        Some(to_usd / from_usd)
    }

    /// Convert an amount from one currency to another.
    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Option<f64> {
        self.rate(from, to).map(|r| amount * r)
    }
}

impl Default for BuiltinRates {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::coerce::CoercionDataSource for BuiltinRates {
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64> {
        self.rate(from, to)
    }
}

/// Reference rates: how many units of currency X equal 1 USD.
///
/// Source: approximate market rates as of January 2026. These are for
/// development/testing only — do not use for financial transactions.
fn usd_rate(code: &str) -> Option<f64> {
    let rate = match code.to_uppercase().as_str() {
        "USD" => 1.0,
        "EUR" => 0.92,
        "GBP" => 0.79,
        "JPY" => 148.0,
        "CAD" => 1.36,
        "AUD" => 1.55,
        "CHF" => 0.88,
        "CNY" => 7.24,
        "INR" => 83.5,
        "KRW" => 1310.0,
        "BRL" => 4.97,
        "MXN" => 17.2,
        "SGD" => 1.34,
        "HKD" => 7.82,
        "NOK" => 10.5,
        "SEK" => 10.3,
        "DKK" => 6.87,
        "NZD" => 1.63,
        "ZAR" => 18.8,
        "RUB" => 89.0,
        "TRY" => 30.2,
        "PLN" => 4.01,
        "THB" => 35.1,
        "TWD" => 31.5,
        "ILS" => 3.65,
        "AED" => 3.67,
        "SAR" => 3.75,
        "PHP" => 56.0,
        "IDR" => 15700.0,
        "MYR" => 4.72,
        "CZK" => 23.1,
        "HUF" => 365.0,
        "CLP" => 910.0,
        "COP" => 3950.0,
        "ARS" => 830.0,
        "EGP" => 30.9,
        "NGN" => 890.0,
        "KES" => 156.0,
        "VND" => 24500.0,
        _ => return None,
    };
    Some(rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_usd_prefix() {
        match detect_currency_format("$12.99") {
            CurrencyFormat::SymbolPrefix { symbol, amount } => {
                assert_eq!(symbol, "$");
                assert_eq!(amount, "12.99");
            }
            other => panic!("expected SymbolPrefix, got {other:?}"),
        }
    }

    #[test]
    fn detect_euro_prefix() {
        match detect_currency_format("€24.50") {
            CurrencyFormat::SymbolPrefix { symbol, .. } => {
                assert_eq!(symbol, "€");
            }
            other => panic!("expected SymbolPrefix, got {other:?}"),
        }
    }

    #[test]
    fn detect_gbp_prefix() {
        match detect_currency_format("£3.49") {
            CurrencyFormat::SymbolPrefix { symbol, .. } => {
                assert_eq!(symbol, "£");
            }
            other => panic!("expected SymbolPrefix, got {other:?}"),
        }
    }

    #[test]
    fn detect_code_suffix() {
        match detect_currency_format("7.99 USD") {
            CurrencyFormat::CodeSuffix { code, amount } => {
                assert_eq!(code, "USD");
                assert_eq!(amount, "7.99");
            }
            other => panic!("expected CodeSuffix, got {other:?}"),
        }
    }

    #[test]
    fn detect_european_locale() {
        match detect_currency_format("2.450,75") {
            CurrencyFormat::EuropeanLocale { amount } => {
                assert_eq!(amount, "2450.75");
            }
            other => panic!("expected EuropeanLocale, got {other:?}"),
        }
    }

    #[test]
    fn detect_not_currency() {
        assert_eq!(detect_currency_format("hello"), CurrencyFormat::NotCurrency);
        assert_eq!(detect_currency_format(""), CurrencyFormat::NotCurrency);
    }

    #[test]
    fn parse_usd() {
        let (amount, code) = parse_currency("$12.99").unwrap();
        assert!((amount - 12.99).abs() < f64::EPSILON);
        assert_eq!(code, Some("USD".into()));
    }

    #[test]
    fn parse_european() {
        let (amount, code) = parse_currency("2.450,75").unwrap();
        assert!((amount - 2450.75).abs() < f64::EPSILON);
        assert_eq!(code, None);
    }

    #[test]
    fn parse_code_suffix() {
        let (amount, code) = parse_currency("7.99 USD").unwrap();
        assert!((amount - 7.99).abs() < f64::EPSILON);
        assert_eq!(code, Some("USD".into()));
    }

    #[test]
    fn coerce_strips_symbol() {
        let result = coerce_currency(&Value::String("$12.99".into()), "price");
        assert!(result.coerced);
        assert_eq!(result.diagnostic.unwrap().risk, RiskLevel::Warning);
    }

    #[test]
    fn coerce_non_currency_unchanged() {
        let result = coerce_currency(&Value::String("hello".into()), "price");
        assert!(!result.coerced);
    }

    // ── BuiltinRates tests ──

    #[test]
    fn builtin_same_currency() {
        let rates = BuiltinRates::new();
        assert_eq!(rates.rate("USD", "USD"), Some(1.0));
        assert_eq!(rates.rate("EUR", "EUR"), Some(1.0));
    }

    #[test]
    fn builtin_usd_to_eur() {
        let rates = BuiltinRates::new();
        let rate = rates.rate("USD", "EUR").unwrap();
        // Should be approximately 0.92
        assert!(rate > 0.8 && rate < 1.1, "USD→EUR rate {rate} out of range");
    }

    #[test]
    fn builtin_eur_to_usd() {
        let rates = BuiltinRates::new();
        let rate = rates.rate("EUR", "USD").unwrap();
        // Should be approximately 1/0.92 ≈ 1.087
        assert!(rate > 0.9 && rate < 1.3, "EUR→USD rate {rate} out of range");
    }

    #[test]
    fn builtin_cross_rate() {
        let rates = BuiltinRates::new();
        // GBP → JPY should work via USD pivot
        let rate = rates.rate("GBP", "JPY").unwrap();
        assert!(
            rate > 150.0 && rate < 250.0,
            "GBP→JPY rate {rate} out of range"
        );
    }

    #[test]
    fn builtin_convert() {
        let rates = BuiltinRates::new();
        let result = rates.convert(100.0, "USD", "EUR").unwrap();
        assert!(result > 80.0 && result < 110.0, "100 USD → {result} EUR");
    }

    #[test]
    fn builtin_unknown_currency() {
        let rates = BuiltinRates::new();
        assert_eq!(rates.rate("USD", "XYZ"), None);
        assert_eq!(rates.rate("XYZ", "USD"), None);
    }

    #[test]
    fn builtin_case_insensitive() {
        let rates = BuiltinRates::new();
        // usd_rate does to_uppercase internally
        assert!(rates.rate("usd", "eur").is_some());
    }

    #[test]
    fn builtin_roundtrip() {
        let rates = BuiltinRates::new();
        // Converting USD→EUR→USD should get back approximately the same amount
        let eur = rates.convert(100.0, "USD", "EUR").unwrap();
        let back = rates.convert(eur, "EUR", "USD").unwrap();
        assert!(
            (back - 100.0).abs() < 0.01,
            "roundtrip: 100 → {eur} → {back}"
        );
    }

    #[test]
    fn builtin_implements_coercion_data_source() {
        use crate::coerce::CoercionDataSource;
        let rates = BuiltinRates::new();
        let rate = rates.exchange_rate("USD", "GBP").unwrap();
        assert!(rate > 0.5 && rate < 1.0);
    }
}
