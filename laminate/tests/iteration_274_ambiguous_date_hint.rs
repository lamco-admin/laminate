//! Iteration 274 — Date-AmbiguousUSvsEU
//! Test detect_format and convert_to_iso8601_with_hint on ambiguous "01/02/2026"

use laminate::packs::time::{convert_to_iso8601_with_hint, detect_format, DateFormat};

#[test]
fn ambiguous_detected() {
    // Both 01 and 02 are ≤ 12, so detect_format cannot determine US vs EU
    assert_eq!(detect_format("01/02/2026"), DateFormat::Ambiguous);
}

#[test]
fn day_first_hint_gives_eu_interpretation() {
    // day_first=true: DD/MM/YYYY → 01 is day, 02 is month → February 1
    let result = convert_to_iso8601_with_hint("01/02/2026", true);
    eprintln!("day_first=true: {:?}", result);
    assert_eq!(result, Some("2026-02-01".to_string()));
}

#[test]
fn month_first_hint_gives_us_interpretation() {
    // day_first=false: MM/DD/YYYY → 01 is month, 02 is day → January 2
    let result = convert_to_iso8601_with_hint("01/02/2026", false);
    eprintln!("day_first=false: {:?}", result);
    assert_eq!(result, Some("2026-01-02".to_string()));
}
