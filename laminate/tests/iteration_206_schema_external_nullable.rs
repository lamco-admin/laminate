#![allow(dead_code)]
//! Iteration 206: external_nullable=false on normally-nullable field.
//!
//! If training data has null values (so field is inferred nullable), but
//! external_nullable=false overrides that, does audit catch null → violation?

use laminate::schema::{ExternalConstraint, InferredSchema};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn external_nullable_false_overrides_inferred() {
    // Training data has nulls — field inferred as nullable
    let rows = vec![
        json!({"score": 95}),
        json!({"score": null}),
        json!({"score": 80}),
    ];
    let schema = InferredSchema::from_values(&rows);

    // Verify inferred: null_count > 0 means nullable by default
    let score = schema.fields.get("score").unwrap();
    assert!(score.null_count > 0, "training data has nulls");

    // Override with external constraint: nullable=false
    let mut constraints = HashMap::new();
    constraints.insert(
        "score".into(),
        ExternalConstraint {
            nullable: false,
            ..ExternalConstraint::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // Audit with null value — should now flag UnexpectedNull
    let test_data = vec![json!({"score": null})];
    let report = schema.audit(&test_data);
    println!("Violations: {:?}", report.violations);

    let null_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "score" && format!("{:?}", v.kind).contains("Null"))
        .collect();
    assert!(
        !null_violations.is_empty(),
        "null should violate external_nullable=false constraint"
    );
}

#[test]
fn external_nullable_true_allows_null() {
    // Training data has NO nulls — field would be inferred as non-nullable
    let rows = vec![json!({"score": 95}), json!({"score": 80})];
    let schema = InferredSchema::from_values(&rows);

    // Verify: null_count == 0 means non-nullable by default
    let score = schema.fields.get("score").unwrap();
    assert_eq!(score.null_count, 0);

    // Override: nullable=true
    let mut constraints = HashMap::new();
    constraints.insert(
        "score".into(),
        ExternalConstraint {
            nullable: true,
            ..ExternalConstraint::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // Audit with null — should be allowed
    let test_data = vec![json!({"score": null})];
    let report = schema.audit(&test_data);
    println!("Violations: {:?}", report.violations);

    let null_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "score" && format!("{:?}", v.kind).contains("Null"))
        .collect();
    assert!(
        null_violations.is_empty(),
        "external_nullable=true should allow null values"
    );
}
