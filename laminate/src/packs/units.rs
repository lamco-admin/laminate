//! Units coercion pack.
//!
//! Handles parsing of values with unit suffixes, unit detection,
//! and conversion between common unit systems.

use serde_json::Value;

use crate::coerce::CoercionResult;
use crate::diagnostic::{Diagnostic, DiagnosticKind, RiskLevel};

/// A parsed value with unit information.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitValue {
    /// The numeric amount.
    pub amount: f64,
    /// The detected unit (normalized lowercase).
    pub unit: String,
    /// The unit category.
    pub category: UnitCategory,
}

/// Categories of units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    /// Mass/weight (kg, lb, oz, g, etc.).
    Weight,
    /// Length/distance (m, km, ft, mi, etc.).
    Length,
    /// Temperature (°C, °F, K).
    Temperature,
    /// Volume (L, mL, gal, etc.).
    Volume,
    /// Time duration (s, min, h, etc.).
    Time,
    /// Digital data (KB, MB, GB, TB, etc.).
    Data,
    /// Frequency (Hz, kHz, MHz, GHz, THz).
    Frequency,
    /// Speed/velocity (m/s, km/h, mph, knots).
    Speed,
    /// Pressure (Pa, hPa, kPa, MPa, psi, atm, bar, mmHg).
    Pressure,
    /// Energy (J, kJ, kWh, Wh, cal, kcal, BTU).
    Energy,
    /// Force (N, kN, lbf).
    Force,
    /// Power (W, kW, MW, GW, hp).
    Power,
    /// Electrical (V, A, Ω, F, etc.).
    Electrical,
    /// Area (m², km², ft², acre, hectare).
    Area,
    /// Unrecognized unit category.
    Unknown,
}

/// UNECE Recommendation 20 / X12 EDI code mappings.
/// (standard_code, normalized_name, category)
const STANDARD_CODES: &[(&str, &str, UnitCategory)] = &[
    // UNECE Rec 20 codes (3-letter uppercase)
    ("KGM", "kg", UnitCategory::Weight),
    ("GRM", "g", UnitCategory::Weight),
    ("MGM", "mg", UnitCategory::Weight),
    ("LBR", "lb", UnitCategory::Weight),
    ("OZA", "oz", UnitCategory::Weight),
    ("TNE", "kg", UnitCategory::Weight), // metric ton → treat as 1000 kg
    ("MTR", "m", UnitCategory::Length),
    ("CMT", "cm", UnitCategory::Length),
    ("MMT", "mm", UnitCategory::Length),
    ("KMT", "km", UnitCategory::Length),
    ("FOT", "ft", UnitCategory::Length),
    ("INH", "in", UnitCategory::Length),
    ("YRD", "m", UnitCategory::Length), // yard → 0.9144 m
    ("LTR", "L", UnitCategory::Volume),
    ("MLT", "mL", UnitCategory::Volume),
    ("GLL", "gal", UnitCategory::Volume), // US gallon
    ("CEL", "°C", UnitCategory::Temperature),
    ("FAH", "°F", UnitCategory::Temperature),
    ("KEL", "K", UnitCategory::Temperature),
    ("SEC", "s", UnitCategory::Time),
    ("MIN", "min", UnitCategory::Time),
    ("HUR", "h", UnitCategory::Time),
    // X12 EDI Element 355 codes (2-letter)
    ("KG", "kg", UnitCategory::Weight),
    ("LB", "lb", UnitCategory::Weight),
    ("OZ", "oz", UnitCategory::Weight),
    ("FT", "ft", UnitCategory::Length),
    ("IN", "in", UnitCategory::Length),
    ("CM", "cm", UnitCategory::Length),
    ("MM", "mm", UnitCategory::Length),
    ("GA", "gal", UnitCategory::Volume),
    ("LT", "L", UnitCategory::Volume),
    ("ML", "mL", UnitCategory::Volume),
    ("CE", "°C", UnitCategory::Temperature),
    ("FA", "°F", UnitCategory::Temperature),
    ("HR", "h", UnitCategory::Time),
    ("DA", "h", UnitCategory::Time), // day → 24h
    // DOD FLIS codes (commonly used)
    ("GL", "gal", UnitCategory::Volume),
    ("MR", "m", UnitCategory::Length),
    ("LI", "L", UnitCategory::Volume),
];

/// Resolve a UNECE/X12/DOD standard code to a normalized unit name.
pub fn resolve_standard_code(code: &str) -> Option<(&'static str, UnitCategory)> {
    let upper = code.to_uppercase();
    STANDARD_CODES
        .iter()
        .find(|(c, _, _)| *c == upper.as_str())
        .map(|(_, name, cat)| (*name, *cat))
}

