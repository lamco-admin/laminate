//! Iteration 232: E2E CSV pipeline — mixed types → schema inference → audit → extract
//!
//! Full pipeline: simulate CSV data with mixed types, infer schema, audit
//! a new batch against the schema, and extract typed values.

use laminate::schema::{InferredSchema, JsonType};
use laminate::value::SourceHint;
use laminate::FlexValue;

fn sample_rows() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"name": "Alice", "age": 30, "score": 95.5}),
        serde_json::json!({"name": "Bob", "age": 25, "score": 88.0}),
        serde_json::json!({"name": "Charlie", "age": 35, "score": 91.2}),
        serde_json::json!({"name": "Diana", "age": 28, "score": 76.8}),
    ]
}

#[test]
fn infer_schema_from_clean_data() {
    let rows = sample_rows();
    let schema = InferredSchema::from_values(&rows);

    println!("Schema: {:#?}", schema);

    let name_def = schema.fields.get("name").expect("name field should exist");
    assert_eq!(name_def.dominant_type, Some(JsonType::String));

    let age_def = schema.fields.get("age").expect("age field should exist");
    assert_eq!(age_def.dominant_type, Some(JsonType::Integer));

    let score_def = schema
        .fields
        .get("score")
        .expect("score field should exist");
    // 88.0 has no fractional part → classified as Integer by JsonType::of
    // So dominant type might be Integer (3 Integer vs 2 Float) — let's observe
    println!("score type counts: {:?}", score_def.type_counts);
}

#[test]
fn audit_new_batch_against_inferred_schema() {
    let training = sample_rows();
    let schema = InferredSchema::from_values(&training);

    // New batch with violations
    let new_batch = vec![
        serde_json::json!({"name": "Eve", "age": 22, "score": 99.1}),
        serde_json::json!({"name": 42, "age": "thirty", "score": "high"}), // all wrong types
        serde_json::json!({"name": "Frank"}),                              // missing fields
    ];

    let report = schema.audit(&new_batch);
    println!("Audit report: {:#?}", report);
    println!("Total violations: {}", report.violations.len());

    // Should have violations for row 2 (type mismatches) and row 3 (missing fields)
    assert!(
        !report.violations.is_empty(),
        "should detect violations in bad data"
    );
}

#[test]
fn csv_strings_to_typed_extraction_after_schema() {
    // CSV-style: all values are strings
    let csv_row = serde_json::json!({"name": "Grace", "age": "40", "score": "87.3"});
    let val = FlexValue::from(csv_row).with_source_hint(SourceHint::Csv);

    let name: String = val.extract("name").unwrap();
    let age: i64 = val.extract("age").unwrap();
    let score: f64 = val.extract("score").unwrap();

    assert_eq!(name, "Grace");
    assert_eq!(age, 40);
    assert!((score - 87.3).abs() < 0.01);
}
