//! Iteration 176: each_iter() drops pack_coercion — elements lose PackCoercion::All
//!
//! When you set with_source_hint(Csv) (which enables PackCoercion::All),
//! then iterate with each(), do child elements retain pack coercion?

use laminate::value::SourceHint;
use laminate::FlexValue;

#[test]
fn each_iter_preserves_pack_coercion_from_source_hint() {
    let data = FlexValue::from_json(r#"{"items": ["$12.99", "€24.50"]}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let elements: Vec<FlexValue> = data.each("items");

    // Each element should still have pack coercion enabled from the CSV hint.
    // Try extracting currency values as f64 — this only works if pack coercion is active.
    let price: f64 = elements[0].extract_root::<f64>().unwrap();
    assert!((price - 12.99).abs() < 0.01, "Expected 12.99, got {price}");

    let price2: f64 = elements[1].extract_root::<f64>().unwrap();
    assert!(
        (price2 - 24.50).abs() < 0.01,
        "Expected 24.50, got {price2}"
    );
}