/// Known unit patterns: (suffix, normalized name, category).
const UNIT_PATTERNS: &[(&str, &str, UnitCategory)] = &[
    // Weight
    ("kg", "kg", UnitCategory::Weight),
    ("kgs", "kg", UnitCategory::Weight),
    ("kilogram", "kg", UnitCategory::Weight),
    ("kilograms", "kg", UnitCategory::Weight),
    ("g", "g", UnitCategory::Weight),
    ("grams", "g", UnitCategory::Weight),
    ("gram", "g", UnitCategory::Weight),
    ("mg", "mg", UnitCategory::Weight),
    ("lb", "lb", UnitCategory::Weight),
    ("lbs", "lb", UnitCategory::Weight),
    ("pound", "lb", UnitCategory::Weight),
    ("pounds", "lb", UnitCategory::Weight),
    ("oz", "oz", UnitCategory::Weight),
    ("ounce", "oz", UnitCategory::Weight),
    ("ounces", "oz", UnitCategory::Weight),
    // Length
    ("km", "km", UnitCategory::Length),
    ("m", "m", UnitCategory::Length),
    ("cm", "cm", UnitCategory::Length),
    ("mm", "mm", UnitCategory::Length),
    ("mi", "mi", UnitCategory::Length),
    ("miles", "mi", UnitCategory::Length),
    ("mile", "mi", UnitCategory::Length),
    ("ft", "ft", UnitCategory::Length),
    ("feet", "ft", UnitCategory::Length),
    ("foot", "ft", UnitCategory::Length),
    ("in", "in", UnitCategory::Length),
    ("inch", "in", UnitCategory::Length),
    ("inches", "in", UnitCategory::Length),
    // Temperature
    (" °c", "°C", UnitCategory::Temperature),
    ("°c", "°C", UnitCategory::Temperature),
    (" °f", "°F", UnitCategory::Temperature),
    ("°f", "°F", UnitCategory::Temperature),
    ("celsius", "°C", UnitCategory::Temperature),
    ("fahrenheit", "°F", UnitCategory::Temperature),
    // Volume
    ("l", "L", UnitCategory::Volume),
    ("liters", "L", UnitCategory::Volume),
    ("litres", "L", UnitCategory::Volume),
    ("liter", "L", UnitCategory::Volume),
    ("litre", "L", UnitCategory::Volume),
    ("ml", "mL", UnitCategory::Volume),
    ("gal", "gal", UnitCategory::Volume),
    ("gallons", "gal", UnitCategory::Volume),
    ("gallon", "gal", UnitCategory::Volume),
    // Data
    ("kb", "KB", UnitCategory::Data),
    ("mb", "MB", UnitCategory::Data),
    ("gb", "GB", UnitCategory::Data),
    ("tb", "TB", UnitCategory::Data),
    ("pb", "PB", UnitCategory::Data),
    // Time
    ("ms", "ms", UnitCategory::Time),
    ("milliseconds", "ms", UnitCategory::Time),
    ("millisecond", "ms", UnitCategory::Time),
    ("seconds", "s", UnitCategory::Time),
    ("second", "s", UnitCategory::Time),
    ("sec", "s", UnitCategory::Time),
    ("minutes", "min", UnitCategory::Time),
    ("minute", "min", UnitCategory::Time),
    ("min", "min", UnitCategory::Time),
    ("hours", "h", UnitCategory::Time),
    ("hour", "h", UnitCategory::Time),
    ("hrs", "h", UnitCategory::Time),
    ("hr", "h", UnitCategory::Time),
    ("h", "h", UnitCategory::Time),
    ("s", "s", UnitCategory::Time),
    // Nautical
    ("nautical miles", "nmi", UnitCategory::Length),
    ("nautical mile", "nmi", UnitCategory::Length),
    ("nmi", "nmi", UnitCategory::Length),
    ("nm", "nmi", UnitCategory::Length),
    // Kelvin
    ("kelvin", "K", UnitCategory::Temperature),
    // Frequency
    ("thz", "THz", UnitCategory::Frequency),
    ("ghz", "GHz", UnitCategory::Frequency),
    ("mhz", "MHz", UnitCategory::Frequency),
    ("khz", "kHz", UnitCategory::Frequency),
    ("hz", "Hz", UnitCategory::Frequency),
    // Power
    ("tw", "TW", UnitCategory::Power),
    ("gw", "GW", UnitCategory::Power),
    ("mw", "MW", UnitCategory::Power),
    ("kw", "kW", UnitCategory::Power),
    ("hp", "hp", UnitCategory::Power),
    ("w", "W", UnitCategory::Power),
    // Electrical
    ("kv", "kV", UnitCategory::Electrical),
    ("mv", "mV", UnitCategory::Electrical),
    ("v", "V", UnitCategory::Electrical),
    ("ma", "mA", UnitCategory::Electrical),
    ("a", "A", UnitCategory::Electrical),
    ("mah", "mAh", UnitCategory::Electrical),
    ("ah", "Ah", UnitCategory::Electrical),
    ("uf", "µF", UnitCategory::Electrical),
    ("nf", "nF", UnitCategory::Electrical),
    ("pf", "pF", UnitCategory::Electrical),
    ("ohm", "Ω", UnitCategory::Electrical),
    ("kohm", "kΩ", UnitCategory::Electrical),
    // Pressure
    ("mmhg", "mmHg", UnitCategory::Pressure),
    ("mpa", "MPa", UnitCategory::Pressure),
    ("kpa", "kPa", UnitCategory::Pressure),
    ("hpa", "hPa", UnitCategory::Pressure),
    ("pa", "Pa", UnitCategory::Pressure),
    ("bar", "bar", UnitCategory::Pressure),
    ("psi", "psi", UnitCategory::Pressure),
    ("atm", "atm", UnitCategory::Pressure),
    // Energy
    ("kwh", "kWh", UnitCategory::Energy),
    ("wh", "Wh", UnitCategory::Energy),
    ("kj", "kJ", UnitCategory::Energy),
    ("j", "J", UnitCategory::Energy),
    ("cal", "cal", UnitCategory::Energy),
    ("kcal", "kcal", UnitCategory::Energy),
    ("btu", "BTU", UnitCategory::Energy),
    // Speed (compound units)
    ("km/h", "km/h", UnitCategory::Speed),
    ("mph", "mph", UnitCategory::Speed),
    ("m/s", "m/s", UnitCategory::Speed),
    ("m/s²", "m/s²", UnitCategory::Speed),
    ("knots", "knots", UnitCategory::Speed),
    ("knot", "knots", UnitCategory::Speed),
    ("kn", "knots", UnitCategory::Speed),
    // Additional volume
    ("qt", "qt", UnitCategory::Volume),
    ("pt", "pt", UnitCategory::Volume),
    ("quart", "qt", UnitCategory::Volume),
    ("quarts", "qt", UnitCategory::Volume),
    ("pint", "pt", UnitCategory::Volume),
    ("pints", "pt", UnitCategory::Volume),
    ("fl oz", "fl oz", UnitCategory::Volume),
    ("cup", "cup", UnitCategory::Volume),
    ("cups", "cup", UnitCategory::Volume),
    // Additional data
    ("bytes", "bytes", UnitCategory::Data),
    ("byte", "bytes", UnitCategory::Data),
    // Additional weight
    ("stone", "stone", UnitCategory::Weight),
    ("tons", "tons", UnitCategory::Weight),
    ("ton", "tons", UnitCategory::Weight),
    ("tonne", "tonne", UnitCategory::Weight),
    ("tonnes", "tonne", UnitCategory::Weight),
    // Additional length
    ("yd", "yd", UnitCategory::Length),
    ("yard", "yd", UnitCategory::Length),
    ("yards", "yd", UnitCategory::Length),
    // Kelvin (single letter — placed after "k" ambiguity note)
    ("k", "K", UnitCategory::Temperature),
    // Area
    ("m²", "m²", UnitCategory::Area),
    ("km²", "km²", UnitCategory::Area),
    ("ft²", "ft²", UnitCategory::Area),
    ("hectare", "hectare", UnitCategory::Area),
    ("hectares", "hectare", UnitCategory::Area),
    ("acre", "acre", UnitCategory::Area),
    ("acres", "acre", UnitCategory::Area),
    // Miscellaneous
    ("lux", "lux", UnitCategory::Unknown),
    ("db", "dB", UnitCategory::Unknown),
    ("ppm", "ppm", UnitCategory::Unknown),
    ("mohm", "mohm", UnitCategory::Unknown),
];

