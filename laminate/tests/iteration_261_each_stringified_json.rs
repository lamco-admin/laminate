//! Iteration 261: each() on stringified JSON array
//!
//! GAP found: at("data[0]") transparently parsed stringified JSON arrays,
//! but each("data") returned empty. Fixed: each_iter() now parses
//! stringified JSON arrays the same way at() does.

use laminate::FlexValue;

#[test]
fn each_on_stringified_json_array() {
    let val = FlexValue::from_json(r#"{"data": "[1, 2, 3]"}"#).unwrap();

    // each() should parse the stringified array and iterate
    let items = val.each("data");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].extract_root::<i64>().unwrap(), 1);
    assert_eq!(items[1].extract_root::<i64>().unwrap(), 2);
    assert_eq!(items[2].extract_root::<i64>().unwrap(), 3);
}

#[test]
fn each_on_stringified_json_objects() {
    let val = FlexValue::from_json(r#"{"users": "[{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]"}"#)
        .unwrap();

    let users = val.each("users");
    assert_eq!(users.len(), 2);

    let name0: String = users[0].extract("name").unwrap();
    let name1: String = users[1].extract("name").unwrap();
    assert_eq!(name0, "Alice");
    assert_eq!(name1, "Bob");
}

#[test]
fn each_on_real_array_still_works() {
    // Regular (non-stringified) arrays should still work
    let val = FlexValue::from_json(r#"{"items": [10, 20, 30]}"#).unwrap();
    let items = val.each("items");
    assert_eq!(items.len(), 3);
}

#[test]
fn each_on_non_json_string_returns_empty() {
    // A plain string that isn't a JSON array should return empty
    let val = FlexValue::from_json(r#"{"name": "hello world"}"#).unwrap();
    let items = val.each("name");
    assert_eq!(items.len(), 0);
}

#[test]
fn each_on_stringified_empty_array() {
    let val = FlexValue::from_json(r#"{"data": "[]"}"#).unwrap();
    let items = val.each("data");
    assert_eq!(items.len(), 0);
}
