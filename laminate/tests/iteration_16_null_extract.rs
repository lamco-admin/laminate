//! Iteration 16: Null Injection — SpaceX latest_launch.json
//! Bug: extract::<Option<T>>() on null values produced Some(default) instead of None.
//! Root cause: Option<T>::coercion_hint() delegated to T, so coercion saw "String"
//! and applied Null→Default (""), then serde wrapped it as Some("").
//! Fix: Added Coercible::is_optional() — when true and value is null, coerce_for
//! skips coercion entirely, letting serde map null → None.

use laminate::FlexValue;

fn load_spacex() -> FlexValue {
    let full = format!(
        "{}/testdata/spacex/latest_launch.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&full).unwrap();
    FlexValue::from_json(&json).unwrap()
}

#[test]
fn iter16_null_extract_as_string_gets_default() {
    // extract::<String>(null) should still coerce to "" (Null→Default for bare types)
    let flex = load_spacex();
    let result: String = flex.extract("static_fire_date_utc").unwrap();
    assert_eq!(result, "");
}

#[test]
fn iter16_null_extract_as_option_string_gets_none() {
    // extract::<Option<String>>(null) must return None, not Some("")
    let flex = load_spacex();
    let result: Option<String> = flex.extract("static_fire_date_utc").unwrap();
    assert_eq!(result, None);
}

#[test]
fn iter16_null_extract_as_option_i64_gets_none() {
    // extract::<Option<i64>>(null) must return None, not Some(0)
    let flex = load_spacex();
    let result: Option<i64> = flex.extract("static_fire_date_unix").unwrap();
    assert_eq!(result, None);
}

#[test]
fn iter16_maybe_still_works() {
    // maybe() should continue to return None for null values
    let flex = load_spacex();
    let result = flex.maybe::<String>("static_fire_date_utc").unwrap();
    assert_eq!(result, None);
}

#[test]
fn iter16_option_non_null_still_coerces() {
    // extract::<Option<i64>> on a non-null value should still coerce normally
    // flight_number is 187 (integer) — extracting as Option<i64> should give Some(187)
    let flex = load_spacex();
    let result: Option<i64> = flex.extract("flight_number").unwrap();
    assert_eq!(result, Some(187));
}

#[test]
fn iter16_option_string_coercion_on_non_null() {
    // success is true (boolean) — extract as Option<String> should coerce bool→string
    let flex = load_spacex();
    let result: Option<String> = flex.extract("success").unwrap();
    assert_eq!(result, Some("true".to_string()));
}