/// How to convert between two units.
enum ConversionRule {
    /// Simple multiplication: result = value * factor
    Factor(f64),
    /// Formula with offset (e.g., temperature): result = value * scale + offset
    Formula { scale: f64, offset: f64 },
}

/// Conversion table: (from_unit, to_unit, rule)
const CONVERSIONS: &[(&str, &str, ConversionRule)] = &[
    // Weight (base: kg)
    ("g", "kg", ConversionRule::Factor(0.001)),
    ("kg", "g", ConversionRule::Factor(1000.0)),
    ("mg", "kg", ConversionRule::Factor(0.000001)),
    ("mg", "g", ConversionRule::Factor(0.001)),
    ("g", "mg", ConversionRule::Factor(1000.0)),
    ("lb", "kg", ConversionRule::Factor(0.453592)),
    ("kg", "lb", ConversionRule::Factor(2.20462)),
    ("oz", "kg", ConversionRule::Factor(0.0283495)),
    ("oz", "lb", ConversionRule::Factor(0.0625)),
    ("lb", "oz", ConversionRule::Factor(16.0)),
    ("oz", "g", ConversionRule::Factor(28.3495)),
    ("g", "oz", ConversionRule::Factor(0.035274)),
    // Length (base: m)
    ("km", "m", ConversionRule::Factor(1000.0)),
    ("m", "km", ConversionRule::Factor(0.001)),
    ("cm", "m", ConversionRule::Factor(0.01)),
    ("mm", "m", ConversionRule::Factor(0.001)),
    ("m", "cm", ConversionRule::Factor(100.0)),
    ("m", "mm", ConversionRule::Factor(1000.0)),
    ("mi", "km", ConversionRule::Factor(1.60934)),
    ("km", "mi", ConversionRule::Factor(0.621371)),
    ("ft", "m", ConversionRule::Factor(0.3048)),
    ("m", "ft", ConversionRule::Factor(3.28084)),
    ("in", "cm", ConversionRule::Factor(2.54)),
    ("cm", "in", ConversionRule::Factor(0.393701)),
    ("in", "m", ConversionRule::Factor(0.0254)),
    ("ft", "in", ConversionRule::Factor(12.0)),
    ("in", "ft", ConversionRule::Factor(1.0 / 12.0)),
    ("nmi", "km", ConversionRule::Factor(1.852)),
    ("km", "nmi", ConversionRule::Factor(0.539957)),
    ("nmi", "mi", ConversionRule::Factor(1.15078)),
    // Volume (base: l)
    ("ml", "l", ConversionRule::Factor(0.001)),
    ("l", "ml", ConversionRule::Factor(1000.0)),
    ("gal", "l", ConversionRule::Factor(3.78541)),
    ("l", "gal", ConversionRule::Factor(0.264172)),
    // Temperature — formula-based: result = value * scale + offset
    // °C → °F: F = C * 9/5 + 32
    (
        "c",
        "f",
        ConversionRule::Formula {
            scale: 9.0 / 5.0,
            offset: 32.0,
        },
    ),
    // °F → °C: C = (F - 32) * 5/9 = F * 5/9 - 32*5/9
    (
        "f",
        "c",
        ConversionRule::Formula {
            scale: 5.0 / 9.0,
            offset: -32.0 * 5.0 / 9.0,
        },
    ),
    // °C → K: K = C + 273.15
    (
        "c",
        "k",
        ConversionRule::Formula {
            scale: 1.0,
            offset: 273.15,
        },
    ),
    // K → °C: C = K - 273.15
    (
        "k",
        "c",
        ConversionRule::Formula {
            scale: 1.0,
            offset: -273.15,
        },
    ),
    // °F → K: K = (F - 32) * 5/9 + 273.15
    (
        "f",
        "k",
        ConversionRule::Formula {
            scale: 5.0 / 9.0,
            offset: -32.0 * 5.0 / 9.0 + 273.15,
        },
    ),
    // K → °F: F = (K - 273.15) * 9/5 + 32
    (
        "k",
        "f",
        ConversionRule::Formula {
            scale: 9.0 / 5.0,
            offset: -273.15 * 9.0 / 5.0 + 32.0,
        },
    ),
    // Data (base: bytes conceptually, but using common units)
    ("kb", "mb", ConversionRule::Factor(0.001)),
    ("mb", "kb", ConversionRule::Factor(1000.0)),
    ("mb", "gb", ConversionRule::Factor(0.001)),
    ("gb", "mb", ConversionRule::Factor(1000.0)),
    ("gb", "tb", ConversionRule::Factor(0.001)),
    ("tb", "gb", ConversionRule::Factor(1000.0)),
    // Time
    ("s", "min", ConversionRule::Factor(1.0 / 60.0)),
    ("min", "s", ConversionRule::Factor(60.0)),
    ("min", "h", ConversionRule::Factor(1.0 / 60.0)),
    ("h", "min", ConversionRule::Factor(60.0)),
    ("h", "s", ConversionRule::Factor(3600.0)),
    ("s", "h", ConversionRule::Factor(1.0 / 3600.0)),
    ("ms", "s", ConversionRule::Factor(0.001)),
    ("s", "ms", ConversionRule::Factor(1000.0)),
];

