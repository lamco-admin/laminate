/// Iteration 124: convert_to_iso8601 validates calendar dates — BUG fixed
///
/// "29 Feb 2023" was producing "2023-02-29" which doesn't exist.
/// Now validates day-of-month against calendar (including leap years).
use laminate::packs::time::convert_to_iso8601;

#[test]
fn leap_year_valid() {
    let result = convert_to_iso8601("29 Feb 2024");
    assert_eq!(result, Some("2024-02-29".to_string()));
}

#[test]
fn non_leap_year_rejected() {
    let result = convert_to_iso8601("29 Feb 2023");
    assert!(
        result.is_none(),
        "Feb 29 in non-leap year should be rejected"
    );
}

#[test]
fn feb_28_non_leap_valid() {
    let result = convert_to_iso8601("28 Feb 2023");
    assert_eq!(result, Some("2023-02-28".to_string()));
}

#[test]
fn day_31_in_30_day_month() {
    let result = convert_to_iso8601("31 Apr 2026");
    assert!(result.is_none(), "April has only 30 days");
}
