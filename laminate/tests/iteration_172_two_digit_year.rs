// Iteration 172: 2-digit year century boundary in convert_to_iso8601
// Fresh target — pivot at 30: 00-29 → 2000-2029, 30-99 → 1930-1999
// Test the boundary values around the pivot.

use laminate::packs::time::convert_to_iso8601;

#[test]
fn two_digit_year_00() {
    // "01-Jan-00" → 2000
    let result = convert_to_iso8601("01-Jan-00");
    assert!(result.is_some(), "01-Jan-00 should parse");
    let iso = result.unwrap();
    assert!(
        iso.starts_with("2000-"),
        "Year 00 should map to 2000, got: {}",
        iso
    );
}

#[test]
fn two_digit_year_29() {
    // "01-Jan-29" → 2029 (last year before pivot)
    let result = convert_to_iso8601("01-Jan-29");
    assert!(result.is_some(), "01-Jan-29 should parse");
    let iso = result.unwrap();
    assert!(
        iso.starts_with("2029-"),
        "Year 29 should map to 2029, got: {}",
        iso
    );
}

#[test]
fn two_digit_year_30() {
    // "01-Jan-30" → 1930 (first year after pivot)
    let result = convert_to_iso8601("01-Jan-30");
    assert!(result.is_some(), "01-Jan-30 should parse");
    let iso = result.unwrap();
    assert!(
        iso.starts_with("1930-"),
        "Year 30 should map to 1930, got: {}",
        iso
    );
}

#[test]
fn two_digit_year_99() {
    // "01-Jan-99" → 1999
    let result = convert_to_iso8601("01-Jan-99");
    assert!(result.is_some(), "01-Jan-99 should parse");
    let iso = result.unwrap();
    assert!(
        iso.starts_with("1999-"),
        "Year 99 should map to 1999, got: {}",
        iso
    );
}

#[test]
fn two_digit_year_26() {
    // "31-Mar-26" → 2026 (current year)
    let result = convert_to_iso8601("31-Mar-26");
    assert!(result.is_some());
    let iso = result.unwrap();
    assert!(
        iso.starts_with("2026-"),
        "Year 26 should map to 2026, got: {}",
        iso
    );
}

#[test]
fn slash_date_with_two_digit_year() {
    // "03/15/26" — US date with 2-digit year
    let result = convert_to_iso8601("03/15/26");
    println!("03/15/26 → {:?}", result);
    // 15 > 12 so must be day → 03=month, 15=day, 26=year → 2026
    if let Some(iso) = result {
        assert!(
            iso.starts_with("2026-"),
            "Slash date year 26 → 2026, got: {}",
            iso
        );
    }
}