/// Parse a string value with a unit suffix.
///
/// Handles patterns like "0.5 kg", "1.2kg", "100g", "50 grams",
/// and compound units like "120 lbs 4 oz", "5 ft 11 in".
pub fn parse_unit_value(s: &str) -> Option<UnitValue> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try standard code resolution first (UNECE/X12/DOD — case-preserved)
    // Handles patterns like "500 KGM", "100 LBR", "37 CEL"
    if let Some(space_pos) = trimmed.rfind(' ') {
        let num_part = trimmed[..space_pos].trim();
        let code_part = trimmed[space_pos + 1..].trim();
        if let Some((normalized, category)) = resolve_standard_code(code_part) {
            if let Some(amount) = parse_numeric(&num_part.to_lowercase()) {
                return Some(UnitValue {
                    amount,
                    unit: normalized.to_string(),
                    category,
                });
            }
        }
    }
    // Also try when code is attached to number with no space: "500KGM"
    // Use char-based slicing to avoid panicking on multi-byte chars like °
    {
        let chars: Vec<char> = trimmed.chars().collect();
        for code_len in [3, 2] {
            if chars.len() > code_len {
                let suffix_chars: String = chars[chars.len() - code_len..].iter().collect();
                if suffix_chars.chars().all(|c| c.is_ascii_alphabetic()) {
                    if let Some((normalized, category)) = resolve_standard_code(&suffix_chars) {
                        let num_str: String = chars[..chars.len() - code_len].iter().collect();
                        let num_part = num_str.trim().to_lowercase();
                        if let Some(amount) = parse_numeric(&num_part) {
                            return Some(UnitValue {
                                amount,
                                unit: normalized.to_string(),
                                category,
                            });
                        }
                    }
                }
            }
        }
    }

    let s = trimmed.to_lowercase();

    // Normalize Unicode confusable: masculine ordinal indicator (º U+00BA)
    // looks identical to degree sign (° U+00B0) in most fonts.
    // Common in OCR output, Spanish/Portuguese locale keyboards, and web data.
    let s = s.replace('\u{00BA}', "\u{00B0}");

    // Try each known unit pattern (longest first to avoid "m" matching before "mm")
    let mut patterns: Vec<_> = UNIT_PATTERNS.iter().collect();
    patterns.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (suffix, normalized, category) in &patterns {
        if s.ends_with(suffix) {
            let num_part = s[..s.len() - suffix.len()].trim();
            if let Some(amount) = parse_numeric(num_part) {
                return Some(UnitValue {
                    amount,
                    unit: normalized.to_string(),
                    category: *category,
                });
            }
        }
    }

    // Try compound pattern: "120 lbs 4 oz", "5 ft 11 in"
    // Split into two unit values and convert the second to the first's unit
    parse_compound(&s, &patterns)
}

