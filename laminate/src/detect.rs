//! Type detection module — identify what kind of data a string contains.
//!
//! Given an unknown string, `guess_type()` returns ranked type candidates
//! with confidence scores. This is the "what IS this?" question, answered
//! by orchestrating all domain packs and format detectors.
//!
//! ```
//! use laminate::detect::{guess_type, GuessedType};
//!
//! let guesses = guess_type("$12.99");
//! assert!(guesses[0].kind == GuessedType::Currency);
//!
//! let guesses = guess_type("2026-04-02");
//! assert!(matches!(guesses[0].kind, GuessedType::Date(_)));
//! ```

use crate::packs::currency::{detect_currency_format, CurrencyFormat};
use crate::packs::time::{detect_format, DateFormat};
use crate::packs::units::{parse_unit_value, UnitCategory};

/// A single type guess with confidence score.
#[derive(Debug, Clone)]
pub struct TypeGuess {
    /// The detected type.
    pub kind: GuessedType,
    /// Confidence score from 0.0 (unlikely) to 1.0 (certain).
    pub confidence: f64,
}

/// Possible detected types for an unknown string value.
#[derive(Debug, Clone, PartialEq)]
pub enum GuessedType {
    /// Parseable as integer (i64).
    Integer,
    /// Parseable as float (f64).
    Float,
    /// Boolean value ("true", "false", "yes", "no", "1", "0", etc.).
    Boolean,
    /// Date or datetime in a recognized format.
    Date(DateFormat),
    /// Currency amount with symbol or code.
    Currency,
    /// Value with a unit suffix (weight, length, temperature, etc.).
    UnitValue(UnitCategory),
    /// JSON object or array as a string.
    Json,
    /// UUID format (8-4-4-4-12 hex).
    Uuid,
    /// Email address pattern.
    Email,
    /// URL/URI pattern.
    Url,
    /// IP address (v4 or v6).
    IpAddress,
    /// Null sentinel ("null", "none", "N/A", etc.).
    NullSentinel,
    /// IBAN (International Bank Account Number).
    Iban,
    /// Credit card number (Luhn-valid).
    CreditCard,
    /// ISBN (10 or 13).
    Isbn,
    /// US Social Security Number.
    Ssn,
    /// US Employer Identification Number.
    Ein,
    /// EU VAT number.
    VatNumber,
    /// Phone number.
    Phone,
    /// Plain string (no special format detected).
    PlainString,
}

