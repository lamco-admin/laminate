//! Iteration 178: FlexValue with_mode::<Strict>() does NOT reject unknown fields
//!
//! with_mode only sets CoercionLevel — it's a FlexValue-level concept.
//! Unknown field rejection is a DERIVE-level concept (shape_strict).
//! This is correct by design: FlexValue doesn't know your struct shape.

use laminate::FlexValue;
use laminate::Strict;

#[test]
fn with_mode_strict_does_not_reject_unknown_fields() {
    // JSON with "extra" unknown field
    let val = FlexValue::from_json(r#"{"name": "test", "age": 30, "unknown": "extra"}"#)
        .unwrap()
        .with_mode::<Strict>();

    // extract on known fields still works — strict only means Exact coercion
    let name: String = val.extract("name").unwrap();
    assert_eq!(name, "test");

    let age: i64 = val.extract("age").unwrap();
    assert_eq!(age, 30);

    // "unknown" field is accessible — not rejected at FlexValue level
    let extra: String = val.extract("unknown").unwrap();
    assert_eq!(extra, "extra");
}

#[test]
fn with_mode_strict_rejects_type_coercion() {
    // Strict = Exact coercion. String "42" should NOT coerce to i64.
    let val = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_mode::<Strict>();

    let result = val.extract::<i64>("x");
    assert!(result.is_err(), "Strict mode should reject string → i64");
}
