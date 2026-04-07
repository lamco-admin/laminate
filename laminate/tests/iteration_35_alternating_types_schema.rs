//! Iteration 35: Boundary — Schema inference with alternating types per field
//!
//! Tests that schema inference handles polymorphic fields (same field has
//! different types across rows) correctly and deterministically.

use laminate::schema::{InferredSchema, JsonType};
use serde_json::{json, Value};

#[test]
fn alternating_types_dominant_type_is_deterministic() {
    // Tied: 2 Integer, 2 String, 1 Bool, 1 null
    let rows: Vec<Value> = vec![
        json!({"v": 1}),
        json!({"v": "hello"}),
        json!({"v": true}),
        json!({"v": null}),
        json!({"v": 42}),
        json!({"v": "world"}),
    ];

    // Must always pick String (widest type) when tied with Integer
    for _ in 0..20 {
        let schema = InferredSchema::from_values(&rows);
        let field = schema.fields.get("v").unwrap();
        assert_eq!(
            field.dominant_type,
            Some(JsonType::String),
            "dominant type must be deterministic (String wins ties)"
        );
    }
}

#[test]
fn four_way_tie_picks_widest() {
    let rows: Vec<Value> = vec![
        json!({"v": 1}),
        json!({"v": "x"}),
        json!({"v": true}),
        json!({"v": 1.5}),
    ];

    for _ in 0..20 {
        let schema = InferredSchema::from_values(&rows);
        let field = schema.fields.get("v").unwrap();
        assert_eq!(field.dominant_type, Some(JsonType::String));
    }
}

#[test]
fn type_counts_track_all_types() {
    let rows: Vec<Value> = vec![
        json!({"v": 1}),
        json!({"v": "hello"}),
        json!({"v": true}),
        json!({"v": null}),
    ];
    let schema = InferredSchema::from_values(&rows);
    let field = schema.fields.get("v").unwrap();

    assert_eq!(field.type_counts.get(&JsonType::Integer), Some(&1));
    assert_eq!(field.type_counts.get(&JsonType::String), Some(&1));
    assert_eq!(field.type_counts.get(&JsonType::Bool), Some(&1));
    assert_eq!(field.null_count, 1);
    assert_eq!(field.present_count, 4);
    assert_eq!(field.absent_count, 0);
}

#[test]
fn all_null_field_has_no_dominant_type() {
    let rows: Vec<Value> = vec![json!({"v": null}), json!({"v": null})];
    let schema = InferredSchema::from_values(&rows);
    let field = schema.fields.get("v").unwrap();
    assert_eq!(field.dominant_type, None);
    assert_eq!(field.null_count, 2);
}

#[test]
fn clear_majority_wins_regardless_of_wideness() {
    // Integer has 3, String has 1 — Integer should win (not a tie)
    let rows: Vec<Value> = vec![
        json!({"v": 1}),
        json!({"v": 2}),
        json!({"v": 3}),
        json!({"v": "x"}),
    ];
    let schema = InferredSchema::from_values(&rows);
    let field = schema.fields.get("v").unwrap();
    assert_eq!(field.dominant_type, Some(JsonType::Integer));
}
