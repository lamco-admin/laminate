#![allow(dead_code, unused_imports, unused_must_use)]
use laminate::FlexError;
/// Iteration 113: Derive struct — required field error quality
///
/// Verified that missing required fields produce PathNotFound with the
/// specific field name, and wrong types produce DeserializeError with path.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Person {
    name: String,
    age: i64,
    email: String,
}

#[test]
fn missing_field_names_the_field() {
    let err = Person::from_json(r#"{"name": "Alice", "email": "a@b.com"}"#).unwrap_err();
    match err {
        FlexError::PathNotFound { path } => assert_eq!(path, "age"),
        other => panic!("expected PathNotFound, got {:?}", other),
    }
}

#[test]
fn wrong_type_names_the_field() {
    let err =
        Person::from_json(r#"{"name": "Alice", "age": "old", "email": "a@b.com"}"#).unwrap_err();
    match err {
        FlexError::DeserializeError { path, .. } => assert_eq!(path, "age"),
        other => panic!("expected DeserializeError, got {:?}", other),
    }
}
