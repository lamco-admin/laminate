//! Identifier detection and validation pack.
//!
//! Unified detection and validation for common identifier formats:
//! IBAN, credit cards (Luhn + BIN), ISBN, US SSN/EIN, EU VAT,
//! UUID, email, US NPI, UK NHS Number, and basic phone numbers.
//!
//! ```
//! use laminate::packs::identifiers::{validate, IdentifierType, ValidationResult};
//!
//! let result = validate("GB29 NWBK 6016 1331 9268 19", IdentifierType::Iban);
//! assert!(result.is_valid);
//!
//! let result = validate("4111111111111111", IdentifierType::CreditCard);
//! assert!(result.is_valid);
//! assert_eq!(result.detail.as_deref(), Some("Visa"));
//! ```

/// Supported identifier types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierType {
    /// International Bank Account Number (mod-97 check).
    Iban,
    /// Credit/debit card number (Luhn algorithm + BIN detection).
    CreditCard,
    /// ISBN-10 (10-digit, legacy format).
    Isbn10,
    /// ISBN-13 (13-digit, current format).
    Isbn13,
    /// US Social Security Number (XXX-XX-XXXX).
    UsSsn,
    /// US Employer Identification Number (XX-XXXXXXX).
    UsEin,
    /// US National Provider Identifier (10-digit, Luhn check).
    UsNpi,
    /// UK National Health Service number (10-digit, mod-11 check).
    UkNhs,
    /// EU Value Added Tax number (country prefix + digits).
    EuVat,
    /// UUID (8-4-4-4-12 hexadecimal).
    Uuid,
    /// Email address (basic pattern match).
    Email,
    /// Phone number (E.164 or common formats).
    Phone,
}

/// Result of validating an identifier.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the identifier is valid.
    pub is_valid: bool,
    /// The normalized identifier (stripped of separators).
    pub normalized: String,
    /// Additional detail (card brand, country, etc.).
    pub detail: Option<String>,
    /// Why validation failed, if applicable.
    pub error: Option<String>,
}

impl ValidationResult {
    fn valid(normalized: String, detail: Option<String>) -> Self {
        Self {
            is_valid: true,
            normalized,
            detail,
            error: None,
        }
    }
    fn invalid(normalized: String, error: &str) -> Self {
        Self {
            is_valid: false,
            normalized,
            detail: None,
            error: Some(error.to_string()),
        }
    }
}

/// Validate an identifier against a specific type.
pub fn validate(s: &str, id_type: IdentifierType) -> ValidationResult {
    let s = s.trim();
    match id_type {
        IdentifierType::Iban => validate_iban(s),
        IdentifierType::CreditCard => validate_credit_card(s),
        IdentifierType::Isbn10 => validate_isbn10(s),
        IdentifierType::Isbn13 => validate_isbn13(s),
        IdentifierType::UsSsn => validate_us_ssn(s),
        IdentifierType::UsEin => validate_us_ein(s),
        IdentifierType::UsNpi => validate_us_npi(s),
        IdentifierType::UkNhs => validate_uk_nhs(s),
        IdentifierType::EuVat => validate_eu_vat(s),
        IdentifierType::Uuid => validate_uuid(s),
        IdentifierType::Email => validate_email(s),
        IdentifierType::Phone => validate_phone(s),
    }
}

