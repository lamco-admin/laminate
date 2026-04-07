//! Iteration 180: Strict mode int→float — does Exact reject integer as f64?
//!
//! In JSON, 42 is an integer. Extracting as f64 is a widening conversion.
//! Exact coercion should reject this since it's technically a type change.

use laminate::FlexValue;
use laminate::Strict;

#[test]
fn strict_mode_int_to_float_extraction() {
    let val = FlexValue::from_json(r#"{"x": 42}"#)
        .unwrap()
        .with_mode::<Strict>();

    // serde_json treats integers and floats as Number. The question is:
    // does Exact coercion reject Number(42) → f64?
    let result = val.extract::<f64>("x");

    // Observe actual behavior before deciding correctness
    match &result {
        Ok(v) => println!("OK: {v}"),
        Err(e) => println!("ERR: {e}"),
    }

    // Exact coercion correctly rejects int→float as a type conversion.
    // Even though 42 → 42.0 is lossless, Strict/Exact means "types must match exactly."
    // Use SafeWidening (Absorbing mode) or higher for this conversion.
    assert!(
        result.is_err(),
        "Exact coercion should reject int→float conversion"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("SafeWidening"),
        "Error should mention required level: {err}"
    );
}

#[test]
fn strict_mode_rejects_string_to_float() {
    let val = FlexValue::from_json(r#"{"x": "42.5"}"#)
        .unwrap()
        .with_mode::<Strict>();

    let result = val.extract::<f64>("x");
    assert!(result.is_err(), "Strict should reject string → f64");
}
