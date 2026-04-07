//! Time/date coercion pack.
//!
//! Handles multi-format date parsing, 12/24 hour conversion, and
//! common date string patterns found in real-world data.

use serde_json::Value;

use crate::coerce::CoercionResult;
use crate::diagnostic::{Diagnostic, DiagnosticKind, RiskLevel};

/// Recognized date/time formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFormat {
    /// ISO 8601: "2026-03-31T15:30:00Z"
    Iso8601,
    /// ISO date only: "2026-03-31"
    IsoDate,
    /// US format: "03/31/2026"
    UsDate,
    /// European format: "31/03/2026"
    EuDate,
    /// Long format: "31 March 2026" or "March 31, 2026"
    LongDate,
    /// Abbreviated: "Mar 31, 2026"
    AbbrevDate,
    /// Year only: "2026" or "1948"
    YearOnly,
    /// Unix timestamp (seconds): "1711900800" or 1711900800
    UnixSeconds,
    /// Unix timestamp (milliseconds): "1711900800000"
    UnixMillis,
    /// 12-hour time: "2:30 PM"
    Time12,
    /// 24-hour time: "14:30"
    Time24,
    /// Ambiguous (e.g., "01/02/2026" — could be Jan 2 or Feb 1)
    Ambiguous,
    /// ISO 8601 week date: "2026W13", "2026-W13", "2026-W13-1"
    IsoWeekDate,
    /// GEDCOM approximate: "ABT 1850", "EST 1850", "CAL 1823"
    GedcomApprox,
    /// GEDCOM range: "BET 1840 AND 1860"
    GedcomRange,
    /// GEDCOM period: "FROM 1840 TO 1860"
    GedcomPeriod,
    /// GEDCOM before/after: "BEF 1899", "AFT 1800" (inclusive in GEDCOM 7.0)
    GedcomBeforeAfter,
    /// GEDCOM interpreted: "INT 1756 (age 30 at death)"
    GedcomInterpreted,
    /// HL7 v2 packed date: "20260402" or "20260402143022"
    Hl7Date,
    /// Unrecognized
    Unknown,
}

