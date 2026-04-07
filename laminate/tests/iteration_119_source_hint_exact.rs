use laminate::coerce::CoercionLevel;
use laminate::value::SourceHint;
/// Iteration 119: SourceHint::Csv should NOT override explicit Exact coercion
///
/// Bug: with_coercion(Exact).with_source_hint(Csv) silently overrode Exact → BestEffort.
/// Fix: explicit with_coercion() takes precedence. Source hints only set
/// coercion if no explicit level was set.
use laminate::FlexValue;

#[test]
fn explicit_exact_survives_csv_hint() {
    let val = FlexValue::from_json(r#"{"port": "8080"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact)
        .with_source_hint(SourceHint::Csv);

    let result: Result<u16, _> = val.extract("port");
    assert!(
        result.is_err(),
        "explicit Exact should NOT be overridden by CSV hint"
    );
}

#[test]
fn csv_hint_without_explicit_coercion_works() {
    let val = FlexValue::from_json(r#"{"port": "8080"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let result: u16 = val.extract("port").unwrap();
    assert_eq!(
        result, 8080,
        "CSV hint without explicit coercion should enable BestEffort"
    );
}

#[test]
fn csv_then_exact_is_exact() {
    let val = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv)
        .with_coercion(CoercionLevel::Exact);

    let result: Result<i64, _> = val.extract("x");
    assert!(result.is_err(), "explicit Exact after CSV should be Exact");
}
