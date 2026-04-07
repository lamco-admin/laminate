#![allow(dead_code)]
//! Iteration 195: Array field with varying element types.
//!
//! Question: Is the dominant type Array regardless of element types?
//! Does audit care if elements change from [int, int] to [string, string]?

use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn schema_array_field_type() {
    let rows = vec![json!({"tags": [1, 2, 3]}), json!({"tags": ["a", "b"]})];

    let schema = InferredSchema::from_values(&rows);
    let tags = schema.fields.get("tags").unwrap();

    println!("tags dominant_type: {:?}", tags.dominant_type);
    println!("tags type_counts: {:?}", tags.type_counts);
}

#[test]
fn schema_audit_array_element_type_change() {
    let training = vec![
        json!({"scores": [90, 85, 92]}),
        json!({"scores": [78, 95, 88]}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Audit with string elements instead of integers
    let test_data = vec![json!({"scores": ["high", "medium", "low"]})];
    let report = schema.audit(&test_data);

    println!(
        "Violations on string-array vs int-array: {:?}",
        report.violations
    );
    // Array type matches — but element types differ. Does audit catch this?
}

#[test]
fn schema_audit_non_array_where_array_expected() {
    let training = vec![json!({"scores": [90, 85]}), json!({"scores": [78, 95]})];
    let schema = InferredSchema::from_values(&training);

    // Audit with a scalar instead of array
    let test_data = vec![json!({"scores": 42})];
    let report = schema.audit(&test_data);

    println!("Violations on scalar vs array: {:?}", report.violations);
    assert!(
        !report.violations.is_empty(),
        "Expected violation: integer where array expected"
    );
}