/// Parse a numeric string, stripping common formatting (commas, underscores, apostrophes).
/// Handles: "1500", "1,500", "1_500", "1'500", "1,234.5", "-3.5".
fn parse_numeric(s: &str) -> Option<f64> {
    // Try plain parse first (fast path for most inputs)
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }

    // European format: "2.500,5" (dot=thousands, comma=decimal)
    if s.contains(',') && s.contains('.') {
        let last_dot = s.rfind('.').expect("guarded by contains check above");
        let last_comma = s.rfind(',').expect("guarded by contains check above");
        if last_comma > last_dot {
            // Comma is decimal separator → European
            let normalized = s.replace('.', "").replace(',', ".");
            if let Ok(v) = normalized.parse::<f64>() {
                return Some(v);
            }
        }
    }

    // Multiple dots without comma: European thousands-only "1.234.567"
    if !s.contains(',') && s.matches('.').count() > 1 {
        let stripped = s.replace('.', "");
        if let Ok(v) = stripped.parse::<f64>() {
            return Some(v);
        }
    }

    // Strip comma/apostrophe/underscore thousands (US format: "1,500", "1'500")
    let stripped = strip_unit_numeric(s);
    if stripped != s {
        if let Ok(v) = stripped.parse::<f64>() {
            return Some(v);
        }
    }
    None
}

/// Strip thousands separators (commas, apostrophes, underscores) from a numeric
/// string in a unit context. Rejects patterns that don't look like thousands grouping.
fn strip_unit_numeric(s: &str) -> String {
    // Only strip if the string contains separators
    if !s.contains(',') && !s.contains('\'') && !s.contains('_') {
        return s.to_string();
    }
    // Reject consecutive separators
    if s.contains(",,") || s.contains("''") || s.contains("__") {
        return s.to_string();
    }
    // Reject leading/trailing separators (not numeric)
    let numeric_start = s.strip_prefix('-').unwrap_or(s);
    if numeric_start.starts_with([',', '\'', '_']) || s.ends_with([',', '\'', '_']) {
        return s.to_string();
    }
    // For commas: validate US-style grouping (reject European "24,51")
    if s.contains(',') {
        if let Some(dot_pos) = s.find('.') {
            if let Some(comma_pos) = s.rfind(',') {
                if comma_pos > dot_pos {
                    return s.to_string(); // European format
                }
            }
        }
        let integer_part = s.trim_start_matches('-').split('.').next().unwrap_or(s);
        let groups: Vec<&str> = integer_part.split(',').collect();
        if groups.len() > 1 {
            for group in &groups[1..] {
                if group.len() != 3 || !group.chars().all(|c| c.is_ascii_digit()) {
                    return s.to_string(); // Invalid grouping
                }
            }
        }
    }
    s.replace([',', '\'', '_'], "")
}

/// Parse compound unit expressions like "120 lbs 4 oz" or "5 ft 11 in".
/// Converts the secondary unit to the primary unit and sums them.
fn parse_compound(s: &str, patterns: &[&(&str, &str, UnitCategory)]) -> Option<UnitValue> {
    // Find the first unit match by scanning for a unit suffix followed by more content
    for (suffix, normalized, category) in patterns {
        // Look for "number unit rest" where rest contains another unit value
        if let Some(pos) = s.find(suffix) {
            let before_unit = s[..pos].trim();
            let after_unit = s[pos + suffix.len()..].trim();
            if before_unit.is_empty() || after_unit.is_empty() {
                continue;
            }
            // Check that before_unit is a number
            let primary_amount: f64 = match parse_numeric(before_unit) {
                Some(v) => v,
                None => continue,
            };
            // Try to parse the remainder as a unit value (recursive single-value parse)
            let mut secondary = None;
            for (suffix2, normalized2, _cat2) in patterns {
                if let Some(before_suffix) = after_unit.strip_suffix(suffix2) {
                    let num2 = before_suffix.trim();
                    if let Ok(amount2) = num2.parse::<f64>() {
                        secondary = Some((amount2, *normalized2));
                        break;
                    }
                }
            }
            if let Some((sec_amount, sec_unit)) = secondary {
                // Convert secondary to primary unit
                if let Some(factor) = conversion_factor(sec_unit, normalized) {
                    return Some(UnitValue {
                        amount: primary_amount + sec_amount * factor,
                        unit: normalized.to_string(),
                        category: *category,
                    });
                }
            }
        }
    }
    None
}

