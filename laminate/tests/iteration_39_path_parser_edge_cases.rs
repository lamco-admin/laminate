//! Iteration 39: Boundary — Path parser with malformed paths
//!
//! Verifies that the path parser rejects malformed paths with clear errors:
//! empty segments, leading/trailing dots, invalid brackets, huge indices.

use laminate::{FlexError, FlexValue};

fn fv() -> FlexValue {
    FlexValue::from_json(r#"{"a": {"b": 1}}"#).unwrap()
}

#[test]
fn leading_dot_rejected() {
    match fv().at(".b") {
        Err(FlexError::InvalidPath { detail }) => {
            assert!(detail.contains("empty key"), "got: {detail}");
        }
        other => panic!("expected InvalidPath, got {:?}", other),
    }
}

#[test]
fn trailing_dot_rejected() {
    match fv().at("a.") {
        Err(FlexError::InvalidPath { detail }) => {
            assert!(detail.contains("trailing dot"), "got: {detail}");
        }
        other => panic!("expected InvalidPath, got {:?}", other),
    }
}

#[test]
fn double_dot_rejected() {
    match fv().at("a..b") {
        Err(FlexError::InvalidPath { detail }) => {
            assert!(detail.contains("empty key"), "got: {detail}");
        }
        other => panic!("expected InvalidPath, got {:?}", other),
    }
}

#[test]
fn empty_brackets_rejected() {
    assert!(matches!(fv().at("a[]"), Err(FlexError::InvalidPath { .. })));
}

#[test]
fn negative_index_rejected() {
    assert!(matches!(
        fv().at("a[-1]"),
        Err(FlexError::InvalidPath { .. })
    ));
}

#[test]
fn huge_index_rejected() {
    match fv().at("a[99999999999999999999]") {
        Err(FlexError::InvalidPath { detail }) => {
            assert!(detail.contains("invalid index"), "got: {detail}");
        }
        other => panic!("expected InvalidPath, got {:?}", other),
    }
}

#[test]
fn quoted_key_with_dot_works() {
    let fv = FlexValue::from_json(r#"{"a.b": 42}"#).unwrap();
    let v: i64 = fv.extract(r#"["a.b"]"#).unwrap();
    assert_eq!(v, 42);
}
