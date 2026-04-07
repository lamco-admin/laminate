#![allow(dead_code)]
//! Iteration 202: external_required=false overrides inferred required.
//!
//! Question: If a field appears in all rows (would be required by inference),
//! but external_required is set to false, does the audit skip the "missing required field" check?

use laminate::schema::{ExternalConstraint, InferredSchema};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn external_required_false_overrides_inferred() {
    // Train: "name" appears in all rows — inferred as required
    let training = vec![
        json!({"name": "Alice", "id": 1}),
        json!({"name": "Bob", "id": 2}),
        json!({"name": "Carol", "id": 3}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Verify it's required by default
    let name_def = schema.fields.get("name").unwrap();
    assert!(
        schema.is_field_required(name_def),
        "name should be required by inference"
    );

    // Now apply external constraint: required = false
    let mut constraints = HashMap::new();
    constraints.insert(
        "name".into(),
        ExternalConstraint {
            required: false,
            ..ExternalConstraint::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // Check: is_field_required should now return false
    let name_def = schema.fields.get("name").unwrap();
    let required = schema.is_field_required(name_def);
    println!("After external_required=false: is_field_required={required}");
    assert!(
        !required,
        "external_required=false should override inference"
    );

    // Audit with missing "name" — should NOT produce a missing-required violation
    let test_data = vec![json!({"id": 4})];
    let report = schema.audit(&test_data);

    println!("Violations: {:?}", report.violations);
    let missing_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "name")
        .collect();
    println!("Name violations: {missing_violations:?}");

    // name is not required anymore — missing it shouldn't be a violation
    assert!(
        missing_violations.is_empty(),
        "name should not cause violation when external_required=false"
    );
}