/// Get the conversion factor between two units.
/// Get the conversion factor between two units (for simple factor conversions only).
/// Returns None for formula-based conversions (use `convert()` instead).
pub fn conversion_factor(from: &str, to: &str) -> Option<f64> {
    let from = normalize_conv_key(from);
    let to = normalize_conv_key(to);
    if from == to {
        return Some(1.0);
    }
    CONVERSIONS
        .iter()
        .find(|(f, t, _)| *f == from && *t == to)
        .and_then(|(_, _, rule)| match rule {
            ConversionRule::Factor(f) => Some(*f),
            ConversionRule::Formula { .. } => None,
        })
}

/// Convert a value from one unit to another.
///
/// Handles both simple factor conversions (kg → lb) and formula-based
/// conversions (°C → °F) transparently.
///
/// Accepts both old-style (`"c"`, `"f"`, `"k"`) and new-style (`"°C"`, `"°F"`, `"K"`)
/// unit names for temperature conversions.
pub fn convert(amount: f64, from: &str, to: &str) -> Option<f64> {
    let from = normalize_conv_key(from);
    let to = normalize_conv_key(to);
    if from == to {
        return Some(amount);
    }
    CONVERSIONS
        .iter()
        .find(|(f, t, _)| *f == from && *t == to)
        .map(|(_, _, rule)| match rule {
            ConversionRule::Factor(factor) => amount * factor,
            ConversionRule::Formula { scale, offset } => amount * scale + offset,
        })
}

/// Normalize unit names so both "°C" and "c" map to the conversion table key "c".
fn normalize_conv_key(unit: &str) -> String {
    match unit {
        "°C" | "°c" => "c".to_string(),
        "°F" | "°f" => "f".to_string(),
        "K" => "k".to_string(),
        "L" => "l".to_string(),
        "mL" => "ml".to_string(),
        "KB" | "MB" | "GB" | "TB" | "PB" => unit.to_lowercase(),
        _ => unit.to_string(),
    }
}

