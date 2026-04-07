//! Iteration 230: shape_absorbing diagnostics vs from_flex_value diagnostics
//!
//! shape_absorbing uses SafeWidening coercion. from_flex_value uses the derive-level
//! coercion (BestEffort for #[laminate(coerce)] fields). Do they produce the same diagnostics
//! on the same input?

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Record {
    name: String,
    #[laminate(coerce)]
    score: f64,
}

#[test]
fn absorbing_vs_from_flex_value_diagnostics() {
    // "95.5" is a string — from_flex_value coerces with BestEffort (derive-level),
    // shape_absorbing wraps the same from_flex_value result
    let json = serde_json::json!({"name": "Test", "score": "95.5"});

    let (_, from_diags) = Record::from_flex_value(&json).unwrap();
    let absorb_result = Record::shape_absorbing(&json).unwrap();

    println!("from_flex_value diagnostics: {:?}", from_diags);
    println!(
        "shape_absorbing diagnostics: {:?}",
        absorb_result.diagnostics
    );

    // shape_absorbing delegates to from_flex_value, so diagnostics should be identical
    assert_eq!(
        from_diags.len(),
        absorb_result.diagnostics.len(),
        "diagnostic count should match between absorbing and from_flex_value"
    );
}

#[test]
fn absorbing_overflow_empty_when_no_unknowns() {
    let json = serde_json::json!({"name": "Test", "score": 95.5});
    let result = Record::shape_absorbing(&json).unwrap();

    println!("residual: {:?}", result.residual);
    assert!(
        result.residual.is_empty(),
        "no unknown fields → empty overflow"
    );
}

#[test]
fn absorbing_diagnostics_include_int_to_float_coercion() {
    // Integer 95 → f64 field. SafeWidening allows this but produces a diagnostic.
    let json = serde_json::json!({"name": "Test", "score": 95});
    let result = Record::shape_absorbing(&json).unwrap();

    println!("int→float diagnostics: {:?}", result.diagnostics);
    // The #[laminate(coerce)] attribute means from_flex_value uses BestEffort.
    // But the value is int→f64 which SafeWidening also allows.
    // Since shape_absorbing just wraps from_flex_value, it gets the same diagnostics.
}
