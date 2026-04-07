//! Iteration 268 — Date-SpaceDateTime
//! convert_to_iso8601 normalizes space-separated datetimes to T-separated ISO 8601

use laminate::packs::time::convert_to_iso8601;

#[test]
fn space_datetime_normalized_to_t() {
    let result = convert_to_iso8601("2026-04-06 14:30:00");
    assert_eq!(result, Some("2026-04-06T14:30:00".to_string()));
}

#[test]
fn t_datetime_unchanged() {
    let result = convert_to_iso8601("2026-04-06T14:30:00");
    assert_eq!(result, Some("2026-04-06T14:30:00".to_string()));
}

#[test]
fn space_datetime_with_tz_normalized() {
    let result = convert_to_iso8601("2026-04-06 14:30:00+05:00");
    assert_eq!(result, Some("2026-04-06T14:30:00+05:00".to_string()));
}

#[test]
fn space_datetime_with_z_normalized() {
    let result = convert_to_iso8601("2026-04-06 14:30:00Z");
    assert_eq!(result, Some("2026-04-06T14:30:00Z".to_string()));
}

#[test]
fn date_only_not_affected() {
    let result = convert_to_iso8601("2026-04-06");
    assert_eq!(result, Some("2026-04-06".to_string()));
}
