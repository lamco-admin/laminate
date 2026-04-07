//! Iteration 29: BUG — overflow field as Option<HashMap<String, Value>> didn't compile
//! Fix: Derive macro now detects Option wrapper on overflow field and generates
//! Some(map) / None instead of bare map assignment.

use laminate_derive::Laminate;
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct WithOptionalOverflow {
    id: String,
    #[laminate(overflow)]
    extra: Option<HashMap<String, serde_json::Value>>,
}

#[test]
fn iter29_optional_overflow_some_when_extras() {
    let (s, _) =
        WithOptionalOverflow::from_json(r#"{"id": "abc", "foo": 1, "bar": "hello"}"#).unwrap();
    assert_eq!(s.id, "abc");
    let extra = s.extra.expect("Should be Some when extra fields exist");
    assert_eq!(extra.len(), 2);
    assert_eq!(extra["foo"], serde_json::json!(1));
}

#[test]
fn iter29_optional_overflow_none_when_no_extras() {
    let (s, _) = WithOptionalOverflow::from_json(r#"{"id": "abc"}"#).unwrap();
    assert_eq!(s.id, "abc");
    assert!(s.extra.is_none(), "Should be None when no extra fields");
}

#[test]
fn iter29_optional_overflow_roundtrip() {
    let (s, _) = WithOptionalOverflow::from_json(r#"{"id": "abc", "extra_key": true}"#).unwrap();
    // Round-trip via to_value
    let val = s.to_value();
    assert_eq!(val["id"], "abc");
    assert!(val["extra_key"]);
}
