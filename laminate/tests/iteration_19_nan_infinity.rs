//! Iteration 19: Type Swap — "NaN" and "Infinity" string coercion to f64
//! Gap: "NaN" parsed as f64 but serde_json::Number can't represent it.
//! Fix: "NaN" treated as null sentinel at BestEffort (maps to None for Option<f64>).
//! "Infinity" flagged with clear diagnostic rather than opaque serde error.

use laminate::FlexValue;

#[test]
fn iter19_nan_as_option_f64_returns_none() {
    // "NaN" is a null sentinel — Option<f64> should get None
    let flex = FlexValue::from_json(r#"{"val": "NaN"}"#).unwrap();
    let result: Option<f64> = flex.extract("val").unwrap();
    assert_eq!(result, None);
}

#[test]
fn iter19_nan_as_bare_f64_defaults_to_zero() {
    // "NaN" is a null sentinel → null → Null→Default → 0.0 at BestEffort
    // Consistent with "unknown", "null", "N/A" all producing 0.0 for bare f64
    let flex = FlexValue::from_json(r#"{"val": "NaN"}"#).unwrap();
    let result: f64 = flex.extract("val").unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn iter19_nan_as_string_preserved() {
    // "NaN" extracted as String should pass through unchanged
    let flex = FlexValue::from_json(r#"{"val": "NaN"}"#).unwrap();
    let result: String = flex.extract("val").unwrap();
    assert_eq!(result, "NaN");
}

#[test]
fn iter19_infinity_as_f64_errors() {
    // Infinity can't be represented in JSON — should error
    let flex = FlexValue::from_json(r#"{"val": "Infinity"}"#).unwrap();
    assert!(flex.extract::<f64>("val").is_err());
}

#[test]
fn iter19_neg_infinity_as_f64_errors() {
    let flex = FlexValue::from_json(r#"{"val": "-Infinity"}"#).unwrap();
    assert!(flex.extract::<f64>("val").is_err());
}

#[test]
fn iter19_nan_case_insensitive() {
    // "nan", "NAN", "NaN" should all be treated as null sentinel
    for variant in &["nan", "NAN", "NaN", "Nan"] {
        let json = format!(r#"{{"val": "{}"}}"#, variant);
        let flex = FlexValue::from_json(&json).unwrap();
        let result: Option<f64> = flex.extract("val").unwrap();
        assert_eq!(result, None, "\"{}\" should map to None", variant);
    }
}
