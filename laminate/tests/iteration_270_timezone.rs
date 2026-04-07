//! Iteration 270 — Date-TimezoneNormalization
//! Verify convert_to_iso8601 preserves timezone offsets (PASS — preserved by design)

use laminate::packs::time::convert_to_iso8601;

#[test]
fn timezone_offsets_preserved() {
    // Positive offset preserved
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00+05:30"),
        Some("2026-04-06T14:30:00+05:30".to_string())
    );
    // Negative offset preserved
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00-04:00"),
        Some("2026-04-06T14:30:00-04:00".to_string())
    );
    // Z preserved
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00Z"),
        Some("2026-04-06T14:30:00Z".to_string())
    );
    // +00:00 NOT normalized to Z (preserves original)
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00+00:00"),
        Some("2026-04-06T14:30:00+00:00".to_string())
    );
    // No timezone stays absent
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00"),
        Some("2026-04-06T14:30:00".to_string())
    );
}
