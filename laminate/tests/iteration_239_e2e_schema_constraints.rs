//! Iteration 239: E2E Schema constraints pipeline
//!
//! Full pipeline: infer schema → apply DB constraints → audit new batch → verify stats.
//!
//! Adversarial: external constraints OVERRIDE inferred properties. What happens when:
//! - External says required but data had it optional?
//! - External max_length is violated?
//! - External allowed_values catches unexpected enum values?
//! - External min/max value catches out-of-range numbers?

use laminate::schema::{ExternalConstraint, InferredSchema};
use std::collections::HashMap;

fn training_data() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"name": "Alice", "age": 30, "status": "active"}),
        serde_json::json!({"name": "Bob", "age": 25, "status": "active"}),
        serde_json::json!({"name": "Charlie", "age": 35, "status": "inactive"}),
        serde_json::json!({"name": "Diana", "age": 28, "status": "active"}),
    ]
}

#[test]
fn schema_with_db_constraints_catches_violations() {
    let schema = InferredSchema::from_values(&training_data());

    // Simulate DB metadata: name is VARCHAR(10), age has range [18,65], status is enum
    let mut constraints = HashMap::new();
    constraints.insert(
        "name".to_string(),
        ExternalConstraint {
            max_length: Some(10),
            ..Default::default()
        },
    );
    constraints.insert(
        "age".to_string(),
        ExternalConstraint {
            min_value: Some(18.0),
            max_value: Some(65.0),
            ..Default::default()
        },
    );
    constraints.insert(
        "status".to_string(),
        ExternalConstraint {
            allowed_values: Some(vec!["active".to_string(), "inactive".to_string()]),
            ..Default::default()
        },
    );

    let schema = schema.with_constraints(constraints);

    // New batch with violations
    let new_batch = vec![
        serde_json::json!({"name": "Eve", "age": 22, "status": "active"}), // clean
        serde_json::json!({"name": "Frank Longname III", "age": 30, "status": "active"}), // name too long
        serde_json::json!({"name": "Grace", "age": 15, "status": "active"}), // age < 18
        serde_json::json!({"name": "Hank", "age": 70, "status": "active"}),  // age > 65
        serde_json::json!({"name": "Ivy", "age": 25, "status": "pending"}),  // invalid status
    ];

    let report = schema.audit(&new_batch);

    println!("Violations:");
    for v in &report.violations {
        println!(
            "  Row {}: {} — {:?} (expected: {}, actual: {})",
            v.row, v.field, v.kind, v.expected, v.actual
        );
    }

    // Row 1: name too long (18 chars > 10)
    // Row 2: age too young (15 < 18)
    // Row 3: age too old (70 > 65)
    // Row 4: invalid status ("pending" not in enum)
    assert!(
        report.violations.len() >= 4,
        "should catch at least 4 violations, got {}",
        report.violations.len()
    );

    // Verify specific violation types
    let name_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "name")
        .collect();
    assert!(
        !name_violations.is_empty(),
        "should catch name length violation"
    );

    let age_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "age")
        .collect();
    assert_eq!(
        age_violations.len(),
        2,
        "should catch 2 age range violations"
    );

    let status_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "status")
        .collect();
    assert!(
        !status_violations.is_empty(),
        "should catch status enum violation"
    );
}

#[test]
fn schema_summary_includes_field_info() {
    let schema = InferredSchema::from_values(&training_data());
    let summary = schema.summary();

    println!("Schema summary:\n{}", summary);

    // Summary should include all 3 fields
    assert!(
        summary.contains("name"),
        "summary should mention name field"
    );
    assert!(summary.contains("age"), "summary should mention age field");
    assert!(
        summary.contains("status"),
        "summary should mention status field"
    );
}

#[test]
fn external_required_overrides_inferred() {
    // Train on data where "email" is always present
    let training = vec![
        serde_json::json!({"name": "Alice", "email": "a@example.com"}),
        serde_json::json!({"name": "Bob", "email": "b@example.com"}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Apply external constraint: email is NOT required (override)
    let mut constraints = HashMap::new();
    constraints.insert(
        "email".to_string(),
        ExternalConstraint {
            required: false,
            ..Default::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // New batch: missing email
    let batch = vec![serde_json::json!({"name": "Charlie"})];
    let report = schema.audit(&batch);

    println!("Missing email violations: {:?}", report.violations);
    // With external_required=false, missing email should NOT be a violation
    let email_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "email")
        .collect();
    assert!(
        email_violations.is_empty(),
        "external_required=false should suppress missing-field violations"
    );
}
