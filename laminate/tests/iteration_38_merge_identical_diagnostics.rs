//! Iteration 38: Boundary — merge_with_diagnostics on identical and partial overlap
//!
//! Verifies that identical merges produce zero diagnostics, and partial
//! overlaps produce exactly one diagnostic per change (no duplicates).

use laminate::FlexValue;

#[test]
fn identical_objects_produce_zero_diagnostics() {
    let a = FlexValue::from_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let b = FlexValue::from_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let (_, diags) = a.merge_with_diagnostics(&b);
    assert!(
        diags.is_empty(),
        "identical objects should produce 0 diagnostics, got {}",
        diags.len()
    );
}

#[test]
fn identical_nested_objects_produce_zero_diagnostics() {
    let a = FlexValue::from_json(r#"{"config": {"debug": true, "level": 3}}"#).unwrap();
    let b = FlexValue::from_json(r#"{"config": {"debug": true, "level": 3}}"#).unwrap();
    let (_, diags) = a.merge_with_diagnostics(&b);
    assert!(diags.is_empty());
}

#[test]
fn same_keys_different_order_produces_zero_diagnostics() {
    let a = FlexValue::from_json(r#"{"b": 2, "a": 1}"#).unwrap();
    let b = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
    let (_, diags) = a.merge_with_diagnostics(&b);
    assert!(diags.is_empty(), "key order should not affect equality");
}

#[test]
fn partial_overlap_produces_exact_diagnostics() {
    let a = FlexValue::from_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let b = FlexValue::from_json(r#"{"name": "Alice", "age": 31, "city": "NYC"}"#).unwrap();
    let (result, diags) = a.merge_with_diagnostics(&b);

    // Exactly 2: one changed field + one new field
    assert_eq!(
        diags.len(),
        2,
        "expected 2 diagnostics (1 change + 1 addition), got {}",
        diags.len()
    );

    // age changed (value override, same type)
    let age_diag = diags
        .iter()
        .find(|d| d.path == "age")
        .expect("missing age diagnostic");
    assert!(
        matches!(
            &age_diag.kind,
            laminate::diagnostic::DiagnosticKind::Overridden { .. }
        ),
        "age change should use Overridden diagnostic, got {:?}",
        age_diag.kind
    );

    // city added
    let city_diag = diags
        .iter()
        .find(|d| d.path == "city")
        .expect("missing city diagnostic");
    assert!(format!("{:?}", city_diag.kind).contains("city"));

    // Result has the merged values
    assert_eq!(result.extract::<i64>("age").unwrap(), 31);
    assert_eq!(result.extract::<String>("city").unwrap(), "NYC");
    assert_eq!(result.extract::<String>("name").unwrap(), "Alice");
}
