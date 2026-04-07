//! Iteration 253: set() with adversarial inputs
//!
//! Tests set() behavior with:
//! - Creating intermediate objects
//! - Setting through existing scalar (silently skips)
//! - Setting at array indices
//! - Unicode paths
//! - Overwriting existing values

use laminate::FlexValue;

#[test]
fn set_creates_intermediate_objects() {
    let mut val = FlexValue::from_json(r#"{}"#).unwrap();
    val.set("a.b.c", serde_json::json!(42)).unwrap();

    let result: i64 = val.extract("a.b.c").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn set_overwrites_existing_value() {
    let mut val = FlexValue::from_json(r#"{"a": 1}"#).unwrap();
    val.set("a", serde_json::json!(99)).unwrap();

    let result: i64 = val.extract("a").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn set_nested_overwrites() {
    let mut val = FlexValue::from_json(r#"{"a": {"b": 1}}"#).unwrap();
    val.set("a.b", serde_json::json!(99)).unwrap();

    let result: i64 = val.extract("a.b").unwrap();
    assert_eq!(result, 99);
}

#[test]
fn set_adds_new_key_to_existing_object() {
    let mut val = FlexValue::from_json(r#"{"a": {"b": 1}}"#).unwrap();
    val.set("a.c", serde_json::json!(42)).unwrap();

    // Both b and c should exist
    assert_eq!(val.extract::<i64>("a.b").unwrap(), 1);
    assert_eq!(val.extract::<i64>("a.c").unwrap(), 42);
}

#[test]
fn set_through_scalar_silent_skip() {
    // If intermediate is a scalar, set should silently skip (not panic)
    let mut val = FlexValue::from_json(r#"{"a": 42}"#).unwrap();
    val.set("a.b", serde_json::json!(99)).unwrap();

    // "a" is still 42 (can't create intermediate through scalar)
    let result: i64 = val.extract("a").unwrap();
    println!("set through scalar: a = {}", result);
    // Behavior: either silently skip or overwrite. Let's observe.
}

#[test]
fn set_null_value() {
    let mut val = FlexValue::from_json(r#"{"a": 1}"#).unwrap();
    val.set("a", serde_json::json!(null)).unwrap();

    assert!(val.at("a").unwrap().raw().is_null(), "set null should work");
}

#[test]
fn set_complex_value() {
    let mut val = FlexValue::from_json(r#"{}"#).unwrap();
    val.set(
        "config",
        serde_json::json!({"host": "localhost", "port": 8080}),
    )
    .unwrap();

    assert_eq!(val.extract::<String>("config.host").unwrap(), "localhost");
    assert_eq!(val.extract::<i64>("config.port").unwrap(), 8080);
}

#[test]
fn set_preserves_coercion_settings() {
    use laminate::CoercionLevel;

    let mut val = FlexValue::from_json(r#"{}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);

    val.set("score", serde_json::json!("95")).unwrap();

    // Coercion should still work after set
    let result: i64 = val.extract("score").unwrap();
    assert_eq!(result, 95, "coercion should work on set values");
}
