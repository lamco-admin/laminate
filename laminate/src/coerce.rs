//! Type coercion engine — the core of laminate's data shaping.
//!
//! Four coercion levels control how aggressively values are transformed:
//! - [`CoercionLevel::Exact`] — no coercion, types must match
//! - [`CoercionLevel::SafeWidening`] — safe numeric widening (int→float)
//! - [`CoercionLevel::StringCoercion`] — parse strings to target types
//! - [`CoercionLevel::BestEffort`] — try everything (null→default, stringified JSON, locale numbers)
//!
//! The [`Coercible`] trait provides compile-time type hints for the coercion engine.
//! The [`CoercionDataSource`] trait allows external data (exchange rates, conversion factors).

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::diagnostic::{Diagnostic, DiagnosticKind, RiskLevel};

/// How aggressively to coerce values to the target type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoercionLevel {
    /// No coercion. Types must match exactly.
    Exact,
    /// Safe numeric widening only (int → float, not float → int).
    SafeWidening,
    /// Parse strings to target types (string → number, string → bool).
    StringCoercion,
    /// Try everything: string coercion + null → default + stringified JSON + array unwrap.
    BestEffort,
}

/// The result of attempting a coercion.
#[derive(Debug)]
pub struct CoercionResult {
    /// The coerced value (or original if no coercion needed/possible).
    pub value: Value,
    /// Whether coercion was applied.
    pub coerced: bool,
    /// Diagnostic produced by the coercion (if any).
    pub diagnostic: Option<Diagnostic>,
}

// ── Diagnostic helpers to reduce boilerplate ──────────────────

fn coerced(
    value: Value,
    path: &str,
    from: &str,
    to: &str,
    risk: RiskLevel,
    suggestion: Option<&str>,
) -> CoercionResult {
    CoercionResult {
        value,
        coerced: true,
        diagnostic: Some(Diagnostic {
            path: path.to_string(),
            kind: DiagnosticKind::Coerced {
                from: from.into(),
                to: to.into(),
            },
            risk,
            suggestion: suggestion.map(|s| s.to_string()),
        }),
    }
}

fn defaulted(value: Value, path: &str, desc: &str, suggestion: Option<&str>) -> CoercionResult {
    CoercionResult {
        value,
        coerced: true,
        diagnostic: Some(Diagnostic {
            path: path.to_string(),
            kind: DiagnosticKind::Defaulted {
                field: path.to_string(),
                value: desc.into(),
            },
            risk: RiskLevel::Warning,
            suggestion: suggestion.map(|s| s.to_string()),
        }),
    }
}

/// Check if an i64 value fits in the target integer type's range.
fn integer_fits_target(i: i64, target: &str) -> bool {
    match target {
        "i8" => i >= i8::MIN as i64 && i <= i8::MAX as i64,
        "i16" => i >= i16::MIN as i64 && i <= i16::MAX as i64,
        "i32" => i >= i32::MIN as i64 && i <= i32::MAX as i64,
        "i64" => true,
        "u8" => i >= 0 && i <= u8::MAX as i64,
        "u16" => i >= 0 && i <= u16::MAX as i64,
        "u32" => i >= 0 && i <= u32::MAX as i64,
        "u64" => i >= 0,
        "isize" => true,
        "usize" => i >= 0,
        _ => true,
    }
}

/// Try to parse a string with a radix prefix (0x, 0o, 0b) as an integer.
/// Returns None if the string doesn't have a recognized prefix or parsing fails.
fn try_parse_radix_int(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() < 3 || !s.is_char_boundary(2) {
        return None;
    }
    let (prefix, digits) = s.split_at(2);
    match prefix {
        "0x" | "0X" => i64::from_str_radix(digits, 16).ok(),
        "0o" | "0O" => i64::from_str_radix(digits, 8).ok(),
        "0b" | "0B" => i64::from_str_radix(digits, 2).ok(),
        _ => None,
    }
}

/// Try to strip US-style comma thousands separators from a numeric string.
/// Returns Some(stripped) only if the pattern looks like US-format commas:
/// - Optional leading minus
/// - Digits with commas every 3 digits (e.g., "1,000", "1,234,567")
/// - Optional decimal point and digits (e.g., "1,234.56")
///
/// Does NOT handle European format (dot thousands, comma decimal).
fn try_strip_comma_thousands(s: &str) -> Option<String> {
    if !s.contains(',') {
        return None;
    }
    // Reject European format: if a comma appears AFTER a dot, it's likely
    // European notation (e.g., "1.234,56") — don't strip.
    if let Some(dot_pos) = s.find('.') {
        if let Some(comma_pos) = s.rfind(',') {
            if comma_pos > dot_pos {
                return None;
            }
        }
    }
    // Reject double/consecutive commas
    if s.contains(",,") {
        return None;
    }
    // Validate US thousands grouping: every comma-separated group after the
    // first must have exactly 3 digits. "1,000" and "1,234,567" are valid;
    // "24,51" (European decimal) and "1,00" are not.
    let numeric_part = s.trim_start_matches('-');
    let integer_part = numeric_part.split('.').next().unwrap_or(numeric_part);
    let groups: Vec<&str> = integer_part.split(',').collect();
    if groups.len() > 1 {
        for group in &groups[1..] {
            if group.len() != 3 || !group.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
        }
        // First group must be 1-3 digits
        if groups[0].is_empty()
            || groups[0].len() > 3
            || !groups[0].chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }
    }
    // Strip commas and check if the result is a valid number
    let stripped: String = s.chars().filter(|c| *c != ',').collect();
    if stripped.parse::<f64>().is_ok() && !stripped.is_empty() {
        Some(stripped)
    } else {
        None
    }
}

/// Try parsing European number format: dot-thousands, comma-decimal.
/// "1.234,56" → "1234.56", "1.234.567" → "1234567"
fn try_parse_european(s: &str) -> Option<String> {
    let has_comma = s.contains(',');
    let has_dot = s.contains('.');

    // Dot+comma with comma after dot → European decimal
    if has_comma && has_dot {
        let last_dot = s.rfind('.').expect("guarded by has_dot");
        let last_comma = s.rfind(',').expect("guarded by has_comma");
        if last_comma > last_dot {
            let normalized = s.replace('.', "").replace(',', ".");
            if normalized.parse::<f64>().is_ok() {
                return Some(normalized);
            }
        }
    }

    // Multiple dots without comma → European thousands-only
    if !has_comma && has_dot && s.matches('.').count() > 1 {
        let stripped = s.replace('.', "");
        if stripped.parse::<f64>().is_ok() {
            return Some(stripped);
        }
    }

    None
}

