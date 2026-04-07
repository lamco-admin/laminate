//! Iteration 272 — Date-YearOnlyExpansion
//! convert_to_iso8601("2026") expands to "2026-01-01" (PASS — by design)

use laminate::packs::time::convert_to_iso8601;

#[test]
fn year_only_expands_to_jan_first() {
    assert_eq!(convert_to_iso8601("2026"), Some("2026-01-01".to_string()));
}

#[test]
fn year_only_historical() {
    assert_eq!(convert_to_iso8601("1948"), Some("1948-01-01".to_string()));
}
