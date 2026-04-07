//! Iteration 198: Non-object row in audit — bare array, bare string, bare null at root.
//!
//! The schema audit expects rows to be JSON objects. What happens with
//! non-object values mixed in?

use laminate::schema::InferredSchema;

#[test]
fn audit_handles_bare_string_row() {
    let training = vec![
        serde_json::json!({"name": "Alice"}),
        serde_json::json!({"name": "Bob"}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Mix in a bare string row
    let test = vec![
        serde_json::json!({"name": "Charlie"}),
        serde_json::json!("just a string"),
    ];
    let report = schema.audit(&test);

    // Non-object rows should be gracefully skipped or flagged,
    // not cause a panic.
    println!("Total violations: {}", report.violations.len());
}

#[test]
fn audit_handles_bare_array_row() {
    let training = vec![serde_json::json!({"x": 1})];
    let schema = InferredSchema::from_values(&training);

    let test = vec![serde_json::json!([1, 2, 3])];
    let report = schema.audit(&test);
    println!("Violations for array row: {}", report.violations.len());
}

#[test]
fn audit_handles_null_row() {
    let training = vec![serde_json::json!({"x": 1})];
    let schema = InferredSchema::from_values(&training);

    let test = vec![serde_json::json!(null)];
    let report = schema.audit(&test);
    println!("Violations for null row: {}", report.violations.len());
}