/// Detect which identifier type(s) a string might be.
/// Returns candidates sorted by confidence.
pub fn detect(s: &str) -> Vec<(IdentifierType, f64)> {
    let s = s.trim();
    let mut candidates = Vec::new();

    // Try each type and collect valid ones with confidence
    let types = [
        IdentifierType::Uuid,
        IdentifierType::Iban,
        IdentifierType::CreditCard,
        IdentifierType::Isbn13,
        IdentifierType::Isbn10,
        IdentifierType::Email,
        IdentifierType::EuVat,
        IdentifierType::UsNpi,
        IdentifierType::UkNhs,
        IdentifierType::UsSsn,
        IdentifierType::UsEin,
        IdentifierType::Phone,
    ];

    for id_type in types {
        let result = validate(s, id_type);
        if result.is_valid {
            let confidence = match id_type {
                IdentifierType::Uuid => 0.98,       // Very specific format
                IdentifierType::Iban => 0.95,       // Checksum verified
                IdentifierType::Email => 0.90,      // Pattern match
                IdentifierType::CreditCard => 0.90, // Luhn verified
                IdentifierType::Isbn13 => 0.90,     // Checksum verified
                IdentifierType::Isbn10 => 0.85,     // Checksum verified
                IdentifierType::UsNpi => 0.80,      // Luhn verified
                IdentifierType::UkNhs => 0.75,      // Mod-11 verified
                IdentifierType::EuVat => 0.75,      // Country-specific
                IdentifierType::UsSsn => 0.50,      // Format only (no checksum)
                IdentifierType::UsEin => 0.50,      // Format only
                IdentifierType::Phone => 0.40,      // Very loose pattern
            };
            candidates.push((id_type, confidence));
        }
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates
}

// ── IBAN ────────────────────────────────────────────────────────

fn validate_iban(s: &str) -> ValidationResult {
    let normalized: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let upper = normalized.to_uppercase();

    if upper.len() < 15 || upper.len() > 34 || !upper.is_ascii() {
        return ValidationResult::invalid(upper, "IBAN must be 15-34 ASCII characters");
    }

    // First 2 chars must be letters (country), next 2 digits (check)
    let country = &upper[..2];
    if !country.chars().all(|c| c.is_ascii_alphabetic()) {
        return ValidationResult::invalid(upper, "IBAN must start with 2-letter country code");
    }
    if !upper[2..4].chars().all(|c| c.is_ascii_digit()) {
        return ValidationResult::invalid(upper, "IBAN check digits must be numeric");
    }

    // MOD-97 validation: move first 4 chars to end, convert letters to numbers
    let rearranged = format!("{}{}", &upper[4..], &upper[..4]);
    let numeric: String = rearranged
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                format!("{}", (c as u32) - ('A' as u32) + 10)
            } else {
                c.to_string()
            }
        })
        .collect();

    // Calculate mod 97 on the large number (process in chunks to avoid overflow)
    let mut remainder: u64 = 0;
    for chunk in numeric.as_bytes().chunks(9) {
        let chunk_str = std::str::from_utf8(chunk).unwrap_or("0");
        let combined = format!("{}{}", remainder, chunk_str);
        remainder = combined.parse::<u64>().unwrap_or(0) % 97;
    }

    if remainder == 1 {
        ValidationResult::valid(upper.clone(), Some(country.to_string()))
    } else {
        ValidationResult::invalid(upper, "IBAN checksum failed (MOD-97)")
    }
}

// ── Credit Card (Luhn + BIN) ────────────────────────────────────

fn validate_credit_card(s: &str) -> ValidationResult {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() < 13 || digits.len() > 19 {
        return ValidationResult::invalid(digits, "credit card must be 13-19 digits");
    }

    // Reject all-zeros (technically passes Luhn but not a valid card)
    if digits.chars().all(|c| c == '0') {
        return ValidationResult::invalid(digits, "credit card cannot be all zeros");
    }

    // Luhn algorithm
    if !luhn_check(&digits) {
        return ValidationResult::invalid(digits, "Luhn checksum failed");
    }

    // BIN prefix detection
    let brand = detect_card_brand(&digits);
    ValidationResult::valid(digits, Some(brand.to_string()))
}

fn luhn_check(digits: &str) -> bool {
    let mut sum: u32 = 0;
    let mut double = false;
    for c in digits.chars().rev() {
        let mut d = c.to_digit(10).unwrap_or(0);
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }
    sum % 10 == 0
}

