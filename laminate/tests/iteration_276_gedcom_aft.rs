//! Iteration 276 — Date-GEDCOM-AFT
//! GEDCOM before/after dates correctly detected and return None (PASS)

use laminate::packs::time::{convert_to_iso8601, detect_format, DateFormat};

#[test]
fn gedcom_aft_detected_and_unconvertible() {
    assert_eq!(detect_format("AFT 1900"), DateFormat::GedcomBeforeAfter);
    assert_eq!(convert_to_iso8601("AFT 1900"), None);
}

#[test]
fn gedcom_bef_detected_and_unconvertible() {
    assert_eq!(detect_format("BEF 1899"), DateFormat::GedcomBeforeAfter);
    assert_eq!(convert_to_iso8601("BEF 1899"), None);
}

#[test]
fn gedcom_after_full_word() {
    assert_eq!(detect_format("AFTER 1950"), DateFormat::GedcomBeforeAfter);
    assert_eq!(convert_to_iso8601("AFTER 1950"), None);
}

#[test]
fn gedcom_before_full_word() {
    assert_eq!(detect_format("BEFORE 1800"), DateFormat::GedcomBeforeAfter);
    assert_eq!(convert_to_iso8601("BEFORE 1800"), None);
}
