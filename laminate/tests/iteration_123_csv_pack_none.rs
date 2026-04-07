use laminate::value::{PackCoercion, SourceHint};
/// Iteration 123: explicit PackCoercion::None survives CSV hint — BUG fixed
///
/// Same class as iteration 119: source hint overrode explicit user setting.
/// Now pack_coercion_explicit flag prevents hints from overriding.
use laminate::FlexValue;

#[test]
fn explicit_none_survives_csv_hint() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_pack_coercion(PackCoercion::None)
        .with_source_hint(SourceHint::Csv);

    let result: Result<f64, _> = val.extract("price");
    assert!(
        result.is_err(),
        "explicit PackCoercion::None should survive CSV hint"
    );
}

#[test]
fn csv_hint_without_explicit_pack_enables_all() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let result: f64 = val.extract("price").unwrap();
    assert!(
        (result - 12.99).abs() < 0.01,
        "CSV hint should enable pack coercion by default"
    );
}
