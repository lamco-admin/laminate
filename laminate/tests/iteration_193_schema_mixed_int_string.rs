#![allow(dead_code)]
//! Iteration 193: Schema from 50% integers + 50% strings.
//!
//! Question: When a field has equal counts of integers and strings,
//! which type wins as dominant? Does wideness favor String?

use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn schema_equal_int_string_mix() {
    let rows: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                json!({"value": i}) // integer
            } else {
                json!({"value": format!("s{i}")}) // string
            }
        })
        .collect();

    let schema = InferredSchema::from_values(&rows);
    let field = schema.fields.get("value").unwrap();

    println!("Dominant type: {:?}", field.dominant_type);
    println!("Type distribution: {:?}", field.type_counts);
    println!("Is mixed: {}", field.is_mixed_type());
    println!("Consistency: {:.2}", field.type_consistency());

    // 5 integers, 5 strings — which wins?
    // Observe the behavior before asserting.
    assert!(field.is_mixed_type(), "Should detect mixed types");
}

#[test]
fn schema_slight_majority_string() {
    // 6 strings, 4 integers — string should dominate
    let rows: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            if i < 4 {
                json!({"value": i})
            } else {
                json!({"value": format!("s{i}")})
            }
        })
        .collect();

    let schema = InferredSchema::from_values(&rows);
    let field = schema.fields.get("value").unwrap();

    println!("6-string/4-int dominant: {:?}", field.dominant_type);
    println!("Distribution: {:?}", field.type_counts);
}