/// Try stripping apostrophe or space thousands separators.
/// Swiss: "1'234'567" → "1234567", French/SI: "1 234 567" → "1234567"
/// Also handles comma-decimal: French "1 234,56" → "1234.56"
fn try_strip_alt_thousands(s: &str) -> Option<String> {
    let has_apostrophe = s.contains('\'');
    let has_space = s.contains(' ');
    if !has_apostrophe && !has_space {
        return None;
    }
    let sep = if has_apostrophe { '\'' } else { ' ' };
    // Validate grouping: separator-separated groups after the first must be 3 digits
    let numeric_part = s.trim_start_matches('-');
    // Split integer part from decimal, accepting either '.' or ',' as decimal separator
    let integer_part = numeric_part
        .split(['.', ','])
        .next()
        .unwrap_or(numeric_part);
    let groups: Vec<&str> = integer_part.split(sep).collect();
    if groups.len() <= 1 {
        return None;
    }
    for group in &groups[1..] {
        if group.len() != 3 || !group.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
    }
    if groups[0].is_empty() || groups[0].len() > 3 || !groups[0].chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    // Strip thousands separator and normalize comma-decimal to dot-decimal
    let stripped: String = s
        .chars()
        .filter(|c| *c != sep)
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    if stripped.parse::<f64>().is_ok() && !stripped.is_empty() {
        Some(stripped)
    } else {
        None
    }
}

/// Try stripping Rust/Python-style underscores from numeric strings.
/// "1_000" → "1000", "3.14_15" → "3.1415", "0xFF_FF" → "0xFFFF".
/// Rejects leading/trailing underscores and consecutive underscores.
fn try_strip_underscores(s: &str) -> Option<String> {
    if !s.contains('_') {
        return None;
    }
    // Reject leading/trailing underscores (looks like identifiers, not numbers)
    let numeric_start = s.strip_prefix('-').unwrap_or(s);
    if numeric_start.starts_with('_') || s.ends_with('_') {
        return None;
    }
    // Reject consecutive underscores
    if s.contains("__") {
        return None;
    }
    let stripped: String = s.chars().filter(|c| *c != '_').collect();
    if stripped.is_empty() {
        return None;
    }
    Some(stripped)
}

/// Check if a string is a null-sentinel value ("null", "None", "N/A", "NaN", etc.)
fn is_null_sentinel(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "null" | "none" | "n/a" | "na" | "nil" | "nan" | "unknown" | "undefined" | "-"
    )
}

/// Try null-sentinel coercion at BestEffort level, otherwise return no_coercion.
fn try_null_sentinel_or_passthrough(
    value: &Value,
    s: &str,
    _target_type: &str,
    level: CoercionLevel,
    path: &str,
) -> CoercionResult {
    if level >= CoercionLevel::BestEffort && is_null_sentinel(s) {
        coerced(
            Value::Null,
            path,
            "null-sentinel string",
            "null",
            RiskLevel::Warning,
            Some(
                "string contains a null-sentinel value; consider using actual null in the source data",
            ),
        )
    } else {
        no_coercion(value)
    }
}

fn no_coercion(value: &Value) -> CoercionResult {
    CoercionResult {
        value: value.clone(),
        coerced: false,
        diagnostic: None,
    }
}

fn flagged_no_coerce(
    value: &Value,
    path: &str,
    from: &str,
    to: &str,
    risk: RiskLevel,
    suggestion: &str,
) -> CoercionResult {
    CoercionResult {
        value: value.clone(),
        coerced: false,
        diagnostic: Some(Diagnostic {
            path: path.to_string(),
            kind: DiagnosticKind::Coerced {
                from: from.into(),
                to: to.into(),
            },
            risk,
            suggestion: Some(suggestion.into()),
        }),
    }
}

// ── CoercionDataSource trait ──────────────────────────────────

/// External data source for coercions that need live data.
///
/// Laminate owns the parsing and plumbing; the user owns the data.
/// Implementations might pull from REST APIs, databases, static tables, etc.
///
/// # Example
///
/// ```
/// use laminate::coerce::CoercionDataSource;
///
/// #[derive(Debug)]
/// struct StaticRates;
///
/// impl CoercionDataSource for StaticRates {
///     fn exchange_rate(&self, from: &str, to: &str) -> Option<f64> {
///         match (from, to) {
///             ("USD", "EUR") => Some(0.92),
///             ("EUR", "USD") => Some(1.09),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait CoercionDataSource: Send + Sync + std::fmt::Debug {
    /// Currency exchange rate lookup.
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64> {
        let _ = (from, to);
        None
    }

    /// Unit conversion factor lookup (multiply source by factor to get target).
    fn conversion_factor(&self, from_unit: &str, to_unit: &str) -> Option<f64> {
        let _ = (from_unit, to_unit);
        None
    }

    /// Custom domain-specific value lookup.
    fn lookup(&self, domain: &str, key: &str) -> Option<serde_json::Value> {
        let _ = (domain, key);
        None
    }
}

/// A no-op data source that returns None for all lookups.
#[derive(Debug, Default)]
pub struct NoDataSource;

impl CoercionDataSource for NoDataSource {}

/// A data source backed by static HashMaps.
#[derive(Debug, Default)]
pub struct StaticDataSource {
    /// Exchange rates keyed by (from_currency, to_currency).
    pub exchange_rates: std::collections::HashMap<(String, String), f64>,
    /// Unit conversion factors keyed by (from_unit, to_unit).
    pub conversion_factors: std::collections::HashMap<(String, String), f64>,
}

impl CoercionDataSource for StaticDataSource {
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64> {
        self.exchange_rates
            .get(&(from.to_string(), to.to_string()))
            .copied()
    }

    fn conversion_factor(&self, from_unit: &str, to_unit: &str) -> Option<f64> {
        self.conversion_factors
            .get(&(from_unit.to_string(), to_unit.to_string()))
            .copied()
    }
}

