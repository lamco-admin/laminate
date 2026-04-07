//! Iteration 283 — Coerce-FrenchThousands
//! French space+comma format "1 234,56" now parses to 1234.56

use laminate::FlexValue;

#[test]
fn french_space_comma_to_f64() {
    let fv = FlexValue::from_json(r#""1 234,56""#)
        .unwrap()
        .with_coercion(laminate::CoercionLevel::BestEffort);
    let result: f64 = fv.extract("").unwrap();
    assert!((result - 1234.56).abs() < 1e-10, "got {}", result);
}

#[test]
fn french_space_only_to_f64() {
    let fv = FlexValue::from_json(r#""1 234""#)
        .unwrap()
        .with_coercion(laminate::CoercionLevel::BestEffort);
    let result: f64 = fv.extract("").unwrap();
    assert!((result - 1234.0).abs() < 1e-10, "got {}", result);
}

#[test]
fn swiss_apostrophe_comma_decimal() {
    // Swiss with comma-decimal: "1'234,56" → 1234.56
    let fv = FlexValue::from_json(r#""1'234,56""#)
        .unwrap()
        .with_coercion(laminate::CoercionLevel::BestEffort);
    let result: f64 = fv.extract("").unwrap();
    assert!((result - 1234.56).abs() < 1e-10, "got {}", result);
}