fn detect_card_brand(digits: &str) -> &'static str {
    if digits.starts_with('4') {
        return "Visa";
    }
    if digits.len() >= 2 {
        let prefix2: u32 = digits[..2].parse().unwrap_or(0);
        if (51..=55).contains(&prefix2) {
            return "Mastercard";
        }
    }
    // Mastercard 2-series range (2221-2720)
    if digits.len() >= 4 {
        let prefix4: u32 = digits[..4].parse().unwrap_or(0);
        if (2221..=2720).contains(&prefix4) {
            return "Mastercard";
        }
    }
    if digits.starts_with("34") || digits.starts_with("37") {
        return "Amex";
    }
    if digits.starts_with("6011") || digits.starts_with("65") {
        return "Discover";
    }
    // Maestro: 5018, 5020, 5038, 6304, 6759, 6761, 6763
    if digits.starts_with("5018")
        || digits.starts_with("5020")
        || digits.starts_with("5038")
        || digits.starts_with("6304")
        || digits.starts_with("6759")
        || digits.starts_with("6761")
        || digits.starts_with("6763")
    {
        return "Maestro";
    }
    if digits.starts_with("35") {
        return "JCB";
    }
    if digits.starts_with("30") || digits.starts_with("36") || digits.starts_with("38") {
        return "Diners Club";
    }
    "Unknown"
}

// ── ISBN ────────────────────────────────────────────────────────

fn validate_isbn10(s: &str) -> ValidationResult {
    let chars: Vec<char> = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
        .collect();
    if chars.len() != 10 {
        return ValidationResult::invalid(s.to_string(), "ISBN-10 must be 10 characters");
    }

    let mut sum: u32 = 0;
    for (i, c) in chars.iter().enumerate() {
        let val = if *c == 'X' || *c == 'x' {
            10
        } else {
            c.to_digit(10).unwrap_or(0)
        };
        sum += val * (10 - i as u32);
    }

    if sum % 11 == 0 {
        let normalized: String = chars.iter().collect();
        ValidationResult::valid(normalized, None)
    } else {
        ValidationResult::invalid(s.to_string(), "ISBN-10 checksum failed (MOD-11)")
    }
}

fn validate_isbn13(s: &str) -> ValidationResult {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 13 {
        return ValidationResult::invalid(digits, "ISBN-13 must be 13 digits");
    }
    if !digits.starts_with("978") && !digits.starts_with("979") {
        return ValidationResult::invalid(digits, "ISBN-13 must start with 978 or 979");
    }

    let mut sum: u32 = 0;
    for (i, c) in digits.chars().enumerate() {
        let d = c.to_digit(10).unwrap_or(0);
        sum += if i % 2 == 0 { d } else { d * 3 };
    }

    if sum % 10 == 0 {
        ValidationResult::valid(digits, None)
    } else {
        ValidationResult::invalid(digits.clone(), "ISBN-13 checksum failed (MOD-10)")
    }
}

// ── US SSN / EIN ────────────────────────────────────────────────

fn validate_us_ssn(s: &str) -> ValidationResult {
    let trimmed = s.trim();
    // SSN format: NNN-NN-NNNN (dashes at positions 3 and 6)
    let has_format =
        trimmed.len() == 11 && trimmed.as_bytes()[3] == b'-' && trimmed.as_bytes()[6] == b'-';
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 9 {
        return ValidationResult::invalid(digits, "SSN must be 9 digits");
    }
    if !has_format {
        return ValidationResult::invalid(digits, "SSN must be formatted as NNN-NN-NNNN");
    }
    // Area (first 3) cannot be 000, 666, or 900-999
    let area: u32 = digits[..3].parse().unwrap_or(0);
    if area == 0 || area == 666 || area >= 900 {
        return ValidationResult::invalid(digits, "invalid SSN area number");
    }
    // Group (middle 2) cannot be 00
    let group: u32 = digits[3..5].parse().unwrap_or(0);
    if group == 0 {
        return ValidationResult::invalid(digits, "invalid SSN group number");
    }
    // Serial (last 4) cannot be 0000
    let serial: u32 = digits[5..].parse().unwrap_or(0);
    if serial == 0 {
        return ValidationResult::invalid(digits, "invalid SSN serial number");
    }
    let formatted = format!("{}-{}-{}", &digits[..3], &digits[3..5], &digits[5..]);
    ValidationResult::valid(formatted, None)
}

