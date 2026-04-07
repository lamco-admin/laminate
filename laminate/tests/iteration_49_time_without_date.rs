use laminate::packs::time::{detect_format, DateFormat};

/// Iteration 49 — Pack Probe: Time-only formats with T prefix and seconds.
/// Result: GAP — T-prefix times and HH:MM:SS all returned Unknown.
/// Fix: replaced narrow len==5 check with is_time_24() supporting T prefix,
/// seconds, fractional seconds, and timezone suffixes.

#[test]
fn iter49_t_prefix_full_time() {
    assert_eq!(detect_format("T15:30:00"), DateFormat::Time24);
}

#[test]
fn iter49_t_prefix_short_time() {
    assert_eq!(detect_format("T15:30"), DateFormat::Time24);
}

#[test]
fn iter49_t_prefix_with_utc() {
    assert_eq!(detect_format("T15:30:00Z"), DateFormat::Time24);
}

#[test]
fn iter49_t_prefix_with_offset() {
    assert_eq!(detect_format("T15:30:00+02:00"), DateFormat::Time24);
    assert_eq!(detect_format("T15:30:00-05:00"), DateFormat::Time24);
}

#[test]
fn iter49_time_with_seconds() {
    assert_eq!(detect_format("14:30:00"), DateFormat::Time24);
}

#[test]
fn iter49_existing_hhmm_still_works() {
    assert_eq!(detect_format("14:30"), DateFormat::Time24);
}
