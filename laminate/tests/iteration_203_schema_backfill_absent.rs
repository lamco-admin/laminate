#![allow(dead_code)]
//! Iteration 203: Field absent in first N rows, present in rest.
//!
//! Question: If a field appears only in the later rows (not in the first few),
//! is absent_count correctly tracking the early absences?

use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn field_absent_then_present() {
    // First 5 rows: no "email" field. Last 5 rows: "email" present.
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for i in 0..5 {
        rows.push(json!({"id": i, "name": format!("user_{i}")}));
    }
    for i in 5..10 {
        rows.push(json!({"id": i, "name": format!("user_{i}"), "email": format!("u{i}@test.com")}));
    }

    let schema = InferredSchema::from_values(&rows);
    let email = schema.fields.get("email").unwrap();

    println!("email present_count: {}", email.present_count);
    println!("email absent_count: {}", email.absent_count);
    println!("email null_count: {}", email.null_count);
    println!("email fill_rate: {:.2}", email.fill_rate());
    println!("email appears_required: {}", email.appears_required());

    // email was absent in first 5 rows
    assert_eq!(email.absent_count, 5, "Should count 5 absences");
    assert_eq!(email.present_count, 5, "Should count 5 presences");
    assert!(!email.appears_required(), "5 absences means not required");
}

#[test]
fn field_present_then_absent() {
    // First 3 rows: "score" present. Last 7: absent.
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for i in 0..3 {
        rows.push(json!({"id": i, "score": 100 - i}));
    }
    for i in 3..10 {
        rows.push(json!({"id": i}));
    }

    let schema = InferredSchema::from_values(&rows);
    let score = schema.fields.get("score").unwrap();

    println!(
        "score present: {}, absent: {}",
        score.present_count, score.absent_count
    );

    assert_eq!(score.present_count, 3);
    assert_eq!(score.absent_count, 7);
}
