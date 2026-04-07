//! Iteration 36: Boundary — merge() where one side is null and other is object
//!
//! Tests merge semantics with null values and verifies that
//! merge_with_diagnostics reports root-level replacements.

use laminate::FlexValue;

#[test]
fn null_merge_object_takes_object() {
    let null_fv = FlexValue::from_json("null").unwrap();
    let obj_fv = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();

    // null.merge(object) → object wins (right-hand side replaces)
    let result = null_fv.merge(&obj_fv);
    assert!(result.is_object());
    assert_eq!(result.extract::<i64>("a").unwrap(), 1);
}

#[test]
fn object_merge_null_takes_null() {
    let null_fv = FlexValue::from_json("null").unwrap();
    let obj_fv = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();

    // object.merge(null) → null wins (RFC 7396 "remove" semantics)
    let result = obj_fv.merge(&null_fv);
    assert!(result.is_null());
}

#[test]
fn merge_with_diagnostics_reports_root_replacement() {
    let null_fv = FlexValue::from_json("null").unwrap();
    let obj_fv = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();

    // object.merge_with_diagnostics(null) must emit a diagnostic
    let (result, diags) = obj_fv.merge_with_diagnostics(&null_fv);
    assert!(result.is_null());
    assert_eq!(diags.len(), 1, "expected 1 diagnostic for root replacement");
    assert_eq!(diags[0].path, "(root)");

    // null.merge_with_diagnostics(object) must also emit a diagnostic
    let (result, diags) = null_fv.merge_with_diagnostics(&obj_fv);
    assert!(result.is_object());
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].path, "(root)");
}

#[test]
fn nested_null_replaces_nested_object() {
    let a = FlexValue::from_json(r#"{"config": {"debug": true, "level": 3}}"#).unwrap();
    let b = FlexValue::from_json(r#"{"config": null}"#).unwrap();

    // Nested config replaced by null
    let result = a.merge(&b);
    assert!(result.at("config").unwrap().is_null());

    // Diagnostics for nested replacement
    let (_, diags) = a.merge_with_diagnostics(&b);
    assert!(
        !diags.is_empty(),
        "should report nested object→null replacement"
    );
}

#[test]
fn merge_identical_nulls_no_diagnostic() {
    let a = FlexValue::from_json("null").unwrap();
    let b = FlexValue::from_json("null").unwrap();

    // null.merge(null) → null, no diagnostic (a == b)
    let (result, diags) = a.merge_with_diagnostics(&b);
    assert!(result.is_null());
    assert!(
        diags.is_empty(),
        "identical values should produce no diagnostics"
    );
}