fn validate_us_ein(s: &str) -> ValidationResult {
    let trimmed = s.trim();
    // EIN format must be NN-NNNNNNN (with dash after 2nd digit)
    let has_dash = trimmed.len() == 10 && trimmed.as_bytes()[2] == b'-';
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 9 {
        return ValidationResult::invalid(digits, "EIN must be 9 digits");
    }
    if !has_dash {
        return ValidationResult::invalid(digits, "EIN must be formatted as NN-NNNNNNN");
    }
    // Prefix 00 is not assigned
    let prefix: u32 = digits[..2].parse().unwrap_or(0);
    if prefix == 0 {
        return ValidationResult::invalid(digits, "EIN prefix 00 is not assigned");
    }
    // All-zero serial is invalid
    if digits[2..].chars().all(|c| c == '0') {
        return ValidationResult::invalid(digits, "EIN serial cannot be all zeros");
    }
    let formatted = format!("{}-{}", &digits[..2], &digits[2..]);
    ValidationResult::valid(formatted, None)
}

// ── US NPI (Luhn with 80840 prefix) ─────────────────────────────

fn validate_us_npi(s: &str) -> ValidationResult {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 10 {
        return ValidationResult::invalid(digits, "NPI must be 10 digits");
    }
    // Luhn check with 80840 prefix
    let prefixed = format!("80840{}", digits);
    if luhn_check(&prefixed) {
        ValidationResult::valid(digits, None)
    } else {
        ValidationResult::invalid(digits, "NPI Luhn checksum failed")
    }
}

// ── UK NHS Number (MOD-11) ──────────────────────────────────────

fn validate_uk_nhs(s: &str) -> ValidationResult {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 10 {
        return ValidationResult::invalid(digits, "NHS number must be 10 digits");
    }
    // All-zeros is not a valid NHS number
    if digits.chars().all(|c| c == '0') {
        return ValidationResult::invalid(digits, "NHS number cannot be all zeros");
    }

    let weights = [10, 9, 8, 7, 6, 5, 4, 3, 2];
    let mut sum: u32 = 0;
    for (i, c) in digits[..9].chars().enumerate() {
        sum += c.to_digit(10).unwrap_or(0) * weights[i];
    }
    let remainder = sum % 11;
    let check = if remainder == 0 { 0 } else { 11 - remainder };

    if check == 10 {
        return ValidationResult::invalid(digits, "NHS number invalid (check digit would be 10)");
    }

    let expected_check = digits.chars().last().unwrap().to_digit(10).unwrap_or(99);
    if check == expected_check {
        ValidationResult::valid(digits, None)
    } else {
        ValidationResult::invalid(digits, "NHS number checksum failed (MOD-11)")
    }
}

// ── EU VAT (basic country-specific format check) ────────────────

