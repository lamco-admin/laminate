#![allow(dead_code, unused_imports, unused_must_use)]
/// Iteration 101: set() path type conflicts — don't destroy existing data
///
/// set_at_path was silently replacing arrays with objects (and vice versa)
/// when the path segment type didn't match the existing value type.
/// Now it bails out instead of destroying existing data.
use laminate::FlexValue;

#[test]
fn set_key_through_array_preserves_array() {
    let mut fv = FlexValue::from(serde_json::json!({"data": [1, 2, 3]}));
    fv.set("data.name", serde_json::json!("test"));
    // Array should survive — set should be a no-op
    assert!(
        fv.at("data").unwrap().is_array(),
        "array should not be destroyed"
    );
}

#[test]
fn set_index_through_object_preserves_object() {
    let mut fv = FlexValue::from(serde_json::json!({"data": {"name": "important"}}));
    fv.set("data[0]", serde_json::json!("overwritten"));
    // Object should survive — set should be a no-op
    assert!(
        fv.at("data").unwrap().is_object(),
        "object should not be destroyed"
    );
    let name: String = fv.extract("data.name").unwrap();
    assert_eq!(name, "important");
}

#[test]
fn set_creates_from_empty() {
    let mut fv = FlexValue::from(serde_json::json!({}));
    fv.set("a.b.c[0].d", serde_json::json!(42));
    let result: i64 = fv.extract("a.b.c[0].d").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn set_extends_existing_array() {
    let mut fv = FlexValue::from(serde_json::json!({"items": [10, 20]}));
    fv.set("items[4]", serde_json::json!(50));
    // Should extend with nulls: [10, 20, null, null, 50]
    let items = fv.at("items").unwrap();
    assert_eq!(items.len(), Some(5));
}
