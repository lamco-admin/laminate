//! Iteration 183: DynamicMode round-trip from string → coercion → extract

use laminate::DynamicMode;
use laminate::FlexValue;

#[test]
fn dynamic_mode_string_roundtrip() {
    // Simulate reading mode from config: "lenient" → parse → apply
    let mode_str = "lenient";
    let mode: DynamicMode = mode_str.parse().unwrap();

    let val = FlexValue::from_json(r#"{"count": "42"}"#)
        .unwrap()
        .with_dynamic_mode(mode);

    // Lenient = BestEffort, so string→int coercion should work
    let count: i64 = val.extract("count").unwrap();
    assert_eq!(count, 42);

    // Now strict from config
    let mode: DynamicMode = "strict".parse().unwrap();
    let val = FlexValue::from_json(r#"{"count": "42"}"#)
        .unwrap()
        .with_dynamic_mode(mode);

    // Strict = Exact, should reject string→int
    let result = val.extract::<i64>("count");
    assert!(result.is_err(), "Strict from string should reject coercion");
}

#[test]
fn dynamic_mode_display_roundtrip() {
    // DynamicMode::Display → str → parse should round-trip
    for mode in [
        DynamicMode::Lenient,
        DynamicMode::Absorbing,
        DynamicMode::Strict,
    ] {
        let s = mode.to_string();
        let parsed: DynamicMode = s.parse().unwrap();
        assert_eq!(mode, parsed, "Round-trip failed for {mode:?}");
    }
}