fn validate_eu_vat(s: &str) -> ValidationResult {
    let normalized: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let upper = normalized.to_uppercase();

    if upper.len() < 4 || !upper.is_ascii() {
        return ValidationResult::invalid(
            upper,
            "VAT number must be ASCII and at least 4 characters",
        );
    }

    let country = &upper[..2];
    let number = &upper[2..];

    // Basic format validation by country.
    // Only known EU/EEA VAT country codes are accepted — the previous catch-all
    // was too permissive (any 2 letters + 2 chars matched "hello" as VAT).
    let valid = match country {
        "AT" => number.starts_with('U') && number.len() == 9,
        "BE" => number.len() == 10 && number.chars().all(|c| c.is_ascii_digit()),
        "BG" => {
            (number.len() == 9 || number.len() == 10) && number.chars().all(|c| c.is_ascii_digit())
        }
        "CY" => number.len() == 9,
        "CZ" => {
            number.len() >= 8 && number.len() <= 10 && number.chars().all(|c| c.is_ascii_digit())
        }
        "DE" => number.len() == 9 && number.chars().all(|c| c.is_ascii_digit()),
        "DK" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "EE" => number.len() == 9 && number.chars().all(|c| c.is_ascii_digit()),
        "EL" | "GR" => number.len() == 9 && number.chars().all(|c| c.is_ascii_digit()),
        "ES" => number.len() == 9,
        "FI" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "FR" => number.len() == 11,
        "GB" => {
            number.len() == 9
                || number.len() == 12
                // Government departments: GD + 3 digits, Health authorities: HA + 3 digits
                || (number.len() == 5
                    && (number.starts_with("GD") || number.starts_with("HA"))
                    && number[2..].chars().all(|c| c.is_ascii_digit()))
        }
        "HR" => number.len() == 11 && number.chars().all(|c| c.is_ascii_digit()),
        "HU" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "IE" => number.len() == 8 || number.len() == 9,
        "IT" => number.len() == 11 && number.chars().all(|c| c.is_ascii_digit()),
        "LT" => {
            (number.len() == 9 || number.len() == 12) && number.chars().all(|c| c.is_ascii_digit())
        }
        "LU" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "LV" => number.len() == 11 && number.chars().all(|c| c.is_ascii_digit()),
        "MT" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "NL" => {
            // Format: 9 digits + B + 2 digits (e.g., NL123456789B01)
            number.len() == 12
                && number.as_bytes()[9] == b'B'
                && number[..9].chars().all(|c| c.is_ascii_digit())
                && number[10..].chars().all(|c| c.is_ascii_digit())
        }
        "PL" => number.len() == 10 && number.chars().all(|c| c.is_ascii_digit()),
        "PT" => number.len() == 9 && number.chars().all(|c| c.is_ascii_digit()),
        "RO" => {
            number.len() >= 2 && number.len() <= 10 && number.chars().all(|c| c.is_ascii_digit())
        }
        "SE" => number.len() == 12 && number.chars().all(|c| c.is_ascii_digit()),
        "SI" => number.len() == 8 && number.chars().all(|c| c.is_ascii_digit()),
        "SK" => number.len() == 10 && number.chars().all(|c| c.is_ascii_digit()),
        // No catch-all: unrecognized country code → invalid
        _ => false,
    };

    if valid {
        ValidationResult::valid(upper.clone(), Some(country.to_string()))
    } else {
        ValidationResult::invalid(upper, "invalid VAT number format for country")
    }
}

// ── UUID ────────────────────────────────────────────────────────

fn validate_uuid(s: &str) -> ValidationResult {
    let lower = s.trim().to_lowercase();
    if lower.len() != 36 {
        return ValidationResult::invalid(lower, "UUID must be 36 characters");
    }
    let parts: Vec<&str> = lower.split('-').collect();
    if parts.len() != 5
        || parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 4
        || parts[4].len() != 12
    {
        return ValidationResult::invalid(lower, "UUID must be 8-4-4-4-12 format");
    }
    if !lower.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return ValidationResult::invalid(lower, "UUID must contain only hex digits and dashes");
    }
    ValidationResult::valid(lower, None)
}

// ── Email (basic RFC 5322) ──────────────────────────────────────

