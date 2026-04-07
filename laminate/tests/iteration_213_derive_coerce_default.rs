#![allow(dead_code)]
//! Iteration 213: #[laminate(coerce, default)] on scalar.
//!
//! When coerce fails (e.g., "abc" → i64), does #[laminate(default)] silently
//! provide the default value? Or does the coercion error propagate?
//! If the error is swallowed, that could be a silent data corruption issue.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(coerce, default)]
    retries: i64,
}

#[test]
fn coerce_succeeds_no_default_needed() {
    let json = serde_json::json!({"name": "api", "retries": "3"});
    let result = Config::from_flex_value(&json);
    println!("Coerce \"3\" → {:?}", result);
    assert!(result.is_ok());
    let (cfg, _) = result.unwrap();
    assert_eq!(cfg.retries, 3);
}

#[test]
fn missing_field_uses_default() {
    let json = serde_json::json!({"name": "api"});
    let result = Config::from_flex_value(&json);
    println!("Missing → {:?}", result);
    assert!(result.is_ok());
    let (cfg, _) = result.unwrap();
    assert_eq!(cfg.retries, 0, "i64 default is 0");
}

#[test]
fn coerce_fails_does_default_rescue() {
    // "abc" can't coerce to i64. Does default kick in? Or does it error?
    let json = serde_json::json!({"name": "api", "retries": "abc"});
    let result = Config::from_flex_value(&json);
    println!("Coerce \"abc\" → {:?}", result);

    // Default rescues the failed coercion, producing retries = 0
    assert!(result.is_ok());
    let (cfg, diags) = result.unwrap();
    assert_eq!(cfg.retries, 0, "Default kicks in when coercion fails");

    // After fix: should emit ErrorDefaulted diagnostic (not silently swallow)
    println!("diagnostics: {:?}", diags);
    assert!(
        !diags.is_empty(),
        "Should emit a diagnostic when coercion fails and default is used"
    );
    assert!(
        diags
            .iter()
            .any(|d| matches!(d.kind, laminate::DiagnosticKind::ErrorDefaulted { .. })),
        "Should be ErrorDefaulted diagnostic"
    );
}

#[test]
fn null_with_coerce_default() {
    // null value + coerce + default — what happens?
    let json = serde_json::json!({"name": "api", "retries": null});
    let result = Config::from_flex_value(&json);
    println!("null → {:?}", result);

    // BestEffort: null → default (0) via Null→Default coercion
    assert!(result.is_ok());
    let (cfg, _) = result.unwrap();
    assert_eq!(cfg.retries, 0);
}
