#![allow(dead_code)]
//! Iteration 190: Full mode pipeline E2E test.
//!
//! from_json → with_mode → each_iter → extract per element.
//! Question: Is the coercion level consistent across the entire pipeline?
//! Does each_iter inherit the mode's coercion level?

use laminate::{FlexValue, Lenient, Strict};

#[test]
fn e2e_lenient_pipeline() {
    let json = r#"{"items": [{"price": "9.99"}, {"price": "24.50"}, {"price": 100}]}"#;
    let val = FlexValue::from_json(json).unwrap().with_mode::<Lenient>();

    let mut prices: Vec<f64> = Vec::new();
    for item in val.each_iter("items") {
        let price: f64 = item.extract("price").unwrap();
        prices.push(price);
    }

    println!("Lenient prices: {prices:?}");
    assert_eq!(prices.len(), 3);
    assert!((prices[0] - 9.99).abs() < f64::EPSILON);
    assert!((prices[1] - 24.50).abs() < f64::EPSILON);
    assert!((prices[2] - 100.0).abs() < f64::EPSILON);
}

#[test]
fn e2e_strict_pipeline_rejects_string_prices() {
    let json = r#"{"items": [{"price": "9.99"}, {"price": 100}]}"#;
    let val = FlexValue::from_json(json).unwrap().with_mode::<Strict>();

    let mut results: Vec<Result<f64, _>> = Vec::new();
    for item in val.each_iter("items") {
        results.push(item.extract::<f64>("price"));
    }

    println!("Strict results: {results:?}");
    // First item "9.99" should fail in Strict (string, not number)
    assert!(results[0].is_err(), "Strict should reject string→f64");
    // Second item 100 should succeed (int→f64 in Exact mode — wait, Exact rejects int→f64!)
    println!("Second result: {:?}", results[1]);
}
