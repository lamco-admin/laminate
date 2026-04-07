use laminate::coerce::CoercionLevel;
use laminate::value::PackCoercion;
/// Iteration 122: PackCoercion respects coercion level — BUG fixed
///
/// Pack coercion fired even in Exact mode, bypassing strict type checking.
/// Fixed: packs only fire at StringCoercion or BestEffort levels.
use laminate::FlexValue;

#[test]
fn exact_mode_blocks_pack_coercion() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact)
        .with_pack_coercion(PackCoercion::All);

    let result: Result<f64, _> = val.extract("price");
    assert!(result.is_err(), "Exact mode should block pack coercion");
}

#[test]
fn best_effort_allows_pack_coercion() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All);

    let result: f64 = val.extract("price").unwrap();
    assert!((result - 12.99).abs() < 0.01);
}
