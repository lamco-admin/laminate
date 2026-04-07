//! Iteration 278 — Date-EuropeanDot
//! Dot-separated European dates: DD.MM.YYYY

use laminate::packs::time::{convert_to_iso8601, detect_format, DateFormat};

#[test]
fn dot_date_detected_as_eu() {
    assert_eq!(detect_format("06.04.2026"), DateFormat::EuDate);
    assert_eq!(detect_format("31.12.2025"), DateFormat::EuDate);
}

#[test]
fn dot_date_converted() {
    assert_eq!(
        convert_to_iso8601("06.04.2026"),
        Some("2026-04-06".to_string())
    );
    assert_eq!(
        convert_to_iso8601("31.12.2025"),
        Some("2025-12-31".to_string())
    );
}

#[test]
fn slash_eu_still_works() {
    // Regression: slash-separated EU dates still work
    assert_eq!(
        convert_to_iso8601("31/12/2025"),
        Some("2025-12-31".to_string())
    );
}
