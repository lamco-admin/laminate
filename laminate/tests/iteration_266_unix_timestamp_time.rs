//! Iteration 266 — Date-UnixTimestamp
//! convert_to_iso8601 on Unix timestamps now includes time component

use laminate::packs::time::convert_to_iso8601;

#[test]
fn unix_timestamp_includes_time() {
    // 1711900800 = 2024-03-31T16:00:00Z
    let result = convert_to_iso8601("1711900800");
    assert_eq!(result, Some("2024-03-31T16:00:00Z".to_string()));
}

#[test]
fn unix_timestamp_midnight() {
    // 1711843200 = 2024-03-31T00:00:00Z (exact midnight)
    let result = convert_to_iso8601("1711843200");
    assert_eq!(result, Some("2024-03-31T00:00:00Z".to_string()));
}

#[test]
fn unix_timestamp_end_of_day() {
    // 1711929599 = 2024-03-31T23:59:59Z
    let result = convert_to_iso8601("1711929599");
    assert_eq!(result, Some("2024-03-31T23:59:59Z".to_string()));
}

#[test]
fn unix_millis_includes_time() {
    // 1711900800000 = same as 1711900800 seconds
    let result = convert_to_iso8601("1711900800000");
    assert_eq!(result, Some("2024-03-31T16:00:00Z".to_string()));
}

#[test]
fn unix_epoch_zero() {
    // Edge: epoch itself (but likely below detection threshold)
    let result = convert_to_iso8601("946684801");
    // 946684801 = 2000-01-01T00:00:01Z
    assert_eq!(result, Some("2000-01-01T00:00:01Z".to_string()));
}
