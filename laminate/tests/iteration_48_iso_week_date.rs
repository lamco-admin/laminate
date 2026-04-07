use laminate::packs::time::{detect_format, DateFormat};

/// Iteration 48 — Pack Probe: ISO 8601 week date format.
/// Result: GAP — week dates completely unrecognized.
/// Fix: added IsoWeekDate variant and is_iso_week_date() detection.

#[test]
fn iter48_iso_week_compact() {
    assert_eq!(detect_format("2026W13"), DateFormat::IsoWeekDate);
}

#[test]
fn iter48_iso_week_hyphenated() {
    assert_eq!(detect_format("2026-W13"), DateFormat::IsoWeekDate);
}

#[test]
fn iter48_iso_week_with_day() {
    assert_eq!(detect_format("2026-W13-1"), DateFormat::IsoWeekDate);
}

#[test]
fn iter48_iso_week_compact_with_day() {
    assert_eq!(detect_format("2026W131"), DateFormat::IsoWeekDate);
}

#[test]
fn iter48_not_week_date() {
    // Too short, invalid patterns
    assert_ne!(detect_format("2026W"), DateFormat::IsoWeekDate);
    assert_ne!(detect_format("2026X13"), DateFormat::IsoWeekDate);
    assert_ne!(detect_format("ABCDW13"), DateFormat::IsoWeekDate);
}
