//! Iteration 341: shape_lenient on {} (empty) — all default/optional fields.
//! GAP: Option<T> without #[laminate(default)] fails with PathNotFound on empty object.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct AllOptional {
    #[laminate(default)]
    name: String,
    #[laminate(default)]
    count: i64,
    #[laminate(default)]
    active: bool,
    #[laminate(default)]
    tags: Option<Vec<String>>,
}

#[test]
fn shape_lenient_empty_object() {
    let json = serde_json::json!({});
    let result = AllOptional::shape_lenient(&json);
    println!("empty object: {:?}", result);

    assert!(
        result.is_ok(),
        "all-defaults struct should handle empty object"
    );
    let lr = result.unwrap();
    assert_eq!(lr.value.name, "", "String default is empty");
    assert_eq!(lr.value.count, 0, "i64 default is 0");
    assert_eq!(lr.value.active, false, "bool default is false");
    assert_eq!(lr.value.tags, None, "Option default is None");
}

// Confirm: Option<T> WITHOUT #[laminate(default)] fails on absent field
#[derive(Debug, Laminate)]
struct NoDefaultOption {
    value: Option<String>,
}

#[test]
fn option_without_default_on_absent_field() {
    let json = serde_json::json!({});
    let result = NoDefaultOption::shape_lenient(&json);
    println!("Option without default: {:?}", result);
    // This is a documented design choice: Option<T> doesn't auto-default
    // You need #[laminate(default)] explicitly
    assert!(
        result.is_err(),
        "Option<T> without #[laminate(default)] should fail on absent"
    );
}

// Confirm: Option<T> with null value works without default
#[test]
fn option_without_default_on_null_value() {
    let json = serde_json::json!({"value": null});
    let result = NoDefaultOption::shape_lenient(&json);
    println!("Option with null value: {:?}", result);
    assert!(
        result.is_ok(),
        "Option<T> with null value should be Ok(None)"
    );
}
