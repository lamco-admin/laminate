//! Iteration 262 — Date-InvalidMonth
//! convert_to_iso8601("2026-13-01") should return None (month 13 is invalid)

use laminate::packs::time::convert_to_iso8601;

#[test]
fn invalid_month_13_rejected() {
    let result = convert_to_iso8601("2026-13-01");
    println!("convert_to_iso8601(\"2026-13-01\") = {:?}", result);
    assert_eq!(result, None, "Month 13 should be rejected");
}

#[test]
fn invalid_month_00_rejected() {
    let result = convert_to_iso8601("2026-00-15");
    println!("convert_to_iso8601(\"2026-00-15\") = {:?}", result);
    assert_eq!(result, None, "Month 0 should be rejected");
}

#[test]
fn invalid_day_32_rejected() {
    let result = convert_to_iso8601("2026-04-32");
    println!("convert_to_iso8601(\"2026-04-32\") = {:?}", result);
    assert_eq!(result, None, "Day 32 should be rejected");
}

#[test]
fn invalid_day_00_rejected() {
    let result = convert_to_iso8601("2026-04-00");
    println!("convert_to_iso8601(\"2026-04-00\") = {:?}", result);
    assert_eq!(result, None, "Day 0 should be rejected");
}

#[test]
fn valid_iso_date_passes() {
    let result = convert_to_iso8601("2026-04-06");
    assert_eq!(result, Some("2026-04-06".to_string()));
}

#[test]
fn valid_iso_datetime_passes() {
    let result = convert_to_iso8601("2026-04-06T14:30:00Z");
    assert_eq!(result, Some("2026-04-06T14:30:00Z".to_string()));
}

#[test]
fn invalid_month_in_iso_datetime() {
    let result = convert_to_iso8601("2026-13-06T14:30:00Z");
    println!(
        "convert_to_iso8601(\"2026-13-06T14:30:00Z\") = {:?}",
        result
    );
    assert_eq!(result, None, "Month 13 in datetime should be rejected");
}

#[test]
fn feb_29_non_leap_year() {
    // 1900 is NOT a leap year (century exception: divisible by 100 but not 400)
    let result = convert_to_iso8601("1900-02-29");
    println!("convert_to_iso8601(\"1900-02-29\") = {:?}", result);
    assert_eq!(result, None, "Feb 29 in 1900 (non-leap) should be rejected");
}

#[test]
fn feb_29_leap_year_valid() {
    let result = convert_to_iso8601("2024-02-29");
    assert_eq!(
        result,
        Some("2024-02-29".to_string()),
        "Feb 29 in 2024 (leap year) is valid"
    );
}
