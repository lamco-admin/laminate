//! Iteration 269 — Date-FractionalSeconds
//! Verify convert_to_iso8601 preserves fractional seconds (PASS — preserved by design)

use laminate::packs::time::convert_to_iso8601;

#[test]
fn fractional_seconds_preserved() {
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00.456789"),
        Some("2026-04-06T14:30:00.456789".to_string())
    );
}

#[test]
fn fractional_seconds_space_normalized() {
    assert_eq!(
        convert_to_iso8601("2026-04-06 14:30:00.123"),
        Some("2026-04-06T14:30:00.123".to_string())
    );
}

#[test]
fn fractional_with_z_preserved() {
    assert_eq!(
        convert_to_iso8601("2026-04-06T14:30:00.123Z"),
        Some("2026-04-06T14:30:00.123Z".to_string())
    );
}
