#![allow(dead_code)]
//! Iteration 201: fill_rate threshold for required fields.
//!
//! Question: A field present in 99/100 rows — is it required at threshold 1.0?
//! What about threshold 0.95? Does the threshold affect appears_required()?

use laminate::schema::{InferenceConfig, InferredSchema};
use serde_json::json;

#[test]
fn fill_rate_99_of_100() {
    // 99 rows have "name", 1 row doesn't
    let mut rows: Vec<serde_json::Value> = (0..99)
        .map(|i| json!({"name": format!("user_{i}"), "id": i}))
        .collect();
    rows.push(json!({"id": 99})); // missing "name"

    let schema = InferredSchema::from_values(&rows);
    let name = schema.fields.get("name").unwrap();

    println!("name fill_rate: {:.4}", name.fill_rate());
    println!("name appears_required: {}", name.appears_required());
    println!("name present_count: {}", name.present_count);
    println!("name absent_count: {}", name.absent_count);
    println!("name null_count: {}", name.null_count);

    // fill_rate should be 0.99, appears_required should be false (absent_count > 0)
    assert!((name.fill_rate() - 0.99).abs() < 0.01);
    assert!(
        !name.appears_required(),
        "1 absence should make it not required"
    );
}

#[test]
fn fill_rate_with_custom_threshold() {
    let mut rows: Vec<serde_json::Value> = (0..95)
        .map(|i| json!({"name": format!("user_{i}"), "id": i}))
        .collect();
    // 5 rows without "name"
    for i in 95..100 {
        rows.push(json!({"id": i}));
    }

    let config = InferenceConfig {
        required_threshold: 0.95,
        ..InferenceConfig::default()
    };
    let schema = InferredSchema::from_values_with_config(&rows, &config);
    let name = schema.fields.get("name").unwrap();

    println!("95% fill_rate: {:.4}", name.fill_rate());
    println!(
        "appears_required (threshold 0.95): {}",
        name.appears_required()
    );
    // fill_rate is 0.95, threshold is 0.95 — borderline. Does it count as required?
}

#[test]
fn fill_rate_all_present() {
    let rows: Vec<serde_json::Value> = (0..10)
        .map(|i| json!({"name": format!("user_{i}")}))
        .collect();

    let schema = InferredSchema::from_values(&rows);
    let name = schema.fields.get("name").unwrap();

    assert_eq!(name.fill_rate(), 1.0);
    assert!(name.appears_required());
}
