//! Iteration 197: allowed_values on Integer field.
//!
//! allowed_values = ["1","2"] but JSON has Value::Number(1).
//! The check uses value.as_str() which returns None for numbers,
//! silently skipping the constraint.

use laminate::schema::InferredSchema;

#[test]
fn allowed_values_checks_non_string_types() {
    let training = vec![
        serde_json::json!({"status": 1}),
        serde_json::json!({"status": 2}),
    ];
    let mut schema = InferredSchema::from_values(&training);

    // Set allowed_values constraint — these are string representations
    schema.fields.get_mut("status").unwrap().allowed_values = Some(vec!["1".into(), "2".into()]);

    // Audit with value 3 (not in allowed set)
    let test = vec![serde_json::json!({"status": 3})];
    let report = schema.audit(&test);

    println!("Violations: {}", report.violations.len());
    for v in &report.violations {
        println!("  {v}");
    }

    // Currently silently passes because as_str() returns None for numbers.
    // This SHOULD flag a constraint violation.
    assert_eq!(
        report.violations.len(),
        1,
        "Integer value 3 should violate allowed_values [1,2]"
    );
}

#[test]
fn allowed_values_passes_valid_integer() {
    let training = vec![serde_json::json!({"status": 1})];
    let mut schema = InferredSchema::from_values(&training);
    schema.fields.get_mut("status").unwrap().allowed_values = Some(vec!["1".into(), "2".into()]);

    let test = vec![serde_json::json!({"status": 1})];
    let report = schema.audit(&test);
    assert_eq!(report.violations.len(), 0, "Value 1 is in allowed set");
}