/// Detect the format of a date/time string.
pub fn detect_format(s: &str) -> DateFormat {
    let s = s.trim();

    if s.is_empty() {
        return DateFormat::Unknown;
    }

    // Early rejection: strings that clearly aren't dates
    // URLs, emails, file paths, and multi-word prose are not dates
    if s.contains("://")
        || s.contains("www.")
        || s.contains('@')
        || s.contains('\\')
        || s.starts_with('/')
    {
        return DateFormat::Unknown;
    }

    // GEDCOM patterns (per GEDCOM 7.0 spec — gedcom.io)
    let upper = s.to_uppercase();
    if upper.starts_with("ABT ")
        || upper.starts_with("ABOUT ")
        || upper.starts_with("EST ")
        || upper.starts_with("CAL ")
        || s.starts_with("circa ")
        || s.starts_with("CIRCA ")
    {
        return DateFormat::GedcomApprox;
    }
    if upper.starts_with("BET ") || upper.starts_with("BETWEEN ") {
        return DateFormat::GedcomRange;
    }
    if upper.starts_with("FROM ") {
        if upper.contains(" TO ") {
            return DateFormat::GedcomPeriod;
        }
        return DateFormat::GedcomPeriod;
    }
    if upper.starts_with("TO ") {
        return DateFormat::GedcomPeriod;
    }
    if upper.starts_with("BEF ") || upper.starts_with("BEFORE ") {
        return DateFormat::GedcomBeforeAfter;
    }
    if upper.starts_with("AFT ") || upper.starts_with("AFTER ") {
        return DateFormat::GedcomBeforeAfter;
    }
    if upper.starts_with("INT ") {
        return DateFormat::GedcomInterpreted;
    }

    // HL7 v2 packed dates: YYYYMMDD or YYYYMMDDHHMMSS
    // Year range includes 9999 (HL7 "end of time" sentinel in validity periods)
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = s[..4].parse().unwrap_or(0);
        let month: u32 = s[4..6].parse().unwrap_or(0);
        let day: u32 = s[6..8].parse().unwrap_or(0);
        if (1900..=9999).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day) {
            return DateFormat::Hl7Date;
        }
    }
    if s.len() >= 14 && s.is_char_boundary(14) && s[..14].chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = s[..4].parse().unwrap_or(0);
        let month: u32 = s[4..6].parse().unwrap_or(0);
        if (1900..=9999).contains(&year) && (1..=12).contains(&month) {
            return DateFormat::Hl7Date;
        }
    }
    if s.starts_with("circa ") || s.starts_with("CIRCA ") || s.starts_with("c. ") {
        return DateFormat::GedcomApprox;
    }

    // ISO 8601 with time — both T-separated and space-separated
    // "2026-03-31T15:30:00Z" (standard ISO 8601)
    // "2026-03-31 15:30:00"  (database-style, MySQL/PostgreSQL/SQLite)
    // ISO dates are always ASCII — reject non-ASCII early to prevent byte-boundary panics
    if s.is_ascii()
        && s.len() >= 19
        && (s.contains('T') || is_space_datetime(s))
        && (s.contains('-') || s.contains(':'))
    {
        return DateFormat::Iso8601;
    }

    // ISO date only: YYYY-MM-DD (always ASCII)
    if s.is_ascii() && s.len() == 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        return DateFormat::IsoDate;
    }

    // ISO 8601 week date: "2026W13", "2026-W13", "2026-W13-1", "2026W131"
    if is_iso_week_date(s) {
        return DateFormat::IsoWeekDate;
    }

    // Year only (4 digits)
    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
        return DateFormat::YearOnly;
    }

    // Unix timestamp (all digits, reasonable range)
    if s.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = s.parse::<u64>() {
            // Unix millis: 13 digits, range ~2001-2100
            if s.len() == 13 && n > 946_684_800_000 && n < 4_200_000_000_000 {
                return DateFormat::UnixMillis;
            }
            // Unix seconds: 9-10 digits, range 2000-2100
            if (s.len() == 9 || s.len() == 10) && n > 946_684_800 && n < 4_200_000_000 {
                return DateFormat::UnixSeconds;
            }
        }
    }

    // Slash-separated dates
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if a > 12 {
                    return DateFormat::EuDate; // first number > 12 → must be day
                }
                if b > 12 {
                    return DateFormat::UsDate; // second number > 12 → must be day
                }
                return DateFormat::Ambiguous; // both ≤ 12 — can't tell
            }
        }
    }

    // Dot-separated dates — European DD.MM.YYYY or ISO-like YYYY.MM.DD
    if s.contains('.') && !s.contains("..") {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 3 {
            if let (Ok(a), Ok(b), Ok(c)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                // DD.MM.YYYY
                if (1..=31).contains(&a) && (1..=12).contains(&b) && parts[2].len() >= 4 {
                    return DateFormat::EuDate;
                }
                // YYYY.MM.DD
                if parts[0].len() == 4
                    && a >= 1900
                    && (1..=12).contains(&b)
                    && (1..=31).contains(&c)
                {
                    return DateFormat::IsoDate;
                }
            }
        }
    }

    // Dash-separated dates: DD-MM-YYYY or MM-DD-YYYY
    // Requires: exactly 3 parts, first two are 1-2 digits, third is 4-digit year.
    // This avoids matching SSN (NNN-NN-NNNN) or EIN (NN-NNNNNNN) formats.
    if s.contains('-') && !s.contains('T') && !s.contains(':') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 && parts[0].len() <= 2 && parts[1].len() <= 2 && parts[2].len() == 4 {
            if let (Ok(a), Ok(b), Ok(c)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                if c >= 1900 {
                    if a > 12 {
                        return DateFormat::EuDate;
                    }
                    if b > 12 {
                        return DateFormat::UsDate;
                    }
                    return DateFormat::Ambiguous;
                }
            }
        }
    }

    // Long/abbreviated month names (word-boundary aware to avoid false positives
    // like "summary" containing "mar", "decimal" containing "dec", etc.)
    let month_names = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let month_abbrevs = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];
    let lower = s.to_lowercase();
    // Only check month names if the string contains at least one digit
    // (pure word strings like "hello may world" are not dates)
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    if has_digit {
        if month_names
            .iter()
            .any(|m| contains_at_word_boundary(&lower, m))
        {
            return DateFormat::LongDate;
        }
        if month_abbrevs
            .iter()
            .any(|m| contains_at_word_boundary(&lower, m))
        {
            return DateFormat::AbbrevDate;
        }
    }

    // 12-hour time — require "am"/"pm" at word boundary AND digits present
    // (avoids "sample" matching "am", "dump" matching "pm")
    if has_digit
        && (contains_at_word_boundary(&lower, "am")
            || contains_at_word_boundary(&lower, "pm")
            || lower.ends_with(" am")
            || lower.ends_with(" pm")
            || lower.contains(" am ")
            || lower.contains(" pm "))
    {
        return DateFormat::Time12;
    }

    // 24-hour time: "14:30", "14:30:00", "T15:30", "T15:30:00Z", "T15:30:00+02:00"
    if is_time_24(s) {
        return DateFormat::Time24;
    }

    DateFormat::Unknown
}

