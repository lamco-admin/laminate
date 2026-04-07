use laminate::value::SourceHint;
/// Source-awareness hint tests.
use laminate::FlexValue;

#[test]
fn csv_hint_enables_coercion() {
    let val = FlexValue::from_json(r#"{"port": "8080", "debug": "true"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let port: u16 = val.extract("port").unwrap();
    assert_eq!(port, 8080);

    let debug: bool = val.extract("debug").unwrap();
    assert_eq!(debug, true);
}

#[test]
fn csv_hint_enables_pack_coercion() {
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let price: f64 = val.extract("price").unwrap();
    assert!((price - 12.99).abs() < 0.01);
}

#[test]
fn json_hint_keeps_current_level() {
    use laminate::coerce::CoercionLevel;

    let val = FlexValue::from_json(r#"{"port": "8080"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact)
        .with_source_hint(SourceHint::Json);

    // JSON hint doesn't change Exact → still rejects string-to-int
    let result: Result<u16, _> = val.extract("port");
    assert!(result.is_err());
}

#[test]
fn env_hint_enables_coercion() {
    let val = FlexValue::from_json(r#"{"workers": "4"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Env);

    let workers: i32 = val.extract("workers").unwrap();
    assert_eq!(workers, 4);
}
