//! Iteration 75 — Domain Pack: Space-separated datetime
//!
//! GAP: "2026-03-31 15:30:00" (database-style datetime) returned Unknown.
//! Fix: Extended ISO 8601 check to accept space separator at position 10.

use laminate::packs::time::{detect_format, DateFormat};

#[test]
fn space_separated_datetime() {
    assert_eq!(detect_format("2026-03-31 15:30:00"), DateFormat::Iso8601);
}

#[test]
fn space_datetime_with_millis() {
    // PostgreSQL-style fractional seconds
    assert_eq!(
        detect_format("2026-03-31 15:30:00.123"),
        DateFormat::Iso8601
    );
}

#[test]
fn space_datetime_with_tz() {
    assert_eq!(
        detect_format("2026-03-31 15:30:00+02:00"),
        DateFormat::Iso8601
    );
}

#[test]
fn t_separated_unchanged() {
    assert_eq!(detect_format("2026-03-31T15:30:00Z"), DateFormat::Iso8601);
}

#[test]
fn iso_date_only_unchanged() {
    // Date-only should still be IsoDate, not Iso8601
    assert_eq!(detect_format("2026-03-31"), DateFormat::IsoDate);
}