/// Check if string is a space-separated datetime: "YYYY-MM-DD HH:MM:SS..."
/// The space must be at position 10 (after the date part) and followed by a digit.
fn is_space_datetime(s: &str) -> bool {
    if s.len() < 19 {
        return false;
    }
    let bytes = s.as_bytes();
    // Position 10 must be a space, position 11 must be a digit (start of time)
    bytes[10] == b' ' && bytes[11].is_ascii_digit()
}

/// Check if string is a 24-hour time, with or without T prefix and timezone.
/// Accepts: "14:30", "14:30:00", "14:30:00.123", "T15:30", "T15:30:00Z",
/// "T15:30:00+02:00", "T15:30:00-05:00"
fn is_time_24(s: &str) -> bool {
    // Strip optional T prefix
    let t = s.strip_prefix('T').unwrap_or(s);
    if t.len() < 5 {
        return false;
    }
    let bytes = t.as_bytes();
    // HH:MM required
    if !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || bytes[2] != b':'
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
    {
        return false;
    }
    let rest = &t[5..];
    if rest.is_empty() {
        return true; // HH:MM
    }
    // Optional :SS
    let rest = if rest.starts_with(':')
        && rest.len() >= 3
        && rest.as_bytes()[1].is_ascii_digit()
        && rest.as_bytes()[2].is_ascii_digit()
    {
        &rest[3..]
    } else {
        return false; // After HH:MM, only :SS or nothing
    };
    if rest.is_empty() {
        return true; // HH:MM:SS
    }
    // Optional .fractional
    let rest = if let Some(after_dot) = rest.strip_prefix('.') {
        let frac_end = after_dot
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_dot.len());
        if frac_end == 0 {
            return false; // "." with no digits
        }
        &after_dot[frac_end..]
    } else {
        rest
    };
    if rest.is_empty() {
        return true; // HH:MM:SS.fff
    }
    // Optional timezone: Z, +HH:MM, -HH:MM
    if rest == "Z" {
        return true;
    }
    if (rest.starts_with('+') || rest.starts_with('-')) && rest.len() == 6 {
        let tz = rest.as_bytes();
        return tz[1].is_ascii_digit()
            && tz[2].is_ascii_digit()
            && tz[3] == b':'
            && tz[4].is_ascii_digit()
            && tz[5].is_ascii_digit();
    }
    false
}

/// Check if string matches ISO 8601 week date pattern.
/// Accepts: "2026W13", "2026-W13", "2026-W13-1", "2026W131"
fn is_iso_week_date(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 7 {
        return false;
    }
    // First 4 chars must be digits (year)
    if !bytes[..4].iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // Find W position (4 or 5 depending on hyphen)
    let (w_pos, expect_hyphen_day) = if bytes[4] == b'W' {
        (4, false)
    } else if bytes[4] == b'-' && bytes.len() > 5 && bytes[5] == b'W' {
        (5, true)
    } else {
        return false;
    };
    // After W, need 2 digits (week number 01-53)
    let week_start = w_pos + 1;
    if bytes.len() < week_start + 2 {
        return false;
    }
    if !bytes[week_start..week_start + 2]
        .iter()
        .all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let remaining = &bytes[week_start + 2..];
    // Optional day suffix: -D or D (1-7)
    match remaining.len() {
        0 => true,
        1 if !expect_hyphen_day && remaining[0].is_ascii_digit() => true,
        2 if expect_hyphen_day && remaining[0] == b'-' && remaining[1].is_ascii_digit() => true,
        _ => false,
    }
}