// ── Coercible trait ───────────────────────────────────────────

/// Trait for types that can be coerced from JSON values.
///
/// This replaces the fragile `std::any::type_name` approach with
/// a stable, trait-based coercion system. Types declare what coercions
/// they accept by returning a type hint string that the coercion engine
/// understands.
///
/// Primitive types have blanket implementations. For types laminate
/// doesn't know about, the default implementation returns `None` (no
/// coercion hint), and the value is passed directly to serde.
pub trait Coercible: DeserializeOwned {
    /// Return the coercion type hint for this type, or None to skip coercion
    /// and use plain serde deserialization.
    fn coercion_hint() -> Option<&'static str> {
        None
    }

    /// Whether this type is `Option<T>`. When true, null values bypass the
    /// Null→Default coercion and are passed directly to serde (which maps
    /// null to `None`). Without this, `extract::<Option<i64>>(null)` would
    /// produce `Some(0)` instead of `None`.
    fn is_optional() -> bool {
        false
    }

    /// For collection types like `Vec<T>`, returns T's coercion hint so that
    /// element-level coercion can be applied to each array element.
    fn element_hint() -> Option<&'static str> {
        None
    }

    /// Whether elements of this collection type are `Option<T>`.
    /// When true, null elements in the array bypass Null→Default coercion
    /// so serde can map them to `None`.
    fn is_element_optional() -> bool {
        false
    }
}

// Blanket impls for primitive types
impl Coercible for i8 {
    fn coercion_hint() -> Option<&'static str> {
        Some("i8")
    }
}
impl Coercible for i16 {
    fn coercion_hint() -> Option<&'static str> {
        Some("i16")
    }
}
impl Coercible for i32 {
    fn coercion_hint() -> Option<&'static str> {
        Some("i32")
    }
}
impl Coercible for i64 {
    fn coercion_hint() -> Option<&'static str> {
        Some("i64")
    }
}
impl Coercible for u8 {
    fn coercion_hint() -> Option<&'static str> {
        Some("u8")
    }
}
impl Coercible for u16 {
    fn coercion_hint() -> Option<&'static str> {
        Some("u16")
    }
}
impl Coercible for u32 {
    fn coercion_hint() -> Option<&'static str> {
        Some("u32")
    }
}
impl Coercible for u64 {
    fn coercion_hint() -> Option<&'static str> {
        Some("u64")
    }
}
impl Coercible for usize {
    fn coercion_hint() -> Option<&'static str> {
        Some("usize")
    }
}
impl Coercible for isize {
    fn coercion_hint() -> Option<&'static str> {
        Some("isize")
    }
}
impl Coercible for f32 {
    fn coercion_hint() -> Option<&'static str> {
        Some("f32")
    }
}
impl Coercible for f64 {
    fn coercion_hint() -> Option<&'static str> {
        Some("f64")
    }
}
impl Coercible for bool {
    fn coercion_hint() -> Option<&'static str> {
        Some("bool")
    }
}
impl Coercible for String {
    fn coercion_hint() -> Option<&'static str> {
        Some("String")
    }
}

// Option<T> delegates to T's hint — coercion applies to the inner value,
// but is_optional=true so null bypasses Null→Default coercion
impl<T: Coercible> Coercible for Option<T> {
    fn coercion_hint() -> Option<&'static str> {
        T::coercion_hint()
    }
    fn is_optional() -> bool {
        true
    }
}

// Vec<T> — element-level coercion applied when T has a hint
impl<T: Coercible> Coercible for Vec<T> {
    fn coercion_hint() -> Option<&'static str> {
        None
    }
    fn element_hint() -> Option<&'static str> {
        T::coercion_hint()
    }
    fn is_element_optional() -> bool {
        T::is_optional()
    }
}

// serde_json::Value — no coercion needed, accepts anything
impl Coercible for serde_json::Value {
    fn coercion_hint() -> Option<&'static str> {
        None
    }
}

/// Coerce a value using the trait-based system.
///
/// This is the preferred public API. Uses `Coercible::coercion_hint()` instead
/// of `type_name` to determine what coercions to apply.
pub fn coerce_for<T: Coercible>(value: &Value, level: CoercionLevel, path: &str) -> CoercionResult {
    // When the target is Option<T> and the value is null, skip coercion
    // entirely so serde maps null → None. Without this, the Null→Default
    // arm would produce Some(default) instead of None.
    if T::is_optional() && value.is_null() {
        return no_coercion(value);
    }

    match T::coercion_hint() {
        Some(hint) => {
            let result = coerce_value(value, hint, level, path);

            // When coercion produced null (e.g., null-sentinel "unknown" → null)
            // and the target is NOT Optional, chain into Null→Default so bare
            // types (f64, i64) get usable default values instead of serde errors.
            // Option<T> targets skip this — serde maps null → None natively.
            if result.value.is_null() && result.coerced && !T::is_optional() {
                let default_result = coerce_value(&Value::Null, hint, level, path);
                if default_result.coerced {
                    return CoercionResult {
                        value: default_result.value,
                        coerced: true,
                        diagnostic: result.diagnostic, // preserve the sentinel diagnostic
                    };
                }
            }

            result
        }
        None => {
            // No coercion hint at the top level. Check for element-level
            // coercion (e.g., Vec<i64> where each element can be coerced).
            if let (Some(elem_hint), Value::Array(arr)) = (T::element_hint(), value) {
                let elem_optional = T::is_element_optional();
                let mut coerced_any = false;
                let mut all_diagnostics: Vec<Diagnostic> = Vec::new();
                let coerced_elements: Vec<Value> = arr
                    .iter()
                    .enumerate()
                    .map(|(idx, elem)| {
                        let elem_path = format!("{}[{}]", path, idx);

                        // When elements are Option<T>, skip coercion on null values
                        // so serde maps null → None (not null→default→Some(0)).
                        if elem_optional && elem.is_null() {
                            return elem.clone();
                        }

                        let result = coerce_value(elem, elem_hint, level, &elem_path);

                        // When coercion produced null (e.g., null sentinel "N/A" → null)
                        // and the element is NOT Optional, chain into Null→Default so
                        // bare types (i64, f64) get usable default values instead of
                        // serde errors. This mirrors the logic in coerce_for's top-level.
                        if result.value.is_null() && result.coerced && !elem_optional {
                            let default_result =
                                coerce_value(&Value::Null, elem_hint, level, &elem_path);
                            if default_result.coerced {
                                coerced_any = true;
                                if let Some(d) = result.diagnostic {
                                    all_diagnostics.push(d);
                                }
                                return default_result.value;
                            }
                        }

                        if result.coerced {
                            coerced_any = true;
                        }
                        if let Some(d) = result.diagnostic {
                            all_diagnostics.push(d);
                        }
                        result.value
                    })
                    .collect();

                if coerced_any {
                    // Return first diagnostic in the standard field, stash the rest
                    // in a combined diagnostic that lists all element coercions.
                    let first = all_diagnostics.first().cloned();
                    if all_diagnostics.len() > 1 {
                        // Combine all element diagnostics into a single summary
                        let paths: Vec<String> =
                            all_diagnostics.iter().map(|d| d.path.clone()).collect();
                        CoercionResult {
                            value: Value::Array(coerced_elements),
                            coerced: true,
                            diagnostic: Some(Diagnostic {
                                path: path.to_string(),
                                kind: DiagnosticKind::Coerced {
                                    from: "mixed array".into(),
                                    to: elem_hint.into(),
                                },
                                risk: RiskLevel::Warning,
                                suggestion: Some(format!(
                                    "{} elements coerced at: {}",
                                    paths.len(),
                                    paths.join(", ")
                                )),
                            }),
                        }
                    } else {
                        CoercionResult {
                            value: Value::Array(coerced_elements),
                            coerced: true,
                            diagnostic: first,
                        }
                    }
                } else {
                    no_coercion(value)
                }
            } else {
                no_coercion(value)
            }
        }
    }
}