/// Coerce a value with units, extracting the numeric amount.
pub fn coerce_unit_value(value: &Value, path: &str) -> CoercionResult {
    match value {
        Value::String(s) => {
            if let Some(uv) = parse_unit_value(s) {
                let new_value = serde_json::Number::from_f64(uv.amount)
                    .map(Value::Number)
                    .unwrap_or_else(|| value.clone());

                CoercionResult {
                    value: new_value,
                    coerced: true,
                    diagnostic: Some(Diagnostic {
                        path: path.to_string(),
                        kind: DiagnosticKind::Coerced {
                            from: format!("unit string ({} {})", uv.amount, uv.unit),
                            to: "f64".into(),
                        },
                        risk: RiskLevel::Warning,
                        suggestion: Some(format!(
                            "unit '{}' stripped from value; consider storing unit separately \
                             or using a structured type",
                            uv.unit
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

/// A parsed pack-size expression.
#[derive(Debug, Clone, PartialEq)]
pub struct PackSize {
    /// Total unit count (e.g., "1x100-count" → 100)
    pub total_units: u64,
    /// Number of inner packs (e.g., "6x500ml" → 6)
    pub packs: Option<u64>,
    /// Per-pack quantity with unit (e.g., "6x500ml" → Some(UnitValue{500, "ml"}))
    pub per_pack: Option<UnitValue>,
    /// The raw input that was parsed
    pub raw: String,
}

/// Parse pack-size notation used in supply chain and product data.
///
/// Handles patterns:
/// - `"1x100-count"` → 100 total
/// - `"case of 12"` → 12 total
/// - `"6x500ml"` → 6 packs × 500ml each = 3000ml total
/// - `"48-ct"` / `"48ct"` → 48 total
/// - `"dozen"` → 12
/// - `"gross"` → 144
/// - `"each"` / `"EA"` → 1
/// - `"pk/12"` → 12
pub fn parse_pack_notation(s: &str) -> Option<PackSize> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let lower = s.to_lowercase();

    // Named quantities
    match lower.as_str() {
        "each" | "ea" => {
            return Some(PackSize {
                total_units: 1,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
        "dozen" | "dz" => {
            return Some(PackSize {
                total_units: 12,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
        "gross" => {
            return Some(PackSize {
                total_units: 144,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
        "hundred" => {
            return Some(PackSize {
                total_units: 100,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
        _ => {}
    }

    // "NxN-count" pattern: "1x100-count", "3x100-count"
    // Skip hex literals like "0x100" — those start with "0x"/"0X"
    if let Some(x_pos) = lower.find('x') {
        let multiplier_str = &lower[..x_pos];
        let rest = &lower[x_pos + 1..];
        if multiplier_str == "0" && rest.chars().next().is_some_and(|c| c.is_ascii_hexdigit()) {
            // Looks like hex (0x...) — skip pack notation
        } else if let Ok(multiplier) = multiplier_str.parse::<u64>() {
            // Try "100-count" pattern
            let count_str = rest.replace("-count", "").replace("count", "");
            let count_str = count_str.trim();
            if let Ok(count) = count_str.parse::<u64>() {
                let total = multiplier.saturating_mul(count);
                return Some(PackSize {
                    total_units: total,
                    packs: Some(multiplier),
                    per_pack: None,
                    raw: s.to_string(),
                });
            }
            // Try "6x500ml" pattern — number with unit
            if let Some(uv) = parse_unit_value(rest) {
                return Some(PackSize {
                    total_units: multiplier.saturating_mul(uv.amount as u64),
                    packs: Some(multiplier),
                    per_pack: Some(uv),
                    raw: s.to_string(),
                });
            }
        }
    }

    // "case of N" / "pack of N" / "box of N"
    for prefix in &["case of ", "pack of ", "box of ", "carton of ", "bag of "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let num_str = rest.trim();
            if let Ok(n) = num_str.parse::<u64>() {
                return Some(PackSize {
                    total_units: n,
                    packs: None,
                    per_pack: None,
                    raw: s.to_string(),
                });
            }
        }
    }

    // "N-ct" / "Nct" pattern
    if lower.ends_with("-ct") || lower.ends_with("ct") {
        let num_str = if lower.ends_with("-ct") {
            &lower[..lower.len() - 3]
        } else {
            &lower[..lower.len() - 2]
        };
        if let Ok(n) = num_str.trim().parse::<u64>() {
            return Some(PackSize {
                total_units: n,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
    }

    // "N-pack" / "N pack"
    let pack_stripped = lower.replace("-pack", "").replace(" pack", "");
    if pack_stripped != lower {
        if let Ok(n) = pack_stripped.trim().parse::<u64>() {
            return Some(PackSize {
                total_units: n,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
    }

    // "pk/N" pattern
    if lower.starts_with("pk/") || lower.starts_with("pk ") {
        let num_str = &lower[3..].trim();
        if let Ok(n) = num_str.parse::<u64>() {
            return Some(PackSize {
                total_units: n,
                packs: None,
                per_pack: None,
                raw: s.to_string(),
            });
        }
    }

    None
}

/// Weight qualifier — distinguishes gross, net, and tare weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightQualifier {
    /// Gross weight (total including packaging/container).
    Gross,
    /// Net weight (contents only).
    Net,
    /// Tare weight (container/packaging only, gross - net).
    Tare,
    /// No qualifier specified.
    Unspecified,
}

/// A weight value with qualifier (gross/net/tare).
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedWeight {
    /// The numeric weight value.
    pub amount: f64,
    /// The unit (kg, lb, etc.).
    pub unit: String,
    /// Weight qualifier.
    pub qualifier: WeightQualifier,
}

/// Parse a weight string with optional qualifier prefix.
///
/// Recognizes patterns like:
/// - `"net wt: 12.5 kg"` → Net, 12.5, "kg"
/// - `"gross weight: 15 kg"` → Gross, 15.0, "kg"
/// - `"tare: 2.5 kg"` → Tare, 2.5, "kg"
/// - `"G.W. 15kg"` → Gross, 15.0, "kg"
/// - `"N.W. 12.5kg"` → Net, 12.5, "kg"
/// - `"T.W. 2.5kg"` → Tare, 2.5, "kg"
/// - `"12.5 kg"` → Unspecified, 12.5, "kg"
pub fn parse_qualified_weight(s: &str) -> Option<QualifiedWeight> {
    let s = s.trim();
    let lower = s.to_lowercase();

    // Detect qualifier prefix and strip it
    let (qualifier, remainder) = if lower.starts_with("net wt")
        || lower.starts_with("net weight")
        || lower.starts_with("n.w.")
        || lower.starts_with("nw:")
    {
        (WeightQualifier::Net, strip_weight_prefix(&lower))
    } else if lower.starts_with("gross wt")
        || lower.starts_with("gross weight")
        || lower.starts_with("g.w.")
        || lower.starts_with("gw:")
    {
        (WeightQualifier::Gross, strip_weight_prefix(&lower))
    } else if lower.starts_with("tare") || lower.starts_with("t.w.") || lower.starts_with("tw:") {
        (WeightQualifier::Tare, strip_weight_prefix(&lower))
    } else {
        (WeightQualifier::Unspecified, lower.clone())
    };

    // Parse the remaining unit value
    let uv = parse_unit_value(&remainder)?;
    if uv.category != UnitCategory::Weight {
        return None;
    }

    Some(QualifiedWeight {
        amount: uv.amount,
        unit: uv.unit,
        qualifier,
    })
}

fn strip_weight_prefix(s: &str) -> String {
    // Strip common prefixes and separators
    let stripped = s
        .trim_start_matches("net weight")
        .trim_start_matches("net wt")
        .trim_start_matches("gross weight")
        .trim_start_matches("gross wt")
        .trim_start_matches("tare weight")
        .trim_start_matches("tare")
        .trim_start_matches("n.w.")
        .trim_start_matches("g.w.")
        .trim_start_matches("t.w.")
        .trim_start_matches("nw:")
        .trim_start_matches("gw:")
        .trim_start_matches("tw:")
        .trim_start_matches(':')
        .trim_start_matches('.')
        .trim();
    stripped.to_string()
}

// ── uom integration (optional feature) ──────────────────────────

/// Convert a parsed UnitValue to a `uom` quantity.
///
/// Available when the `uom-integration` feature is enabled.
/// Returns the value as a `uom::si::f64` quantity for type-safe dimensional analysis.
#[cfg(feature = "uom-integration")]
pub mod uom_convert {
    use super::*;

    /// Convert a UnitValue to uom Mass (kg).
    pub fn to_mass(uv: &UnitValue) -> Option<uom::si::f64::Mass> {
        use uom::si::mass;
        match uv.unit.as_str() {
            "kg" => Some(uom::si::f64::Mass::new::<mass::kilogram>(uv.amount)),
            "g" => Some(uom::si::f64::Mass::new::<mass::gram>(uv.amount)),
            "mg" => Some(uom::si::f64::Mass::new::<mass::milligram>(uv.amount)),
            "lb" => Some(uom::si::f64::Mass::new::<mass::pound>(uv.amount)),
            _ => None,
        }
    }

    /// Convert a UnitValue to uom Length (m).
    pub fn to_length(uv: &UnitValue) -> Option<uom::si::f64::Length> {
        use uom::si::length;
        match uv.unit.as_str() {
            "m" => Some(uom::si::f64::Length::new::<length::meter>(uv.amount)),
            "km" => Some(uom::si::f64::Length::new::<length::kilometer>(uv.amount)),
            "cm" => Some(uom::si::f64::Length::new::<length::centimeter>(uv.amount)),
            "mm" => Some(uom::si::f64::Length::new::<length::millimeter>(uv.amount)),
            "ft" => Some(uom::si::f64::Length::new::<length::foot>(uv.amount)),
            "in" => Some(uom::si::f64::Length::new::<length::inch>(uv.amount)),
            "mi" => Some(uom::si::f64::Length::new::<length::mile>(uv.amount)),
            _ => None,
        }
    }

    /// Convert a UnitValue to uom ThermodynamicTemperature.
    pub fn to_temperature(uv: &UnitValue) -> Option<uom::si::f64::ThermodynamicTemperature> {
        use uom::si::thermodynamic_temperature;
        match uv.unit.as_str() {
            "c" => Some(uom::si::f64::ThermodynamicTemperature::new::<
                thermodynamic_temperature::degree_celsius,
            >(uv.amount)),
            "f" => Some(uom::si::f64::ThermodynamicTemperature::new::<
                thermodynamic_temperature::degree_fahrenheit,
            >(uv.amount)),
            "k" => Some(uom::si::f64::ThermodynamicTemperature::new::<
                thermodynamic_temperature::kelvin,
            >(uv.amount)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kg() {
        let uv = parse_unit_value("0.5 kg").unwrap();
        assert!((uv.amount - 0.5).abs() < f64::EPSILON);
        assert_eq!(uv.unit, "kg");
        assert_eq!(uv.category, UnitCategory::Weight);
    }

    #[test]
    fn parse_grams_no_space() {
        let uv = parse_unit_value("100g").unwrap();
        assert!((uv.amount - 100.0).abs() < f64::EPSILON);
        assert_eq!(uv.unit, "g");
    }

    #[test]
    fn parse_pounds() {
        let uv = parse_unit_value("2.5 lbs").unwrap();
        assert!((uv.amount - 2.5).abs() < f64::EPSILON);
        assert_eq!(uv.unit, "lb");
    }

    #[test]
    fn parse_grams_full_word() {
        let uv = parse_unit_value("50 grams").unwrap();
        assert!((uv.amount - 50.0).abs() < f64::EPSILON);
        assert_eq!(uv.unit, "g");
    }

    #[test]
    fn parse_mm() {
        let uv = parse_unit_value("3.2 mm").unwrap();
        assert_eq!(uv.unit, "mm");
        assert_eq!(uv.category, UnitCategory::Length);
    }

    #[test]
    fn parse_miles() {
        let uv = parse_unit_value("5.5 miles").unwrap();
        assert_eq!(uv.unit, "mi");
    }

    #[test]
    fn parse_gb() {
        let uv = parse_unit_value("16 GB").unwrap();
        assert_eq!(uv.unit, "GB");
        assert_eq!(uv.category, UnitCategory::Data);
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_unit_value("hello").is_none());
        assert!(parse_unit_value("").is_none());
    }

    #[test]
    fn convert_kg_to_lb() {
        let result = convert(1.0, "kg", "lb").unwrap();
        assert!((result - 2.20462).abs() < 0.001);
    }

    #[test]
    fn convert_lb_to_kg() {
        let result = convert(1.0, "lb", "kg").unwrap();
        assert!((result - 0.453592).abs() < 0.001);
    }

    #[test]
    fn convert_km_to_mi() {
        let result = convert(1.0, "km", "mi").unwrap();
        assert!((result - 0.621371).abs() < 0.001);
    }

    #[test]
    fn convert_same_unit() {
        let result = convert(42.0, "kg", "kg").unwrap();
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_unknown_pair() {
        assert!(convert(1.0, "kg", "miles").is_none());
    }

    #[test]
    fn coerce_strips_unit() {
        let result = coerce_unit_value(&Value::String("0.5 kg".into()), "weight");
        assert!(result.coerced);
        let num = result.value.as_f64().unwrap();
        assert!((num - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn coerce_non_unit_unchanged() {
        let result = coerce_unit_value(&Value::String("hello".into()), "weight");
        assert!(!result.coerced);
    }
}