/// Check if `needle` appears in `haystack` at a word boundary.
/// A word boundary means the character before the match is not ASCII alphabetic
/// (or is start of string), AND the character after is not ASCII alphabetic
/// (or is end of string). Digits, punctuation, spaces all count as boundaries.
/// This prevents "summary" matching "mar" or "decimal" matching "dec".
fn contains_at_word_boundary(haystack: &str, needle: &str) -> bool {
    let hay = haystack.as_bytes();
    let need = needle.as_bytes();
    let need_len = need.len();
    if hay.len() < need_len {
        return false;
    }
    for i in 0..=(hay.len() - need_len) {
        if &hay[i..i + need_len] == need {
            let before_ok = i == 0 || !hay[i - 1].is_ascii_alphabetic();
            let after_ok = i + need_len >= hay.len() || !hay[i + need_len].is_ascii_alphabetic();
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

/// Convert a date/time string to ISO 8601 format.
///
/// Detects the input format and normalizes to `"YYYY-MM-DDTHH:MM:SSZ"` or
/// `"YYYY-MM-DD"` for date-only formats. Returns `None` if the format
/// cannot be parsed or is ambiguous (use `convert_date_with_hint` for ambiguous dates).
///
/// ```
/// use laminate::packs::time::convert_to_iso8601;
///
/// assert_eq!(convert_to_iso8601("03/31/2026"), Some("2026-03-31".to_string()));
/// assert_eq!(convert_to_iso8601("31 March 2026"), Some("2026-03-31".to_string()));
/// assert_eq!(convert_to_iso8601("20260402"), Some("2026-04-02".to_string())); // HL7
/// ```
pub fn convert_to_iso8601(s: &str) -> Option<String> {
    // For ambiguous dates (both parts ≤12), defaults to US format (MM/DD).
    // Use convert_to_iso8601_with_hint(s, true) for EU (DD/MM) convention.
    // Use detect_format() to check for DateFormat::Ambiguous before converting.
    convert_to_iso8601_with_hint(s, false)
}

/// Convert a date/time string to ISO 8601, with a day-first hint for ambiguous dates.
///
/// When `day_first` is true, "01/02/2026" is interpreted as February 1 (DD/MM).
/// When false, it's January 2 (MM/DD).
pub fn convert_to_iso8601_with_hint(s: &str, day_first: bool) -> Option<String> {
    let s = s.trim();
    let format = detect_format(s);

    match format {
        DateFormat::Iso8601 | DateFormat::IsoDate => {
            // Validate the date portion (first 10 chars: YYYY?MM?DD where ? is any separator)
            // Guard against multi-byte UTF-8 characters at slice boundaries
            if s.len() >= 10
                && s.is_char_boundary(4)
                && s.is_char_boundary(5)
                && s.is_char_boundary(7)
                && s.is_char_boundary(8)
                && s.is_char_boundary(10)
            {
                let year: u32 = s[..4].parse().ok()?;
                let month: u32 = s[5..7].parse().ok()?;
                let day: u32 = s[8..10].parse().ok()?;
                if !is_valid_date(year, month, day) {
                    return None;
                }
            }
            // Normalize to ISO 8601
            let mut normalized = s.to_string();
            // Normalize dot-separated YYYY.MM.DD to YYYY-MM-DD (only date-part dots)
            if normalized.len() >= 10 {
                let b4 = normalized.as_bytes()[4];
                let b7 = normalized.as_bytes()[7];
                if b4 == b'.' || b7 == b'.' {
                    let mut chars: Vec<u8> = normalized.into_bytes();
                    if chars[4] == b'.' {
                        chars[4] = b'-';
                    }
                    if chars[7] == b'.' {
                        chars[7] = b'-';
                    }
                    normalized = String::from_utf8(chars).expect("only ASCII bytes modified");
                }
            }
            // Normalize space-separated datetimes to T-separated
            if normalized.len() > 10 && normalized.as_bytes()[10] == b' ' {
                normalized.replace_range(10..11, "T");
            }
            Some(normalized)
        }

        DateFormat::UsDate => {
            // MM/DD/YYYY, MM/DD/YY, or MM-DD-YYYY
            let sep = if s.contains('/') { '/' } else { '-' };
            let parts: Vec<&str> = s.split(sep).collect();
            if parts.len() == 3 {
                let month: u32 = parts[0].parse().ok()?;
                let day: u32 = parts[1].parse().ok()?;
                let year = parse_year(parts[2])?;
                if !is_valid_date(year, month, day) {
                    return None;
                }
                Some(format!("{:04}-{:02}-{:02}", year, month, day))
            } else {
                None
            }
        }

        DateFormat::EuDate => {
            // DD/MM/YYYY, DD/MM/YY, DD.MM.YYYY, or DD-MM-YYYY
            let sep = if s.contains('.') {
                '.'
            } else if s.contains('/') {
                '/'
            } else {
                '-'
            };
            let parts: Vec<&str> = s.split(sep).collect();
            if parts.len() == 3 {
                let day: u32 = parts[0].parse().ok()?;
                let month: u32 = parts[1].parse().ok()?;
                let year = parse_year(parts[2])?;
                if !is_valid_date(year, month, day) {
                    return None;
                }
                Some(format!("{:04}-{:02}-{:02}", year, month, day))
            } else {
                None
            }
        }

        DateFormat::Ambiguous => {
            let sep = if s.contains('/') { '/' } else { '-' };
            let parts: Vec<&str> = s.split(sep).collect();
            if parts.len() == 3 {
                let a: u32 = parts[0].parse().ok()?;
                let b: u32 = parts[1].parse().ok()?;
                let year = parse_year(parts[2])?;
                let (month, day) = if day_first { (b, a) } else { (a, b) };
                if !is_valid_date(year, month, day) {
                    return None;
                }
                Some(format!("{:04}-{:02}-{:02}", year, month, day))
            } else {
                None
            }
        }

        DateFormat::LongDate => parse_long_date_to_iso(s),
        DateFormat::AbbrevDate => parse_abbrev_date_to_iso(s),
        DateFormat::YearOnly => Some(format!("{}-01-01", s)),

        DateFormat::UnixSeconds => {
            let ts: i64 = s.parse().ok()?;
            let (y, m, d, h, min, sec) = unix_secs_to_ymdhms(ts);
            Some(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                y, m, d, h, min, sec
            ))
        }

        DateFormat::UnixMillis => {
            let ts: i64 = s.parse().ok()?;
            let secs = ts / 1000;
            let (y, m, d, h, min, sec) = unix_secs_to_ymdhms(secs);
            Some(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                y, m, d, h, min, sec
            ))
        }

        DateFormat::Hl7Date => {
            if s.len() >= 8 {
                let iso = format!("{}-{}-{}", &s[..4], &s[4..6], &s[6..8]);
                if s.len() >= 14 {
                    let base = format!("{}T{}:{}:{}", iso, &s[8..10], &s[10..12], &s[12..14]);
                    // Preserve fractional seconds and timezone suffix
                    let suffix = &s[14..];
                    if suffix.is_empty() {
                        Some(base)
                    } else {
                        // Normalize HL7 timezone: +/-HHMM → +/-HH:MM
                        let normalized = normalize_hl7_suffix(suffix);
                        Some(format!("{}{}", base, normalized))
                    }
                } else {
                    Some(iso)
                }
            } else {
                None
            }
        }

        DateFormat::IsoWeekDate => iso_week_to_calendar(s),

        _ => None,
    }
}

/// Normalize HL7 suffix: fractional seconds + timezone.
/// HL7 uses +/-HHMM without colon; ISO 8601 uses +/-HH:MM.
fn normalize_hl7_suffix(suffix: &str) -> String {
    // Find timezone offset pattern: +/-HHMM at end
    let bytes = suffix.as_bytes();
    let len = bytes.len();
    // Look for +/- followed by exactly 4 digits at end
    if len >= 5 {
        let tz_start = len - 5;
        let sign = bytes[tz_start];
        if (sign == b'+' || sign == b'-')
            && bytes[tz_start + 1..].iter().all(|b| b.is_ascii_digit())
        {
            let frac = &suffix[..tz_start];
            let hh = &suffix[tz_start + 1..tz_start + 3];
            let mm = &suffix[tz_start + 3..];
            return format!("{}{}{hh}:{mm}", frac, sign as char);
        }
    }
    suffix.to_string()
}

/// Batch-level date format detection for a column of values.
///
/// Analyzes multiple values to determine the dominant date format,
/// percentage of dates, and whether ambiguous dates can be disambiguated.
///
/// ```
/// use laminate::packs::time::detect_column_format;
///
/// let values = vec!["2026-01-01", "2026-02-15", "2026-03-31", "not a date"];
/// let info = detect_column_format(&values);
/// assert!(info.date_percentage > 0.7);
/// ```
#[derive(Debug)]
pub struct ColumnDateInfo {
    /// The most common date format in the column.
    pub dominant_format: DateFormat,
    /// Percentage of values that parse as dates (0.0 – 1.0).
    pub date_percentage: f64,
    /// Number of ambiguous dates in the column.
    pub ambiguous_count: usize,
    /// Whether batch context can disambiguate (e.g., a value > 12 proves DD/MM).
    pub disambiguated: bool,
    /// If disambiguated, whether the column is day-first (EU) or month-first (US).
    pub day_first: Option<bool>,
    /// Total values analyzed.
    pub total: usize,
}

/// Analyze a column of string values and detect the dominant date/time format.
pub fn detect_column_format(values: &[&str]) -> ColumnDateInfo {
    let mut format_counts: std::collections::HashMap<
        std::mem::Discriminant<DateFormat>,
        (DateFormat, usize),
    > = std::collections::HashMap::new();
    let mut date_count = 0;
    let mut ambiguous_count = 0;
    let mut has_day_gt_12 = false; // proves DD/MM (EU)
    let mut has_month_gt_12 = false; // proves MM/DD (US) — shouldn't happen as first field > 12

    for &v in values {
        let fmt = detect_format(v);
        match fmt {
            DateFormat::Unknown => {}
            DateFormat::Ambiguous => {
                date_count += 1;
                ambiguous_count += 1;
                // Check if any value in the batch can disambiguate
                if let Some(first) = v.split('/').next() {
                    if let Ok(n) = first.parse::<u32>() {
                        if n > 12 {
                            has_day_gt_12 = true;
                        }
                    }
                }
            }
            _ => {
                date_count += 1;
                if matches!(fmt, DateFormat::EuDate) {
                    has_day_gt_12 = true;
                }
                if matches!(fmt, DateFormat::UsDate) {
                    has_month_gt_12 = true;
                }
            }
        }
        let disc = std::mem::discriminant(&fmt);
        format_counts
            .entry(disc)
            .and_modify(|(_, c)| *c += 1)
            .or_insert((fmt, 1));
    }

    let total = values.len();
    let date_percentage = if total > 0 {
        date_count as f64 / total as f64
    } else {
        0.0
    };

    let dominant = format_counts
        .into_values()
        .max_by_key(|(_, c)| *c)
        .map(|(f, _)| f)
        .unwrap_or(DateFormat::Unknown);

    let (disambiguated, day_first) = if ambiguous_count > 0 {
        if has_day_gt_12 && !has_month_gt_12 {
            (true, Some(true)) // EU format (DD/MM)
        } else if has_month_gt_12 && !has_day_gt_12 {
            (true, Some(false)) // US format (MM/DD) — rare since first > 12 hits EuDate
        } else {
            (false, None) // Still ambiguous
        }
    } else {
        (false, None) // No ambiguous dates
    };

    ColumnDateInfo {
        dominant_format: dominant,
        date_percentage,
        ambiguous_count,
        disambiguated,
        day_first,
        total,
    }
}

// ── Helper functions for date conversion ────────────────────────

/// Convert day count from epoch to (year, month, day).
/// Uses the algorithm from http://howardhinnant.github.io/date_algorithms.html
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn unix_secs_to_ymdhms(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    // Euclidean division so remainder is always non-negative
    let days = secs.div_euclid(86400);
    let day_secs = secs.rem_euclid(86400) as u32;
    let (y, m, d) = days_to_ymd(days + 719468);
    let h = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    (y, m, d, h, min, sec)
}

/// Convert ISO week date (e.g. "2026-W14", "2026-W14-1", "2026W145") to calendar date.
fn iso_week_to_calendar(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let year: i64 = std::str::from_utf8(&bytes[..4]).ok()?.parse().ok()?;

    // Find W position
    let w_pos = if bytes[4] == b'W' { 4 } else { 5 };
    let week_start = w_pos + 1;
    let week: i64 = std::str::from_utf8(&bytes[week_start..week_start + 2])
        .ok()?
        .parse()
        .ok()?;
    if !(1..=53).contains(&week) {
        return None;
    }

    // Parse optional day-of-week (1=Monday, 7=Sunday), default to 1
    let day_of_week: i64 = {
        let rest = &bytes[week_start + 2..];
        if rest.is_empty() {
            1
        } else if rest[0] == b'-' && rest.len() >= 2 && rest[1].is_ascii_digit() {
            (rest[1] - b'0') as i64
        } else if rest[0].is_ascii_digit() {
            (rest[0] - b'0') as i64
        } else {
            1
        }
    };
    if !(1..=7).contains(&day_of_week) {
        return None;
    }

    // January 4th is always in ISO week 1. Find its day-of-week,
    // then compute the Monday of week 1.
    // ymd_to_days returns Unix epoch days (day 0 = 1970-01-01 = Thursday).
    // To get Monday-based day-of-week: (days + 3) mod 7 gives 0=Mon..6=Sun.
    let jan4 = ymd_to_days(year, 1, 4);
    let jan4_dow = (jan4 + 3).rem_euclid(7); // 0=Mon, 1=Tue, ..., 6=Sun
    let week1_monday = jan4 - jan4_dow;

    let target_day = week1_monday + (week - 1) * 7 + (day_of_week - 1);
    let (y, m, d) = days_to_ymd(target_day + 719468);

    if is_valid_date(y as u32, m, d) {
        Some(format!("{:04}-{:02}-{:02}", y, m, d))
    } else {
        None
    }
}

/// Convert a calendar date to a day count (inverse of days_to_ymd shifted by epoch).
fn ymd_to_days(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 9 } else { month - 3 } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * m as u64 + 2) / 5 + day as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe as i64) - 719468
}

