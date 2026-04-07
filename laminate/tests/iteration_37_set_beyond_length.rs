#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration 37: Boundary — set() to create array index beyond current length
//!
//! Verifies that set() pads arrays with null to reach the target index,
//! and handles nested set-beyond-length correctly.

use laminate::FlexValue;
use serde_json::json;

#[test]
fn set_beyond_length_pads_with_null() {
    let mut fv = FlexValue::from_json("[10, 20, 30]").unwrap();
    fv.set("[5]", json!("hello"));

    let arr = fv.raw().as_array().unwrap();
    assert_eq!(arr.len(), 6);
    assert_eq!(arr[0], json!(10));
    assert_eq!(arr[3], json!(null)); // padded
    assert_eq!(arr[4], json!(null)); // padded
    assert_eq!(arr[5], json!("hello"));
}

#[test]
fn set_on_empty_array() {
    let mut fv = FlexValue::from_json("[]").unwrap();
    fv.set("[2]", json!("x"));

    let arr = fv.raw().as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], json!(null));
    assert_eq!(arr[1], json!(null));
    assert_eq!(arr[2], json!("x"));
}

#[test]
fn nested_set_beyond_length_creates_intermediate_objects() {
    let mut fv = FlexValue::from_json(r#"[{"name":"a"}]"#).unwrap();
    fv.set("[3].name", json!("far"));

    let arr = fv.raw().as_array().unwrap();
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[1], json!(null)); // padded
    assert_eq!(arr[2], json!(null)); // padded
    assert_eq!(arr[3], json!({"name": "far"}));
}

#[test]
fn set_overwrite_existing_index() {
    let mut fv = FlexValue::from_json("[10, 20, 30]").unwrap();
    fv.set("[1]", json!(99));
    assert_eq!(fv.extract::<i64>("[1]").unwrap(), 99);
    // Other elements unchanged
    assert_eq!(fv.extract::<i64>("[0]").unwrap(), 10);
    assert_eq!(fv.extract::<i64>("[2]").unwrap(), 30);
}
