#![allow(dead_code)]
//! Iteration 194: Nested object field — dominant type is Object, not recursive.
//!
//! Question: Does schema inference just say "Object" for nested fields,
//! or does it recurse into the structure? And if the nested structure changes
//! between rows, does audit flag it?

use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn schema_nested_object_type() {
    let rows = vec![
        json!({"id": 1, "meta": {"color": "red", "size": 10}}),
        json!({"id": 2, "meta": {"color": "blue", "size": 20}}),
    ];

    let schema = InferredSchema::from_values(&rows);
    let meta = schema.fields.get("meta").unwrap();

    println!("meta dominant_type: {:?}", meta.dominant_type);
    println!("meta type_counts: {:?}", meta.type_counts);
    // Is the nested structure tracked, or just "Object"?
}

#[test]
fn schema_audit_different_nested_structure() {
    let training = vec![
        json!({"id": 1, "meta": {"color": "red"}}),
        json!({"id": 2, "meta": {"color": "blue"}}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Audit with a different nested structure — "meta" now has "size" instead of "color"
    let test_data = vec![json!({"id": 3, "meta": {"size": 42}})];
    let report = schema.audit(&test_data);

    println!("Violations: {:?}", report.violations);
    println!("Violation count: {}", report.violations.len());
    // Does the audit care about internal structure changes in Object fields?
}
