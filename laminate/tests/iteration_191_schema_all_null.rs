//! Iteration 191: All-null column → schema infers dominant_type=None → audit with non-null value
//!
//! If every value in a column is null, there's no non-null type to infer.
//! What happens when we audit with a row that has a non-null value?

use laminate::schema::InferredSchema;

#[test]
fn all_null_column_infers_no_dominant_type() {
    let rows = vec![
        serde_json::json!({"name": "Alice", "score": null}),
        serde_json::json!({"name": "Bob", "score": null}),
        serde_json::json!({"name": "Charlie", "score": null}),
    ];

    let schema = InferredSchema::from_values(&rows);
    let score_def = schema.fields.get("score").unwrap();

    // All null → no dominant type
    assert_eq!(
        score_def.dominant_type, None,
        "All-null column should have no dominant type"
    );
    assert_eq!(score_def.null_count, 3);
    assert_eq!(score_def.present_count, 3); // present but null
}

#[test]
fn audit_non_null_value_against_all_null_schema() {
    // Infer from all-null data
    let training = vec![
        serde_json::json!({"name": "Alice", "score": null}),
        serde_json::json!({"name": "Bob", "score": null}),
    ];
    let schema = InferredSchema::from_values(&training);

    // Audit with non-null data
    let test = vec![serde_json::json!({"name": "Dave", "score": 95})];
    let report = schema.audit(&test);

    // Should the audit flag a type violation? With no dominant type,
    // any value is technically "unexpected." Let's observe.
    println!("Violations: {:?}", report.violations.len());
    for v in &report.violations {
        println!("  {v}");
    }

    // With no expected type, the audit should NOT flag a violation
    // (there's nothing to violate against). This is the sane default.
    assert_eq!(
        report.violations.len(),
        0,
        "No expected type = no violation. Got {} violations.",
        report.violations.len()
    );
}