/// Detect what type(s) an unknown string value might represent.
///
/// Returns a list of type guesses sorted by confidence (highest first).
/// Multiple types may match — e.g., "42" is both an Integer (high confidence)
/// and a Float (lower confidence).
///
/// This function is the "what IS this?" question for unknown data. It
/// orchestrates all domain packs and format detectors.
pub fn guess_type(s: &str) -> Vec<TypeGuess> {
    let s = s.trim();

    let mut guesses = Vec::new();

    // Null sentinels — check first (common in messy data).
    // Empty/whitespace-only strings are the #1 most common null sentinel in
    // CSV and database exports (pandas treats "" as NaN by default), so they
    // must reach this check — NOT short-circuit to PlainString.
    let lower = s.to_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "none" | "nil" | "n/a" | "na" | "nan" | "undefined" | "-" | "" | "unknown"
    ) {
        guesses.push(TypeGuess {
            kind: GuessedType::NullSentinel,
            confidence: 0.95,
        });
    }

    // Boolean — check before number (since "1"/"0" are both)
    if matches!(
        lower.as_str(),
        "true" | "false" | "yes" | "no" | "y" | "n" | "on" | "off" | "t" | "f"
    ) {
        guesses.push(TypeGuess {
            kind: GuessedType::Boolean,
            confidence: 0.9,
        });
    }

    // Integer — fast path
    if s.parse::<i64>().is_ok() {
        // "1" and "0" are more commonly Boolean in datasets than standalone integers
        let is_bool_like = s == "1" || s == "0";
        guesses.push(TypeGuess {
            kind: GuessedType::Integer,
            confidence: if is_bool_like { 0.5 } else { 0.95 },
        });
        // Also a valid float (demote for bool-like values)
        guesses.push(TypeGuess {
            kind: GuessedType::Float,
            confidence: if is_bool_like { 0.5 } else { 0.7 },
        });
    } else if let Ok(f) = s.parse::<f64>() {
        // Float but not integer
        if !f.is_nan() && !f.is_infinite() {
            guesses.push(TypeGuess {
                kind: GuessedType::Float,
                confidence: 0.95,
            });
        }
    }

    // "1" and "0" are also booleans — in many datasets these represent true/false.
    // Higher confidence than before (0.6) since they're equally valid as Boolean or Integer.
    if s == "1" || s == "0" {
        guesses.push(TypeGuess {
            kind: GuessedType::Boolean,
            confidence: 0.6,
        });
    }

    // Email pattern (check BEFORE date — "alice@example.com" has "am" which
    // tricks the time pack into detecting Time12)
    // But skip email early-return for URLs with @ (like https://user:pass@host)
    let is_url_prefix =
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://");
    if !is_url_prefix && s.contains('@') && s.contains('.') && !s.contains(' ') {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.') {
            guesses.push(TypeGuess {
                kind: GuessedType::Email,
                confidence: 0.9,
            });
            // Skip date detection for emails
            guesses.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return guesses;
        }
    }

    // Date/time detection
    let date_format = detect_format(s);
    match date_format {
        DateFormat::Unknown => {}
        DateFormat::Ambiguous => {
            guesses.push(TypeGuess {
                kind: GuessedType::Date(date_format),
                confidence: 0.6,
            });
        }
        _ => {
            guesses.push(TypeGuess {
                kind: GuessedType::Date(date_format),
                confidence: 0.85,
            });
        }
    }

    // Currency detection
    let currency_format = detect_currency_format(s);
    match currency_format {
        CurrencyFormat::NotCurrency | CurrencyFormat::PlainNumber => {}
        CurrencyFormat::EuropeanLocale { .. } => {
            guesses.push(TypeGuess {
                kind: GuessedType::Currency,
                confidence: 0.7,
            });
        }
        _ => {
            guesses.push(TypeGuess {
                kind: GuessedType::Currency,
                confidence: 0.9,
            });
        }
    }

    // Accounting negative: (1,234.56) — parenthesized number
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        // Check if inner is a number with optional comma thousands
        let stripped: String = inner.chars().filter(|c| *c != ',').collect();
        if stripped.parse::<f64>().is_ok() {
            guesses.push(TypeGuess {
                kind: GuessedType::Currency,
                confidence: 0.85,
            });
        }
    }

    // Unit value detection (includes qualified weight like "G.W. 15.5kg")
    if let Some(uv) = parse_unit_value(s) {
        guesses.push(TypeGuess {
            kind: GuessedType::UnitValue(uv.category),
            confidence: 0.85,
        });
    } else if crate::packs::units::parse_qualified_weight(s).is_some() {
        guesses.push(TypeGuess {
            kind: GuessedType::UnitValue(crate::packs::units::UnitCategory::Weight),
            confidence: 0.85,
        });
    } else if crate::packs::units::parse_pack_notation(s).is_some() {
        guesses.push(TypeGuess {
            kind: GuessedType::UnitValue(crate::packs::units::UnitCategory::Weight),
            confidence: 0.8,
        });
    }

    // UUID pattern (8-4-4-4-12 hex)
    if s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4 {
        let hex_parts: Vec<&str> = s.split('-').collect();
        if hex_parts.len() == 5
            && hex_parts[0].len() == 8
            && hex_parts[1].len() == 4
            && hex_parts[2].len() == 4
            && hex_parts[3].len() == 4
            && hex_parts[4].len() == 12
            && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
        {
            guesses.push(TypeGuess {
                kind: GuessedType::Uuid,
                confidence: 0.95,
            });
        }
    }

    // Email already checked above (early return)

    // URL pattern
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://") {
        guesses.push(TypeGuess {
            kind: GuessedType::Url,
            confidence: 0.95,
        });
    }

    // IP address (v4)
    let looks_like_ipv4 = s.split('.').count() == 4
        && s.split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    if looks_like_ipv4 {
        if s.split('.').all(|p| p.parse::<u8>().is_ok()) {
            guesses.push(TypeGuess {
                kind: GuessedType::IpAddress,
                confidence: 0.9,
            });
        } else {
            // Looks like an IP but invalid octets — suppress further detection
            // (it's not a phone number or identifier)
            guesses.push(TypeGuess {
                kind: GuessedType::PlainString,
                confidence: 0.95,
            });
            guesses.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return guesses;
        }
    }

    // IPv6 (basic check)
    if s.contains(':')
        && s.split(':').count() >= 3
        && s.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
    {
        guesses.push(TypeGuess {
            kind: GuessedType::IpAddress,
            confidence: 0.8,
        });
    }

    // JSON object or array
    let trimmed = s.trim();
    if ((trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        && serde_json::from_str::<serde_json::Value>(s).is_ok()
    {
        guesses.push(TypeGuess {
            kind: GuessedType::Json,
            confidence: 0.95,
        });
    }

    // IBAN format detection (pattern-only, without MOD-97 checksum validation).
    // This catches formatted IBANs with spaces like "GB23 IHAV 7261 1691 57".
    {
        let stripped: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let upper = stripped.to_uppercase();
        if upper.len() >= 15
            && upper.len() <= 34
            && upper.is_ascii()
            && upper[..2].chars().all(|c| c.is_ascii_alphabetic())
            && upper[2..4].chars().all(|c| c.is_ascii_digit())
            && upper[4..].chars().all(|c| c.is_ascii_alphanumeric())
        {
            // Reject obviously invalid patterns: check digits 00/01, all-same body
            let check: u32 = upper[2..4].parse().unwrap_or(0);
            let body = &upper[4..];
            let has_variety = body.chars().collect::<std::collections::HashSet<_>>().len() > 2;
            if check >= 2 && has_variety {
                guesses.push(TypeGuess {
                    kind: GuessedType::Iban,
                    confidence: 0.8,
                });
            }
        }
    }

    // Identifier detection (IBAN, credit card, ISBN, SSN, etc.)
    let id_candidates = crate::packs::identifiers::detect(s);
    for (id_type, confidence) in id_candidates {
        let kind = match id_type {
            crate::packs::identifiers::IdentifierType::Iban => GuessedType::Iban,
            crate::packs::identifiers::IdentifierType::CreditCard => GuessedType::CreditCard,
            crate::packs::identifiers::IdentifierType::Isbn10
            | crate::packs::identifiers::IdentifierType::Isbn13 => GuessedType::Isbn,
            crate::packs::identifiers::IdentifierType::UsSsn => GuessedType::Ssn,
            crate::packs::identifiers::IdentifierType::UsEin => GuessedType::Ein,
            crate::packs::identifiers::IdentifierType::UsNpi
            | crate::packs::identifiers::IdentifierType::UkNhs => continue, // skip niche
            crate::packs::identifiers::IdentifierType::EuVat => GuessedType::VatNumber,
            crate::packs::identifiers::IdentifierType::Uuid => continue, // already detected above
            crate::packs::identifiers::IdentifierType::Email => continue, // already detected above
            crate::packs::identifiers::IdentifierType::Phone => GuessedType::Phone,
        };
        guesses.push(TypeGuess { kind, confidence });
    }

    // When a specific identifier (CreditCard, ISBN, etc.) was detected with high
    // confidence on an all-digit string, demote Integer since the digits have
    // semantic meaning beyond being a number.
    let has_high_confidence_id = guesses.iter().any(|g| {
        matches!(g.kind, GuessedType::CreditCard | GuessedType::Isbn) && g.confidence >= 0.85
    });
    if has_high_confidence_id {
        for g in guesses.iter_mut() {
            if g.kind == GuessedType::Integer {
                g.confidence = 0.5;
            }
        }
    }

    // When all-digit string matches a specific date format (Hl7Date, Unix timestamps),
    // demote Integer — the digits represent a date, not a number.
    // Don't demote for YearOnly (4-digit years are usually integers).
    let has_specific_date_on_digits = guesses.iter().any(|g| {
        matches!(
            g.kind,
            GuessedType::Date(DateFormat::Hl7Date)
                | GuessedType::Date(DateFormat::UnixSeconds)
                | GuessedType::Date(DateFormat::UnixMillis)
        )
    }) && s.chars().all(|c| c.is_ascii_digit());
    if has_specific_date_on_digits {
        for g in guesses.iter_mut() {
            if g.kind == GuessedType::Integer || g.kind == GuessedType::Float {
                g.confidence = 0.4;
            }
        }
    }

    // If nothing matched, it's a plain string
    if guesses.is_empty() {
        guesses.push(TypeGuess {
            kind: GuessedType::PlainString,
            confidence: 1.0,
        });
    }

    // Sort by confidence (highest first)
    guesses.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    guesses
}
