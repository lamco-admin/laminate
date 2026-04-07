//! Iteration 275 — Date-GEDCOM-BET
//! convert_to_iso8601("BET 1840 AND 1860") returns None (PASS — range isn't a single date)

use laminate::packs::time::{convert_to_iso8601, detect_format, DateFormat};

#[test]
fn gedcom_bet_detected_as_range() {
    assert_eq!(detect_format("BET 1840 AND 1860"), DateFormat::GedcomRange);
}

#[test]
fn gedcom_bet_not_convertible_to_single_date() {
    // A GEDCOM range ("between X and Y") can't be a single ISO date
    assert_eq!(convert_to_iso8601("BET 1840 AND 1860"), None);
}

#[test]
fn gedcom_between_full_word() {
    assert_eq!(
        detect_format("BETWEEN 1900 AND 1920"),
        DateFormat::GedcomRange
    );
    assert_eq!(convert_to_iso8601("BETWEEN 1900 AND 1920"), None);
}
