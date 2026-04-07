// Iteration 173: keys() and len() on various value types
// Fresh target — keys() returns Some for objects, None for others.
// len() returns element count for arrays/objects. What about strings?

use laminate::FlexValue;

#[test]
fn keys_on_object() {
    let val = FlexValue::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
    let keys = val.keys();
    assert!(keys.is_some());
    let keys = keys.unwrap();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"a"));
    assert!(keys.contains(&"b"));
    assert!(keys.contains(&"c"));
}

#[test]
fn keys_on_array() {
    let val = FlexValue::from_json(r#"[1, 2, 3]"#).unwrap();
    let keys = val.keys();
    assert!(keys.is_none(), "keys() on array should be None");
}

#[test]
fn keys_on_string() {
    let val = FlexValue::from_json(r#""hello""#).unwrap();
    let keys = val.keys();
    assert!(keys.is_none(), "keys() on string should be None");
}

#[test]
fn keys_on_null() {
    let val = FlexValue::from_json(r#"null"#).unwrap();
    let keys = val.keys();
    assert!(keys.is_none(), "keys() on null should be None");
}

#[test]
fn keys_on_nested_object() {
    let val = FlexValue::from_json(r#"{"outer": {"inner": 1}}"#).unwrap();
    // keys() should only return top-level keys
    let keys = val.keys().unwrap();
    assert_eq!(keys, vec!["outer"]);
}

#[test]
fn len_on_array() {
    let val = FlexValue::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
    assert_eq!(val.len(), Some(5));
}

#[test]
fn len_on_object() {
    let val = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
    assert_eq!(val.len(), Some(2));
}

#[test]
fn len_on_string() {
    let val = FlexValue::from_json(r#""hello""#).unwrap();
    // What does len() return for a string? None (not a container) or Some(5)?
    let result = val.len();
    println!("len() on string: {:?}", result);
}

#[test]
fn is_empty_on_empty_object() {
    let val = FlexValue::from_json(r#"{}"#).unwrap();
    assert_eq!(val.is_empty(), Some(true));
}

#[test]
fn is_empty_on_nonempty_array() {
    let val = FlexValue::from_json(r#"[1]"#).unwrap();
    assert_eq!(val.is_empty(), Some(false));
}

#[test]
fn keys_on_empty_object() {
    let val = FlexValue::from_json(r#"{}"#).unwrap();
    let keys = val.keys().unwrap();
    assert!(keys.is_empty());
}
