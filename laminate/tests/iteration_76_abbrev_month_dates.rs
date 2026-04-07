//! Iteration 76 probe: abbreviated month dates with various separators
//! Target: "31-Mar-2026", "31 Mar 2026", and edge cases including false positives

use laminate::packs::time::{detect_format, DateFormat};

#[test]
fn abbrev_dash_day_first() {
    // RFC 2822 / financial style: "31-Mar-2026"
    assert_eq!(detect_format("31-Mar-2026"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_space_day_first() {
    // UK/military style: "31 Mar 2026"
    assert_eq!(detect_format("31 Mar 2026"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_us_comma_style() {
    // Already known to work: "Mar 31, 2026"
    assert_eq!(detect_format("Mar 31, 2026"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_all_caps() {
    // Financial/SWIFT style: "31-MAR-2026"
    assert_eq!(detect_format("31-MAR-2026"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_two_digit_year() {
    // Legacy financial: "31-MAR-26"
    assert_eq!(detect_format("31-MAR-26"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_no_separator() {
    // Airline/IATA style: "31Mar2026"
    assert_eq!(detect_format("31Mar2026"), DateFormat::AbbrevDate);
}

#[test]
fn abbrev_compressed_airline() {
    // Compressed airline: "31MAR26"
    assert_eq!(detect_format("31MAR26"), DateFormat::AbbrevDate);
}

// --- False positive probes ---
// These are common English words that contain 3-letter month substrings.
// Current implementation uses lower.contains("mar"), lower.contains("jun"), etc.
// These should NOT be classified as dates.

#[test]
fn false_positive_summary() {
    // "summary" contains "mar" (March)
    let result = detect_format("summary");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"summary\" should not be classified as an abbreviated date"
    );
}

#[test]
fn false_positive_decimal() {
    // "decimal" contains "dec" (December)
    let result = detect_format("decimal");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"decimal\" should not be classified as an abbreviated date"
    );
}

#[test]
fn false_positive_juniper() {
    // "juniper" contains "jun" (June)
    let result = detect_format("juniper");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"juniper\" should not be classified as an abbreviated date"
    );
}

#[test]
fn false_positive_approval() {
    // "approval" contains "apr" (April)
    let result = detect_format("approval");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"approval\" should not be classified as an abbreviated date"
    );
}

#[test]
fn false_positive_octopus() {
    // "octopus" contains "oct" (October)
    let result = detect_format("octopus");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"octopus\" should not be classified as an abbreviated date"
    );
}

#[test]
fn false_positive_marigold() {
    // "marigold" contains "mar" (March)
    let result = detect_format("marigold");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"marigold\" should not be classified as an abbreviated date"
    );
}

// Also check full month name false positives
#[test]
fn false_positive_augustine() {
    // "augustine" contains "august"
    let result = detect_format("augustine");
    assert_ne!(
        result,
        DateFormat::LongDate,
        "\"augustine\" should not be classified as a long date"
    );
}

#[test]
fn false_positive_separation() {
    // "separation" contains "sep" (September)
    let result = detect_format("separation");
    assert_ne!(
        result,
        DateFormat::AbbrevDate,
        "\"separation\" should not be classified as an abbreviated date"
    );
}