fn validate_email(s: &str) -> ValidationResult {
    let s = s.trim();
    if !s.contains('@') || s.contains(' ') {
        return ValidationResult::invalid(s.to_string(), "email must contain @ without spaces");
    }
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || !parts[1].contains('.') {
        return ValidationResult::invalid(s.to_string(), "invalid email format");
    }
    let local = parts[0];
    let domain = parts[1];
    // RFC 5321: local-part max 64 chars
    if local.len() > 64 {
        return ValidationResult::invalid(s.to_string(), "local part exceeds 64 characters");
    }
    // RFC 5322: no leading/trailing dots or consecutive dots in local part
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        return ValidationResult::invalid(s.to_string(), "invalid email local part");
    }
    if domain.starts_with('.') || domain.ends_with('.') || domain.contains("..") {
        return ValidationResult::invalid(s.to_string(), "invalid email domain");
    }
    // Domain labels: no leading/trailing hyphens, no underscores
    for label in domain.split('.') {
        if label.is_empty() || label.starts_with('-') || label.ends_with('-') || label.contains('_')
        {
            return ValidationResult::invalid(s.to_string(), "invalid email domain label");
        }
    }
    ValidationResult::valid(s.to_string(), Some(domain.to_string()))
}

// ── Phone (basic international format) ──────────────────────────

fn validate_phone(s: &str) -> ValidationResult {
    let trimmed = s.trim();
    // Reject double-plus or other malformed prefixes
    if trimmed.starts_with("++") {
        return ValidationResult::invalid(trimmed.to_string(), "phone number has invalid prefix");
    }
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    // Require at least 10 digits for a valid phone (country code + number)
    // 7-digit local numbers without country code are not internationally valid
    if digits.len() < 10 || digits.len() > 15 {
        return ValidationResult::invalid(digits, "phone number must be 10-15 digits");
    }
    // Country code cannot start with 0
    let has_plus = trimmed.starts_with('+');
    if has_plus {
        let after_plus: String = trimmed[1..]
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        if after_plus.starts_with('0') {
            return ValidationResult::invalid(digits, "country code cannot start with 0");
        }
    }
    let stripped = trimmed.trim_start_matches('+');
    if stripped.is_empty() || !stripped.chars().any(|c| c.is_ascii_digit()) {
        return ValidationResult::invalid(digits, "phone number must contain digits");
    }
    let has_invalid = stripped
        .chars()
        .any(|c| !c.is_ascii_digit() && !" ()-./".contains(c));
    if has_invalid {
        return ValidationResult::invalid(digits, "phone number contains invalid characters");
    }
    ValidationResult::valid(digits, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iban_valid() {
        assert!(validate("GB29 NWBK 6016 1331 9268 19", IdentifierType::Iban).is_valid);
        assert!(validate("DE89370400440532013000", IdentifierType::Iban).is_valid);
    }

    #[test]
    fn iban_invalid() {
        assert!(!validate("GB29 NWBK 6016 1331 9268 18", IdentifierType::Iban).is_valid);
    }

    #[test]
    fn credit_card_visa() {
        let r = validate("4111111111111111", IdentifierType::CreditCard);
        assert!(r.is_valid);
        assert_eq!(r.detail.as_deref(), Some("Visa"));
    }

    #[test]
    fn credit_card_luhn_fail() {
        assert!(!validate("4111111111111112", IdentifierType::CreditCard).is_valid);
    }

    #[test]
    fn isbn13_valid() {
        assert!(validate("978-0-306-40615-7", IdentifierType::Isbn13).is_valid);
    }

    #[test]
    fn isbn10_valid() {
        assert!(validate("0306406152", IdentifierType::Isbn10).is_valid);
    }

    #[test]
    fn us_ssn_valid() {
        assert!(validate("123-45-6789", IdentifierType::UsSsn).is_valid);
    }

    #[test]
    fn us_ssn_invalid_area() {
        assert!(!validate("000-45-6789", IdentifierType::UsSsn).is_valid);
        assert!(!validate("666-45-6789", IdentifierType::UsSsn).is_valid);
    }

    #[test]
    fn uuid_valid() {
        assert!(validate("550e8400-e29b-41d4-a716-446655440000", IdentifierType::Uuid).is_valid);
    }

    #[test]
    fn email_valid() {
        assert!(validate("alice@example.com", IdentifierType::Email).is_valid);
    }

    #[test]
    fn detect_identifies_types() {
        let results = detect("4111111111111111");
        assert!(results
            .iter()
            .any(|(t, _)| *t == IdentifierType::CreditCard));
    }
}
