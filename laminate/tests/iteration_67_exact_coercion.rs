//! Iteration 67 — CoercionLevel::Exact with plain extract
//!
//! PASS: Exact level correctly prevents all type coercion.
//! Same-type works, cross-type fails with DeserializeError.

use laminate::coerce::CoercionLevel;
use laminate::value::FlexValue;

#[test]
fn exact_allows_same_type_extraction() {
    let fv = FlexValue::from_json(r#"{"name":"Alice","count":42,"flag":true}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);

    let name: String = fv.extract("name").unwrap();
    assert_eq!(name, "Alice");

    let count: i64 = fv.extract("count").unwrap();
    assert_eq!(count, 42);

    let flag: bool = fv.extract("flag").unwrap();
    assert!(flag);
}

#[test]
fn exact_blocks_string_to_number_coercion() {
    let fv = FlexValue::from_json(r#"{"port":"8080"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);

    let result: Result<u16, _> = fv.extract("port");
    assert!(result.is_err());

    // Same input with BestEffort succeeds
    let fv2 = FlexValue::from_json(r#"{"port":"8080"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let port: u16 = fv2.extract("port").unwrap();
    assert_eq!(port, 8080);
}

#[test]
fn exact_blocks_string_to_bool_coercion() {
    let fv = FlexValue::from_json(r#"{"flag":"true"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);

    let result: Result<bool, _> = fv.extract("flag");
    assert!(result.is_err());
}

#[test]
fn exact_rejects_json_number_widening() {
    // Iteration 81 fix: Exact mode now rejects integer → float widening
    // (previously serde silently allowed this, violating "types must match exactly")
    let fv = FlexValue::from_json(r#"{"n":42}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);

    let result: Result<f64, _> = fv.extract("n");
    assert!(result.is_err(), "Exact mode should reject integer → f64");
}