fn parse_long_date_to_iso(s: &str) -> Option<String> {
    // "31 March 2026" or "March 31, 2026"
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let (day, month_name, year) = if parts[0].chars().next()?.is_ascii_digit() {
        // "31 March 2026"
        (parts[0].trim_end_matches(','), parts[1], parts[2])
    } else {
        // "March 31, 2026"
        (parts[1].trim_end_matches(','), parts[0], parts[2])
    };

    let month = month_name_to_number(month_name)?;
    let d: u32 = day.parse().ok()?;
    let y: u32 = year.parse().ok()?;
    if !is_valid_date(y, month, d) {
        return None;
    }
    Some(format!("{:04}-{:02}-{:02}", y, month, d))
}

fn parse_abbrev_date_to_iso(s: &str) -> Option<String> {
    // "Mar 31, 2026", "31-Mar-2026", "31MAR26"
    let s = s.replace(['-', ','], " ");
    let parts: Vec<&str> = s.split_whitespace().filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 {
        return None;
    }

    // Try both orderings
    if parts[0].chars().next()?.is_ascii_digit() {
        // "31 Mar 2026"
        let d: u32 = parts[0].parse().ok()?;
        let m = month_name_to_number(parts[1])?;
        let y = parse_year(parts[2])?;
        if !is_valid_date(y, m, d) {
            return None;
        }
        Some(format!("{:04}-{:02}-{:02}", y, m, d))
    } else {
        // "Mar 31 2026"
        let m = month_name_to_number(parts[0])?;
        let d: u32 = parts[1].parse().ok()?;
        let y = parse_year(parts[2])?;
        if !is_valid_date(y, m, d) {
            return None;
        }
        Some(format!("{:04}-{:02}-{:02}", y, m, d))
    }
}

