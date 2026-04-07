//! Iteration 177: each_iter() preserves data_source after fix in iteration 176.

use laminate::value::SourceHint;
use laminate::FlexValue;

#[test]
fn each_iter_preserves_data_source() {
    // with_data_source requires an impl of CoercionDataSource.
    // The simplest test: set source hint (which affects coercion) and verify
    // elements still have the hint's coercion level (BestEffort from CSV).
    let data = FlexValue::from_json(r#"{"items": ["42", "true", "3.14"]}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let elements: Vec<FlexValue> = data.each("items");

    // CSV hint sets BestEffort coercion. If propagated, "42" → i64 works.
    let val: i64 = elements[0].extract_root::<i64>().unwrap();
    assert_eq!(val, 42);

    let val: bool = elements[1].extract_root::<bool>().unwrap();
    assert!(val);

    let val: f64 = elements[2].extract_root::<f64>().unwrap();
    assert!((val - 3.14).abs() < 0.001);
}
