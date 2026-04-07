//! Iteration 192: Mixed Integer and Float column — does Float win as dominant?
//!
//! When a column has both integers and floats, the wideness tiebreak
//! should favor Float (wideness 3) over Integer (wideness 2).

use laminate::schema::{InferredSchema, JsonType};

#[test]
fn mixed_int_float_float_dominates() {
    let rows = vec![
        serde_json::json!({"val": 1}),
        serde_json::json!({"val": 2}),
        serde_json::json!({"val": 3.5}),
        serde_json::json!({"val": 4}),
    ];

    let schema = InferredSchema::from_values(&rows);
    let val_def = schema.fields.get("val").unwrap();

    // 3 integers, 1 float — Integer has higher count.
    // But does wideness tiebreak apply when counts differ?
    println!("dominant_type: {:?}", val_def.dominant_type);
    println!("type_counts: {:?}", val_def.type_counts);

    // Integer should be dominant (3 vs 1) — wideness only breaks TIES
    assert_eq!(
        val_def.dominant_type,
        Some(JsonType::Integer),
        "Integer should be dominant with 3 vs 1 count"
    );
}

#[test]
fn equal_int_float_float_wins_by_wideness() {
    let rows = vec![
        serde_json::json!({"val": 1}),
        serde_json::json!({"val": 2.5}),
    ];

    let schema = InferredSchema::from_values(&rows);
    let val_def = schema.fields.get("val").unwrap();

    // 1 integer, 1 float — tied count, wideness should break tie → Float
    assert_eq!(
        val_def.dominant_type,
        Some(JsonType::Float),
        "Float should win by wideness when count is tied"
    );
}
