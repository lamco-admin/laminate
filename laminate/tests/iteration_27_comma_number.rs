//! Iteration 27: GAP — comma-separated thousands "1,000" failed to coerce
//! Fix: Added try_strip_comma_thousands() for US-style comma thousands.
//! Rejects European format (comma after dot) and malformed double-commas.

use laminate::FlexValue;

#[test]
fn iter27_us_comma_thousands_to_integer() {
    let json = serde_json::json!({"s": "1,000", "m": "1,234,567", "l": "1,000,000,000"});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<i64>("s").unwrap(), 1_000);
    assert_eq!(flex.extract::<i64>("m").unwrap(), 1_234_567);
    assert_eq!(flex.extract::<i64>("l").unwrap(), 1_000_000_000);
}

#[test]
fn iter27_negative_comma_thousands() {
    let json = serde_json::json!({"val": "-1,000"});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<i64>("val").unwrap(), -1000);
}

#[test]
fn iter27_us_comma_with_decimal_to_float() {
    let json = serde_json::json!({"val": "1,234.56"});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<f64>("val").unwrap(), 1234.56);
}

#[test]
fn iter27_european_format_parsed() {
    // "1.234,56" is European format — now correctly parsed as 1234.56
    let json = serde_json::json!({"val": "1.234,56"});
    let flex = FlexValue::new(json);
    let result: f64 = flex.extract("val").unwrap();
    assert!((result - 1234.56).abs() < 0.01);
}

#[test]
fn iter27_malformed_double_comma_rejected() {
    let json = serde_json::json!({"val": "1,,000"});
    let flex = FlexValue::new(json);
    assert!(flex.extract::<i64>("val").is_err());
}

#[test]
fn iter27_comma_diagnostic_warns() {
    let json = serde_json::json!({"val": "1,000"});
    let flex = FlexValue::new(json);
    let (val, diags) = flex.extract_with_diagnostics::<i64>("val").unwrap();
    assert_eq!(val, 1000);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].risk, laminate::RiskLevel::Warning);
}

#[test]
fn iter27_comma_as_string_preserved() {
    let json = serde_json::json!({"val": "1,000"});
    let flex = FlexValue::new(json);
    assert_eq!(flex.extract::<String>("val").unwrap(), "1,000");
}
