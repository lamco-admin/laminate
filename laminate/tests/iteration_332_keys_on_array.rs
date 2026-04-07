//! Iteration 332: keys() on a JSON array — returns None (not indices).
//! Also: len() on a scalar string.

use laminate::FlexValue;

#[test]
fn keys_on_array_returns_none() {
    let fv = FlexValue::from_json(r#"[1, 2, 3]"#).unwrap();
    let keys = fv.keys();
    println!("keys on array: {:?}", keys);
    assert!(keys.is_none(), "keys() on array should return None");
}

#[test]
fn keys_on_object() {
    let fv = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
    let keys = fv.keys();
    println!("keys on object: {:?}", keys);
    assert!(keys.is_some());
    let keys = keys.unwrap();
    assert!(keys.contains(&"a"));
    assert!(keys.contains(&"b"));
}

#[test]
fn len_on_scalar_string() {
    let fv = FlexValue::from_json(r#""hello""#).unwrap();
    let len = fv.len();
    println!("len on string: {:?}", len);
    // len() only works on arrays and objects
    assert!(len.is_none(), "len() on string should return None");
}

#[test]
fn len_on_number() {
    let fv = FlexValue::from_json(r#"42"#).unwrap();
    let len = fv.len();
    println!("len on number: {:?}", len);
    assert!(len.is_none(), "len() on number should return None");
}

#[test]
fn is_empty_on_null() {
    let fv = FlexValue::from_json(r#"null"#).unwrap();
    let empty = fv.is_empty();
    println!("is_empty on null: {:?}", empty);
    assert!(empty.is_none(), "is_empty() on null should return None");
}
