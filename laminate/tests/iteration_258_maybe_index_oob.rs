//! Iteration 258: maybe() with IndexOutOfBounds
//!
//! Tests whether `maybe()` treats an out-of-bounds array index as "absent"
//! (returning None) or as a hard error.
//!
//! GAP found: maybe() returned Err(IndexOutOfBounds) instead of None.
//! Fixed: IndexOutOfBounds is now treated as absence, like PathNotFound.

use laminate::FlexValue;

#[test]
fn maybe_on_empty_array_returns_none() {
    let val = FlexValue::from_json(r#"{"items": []}"#).unwrap();

    // Empty array — index 0 doesn't exist, should be None
    let result: Option<String> = val.maybe("items[0]").unwrap();
    assert_eq!(result, None);
}

#[test]
fn maybe_on_empty_array_deep_path_returns_none() {
    let val = FlexValue::from_json(r#"{"items": []}"#).unwrap();

    // Path through missing element — should also be None
    let result: Option<String> = val.maybe("items[0].name").unwrap();
    assert_eq!(result, None);
}

#[test]
fn maybe_on_short_array_returns_none() {
    let val = FlexValue::from_json(r#"{"items": [{"name": "Alice"}]}"#).unwrap();

    // Index 0 exists
    let result: Option<String> = val.maybe("items[0].name").unwrap();
    assert_eq!(result, Some("Alice".to_string()));

    // Index 1 doesn't exist — should be None, not error
    let result: Option<String> = val.maybe("items[1].name").unwrap();
    assert_eq!(result, None);
}

#[test]
fn maybe_on_root_array_oob_returns_none() {
    let val = FlexValue::from_json(r#"[10, 20, 30]"#).unwrap();

    // Index 2 exists
    let result: Option<i64> = val.maybe("[2]").unwrap();
    assert_eq!(result, Some(30));

    // Index 5 doesn't exist
    let result: Option<i64> = val.maybe("[5]").unwrap();
    assert_eq!(result, None);
}

#[test]
fn extract_on_oob_still_errors() {
    // extract() should still return a hard error for OOB —
    // only maybe() treats it as absence
    let val = FlexValue::from_json(r#"{"items": []}"#).unwrap();
    let result = val.extract::<String>("items[0]");
    assert!(result.is_err());
}
