//! Iteration 31: Boundary — root value edge cases and error quality
//!
//! Tests FlexValue behavior with unusual root JSON values: empty containers,
//! root null, root scalars. Verifies that navigation into non-object roots
//! produces TypeMismatch (not misleading PathNotFound).

use laminate::{FlexError, FlexValue};
use serde_json::Value;

#[test]
fn empty_object_root() {
    let fv = FlexValue::from_json("{}").unwrap();
    assert!(fv.is_object());
    assert!(!fv.is_null());

    // Empty object has no keys
    assert_eq!(fv.keys(), Some(vec![]));
    assert!(!fv.has("x"));

    // Navigating missing key → PathNotFound (correct: object exists, key doesn't)
    assert!(matches!(fv.at("x"), Err(FlexError::PathNotFound { .. })));

    // Extract as Value works
    let v: Value = fv.extract_root().unwrap();
    assert_eq!(v, serde_json::json!({}));

    // Object→String coercion produces "{}"
    let s: String = fv.extract_root().unwrap();
    assert_eq!(s, "{}");
}

#[test]
fn empty_array_root() {
    let fv = FlexValue::from_json("[]").unwrap();
    assert!(fv.is_array());
    assert_eq!(fv.len(), Some(0));

    // Index 0 on empty array → IndexOutOfBounds
    match fv.at("[0]") {
        Err(FlexError::IndexOutOfBounds {
            index: 0, len: 0, ..
        }) => {}
        other => panic!("expected IndexOutOfBounds, got {:?}", other),
    }

    // Extract as Vec works
    let v: Vec<Value> = fv.extract_root().unwrap();
    assert!(v.is_empty());

    // Array→String coercion produces "[]"
    let s: String = fv.extract_root().unwrap();
    assert_eq!(s, "[]");
}

#[test]
fn root_null_navigation_gives_type_mismatch() {
    let fv = FlexValue::from_json("null").unwrap();
    assert!(fv.is_null());

    // Key access on null → TypeMismatch (not PathNotFound)
    match fv.at("x") {
        Err(FlexError::TypeMismatch {
            expected, actual, ..
        }) => {
            assert_eq!(expected, "object");
            assert_eq!(actual, "null");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }

    // Index access on null → TypeMismatch
    match fv.at("[0]") {
        Err(FlexError::TypeMismatch {
            expected, actual, ..
        }) => {
            assert_eq!(expected, "array");
            assert_eq!(actual, "null");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }

    // has/keys/len on null root
    assert!(!fv.has("x"));
    assert_eq!(fv.keys(), None);
    assert_eq!(fv.len(), None);

    // Extract: Option<T> → None, bare T → default (BestEffort)
    let opt: Option<String> = fv.extract_root().unwrap();
    assert_eq!(opt, None);
    let s: String = fv.extract_root().unwrap();
    assert_eq!(s, "");
}

#[test]
fn scalar_root_navigation_gives_type_mismatch() {
    // String root — key access → TypeMismatch
    let fv = FlexValue::from_json(r#""hello""#).unwrap();
    assert!(fv.is_string());
    match fv.at("x") {
        Err(FlexError::TypeMismatch {
            expected, actual, ..
        }) => {
            assert_eq!(expected, "object");
            assert_eq!(actual, "string");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }

    // Number root — key access → TypeMismatch
    let fv = FlexValue::from_json("42").unwrap();
    match fv.at("x") {
        Err(FlexError::TypeMismatch {
            expected, actual, ..
        }) => {
            assert_eq!(expected, "object");
            assert_eq!(actual, "number");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }

    // Boolean root — key access → TypeMismatch
    let fv = FlexValue::from_json("true").unwrap();
    match fv.at("x") {
        Err(FlexError::TypeMismatch {
            expected, actual, ..
        }) => {
            assert_eq!(expected, "object");
            assert_eq!(actual, "bool");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }
}

#[test]
fn array_root_key_access_gives_path_not_found() {
    // Arrays support serde string key access, so missing key → PathNotFound (not TypeMismatch)
    let fv = FlexValue::from_json("[1,2,3]").unwrap();
    assert!(matches!(fv.at("x"), Err(FlexError::PathNotFound { .. })));
}
