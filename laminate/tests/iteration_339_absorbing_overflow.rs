//! Iteration 339: shape_absorbing with 50 unknown fields — verify all captured in residual.

use laminate::Laminate;
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Minimal {
    name: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn absorbing_50_unknown_fields() {
    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), serde_json::json!("Alice"));
    for i in 0..50 {
        obj.insert(format!("field_{}", i), serde_json::json!(i));
    }
    let json = serde_json::Value::Object(obj);

    let result = Minimal::shape_absorbing(&json);
    println!(
        "absorbing 50 unknowns: {:?}",
        result.as_ref().map(|r| r.value.extra.len())
    );

    assert!(result.is_ok(), "absorbing should accept unknowns");
    let lr = result.unwrap();
    assert_eq!(lr.value.name, "Alice");
    assert_eq!(
        lr.value.extra.len(),
        50,
        "all 50 unknown fields should be in overflow"
    );

    // Spot-check some values
    assert_eq!(lr.value.extra["field_0"], serde_json::json!(0));
    assert_eq!(lr.value.extra["field_49"], serde_json::json!(49));
}
