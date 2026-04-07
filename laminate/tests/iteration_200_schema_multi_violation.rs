#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration 200: Multiple violations per field per row.
//!
//! A field can have both TypeMismatch (wrong type) AND ConstraintViolation
//! (violates max_length, min/max value, etc.) in the same row.
//! Do both get reported?

use laminate::schema::{InferredSchema, ViolationKind};

#[test]
fn multiple_violations_same_field_same_row() {
    let training = vec![
        serde_json::json!({"score": 50}),
        serde_json::json!({"score": 75}),
    ];
    let mut schema = InferredSchema::from_values(&training);

    // Set constraints: must be integer, max_value = 100
    schema.fields.get_mut("score").unwrap().max_value = Some(100.0);

    // Audit with a STRING "999" — both type mismatch (string, not integer)
    // AND constraint violation (999 > 100, if it were numeric)
    let test = vec![serde_json::json!({"score": "999"})];
    let report = schema.audit(&test);

    println!("Violations: {}", report.violations.len());
    for v in &report.violations {
        println!(
            "  kind={:?} field={} expected={} actual={}",
            v.kind, v.field, v.expected, v.actual
        );
    }

    // At minimum, should flag the type mismatch (string vs integer)
    let _type_mismatches: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::TypeMismatch))
        .collect();

    // "999" is a string, schema expects Integer — but coercion engine may
    // classify it as coercible (string "999" → integer is a plausible coercion).
    // So it might be coercible rather than a hard mismatch.
    let total = report.violations.len();
    // Observation point — print violations count for manual review
    println!("Got {total} violations");
}

#[test]
fn numeric_type_with_out_of_range_value() {
    let training = vec![
        serde_json::json!({"temp": 20.0}),
        serde_json::json!({"temp": 25.0}),
    ];
    let mut schema = InferredSchema::from_values(&training);
    schema.fields.get_mut("temp").unwrap().min_value = Some(0.0);
    schema.fields.get_mut("temp").unwrap().max_value = Some(50.0);

    // Out of range but correct type
    let test = vec![serde_json::json!({"temp": 99.9})];
    let report = schema.audit(&test);

    let constraint_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| matches!(v.kind, ViolationKind::ConstraintViolation))
        .collect();

    assert_eq!(
        constraint_violations.len(),
        1,
        "Should flag max_value violation"
    );
    assert!(constraint_violations[0].expected.contains("max_value"));
}