/// Validate that a date is calendrically valid (accounts for leap years).
fn is_valid_date(year: u32, month: u32, day: u32) -> bool {
    if month == 0 || month > 12 || day == 0 {
        return false;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return false,
    };
    day <= max_day
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn month_name_to_number(name: &str) -> Option<u32> {
    match name.to_lowercase().get(..3)? {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

fn parse_year(s: &str) -> Option<u32> {
    let n: u32 = s.parse().ok()?;
    // Only apply 2-digit year pivot for actual 2-digit strings (not "0001")
    if s.len() <= 2 && n < 100 {
        // 2-digit year: 00-29 → 2000-2029, 30-99 → 1930-1999
        Some(if n < 30 { 2000 + n } else { 1900 + n })
    } else {
        Some(n)
    }
}

/// Attempt to coerce a date/time value, producing diagnostics.
///
/// Returns the value as-is (dates stay as strings) but with format
/// detection diagnostics and risk assessment.
pub fn coerce_datetime(value: &Value, path: &str) -> CoercionResult {
    match value {
        Value::String(s) => {
            let format = detect_format(s);
            let (risk, suggestion) = match &format {
                DateFormat::Ambiguous => (
                    RiskLevel::Risky,
                    Some(
                        "ambiguous date format (could be MM/DD or DD/MM); specify format explicitly",
                    ),
                ),
                DateFormat::GedcomApprox | DateFormat::GedcomRange => (
                    RiskLevel::Warning,
                    Some(
                        "approximate/range date from genealogical data; precision is inherently limited",
                    ),
                ),
                DateFormat::Unknown => (
                    RiskLevel::Warning,
                    Some("unrecognized date format; consider standardizing to ISO 8601"),
                ),
                DateFormat::UnixSeconds | DateFormat::UnixMillis => (
                    RiskLevel::Info,
                    Some(
                        "unix timestamp detected; consider converting to ISO 8601 for readability",
                    ),
                ),
                _ => (RiskLevel::Info, None),
            };

            CoercionResult {
                value: value.clone(),
                coerced: false, // We don't transform dates — we detect and diagnose
                diagnostic: Some(Diagnostic {
                    path: path.to_string(),
                    kind: DiagnosticKind::Coerced {
                        from: format!("date string ({format:?})"),
                        to: "detected format".into(),
                    },
                    risk,
                    suggestion: suggestion.map(|s| s.to_string()),
                }),
            }
        }
        Value::Number(n) => {
            // Numeric timestamp
            let is_millis = n.as_u64().map(|v| v > 1_000_000_000_000).unwrap_or(false);
            let fmt = if is_millis {
                "unix_millis"
            } else {
                "unix_seconds"
            };
            CoercionResult {
                value: value.clone(),
                coerced: false,
                diagnostic: Some(Diagnostic {
                    path: path.to_string(),
                    kind: DiagnosticKind::Coerced {
                        from: format!("numeric timestamp ({fmt})"),
                        to: "detected format".into(),
                    },
                    risk: RiskLevel::Info,
                    suggestion: Some("numeric timestamp; consider converting to ISO 8601".into()),
                }),
            }
        }
        _ => CoercionResult {
            value: value.clone(),
            coerced: false,
            diagnostic: None,
        },
    }
}

// ── Chrono integration (optional feature) ───────────────────────

/// Convert a date string to a `chrono::NaiveDate`.
///
/// Available when the `chrono-integration` feature is enabled.
/// Detects the input format automatically and converts to chrono's date type.
#[cfg(feature = "chrono-integration")]
pub fn to_naive_date(s: &str) -> Option<chrono::NaiveDate> {
    let iso = convert_to_iso8601(s)?;
    // Parse YYYY-MM-DD from the ISO string
    if iso.len() >= 10 {
        chrono::NaiveDate::parse_from_str(&iso[..10], "%Y-%m-%d").ok()
    } else {
        None
    }
}

/// Convert a datetime string to a `chrono::NaiveDateTime`.
///
/// Available when the `chrono-integration` feature is enabled.
#[cfg(feature = "chrono-integration")]
pub fn to_naive_datetime(s: &str) -> Option<chrono::NaiveDateTime> {
    let iso = convert_to_iso8601(s)?;
    // Try full datetime first
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&iso, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt);
    }
    // Try with timezone (strip it for NaiveDateTime)
    let clean = iso.trim_end_matches('Z');
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(clean, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt);
    }
    // Date-only → midnight
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&iso, "%Y-%m-%d") {
        return date.and_hms_opt(0, 0, 0);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_iso8601() {
        assert_eq!(detect_format("2026-03-31T15:30:00Z"), DateFormat::Iso8601);
    }

    #[test]
    fn detect_iso_date() {
        assert_eq!(detect_format("2026-03-31"), DateFormat::IsoDate);
    }

    #[test]
    fn detect_us_date() {
        assert_eq!(detect_format("03/31/2026"), DateFormat::UsDate);
    }

    #[test]
    fn detect_eu_date() {
        assert_eq!(detect_format("31/03/2026"), DateFormat::EuDate);
    }

    #[test]
    fn detect_ambiguous_date() {
        assert_eq!(detect_format("01/02/2026"), DateFormat::Ambiguous);
    }

    #[test]
    fn detect_year_only() {
        assert_eq!(detect_format("1948"), DateFormat::YearOnly);
    }

    #[test]
    fn detect_unix_seconds() {
        assert_eq!(detect_format("1711900800"), DateFormat::UnixSeconds);
    }

    #[test]
    fn detect_unix_millis() {
        assert_eq!(detect_format("1711900800000"), DateFormat::UnixMillis);
    }

    #[test]
    fn detect_long_date() {
        assert_eq!(detect_format("31 March 2026"), DateFormat::LongDate);
    }

    #[test]
    fn detect_abbrev_date() {
        assert_eq!(detect_format("Mar 31, 2026"), DateFormat::AbbrevDate);
    }

    #[test]
    fn detect_gedcom_approx() {
        assert_eq!(detect_format("ABT 1850"), DateFormat::GedcomApprox);
        assert_eq!(detect_format("circa 1900"), DateFormat::GedcomApprox);
    }

    #[test]
    fn detect_gedcom_range() {
        assert_eq!(detect_format("BET 1840 AND 1860"), DateFormat::GedcomRange);
    }

    #[test]
    fn detect_time_12() {
        assert_eq!(detect_format("2:30 PM"), DateFormat::Time12);
    }

    #[test]
    fn detect_time_24() {
        assert_eq!(detect_format("14:30"), DateFormat::Time24);
    }

    #[test]
    fn coerce_ambiguous_is_risky() {
        let result = coerce_datetime(&Value::String("01/02/2026".into()), "date");
        assert_eq!(result.diagnostic.unwrap().risk, RiskLevel::Risky);
    }

    #[test]
    fn coerce_iso_is_info() {
        let result = coerce_datetime(&Value::String("2026-03-31".into()), "date");
        assert_eq!(result.diagnostic.unwrap().risk, RiskLevel::Info);
    }
}
