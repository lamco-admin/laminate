// Iteration 168: each() on non-array fields
// Fresh target — each() should return empty Vec on non-array types.
// What about: string that looks like array, null, nested object, number?

use laminate::FlexValue;

#[test]
fn each_on_string() {
    let val = FlexValue::from_json(r#"{"items": "not an array"}"#).unwrap();
    let result = val.each("items");
    assert!(
        result.is_empty(),
        "each() on string should return empty Vec"
    );
}

#[test]
fn each_on_null() {
    let val = FlexValue::from_json(r#"{"items": null}"#).unwrap();
    let result = val.each("items");
    assert!(result.is_empty(), "each() on null should return empty Vec");
}

#[test]
fn each_on_number() {
    let val = FlexValue::from_json(r#"{"items": 42}"#).unwrap();
    let result = val.each("items");
    assert!(
        result.is_empty(),
        "each() on number should return empty Vec"
    );
}

#[test]
fn each_on_object() {
    let val = FlexValue::from_json(r#"{"items": {"a": 1, "b": 2}}"#).unwrap();
    let result = val.each("items");
    assert!(
        result.is_empty(),
        "each() on object should return empty Vec"
    );
}

#[test]
fn each_on_missing_path() {
    let val = FlexValue::from_json(r#"{"data": 1}"#).unwrap();
    let result = val.each("nonexistent");
    assert!(
        result.is_empty(),
        "each() on missing path should return empty Vec"
    );
}

#[test]
fn each_on_string_that_looks_like_json_array() {
    // A string value that contains JSON array syntax — since iter 261,
    // each() transparently parses stringified JSON arrays (same as at() does
    // for path navigation).
    let val = FlexValue::from_json(r#"{"items": "[1,2,3]"}"#).unwrap();
    let result = val.each("items");
    assert_eq!(
        result.len(),
        3,
        "each() should parse stringified JSON array"
    );
}

#[test]
fn each_on_boolean() {
    let val = FlexValue::from_json(r#"{"flag": true}"#).unwrap();
    let result = val.each("flag");
    assert!(result.is_empty(), "each() on bool should return empty Vec");
}
