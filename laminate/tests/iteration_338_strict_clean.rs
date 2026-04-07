//! Iteration 338: shape_strict on perfectly clean data — zero diagnostics.

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Record {
    name: String,
    age: i64,
    active: bool,
}

#[test]
fn shape_strict_clean_data() {
    let json = serde_json::json!({
        "name": "Alice",
        "age": 30,
        "active": true
    });

    let result = Record::shape_strict(&json);
    println!("strict clean: {:?}", result);

    assert!(
        result.is_ok(),
        "clean data should pass strict: {:?}",
        result.err()
    );
    let record = result.unwrap();
    assert_eq!(record.name, "Alice");
    assert_eq!(record.age, 30);
    assert!(record.active);
}

#[test]
fn shape_strict_wrong_type_fails() {
    let json = serde_json::json!({
        "name": "Alice",
        "age": "30",  // string instead of i64
        "active": true
    });

    let result = Record::shape_strict(&json);
    println!("strict wrong type: {:?}", result);
    assert!(result.is_err(), "strict should reject string for i64 field");
}

#[test]
fn shape_strict_unknown_field_fails() {
    let json = serde_json::json!({
        "name": "Alice",
        "age": 30,
        "active": true,
        "extra": "unknown"
    });

    let result = Record::shape_strict(&json);
    println!("strict unknown field: {:?}", result);
    assert!(result.is_err(), "strict should reject unknown fields");
}
