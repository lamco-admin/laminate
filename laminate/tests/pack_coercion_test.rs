use laminate::value::PackCoercion;
/// Pack coercion integration — domain packs participate in extract() pipeline.
use laminate::FlexValue;

#[test]
fn currency_pack_coercion_in_extract() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_pack_coercion(PackCoercion::All);

    let price: f64 = val.extract("price").unwrap();
    assert!(
        (price - 12.99).abs() < 0.01,
        "pack coercion should strip currency symbol"
    );
}

#[test]
fn units_pack_coercion_in_extract() {
    let val = FlexValue::from_json(r#"{"weight": "5.2 kg"}"#)
        .unwrap()
        .with_pack_coercion(PackCoercion::All);

    let weight: f64 = val.extract("weight").unwrap();
    assert!(
        (weight - 5.2).abs() < 0.01,
        "pack coercion should strip unit suffix"
    );
}

#[test]
fn no_pack_coercion_by_default() {
    // Without pack coercion, "$12.99" should fail to extract as f64
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#).unwrap();
    let result: Result<f64, _> = val.extract("price");
    assert!(
        result.is_err(),
        "without pack coercion, currency string should fail"
    );
}

#[test]
fn pack_coercion_currency_only() {
    let val = FlexValue::from_json(r#"{"price": "$12.99", "weight": "5.2 kg"}"#)
        .unwrap()
        .with_pack_coercion(PackCoercion::Currency);

    // Currency should work
    let price: f64 = val.extract("price").unwrap();
    assert!((price - 12.99).abs() < 0.01);

    // Units should NOT work (only currency enabled)
    let result: Result<f64, _> = val.extract("weight");
    assert!(
        result.is_err(),
        "units coercion should not fire with Currency-only pack"
    );
}
