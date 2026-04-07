// Iteration 169: merge() with conflicting types at same path
// Fresh target — what happens when merging an object field with a scalar,
// an array with an object, etc.? Does merge_with_diagnostics report the conflict?

use laminate::FlexValue;

#[test]
fn merge_object_with_scalar_override() {
    let a = FlexValue::from_json(r#"{"x": {"nested": 1}}"#).unwrap();
    let b = FlexValue::from_json(r#"{"x": "replaced"}"#).unwrap();
    let merged = a.merge(&b);
    // b should win — object replaced by string
    let result: String = merged.extract("x").unwrap();
    assert_eq!(result, "replaced");
}

#[test]
fn merge_scalar_with_object_override() {
    let a = FlexValue::from_json(r#"{"x": "was string"}"#).unwrap();
    let b = FlexValue::from_json(r#"{"x": {"now": "object"}}"#).unwrap();
    let merged = a.merge(&b);
    // b should win — string replaced by object
    let result: String = merged.extract("x.now").unwrap();
    assert_eq!(result, "object");
}

#[test]
fn merge_array_with_scalar() {
    let a = FlexValue::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
    let b = FlexValue::from_json(r#"{"items": 99}"#).unwrap();
    let merged = a.merge(&b);
    let result: i64 = merged.extract("items").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn merge_with_diagnostics_reports_override() {
    let a = FlexValue::from_json(r#"{"x": [1,2,3]}"#).unwrap();
    let b = FlexValue::from_json(r#"{"x": "replaced"}"#).unwrap();
    let (merged, diagnostics) = a.merge_with_diagnostics(&b);

    println!("Diagnostics: {:?}", diagnostics);

    // Should report that "x" was overridden
    let result: String = merged.extract("x").unwrap();
    assert_eq!(result, "replaced");

    // Check that diagnostics mention the override
    assert!(
        !diagnostics.is_empty(),
        "Diagnostics should report the type-changing override"
    );
}

#[test]
fn merge_null_with_value() {
    let a = FlexValue::from_json(r#"{"x": null}"#).unwrap();
    let b = FlexValue::from_json(r#"{"x": 42}"#).unwrap();
    let merged = a.merge(&b);
    let result: i64 = merged.extract("x").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn merge_value_with_null() {
    // Merging with null — null should win (b overrides a)
    let a = FlexValue::from_json(r#"{"x": 42}"#).unwrap();
    let b = FlexValue::from_json(r#"{"x": null}"#).unwrap();
    let merged = a.merge(&b);
    assert!(merged.at("x").unwrap().is_null());
}
