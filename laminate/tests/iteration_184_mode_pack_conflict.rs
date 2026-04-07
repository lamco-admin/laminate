//! Iteration 184: with_mode::<Lenient>() + with_pack_coercion(None)
//!
//! Mode sets BestEffort coercion. PackCoercion::None explicitly disables packs.
//! The explicit None should win — "$12.99" should NOT parse as f64.

use laminate::value::PackCoercion;
use laminate::FlexValue;
use laminate::Lenient;

#[test]
fn lenient_mode_with_explicit_no_packs() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_mode::<Lenient>()
        .with_pack_coercion(PackCoercion::None);

    // Even though BestEffort enables lots of coercion, explicit PackCoercion::None
    // should prevent currency parsing
    let result = val.extract::<f64>("price");
    assert!(
        result.is_err(),
        "Explicit PackCoercion::None should block currency parsing"
    );
}

#[test]
fn lenient_mode_with_explicit_all_packs() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_mode::<Lenient>()
        .with_pack_coercion(PackCoercion::All);

    // With All packs enabled, currency parsing should work
    let price: f64 = val.extract("price").unwrap();
    assert!((price - 12.99).abs() < 0.01);
}
