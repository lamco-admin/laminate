//! Iteration 267 — Date-ISOWeek
//! convert_to_iso8601 on ISO week dates (e.g. "2026-W14")

use laminate::packs::time::{convert_to_iso8601, detect_format, DateFormat};

#[test]
fn iso_week_detected() {
    assert_eq!(detect_format("2026-W14"), DateFormat::IsoWeekDate);
    assert_eq!(detect_format("2026W14"), DateFormat::IsoWeekDate);
    assert_eq!(detect_format("2026-W14-1"), DateFormat::IsoWeekDate);
    assert_eq!(detect_format("2026W141"), DateFormat::IsoWeekDate);
}

#[test]
fn iso_week_monday_of_w14() {
    // Python: datetime.date.fromisocalendar(2026, 14, 1) = 2026-03-30
    let result = convert_to_iso8601("2026-W14");
    assert_eq!(
        result,
        Some("2026-03-30".to_string()),
        "W14 Monday = Mar 30"
    );
}

#[test]
fn iso_week_with_day_monday() {
    let result = convert_to_iso8601("2026-W14-1");
    assert_eq!(result, Some("2026-03-30".to_string()));
}

#[test]
fn iso_week_compact_friday() {
    // 2026-W14-5 = Friday = 2026-04-03
    let result = convert_to_iso8601("2026W145");
    assert_eq!(result, Some("2026-04-03".to_string()));
}

#[test]
fn iso_week_01_crosses_year_boundary() {
    // 2026-W01-1 = Monday 2025-12-29 (week 1 can start in previous year)
    let result = convert_to_iso8601("2026-W01");
    assert_eq!(result, Some("2025-12-29".to_string()));
}

#[test]
fn iso_week_53_2020() {
    // 2020 has 53 weeks. W53-4 (Thursday) = 2020-12-31
    let result = convert_to_iso8601("2020-W53-4");
    assert_eq!(result, Some("2020-12-31".to_string()));
}
