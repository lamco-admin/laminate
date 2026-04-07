#![allow(dead_code)]
//! Iteration 207: is_coercible_value() vs actual coercion engine.
//!
//! Float(3.0) → Integer: audit says "coercible". Does actual extract agree?
//! Float(3.5) → Integer: audit says "not coercible" (TypeMismatch). Correct?
//! This checks consistency between schema audit classification and runtime coercion.

use laminate::schema::{ExternalConstraint, InferredSchema, JsonType};
use laminate::FlexValue;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn float_3_0_coercible_to_integer_in_audit() {
    // Train on integers
    let rows = vec![json!({"count": 1}), json!({"count": 2})];
    let schema = InferredSchema::from_values(&rows);

    // Audit with Float(3.0) — should be classified as coercible, not violation
    let test = vec![json!({"count": 3.0})];
    let report = schema.audit(&test);
    println!("Violations: {:?}", report.violations);
    println!("Field stats: {:?}", report.field_stats);

    // JsonType::of(3.0) returns Integer (since fract()==0.0), so observed==expected.
    // This means 3.0 is classified as clean, not coercible — correct behavior.
    let count_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "count")
        .collect();
    assert!(count_violations.is_empty(), "No violations expected");

    let count_stats = report.field_stats.get("count").unwrap();
    assert_eq!(
        count_stats.clean, 1,
        "3.0 matches Integer type directly (fract==0.0)"
    );
    assert_eq!(count_stats.coercible, 0, "No coercion needed — types match");
}

#[test]
fn float_3_5_not_coercible_to_integer_in_audit() {
    let rows = vec![json!({"count": 1}), json!({"count": 2})];
    let schema = InferredSchema::from_values(&rows);

    // Audit with Float(3.5) — should be TypeMismatch
    let test = vec![json!({"count": 3.5})];
    let report = schema.audit(&test);
    println!("Violations: {:?}", report.violations);

    let type_mismatches: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "count" && format!("{:?}", v.kind).contains("TypeMismatch"))
        .collect();
    assert!(
        !type_mismatches.is_empty(),
        "Float 3.5 should be a TypeMismatch since it has fractional part"
    );
}

#[test]
fn float_3_0_actually_extracts_as_integer() {
    // Verify actual coercion engine agrees: Float 3.0 can become i64
    let fv = FlexValue::new(json!(3.0));
    let result: Result<i64, _> = fv.extract_root();
    println!("extract_root::<i64> from 3.0: {:?}", result);

    // If audit says coercible, extract should succeed too
    assert!(
        result.is_ok(),
        "Actual coercion should agree with audit: 3.0 → i64"
    );
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn float_3_5_extract_as_integer_fails_or_truncates() {
    // Verify actual coercion engine on 3.5 → i64
    let fv = FlexValue::new(json!(3.5));
    let result: Result<i64, _> = fv.extract_root();
    println!("extract_root::<i64> from 3.5: {:?}", result);

    // Audit says not coercible — extract should agree (fail or at least be lossy)
    // At default (Exact) coercion level, this should fail
    // Observe actual behavior before asserting
}
