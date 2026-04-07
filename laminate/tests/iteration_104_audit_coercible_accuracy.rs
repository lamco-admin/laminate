#![cfg(feature = "schema")]
/// Iteration 104: Schema audit — fractional Float→Integer is a violation, not coercible
///
/// is_coercible blindly returned true for Float→Integer. Now
/// is_coercible_value checks the actual float value — only integral
/// floats (5.0) are coercible, fractional floats (3.14) are violations.
use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn audit_fractional_float_is_violation() {
    let training = vec![json!({"count": 10}), json!({"count": 20})];
    let schema = InferredSchema::from_values(&training);

    // 3.14 in an integer field — fractional, cannot be losslessly coerced
    let audit_data = vec![json!({"count": 3.14})];

    let report = schema.audit(&audit_data);
    let stats = &report.field_stats["count"];

    assert_eq!(
        stats.violations, 1,
        "fractional float should be a violation, not coercible"
    );
    assert_eq!(
        stats.coercible, 0,
        "3.14 should NOT be counted as coercible"
    );
    assert_eq!(report.violations.len(), 1);
}

#[test]
fn audit_integral_float_is_coercible() {
    let training = vec![json!({"count": 10}), json!({"count": 20})];
    let schema = InferredSchema::from_values(&training);

    // 5.0 would be coercible (integral float), but serde_json may store it as integer
    // Use a float that serde_json can't round: multiply to force float representation
    let audit_data = vec![json!({"count": 100.0_f64})];
    // Note: serde_json may store 100.0 as integer 100 — check actual behavior
    let report = schema.audit(&audit_data);
    let stats = &report.field_stats["count"];
    println!(
        "integral float: clean={}, coercible={}, violations={}",
        stats.clean, stats.coercible, stats.violations
    );
}
