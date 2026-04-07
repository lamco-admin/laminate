use laminate::value::{PackCoercion, SourceHint};
use laminate::CoercionLevel;
/// Iteration 129: SourceHint::Json with BestEffort coercion
///
/// Target #97: JSON hint should NOT change coercion level.
/// Specifically: explicit BestEffort + Json hint should keep BestEffort,
/// and pack coercion should still work if explicitly enabled.
use laminate::FlexValue;

#[test]
fn json_hint_preserves_explicit_besteffort() {
    // Set BestEffort explicitly, then apply Json hint
    let val = FlexValue::from_json(r#"{"temp": "72"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort)
        .with_source_hint(SourceHint::Json);

    // String "72" should coerce to f64 under BestEffort
    let result: f64 = val.extract("temp").unwrap();
    assert!(
        (result - 72.0).abs() < 0.01,
        "BestEffort should allow string→f64 coercion"
    );
}

#[test]
fn json_hint_does_not_enable_packs() {
    // Json hint alone should NOT enable pack coercion
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Json);

    let result: Result<f64, _> = val.extract("price");
    assert!(
        result.is_err(),
        "Json hint alone should NOT enable pack coercion — $12.99 should fail as f64"
    );
}

#[test]
fn json_hint_with_explicit_packs_works() {
    // Json hint + explicit pack coercion should allow currency stripping
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All)
        .with_source_hint(SourceHint::Json);

    let result: f64 = val.extract("price").unwrap();
    assert!(
        (result - 12.99).abs() < 0.01,
        "Explicit BestEffort + PackCoercion::All + Json hint should extract currency"
    );
}

#[test]
fn json_hint_preserves_exact_coercion() {
    // Explicit Exact + Json hint should keep Exact (no coercion)
    let val = FlexValue::from_json(r#"{"temp": "72"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact)
        .with_source_hint(SourceHint::Json);

    let result: Result<f64, _> = val.extract("temp");
    assert!(
        result.is_err(),
        "Exact coercion + Json hint should NOT coerce string→f64"
    );
}