// ── Core coercion engine (string-based, used by derive macro and trait) ──

/// Attempt to coerce a JSON value toward a target type suitable for serde deserialization.
///
/// The `target_type` parameter is a type hint string (e.g., "i64", "bool", "String").
/// Prefer `coerce_for::<T>()` for the public API — this function is also used
/// internally by the derive macro which knows the type hint at compile time.
pub fn coerce_value(
    value: &Value,
    target_type: &str,
    level: CoercionLevel,
    path: &str,
) -> CoercionResult {
    if level == CoercionLevel::Exact {
        // Exact mode: reject cross-subtype numeric coercions that serde would allow.
        // serde_json silently widens integer → float, but Exact means "types must match exactly".
        // We substitute Value::Null so serde rejects it, and the diagnostic carries the real reason.
        if let Value::Number(n) = value {
            match target_type {
                "f32" | "f64" if n.is_i64() || n.is_u64() => {
                    return CoercionResult {
                        value: Value::Null,
                        coerced: false,
                        diagnostic: Some(Diagnostic {
                            path: path.to_string(),
                            kind: DiagnosticKind::Coerced {
                                from: "integer".into(),
                                to: target_type.into(),
                            },
                            risk: RiskLevel::Warning,
                            suggestion: Some(
                                "integer → float requires SafeWidening coercion level or higher"
                                    .into(),
                            ),
                        }),
                    };
                }
                _ => {}
            }
        }
        return no_coercion(value);
    }

    match (value, target_type) {
        // ── String → Numeric ──────────────────────────────────────
        (
            Value::String(s),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize",
        ) if level >= CoercionLevel::StringCoercion => {
            let s = s.trim();
            if let Ok(n) = s.parse::<i64>() {
                if integer_fits_target(n, target_type) {
                    coerced(
                        Value::Number(n.into()),
                        path,
                        "string",
                        target_type,
                        RiskLevel::Info,
                        Some("use an integer type in the source data"),
                    )
                } else {
                    flagged_no_coerce(
                        value,
                        path,
                        "string",
                        target_type,
                        RiskLevel::Risky,
                        "integer value from string overflows the target type",
                    )
                }
            } else if let Ok(n) = s.parse::<u64>() {
                // u64 values that didn't fit in i64 — check if they fit the target
                let fits = match target_type {
                    "u64" => true,
                    "usize" => true,
                    // u64 values > i64::MAX can't fit in any signed type
                    _ => false,
                };
                if fits {
                    coerced(
                        Value::Number(n.into()),
                        path,
                        "string",
                        target_type,
                        RiskLevel::Info,
                        Some("use an integer type in the source data"),
                    )
                } else {
                    flagged_no_coerce(
                        value,
                        path,
                        "string",
                        target_type,
                        RiskLevel::Risky,
                        "integer value from string overflows the target type",
                    )
                }
            } else if let Some(n) = try_parse_radix_int(s) {
                // Hex (0x1F), octal (0o77), binary (0b1010) string
                if integer_fits_target(n, target_type) {
                    coerced(
                        Value::Number(n.into()),
                        path,
                        "radix-prefixed string",
                        target_type,
                        RiskLevel::Warning,
                        Some("use a decimal integer in the source data"),
                    )
                } else {
                    flagged_no_coerce(
                        value,
                        path,
                        "radix-prefixed string",
                        target_type,
                        RiskLevel::Risky,
                        "radix-prefixed integer overflows the target type",
                    )
                }
            } else if level >= CoercionLevel::BestEffort {
                // Try comma-stripped thousands (e.g., "1,000" → "1000")
                if let Some(stripped) = try_strip_comma_thousands(s) {
                    if let Ok(n) = stripped.parse::<i64>() {
                        return if integer_fits_target(n, target_type) {
                            coerced(
                                Value::Number(n.into()),
                                path,
                                "comma-formatted string",
                                target_type,
                                RiskLevel::Warning,
                                Some("remove thousands separators from numeric strings"),
                            )
                        } else {
                            flagged_no_coerce(
                                value,
                                path,
                                "comma-formatted string",
                                target_type,
                                RiskLevel::Risky,
                                "comma-formatted integer overflows the target type",
                            )
                        };
                    }
                }
                // Try European format (e.g., "1.234,56" → "1234.56", "1.234.567" → "1234567")
                if let Some(normalized) = try_parse_european(s) {
                    if let Ok(n) = normalized.parse::<i64>() {
                        return if integer_fits_target(n, target_type) {
                            coerced(
                                Value::Number(n.into()),
                                path,
                                "European-formatted string",
                                target_type,
                                RiskLevel::Warning,
                                Some("use US number format (dot for decimal, comma for thousands)"),
                            )
                        } else {
                            flagged_no_coerce(
                                value,
                                path,
                                "European-formatted string",
                                target_type,
                                RiskLevel::Risky,
                                "European-formatted integer overflows the target type",
                            )
                        };
                    }
                }
                // Try apostrophe/space thousands (Swiss: "1'234", French: "1 234")
                if let Some(stripped) = try_strip_alt_thousands(s) {
                    if let Ok(n) = stripped.parse::<i64>() {
                        return if integer_fits_target(n, target_type) {
                            coerced(
                                Value::Number(n.into()),
                                path,
                                "locale-formatted string",
                                target_type,
                                RiskLevel::Warning,
                                Some("remove thousands separators from numeric strings"),
                            )
                        } else {
                            flagged_no_coerce(
                                value,
                                path,
                                "locale-formatted string",
                                target_type,
                                RiskLevel::Risky,
                                "locale-formatted integer overflows the target type",
                            )
                        };
                    }
                }
                // Try underscore-stripped (e.g., "1_000" → "1000")
                if let Some(stripped) = try_strip_underscores(s) {
                    if let Ok(n) = stripped.parse::<i64>() {
                        return if integer_fits_target(n, target_type) {
                            coerced(
                                Value::Number(n.into()),
                                path,
                                "underscore-formatted string",
                                target_type,
                                RiskLevel::Warning,
                                Some("remove underscores from numeric strings"),
                            )
                        } else {
                            flagged_no_coerce(
                                value,
                                path,
                                "underscore-formatted string",
                                target_type,
                                RiskLevel::Risky,
                                "underscore-formatted integer overflows the target type",
                            )
                        };
                    }
                    // Also try radix after stripping underscores (e.g., "0xFF_FF" → "0xFFFF")
                    if let Some(n) = try_parse_radix_int(&stripped) {
                        return if integer_fits_target(n, target_type) {
                            coerced(
                                Value::Number(n.into()),
                                path,
                                "underscore-formatted radix string",
                                target_type,
                                RiskLevel::Warning,
                                Some("remove underscores and use decimal integers"),
                            )
                        } else {
                            flagged_no_coerce(
                                value,
                                path,
                                "underscore-formatted radix string",
                                target_type,
                                RiskLevel::Risky,
                                "underscore-formatted radix integer overflows the target type",
                            )
                        };
                    }
                }
                try_null_sentinel_or_passthrough(value, s, target_type, level, path)
            } else {
                try_null_sentinel_or_passthrough(value, s, target_type, level, path)
            }
        }

        // "3.14" → 3.14
        (Value::String(s), "f32" | "f64") if level >= CoercionLevel::StringCoercion => {
            let s = s.trim();
            if let Ok(n) = s.parse::<f64>() {
                if n.is_nan() {
                    // "NaN" parses as f64 but can't be represented in JSON.
                    // Treat as null sentinel — maps to None for Option<f64>.
                    if level >= CoercionLevel::BestEffort {
                        coerced(
                            Value::Null,
                            path,
                            "NaN string",
                            "null",
                            RiskLevel::Warning,
                            Some(
                                "\"NaN\" represents missing/undefined data; consider using null instead",
                            ),
                        )
                    } else {
                        no_coercion(value)
                    }
                } else if n.is_infinite() {
                    // "Infinity"/"-Infinity" parse as f64 but can't be in JSON.
                    // Flag clearly rather than producing an opaque serde error.
                    flagged_no_coerce(
                        value,
                        path,
                        "string",
                        target_type,
                        RiskLevel::Risky,
                        "\"Infinity\" cannot be represented in JSON; use a sentinel value or null",
                    )
                } else if target_type == "f32" {
                    // Check f32 overflow/underflow before creating the Number.
                    // serde_json silently produces f32::INFINITY, which is never valid data.
                    let as_f32 = n as f32;
                    if as_f32.is_infinite() {
                        CoercionResult {
                            value: Value::Null,
                            coerced: false,
                            diagnostic: Some(Diagnostic {
                                path: path.to_string(),
                                kind: DiagnosticKind::Coerced {
                                    from: "string".into(),
                                    to: "f32".into(),
                                },
                                risk: RiskLevel::Risky,
                                suggestion: Some(format!(
                                    "value {:.6e} overflows f32 range (max ~3.4e38); use f64",
                                    n
                                )),
                            }),
                        }
                    } else if as_f32 == 0.0 && n != 0.0 {
                        CoercionResult {
                            value: Value::Null,
                            coerced: false,
                            diagnostic: Some(Diagnostic {
                                path: path.to_string(),
                                kind: DiagnosticKind::Coerced {
                                    from: "string".into(),
                                    to: "f32".into(),
                                },
                                risk: RiskLevel::Warning,
                                suggestion: Some(format!(
                                    "value {:.6e} underflows f32 (too small to represent); use f64",
                                    n
                                )),
                            }),
                        }
                    } else {
                        match serde_json::Number::from_f64(n) {
                            Some(num) => coerced(
                                Value::Number(num),
                                path,
                                "string",
                                target_type,
                                RiskLevel::Warning,
                                Some(
                                    "string-to-float coercion may lose decimal precision; consider a Decimal type for financial data",
                                ),
                            ),
                            None => no_coercion(value),
                        }
                    }
                } else {
                    match serde_json::Number::from_f64(n) {
                        Some(num) => coerced(
                            Value::Number(num),
                            path,
                            "string",
                            target_type,
                            RiskLevel::Warning,
                            Some(
                                "string-to-float coercion may lose decimal precision; consider a Decimal type for financial data",
                            ),
                        ),
                        None => no_coercion(value),
                    }
                }
            } else if level >= CoercionLevel::BestEffort {
                // Try comma-stripped thousands (e.g., "1,234.56" → "1234.56")
                if let Some(stripped) = try_strip_comma_thousands(s) {
                    if let Ok(n) = stripped.parse::<f64>() {
                        if !n.is_nan() && !n.is_infinite() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                return coerced(
                                    Value::Number(num),
                                    path,
                                    "comma-formatted string",
                                    target_type,
                                    RiskLevel::Warning,
                                    Some("remove thousands separators from numeric strings"),
                                );
                            }
                        }
                    }
                }
                // Try European format (e.g., "1.234,56" → "1234.56")
                if let Some(normalized) = try_parse_european(s) {
                    if let Ok(n) = normalized.parse::<f64>() {
                        if !n.is_nan() && !n.is_infinite() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                return coerced(
                                    Value::Number(num),
                                    path,
                                    "European-formatted string",
                                    target_type,
                                    RiskLevel::Warning,
                                    Some(
                                        "use US number format (dot for decimal, comma for thousands)",
                                    ),
                                );
                            }
                        }
                    }
                }
                // Try apostrophe/space thousands (Swiss: "1'234.56", French: "1 234.56")
                if let Some(stripped) = try_strip_alt_thousands(s) {
                    if let Ok(n) = stripped.parse::<f64>() {
                        if !n.is_nan() && !n.is_infinite() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                return coerced(
                                    Value::Number(num),
                                    path,
                                    "locale-formatted string",
                                    target_type,
                                    RiskLevel::Warning,
                                    Some("remove thousands separators from numeric strings"),
                                );
                            }
                        }
                    }
                }
                // Try underscore-stripped (e.g., "3.14_15" → "3.1415")
                if let Some(stripped) = try_strip_underscores(s) {
                    if let Ok(n) = stripped.parse::<f64>() {
                        if !n.is_nan() && !n.is_infinite() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                return coerced(
                                    Value::Number(num),
                                    path,
                                    "underscore-formatted string",
                                    target_type,
                                    RiskLevel::Warning,
                                    Some("remove underscores from numeric strings"),
                                );
                            }
                        }
                    }
                }
                try_null_sentinel_or_passthrough(value, s, target_type, level, path)
            } else {
                try_null_sentinel_or_passthrough(value, s, target_type, level, path)
            }
        }

        // ── String → Bool ─────────────────────────────────────────
        (Value::String(s), "bool") if level >= CoercionLevel::StringCoercion => {
            match s.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" | "y" | "t" => coerced(
                    Value::Bool(true),
                    path,
                    "string",
                    "bool",
                    RiskLevel::Info,
                    None,
                ),
                "false" | "0" | "no" | "off" | "n" | "f" => coerced(
                    Value::Bool(false),
                    path,
                    "string",
                    "bool",
                    RiskLevel::Info,
                    None,
                ),
                _ => try_null_sentinel_or_passthrough(value, s, target_type, level, path),
            }
        }

        // ── Number → Bool ─────────────────────────────────────────
        (Value::Number(n), "bool") if level >= CoercionLevel::SafeWidening => {
            if let Some(i) = n.as_i64() {
                match i {
                    0 => coerced(
                        Value::Bool(false),
                        path,
                        "integer",
                        "bool",
                        RiskLevel::Info,
                        Some("use a boolean type in the source data"),
                    ),
                    1 => coerced(
                        Value::Bool(true),
                        path,
                        "integer",
                        "bool",
                        RiskLevel::Info,
                        Some("use a boolean type in the source data"),
                    ),
                    _ => flagged_no_coerce(
                        value,
                        path,
                        "integer",
                        "bool",
                        RiskLevel::Risky,
                        "integer value other than 0/1 cannot be coerced to bool",
                    ),
                }
            } else {
                no_coercion(value)
            }
        }

        // ── Number → String ───────────────────────────────────────
        (Value::Number(n), "String" | "string") if level >= CoercionLevel::StringCoercion => {
            coerced(
                Value::String(n.to_string()),
                path,
                "number",
                "string",
                RiskLevel::Info,
                None,
            )
        }

        // ── Bool → String ─────────────────────────────────────────
        (Value::Bool(b), "String" | "string") if level >= CoercionLevel::StringCoercion => coerced(
            Value::String(b.to_string()),
            path,
            "bool",
            "string",
            RiskLevel::Info,
            None,
        ),

        // ── Bool → Integer ───────────────────────────────────────
        (
            Value::Bool(b),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize",
        ) if level >= CoercionLevel::SafeWidening => {
            let n = if *b { 1i64 } else { 0i64 };
            coerced(
                Value::Number(n.into()),
                path,
                "bool",
                target_type,
                RiskLevel::Info,
                Some("use an integer type in the source data"),
            )
        }

        // ── Bool → Float ─────────────────────────────────────────
        (Value::Bool(b), "f32" | "f64") if level >= CoercionLevel::SafeWidening => {
            let n = if *b { 1.0 } else { 0.0 };
            match serde_json::Number::from_f64(n) {
                Some(num) => coerced(
                    Value::Number(num),
                    path,
                    "bool",
                    target_type,
                    RiskLevel::Info,
                    Some("use a numeric type in the source data"),
                ),
                None => no_coercion(value),
            }
        }

        // ── Number → Integer (with overflow check) ─────────────────
        (
            Value::Number(n),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize",
        ) if level >= CoercionLevel::SafeWidening => {
            if let Some(i) = n.as_i64() {
                // Already an integer — just check range for narrowing
                if integer_fits_target(i, target_type) {
                    no_coercion(value)
                } else {
                    flagged_no_coerce(
                        value,
                        path,
                        "integer",
                        target_type,
                        RiskLevel::Risky,
                        &format!("value {} overflows {} range", i, target_type),
                    )
                }
            } else if let Some(u) = n.as_u64() {
                // Large unsigned that doesn't fit i64 — check unsigned targets
                match target_type {
                    "u64" | "usize" => no_coercion(value),
                    _ => flagged_no_coerce(
                        value,
                        path,
                        "integer",
                        target_type,
                        RiskLevel::Risky,
                        &format!("value {} overflows {} range", u, target_type),
                    ),
                }
            } else if let Some(f) = n.as_f64() {
                // Float → Integer (lossless conversion)
                if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    let i = f as i64;
                    if integer_fits_target(i, target_type) {
                        coerced(
                            Value::Number(i.into()),
                            path,
                            "float",
                            target_type,
                            RiskLevel::Info,
                            Some("use an integer in the source data"),
                        )
                    } else {
                        flagged_no_coerce(
                            value,
                            path,
                            "float",
                            target_type,
                            RiskLevel::Risky,
                            &format!("value {} overflows {} range", i, target_type),
                        )
                    }
                } else {
                    flagged_no_coerce(
                        value,
                        path,
                        "float",
                        target_type,
                        RiskLevel::Risky,
                        "float has fractional part; truncation would lose data",
                    )
                }
            } else {
                no_coercion(value)
            }
        }

        // ── Integer → Float (safe widening) ───────────────────────
        (Value::Number(n), "f32" | "f64") if level >= CoercionLevel::SafeWidening => {
            if let Some(i) = n.as_i64() {
                let f = i as f64;
                match serde_json::Number::from_f64(f) {
                    Some(num) => {
                        // f64 mantissa is 53 bits — integers beyond 2^53 lose precision.
                        // Can't use `f as i64 == i` because saturating casts mask the loss.
                        let lossless = f == (i as f64) && (i.unsigned_abs() <= (1_u64 << 53));
                        let risk = if lossless {
                            RiskLevel::Info
                        } else {
                            RiskLevel::Warning
                        };
                        let suggestion = if lossless {
                            None
                        } else {
                            Some("integer exceeds f64 exact range (2^53); precision may be lost")
                        };
                        coerced(
                            Value::Number(num),
                            path,
                            "integer",
                            target_type,
                            risk,
                            suggestion,
                        )
                    }
                    None => no_coercion(value),
                }
            } else if let Some(u) = n.as_u64() {
                // u64 values that don't fit in i64 (> i64::MAX)
                let f = u as f64;
                match serde_json::Number::from_f64(f) {
                    Some(num) => {
                        let lossless = u <= (1_u64 << 53);
                        let risk = if lossless {
                            RiskLevel::Info
                        } else {
                            RiskLevel::Warning
                        };
                        let suggestion = if lossless {
                            None
                        } else {
                            Some("integer exceeds f64 exact range (2^53); precision may be lost")
                        };
                        coerced(
                            Value::Number(num),
                            path,
                            "integer",
                            target_type,
                            risk,
                            suggestion,
                        )
                    }
                    None => no_coercion(value),
                }
            } else if target_type == "f32" {
                // Float value targeting f32 — check for overflow/underflow.
                // serde_json silently produces f32::INFINITY, which is never valid data.
                // We substitute Value::Null to force serde rejection (like Exact int→float).
                if let Some(f) = n.as_f64() {
                    let as_f32 = f as f32;
                    if as_f32.is_infinite() && f.is_finite() {
                        CoercionResult {
                            value: Value::Null,
                            coerced: false,
                            diagnostic: Some(Diagnostic {
                                path: path.to_string(),
                                kind: DiagnosticKind::Coerced {
                                    from: "f64".into(),
                                    to: "f32".into(),
                                },
                                risk: RiskLevel::Risky,
                                suggestion: Some(format!(
                                    "value {:.6e} overflows f32 range (max ~3.4e38); use f64",
                                    f
                                )),
                            }),
                        }
                    } else if as_f32 == 0.0 && f != 0.0 {
                        CoercionResult {
                            value: Value::Null,
                            coerced: false,
                            diagnostic: Some(Diagnostic {
                                path: path.to_string(),
                                kind: DiagnosticKind::Coerced {
                                    from: "f64".into(),
                                    to: "f32".into(),
                                },
                                risk: RiskLevel::Warning,
                                suggestion: Some(format!(
                                    "value {:.6e} underflows f32 (too small to represent); use f64",
                                    f
                                )),
                            }),
                        }
                    } else {
                        no_coercion(value)
                    }
                } else {
                    no_coercion(value)
                }
            } else {
                no_coercion(value)
            }
        }

        // ── Object/Array → String (JSON serialization) ──────────
        (Value::Object(_) | Value::Array(_), "String" | "string")
            if level >= CoercionLevel::BestEffort =>
        {
            let json_str = serde_json::to_string(value).unwrap_or_default();
            coerced(
                Value::String(json_str),
                path,
                "object/array",
                "String",
                RiskLevel::Warning,
                Some("complex value serialized to JSON string; consider using a structured type"),
            )
        }

        // ── Null → Default ────────────────────────────────────────
        (Value::Null, target) if level >= CoercionLevel::BestEffort => {
            let default_val = match target {
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                    Value::Number(0.into())
                }
                "f32" | "f64" => match serde_json::Number::from_f64(0.0) {
                    Some(n) => Value::Number(n),
                    None => return no_coercion(value),
                },
                "bool" => Value::Bool(false),
                "String" | "string" => Value::String(String::new()),
                _ => return no_coercion(value),
            };
            defaulted(
                default_val,
                path,
                "null → default",
                Some("null was replaced with a default value; consider making this field Optional"),
            )
        }

        // ── Stringified JSON → parsed ─────────────────────────────
        // Only when target is NOT String — if you asked for a String, you get the string as-is
        (Value::String(s), target)
            if level >= CoercionLevel::BestEffort && target != "String" && target != "string" =>
        {
            if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                if parsed.is_object() || parsed.is_array() {
                    return coerced(
                        parsed,
                        path,
                        "stringified JSON",
                        "parsed value",
                        RiskLevel::Warning,
                        Some(
                            "source embeds JSON as a string; consider fixing the upstream to send structured data",
                        ),
                    );
                }
            }
            no_coercion(value)
        }

        // ── Single-element array → scalar ─────────────────────────
        (Value::Array(arr), _) if arr.len() == 1 && level >= CoercionLevel::BestEffort => coerced(
            arr[0].clone(),
            path,
            "single-element array",
            "scalar",
            RiskLevel::Warning,
            Some("array with one element was unwrapped to scalar; verify this is intentional"),
        ),

        // ── Vec<T> element-level coercion ────────────────────────
        // When the derive macro passes "Vec<i64>" as target_type, coerce each element.
        (Value::Array(arr), target) if target.starts_with("Vec<") && target.ends_with('>') => {
            let inner_type = &target[4..target.len() - 1];
            let mut coerced_any = false;
            let mut all_diagnostics: Vec<Diagnostic> = Vec::new();
            let coerced_elements: Vec<Value> = arr
                .iter()
                .enumerate()
                .map(|(idx, elem)| {
                    let elem_path = format!("{}[{}]", path, idx);
                    let result = coerce_value(elem, inner_type, level, &elem_path);
                    if result.coerced {
                        coerced_any = true;
                    }
                    if let Some(d) = result.diagnostic {
                        all_diagnostics.push(d);
                    }
                    result.value
                })
                .collect();

            if coerced_any {
                let first = all_diagnostics.first().cloned();
                if all_diagnostics.len() > 1 {
                    let paths: Vec<String> =
                        all_diagnostics.iter().map(|d| d.path.clone()).collect();
                    CoercionResult {
                        value: Value::Array(coerced_elements),
                        coerced: true,
                        diagnostic: Some(Diagnostic {
                            path: path.to_string(),
                            kind: DiagnosticKind::Coerced {
                                from: "mixed array".into(),
                                to: inner_type.into(),
                            },
                            risk: RiskLevel::Warning,
                            suggestion: Some(format!(
                                "{} elements coerced at: {}",
                                paths.len(),
                                paths.join(", ")
                            )),
                        }),
                    }
                } else {
                    CoercionResult {
                        value: Value::Array(coerced_elements),
                        coerced: true,
                        diagnostic: first,
                    }
                }
            } else {
                no_coercion(value)
            }
        }

        // ── No coercion applicable ────────────────────────────────
        _ => no_coercion(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_integer() {
        let result = coerce_value(
            &Value::String("42".into()),
            "i64",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(42.into()));
    }

    #[test]
    fn string_to_float() {
        let result = coerce_value(
            &Value::String("3.14".into()),
            "f64",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
    }

    #[test]
    fn string_to_bool_true() {
        let result = coerce_value(
            &Value::String("true".into()),
            "bool",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn string_to_bool_yes() {
        let result = coerce_value(
            &Value::String("yes".into()),
            "bool",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn number_to_string() {
        let result = coerce_value(
            &Value::Number(42.into()),
            "String",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::String("42".into()));
    }

    #[test]
    fn lossless_float_to_int() {
        let v = serde_json::Number::from_f64(3.0).unwrap();
        let result = coerce_value(
            &Value::Number(v),
            "i64",
            CoercionLevel::SafeWidening,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(3.into()));
    }

    #[test]
    fn lossy_float_to_int_flagged() {
        let v = serde_json::Number::from_f64(3.7).unwrap();
        let result = coerce_value(
            &Value::Number(v),
            "i64",
            CoercionLevel::SafeWidening,
            "test",
        );
        assert!(!result.coerced);
        assert!(result.diagnostic.is_some());
        assert_eq!(result.diagnostic.unwrap().risk, RiskLevel::Risky);
    }

    #[test]
    fn null_to_default_int() {
        let result = coerce_value(&Value::Null, "i64", CoercionLevel::BestEffort, "test");
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(0.into()));
    }

    #[test]
    fn null_to_default_string() {
        let result = coerce_value(&Value::Null, "String", CoercionLevel::BestEffort, "test");
        assert!(result.coerced);
        assert_eq!(result.value, Value::String(String::new()));
    }

    #[test]
    fn stringified_json() {
        let result = coerce_value(
            &Value::String(r#"{"a":1}"#.into()),
            "object",
            CoercionLevel::BestEffort,
            "test",
        );
        assert!(result.coerced);
        assert!(result.value.is_object());
    }

    #[test]
    fn single_element_array() {
        let result = coerce_value(
            &Value::Array(vec![Value::Number(42.into())]),
            "i64",
            CoercionLevel::BestEffort,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(42.into()));
    }

    #[test]
    fn exact_mode_no_coercion() {
        let result = coerce_value(
            &Value::String("42".into()),
            "i64",
            CoercionLevel::Exact,
            "test",
        );
        assert!(!result.coerced);
        assert_eq!(result.value, Value::String("42".into()));
    }

    #[test]
    fn int_one_to_bool_true() {
        let result = coerce_value(
            &Value::Number(1.into()),
            "bool",
            CoercionLevel::SafeWidening,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn int_zero_to_bool_false() {
        let result = coerce_value(
            &Value::Number(0.into()),
            "bool",
            CoercionLevel::SafeWidening,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Bool(false));
    }

    #[test]
    fn int_other_to_bool_risky() {
        let result = coerce_value(
            &Value::Number(42.into()),
            "bool",
            CoercionLevel::SafeWidening,
            "test",
        );
        assert!(!result.coerced);
        assert!(result.diagnostic.is_some());
        assert_eq!(result.diagnostic.unwrap().risk, RiskLevel::Risky);
    }

    #[test]
    fn bool_to_string() {
        let result = coerce_value(
            &Value::Bool(true),
            "String",
            CoercionLevel::StringCoercion,
            "test",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::String("true".into()));
    }

    // ── Trait-based coercion tests ──

    #[test]
    fn coerce_for_primitive() {
        let result = coerce_for::<u16>(
            &Value::String("8080".into()),
            CoercionLevel::BestEffort,
            "port",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(8080.into()));
    }

    #[test]
    fn coerce_for_bool() {
        let result = coerce_for::<bool>(
            &Value::String("yes".into()),
            CoercionLevel::BestEffort,
            "flag",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn coerce_for_option_uses_inner_hint() {
        // Option<u32> should coerce string → u32 just like u32
        let result = coerce_for::<Option<u32>>(
            &Value::String("42".into()),
            CoercionLevel::BestEffort,
            "val",
        );
        assert!(result.coerced);
        assert_eq!(result.value, Value::Number(42.into()));
    }

    #[test]
    fn coerce_for_vec_no_coercion() {
        // Vec<String> has no coercion hint — passes through to serde
        let arr = Value::Array(vec![Value::String("a".into())]);
        let result = coerce_for::<Vec<String>>(&arr, CoercionLevel::BestEffort, "val");
        assert!(!result.coerced);
    }

    #[test]
    fn coerce_for_value_no_coercion() {
        // serde_json::Value accepts anything — no coercion needed
        let result = coerce_for::<serde_json::Value>(
            &Value::Number(42.into()),
            CoercionLevel::BestEffort,
            "val",
        );
        assert!(!result.coerced);
    }

    #[test]
    fn coerce_for_string_from_number() {
        let result =
            coerce_for::<String>(&Value::Number(42.into()), CoercionLevel::BestEffort, "val");
        assert!(result.coerced);
        assert_eq!(result.value, Value::String("42".into()));
    }
}
