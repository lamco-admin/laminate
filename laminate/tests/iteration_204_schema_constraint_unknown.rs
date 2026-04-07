#![allow(dead_code)]
//! Iteration 204: with_constraints() on a field not in training data.
//!
//! Question: If you add an ExternalConstraint for a field that doesn't exist
//! in the inferred schema, is it silently ignored? Does it cause a panic?

use laminate::schema::{ExternalConstraint, InferredSchema, JsonType};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn constraint_on_nonexistent_field() {
    let rows = vec![json!({"name": "Alice"}), json!({"name": "Bob"})];
    let schema = InferredSchema::from_values(&rows);

    // Apply constraint to a field that doesn't exist in the data
    let mut constraints = HashMap::new();
    constraints.insert(
        "email".into(),
        ExternalConstraint {
            expected_type: Some(JsonType::String),
            required: true,
            ..ExternalConstraint::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // Does "email" now appear in the schema?
    let has_email = schema.fields.contains_key("email");
    println!("Has email field after constraint: {has_email}");

    // Audit data without "email" — does the required constraint fire?
    let test_data = vec![json!({"name": "Carol"})];
    let report = schema.audit(&test_data);
    println!("Violations: {:?}", report.violations);

    // The field wasn't in training, so the constraint was silently ignored
    // per the code comment: "If the constraint names a field not in the data, we skip"
}
