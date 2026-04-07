//! Iteration 199: Unknown field detection in audit.

use laminate::schema::{InferredSchema, ViolationKind};

#[test]
fn audit_flags_unknown_fields() {
    let training = vec![serde_json::json!({"name": "Alice", "age": 30})];
    let schema = InferredSchema::from_values(&training);

    // Audit with an extra "role" field not in schema
    let test = vec![serde_json::json!({"name": "Bob", "age": 25, "role": "admin"})];
    let report = schema.audit(&test);

    let unknown_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::UnknownField))
        .collect();

    assert_eq!(unknown_violations.len(), 1, "Should flag one unknown field");
    assert_eq!(unknown_violations[0].field, "role");
}

#[test]
fn audit_no_unknown_when_fields_match() {
    let training = vec![serde_json::json!({"name": "Alice", "age": 30})];
    let schema = InferredSchema::from_values(&training);

    let test = vec![serde_json::json!({"name": "Bob", "age": 25})];
    let report = schema.audit(&test);

    let unknown_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::UnknownField))
        .collect();

    assert_eq!(unknown_violations.len(), 0);
}
