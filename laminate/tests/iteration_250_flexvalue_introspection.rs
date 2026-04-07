//! Iteration 250: FlexValue introspection methods — keys(), is_empty(), len()
//!
//! Tests the container introspection methods on various types:
//! objects, arrays, scalars, null, nested values.
//!
//! Adversarial: keys() on non-object, len() on string, is_empty() on null.

use laminate::FlexValue;

#[test]
fn keys_on_object() {
    let val = FlexValue::from(serde_json::json!({"a": 1, "b": 2, "c": 3}));
    let keys = val.keys().expect("object should have keys");
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"a"));
    assert!(keys.contains(&"b"));
    assert!(keys.contains(&"c"));
}

#[test]
fn keys_on_empty_object() {
    let val = FlexValue::from(serde_json::json!({}));
    let keys = val.keys().expect("empty object should return Some([])");
    assert!(keys.is_empty());
}

#[test]
fn keys_on_non_object() {
    let val = FlexValue::from(serde_json::json!([1, 2, 3]));
    assert_eq!(val.keys(), None, "array should not have keys");

    let val = FlexValue::from(serde_json::json!("hello"));
    assert_eq!(val.keys(), None, "string should not have keys");

    let val = FlexValue::from(serde_json::json!(42));
    assert_eq!(val.keys(), None, "number should not have keys");

    let val = FlexValue::from(serde_json::json!(null));
    assert_eq!(val.keys(), None, "null should not have keys");
}

#[test]
fn len_on_array() {
    let val = FlexValue::from(serde_json::json!([1, 2, 3]));
    assert_eq!(val.len(), Some(3));
}

#[test]
fn len_on_object() {
    let val = FlexValue::from(serde_json::json!({"x": 1, "y": 2}));
    assert_eq!(val.len(), Some(2));
}

#[test]
fn len_on_empty_containers() {
    let val = FlexValue::from(serde_json::json!([]));
    assert_eq!(val.len(), Some(0));

    let val = FlexValue::from(serde_json::json!({}));
    assert_eq!(val.len(), Some(0));
}

#[test]
fn len_on_scalars() {
    let val = FlexValue::from(serde_json::json!("hello"));
    assert_eq!(val.len(), None, "string should return None for len");

    let val = FlexValue::from(serde_json::json!(42));
    assert_eq!(val.len(), None, "number should return None for len");

    let val = FlexValue::from(serde_json::json!(true));
    assert_eq!(val.len(), None, "bool should return None for len");

    let val = FlexValue::from(serde_json::json!(null));
    assert_eq!(val.len(), None, "null should return None for len");
}

#[test]
fn is_empty_on_containers() {
    assert_eq!(
        FlexValue::from(serde_json::json!([])).is_empty(),
        Some(true)
    );
    assert_eq!(
        FlexValue::from(serde_json::json!({})).is_empty(),
        Some(true)
    );
    assert_eq!(
        FlexValue::from(serde_json::json!([1])).is_empty(),
        Some(false)
    );
    assert_eq!(
        FlexValue::from(serde_json::json!({"a": 1})).is_empty(),
        Some(false)
    );
}

#[test]
fn is_empty_on_scalars() {
    assert_eq!(FlexValue::from(serde_json::json!("")).is_empty(), None);
    assert_eq!(FlexValue::from(serde_json::json!(0)).is_empty(), None);
    assert_eq!(FlexValue::from(serde_json::json!(null)).is_empty(), None);
}

#[test]
fn keys_on_nested_then_navigated() {
    let val = FlexValue::from(serde_json::json!({
        "outer": {"inner_a": 1, "inner_b": 2}
    }));

    let inner = val.at("outer").unwrap();
    let keys = inner.keys().expect("navigated object should have keys");
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"inner_a"));
    assert!(keys.contains(&"inner_b"));
}
