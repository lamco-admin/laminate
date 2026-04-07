#![allow(dead_code)]
//! Iteration 217: #[laminate(overflow)] as Option<HashMap>.
//!
//! When overflow is Option<HashMap<String, Value>>:
//! - No unknown fields → None (not Some(empty map))
//! - Unknown fields → Some(map)
//! This distinction matters for serialization and downstream logic.

use laminate::Laminate;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Event {
    event_type: String,
    #[laminate(overflow)]
    metadata: Option<HashMap<String, Value>>,
}

#[test]
fn no_overflow_becomes_none() {
    let json = json!({"event_type": "click"});
    let result = Event::from_flex_value(&json);
    println!("No overflow: {:?}", result);
    assert!(result.is_ok());
    let (event, _) = result.unwrap();
    assert_eq!(
        event.metadata, None,
        "No unknown fields should give None, not Some(empty)"
    );
}

#[test]
fn overflow_becomes_some() {
    let json = json!({"event_type": "click", "x": 100, "y": 200});
    let result = Event::from_flex_value(&json);
    println!("With overflow: {:?}", result);
    assert!(result.is_ok());
    let (event, _) = result.unwrap();
    assert!(event.metadata.is_some());
    let meta = event.metadata.unwrap();
    assert_eq!(meta.get("x"), Some(&json!(100)));
    assert_eq!(meta.get("y"), Some(&json!(200)));
}

#[test]
fn option_overflow_roundtrip() {
    // With overflow
    let json = json!({"event_type": "submit", "form_id": "abc123"});
    let result = Event::from_flex_value(&json);
    assert!(result.is_ok());
    let (event, _) = result.unwrap();
    let output = event.to_value();
    println!(
        "Round-trip: {}",
        serde_json::to_string_pretty(&output).unwrap()
    );
    assert_eq!(output.get("form_id"), Some(&json!("abc123")));
}

#[test]
fn none_overflow_roundtrip() {
    // Without overflow — to_value should not include empty metadata
    let json = json!({"event_type": "pageview"});
    let result = Event::from_flex_value(&json);
    assert!(result.is_ok());
    let (event, _) = result.unwrap();
    let output = event.to_value();
    println!(
        "No-overflow round-trip: {}",
        serde_json::to_string_pretty(&output).unwrap()
    );

    // Should only have event_type, no extra keys from empty overflow
    let obj = output.as_object().unwrap();
    assert_eq!(obj.len(), 1, "Only event_type should be present");
}
