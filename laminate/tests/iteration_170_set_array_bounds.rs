// Iteration 170: set() with array index beyond bounds
// Fresh target — set() auto-grows arrays with null padding.
// What happens with: huge index, existing array, type conflicts?

use laminate::FlexValue;
use serde_json::json;

#[test]
fn set_beyond_array_length() {
    let mut val = FlexValue::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
    val.set("items[5]", json!(99)).unwrap();

    // Should pad with nulls at indices 3 and 4
    let items = val.each("items");
    assert_eq!(items.len(), 6);
    assert!(items[3].is_null());
    assert!(items[4].is_null());
    let v: i64 = items[5].extract_root().unwrap();
    assert_eq!(v, 99);
}

#[test]
fn set_at_zero_on_empty_array() {
    let mut val = FlexValue::from_json(r#"{"items": []}"#).unwrap();
    val.set("items[0]", json!("first")).unwrap();

    let result: String = val.extract("items[0]").unwrap();
    assert_eq!(result, "first");
}

#[test]
fn set_creates_nested_array() {
    // Path creates array from null
    let mut val = FlexValue::from_json(r#"{"data": null}"#).unwrap();
    val.set("data[0]", json!("hello")).unwrap();

    let result: String = val.extract("data[0]").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn set_moderate_index_padding() {
    // Index 20 — should create 21 elements (0..=20)
    let mut val = FlexValue::from_json(r#"{"arr": []}"#).unwrap();
    val.set("arr[20]", json!(42)).unwrap();

    let items = val.each("arr");
    assert_eq!(items.len(), 21);
    let v: i64 = items[20].extract_root().unwrap();
    assert_eq!(v, 42);
}

#[test]
fn set_on_non_array_is_noop() {
    // set() with array index on a string — should silently fail (not destroy data)
    let mut val = FlexValue::from_json(r#"{"name": "hello"}"#).unwrap();
    val.set("name[0]", json!("x")).unwrap();

    // Original value should be unchanged
    let result: String = val.extract("name").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn set_nested_array_in_object() {
    let mut val = FlexValue::from_json(r#"{}"#).unwrap();
    val.set("a.b[2].c", json!(true)).unwrap();

    let result: bool = val.extract("a.b[2].c").unwrap();
    assert!(result);
}
