//! Iteration 329: with_mode::<Strict>().with_source_hint(Csv) — does CSV override Strict?

use laminate::mode::Strict;
use laminate::value::SourceHint;
use laminate::FlexValue;

#[test]
fn strict_then_csv_hint() {
    // Strict sets Exact coercion. CSV hint sets StringCoercion.
    // Question: which wins when chained?
    let fv = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_mode::<Strict>()
        .with_source_hint(SourceHint::Csv);

    let result: Result<i64, _> = fv.extract("x");
    println!("Strict + CSV: {:?}", result);

    // If Strict's Exact wins: "42" → i64 should fail (no string coercion)
    // If CSV's StringCoercion wins: "42" → 42 should succeed
    // Observe actual behavior:
    match &result {
        Ok(v) => println!("CSV hint overrode Strict: got {}", v),
        Err(e) => println!("Strict won over CSV hint: {:?}", e),
    }
}

#[test]
fn csv_then_strict() {
    // Reverse order: CSV first, then Strict
    let fv = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv)
        .with_mode::<Strict>();

    let result: Result<i64, _> = fv.extract("x");
    println!("CSV + Strict: {:?}", result);
}

#[test]
fn mode_explicit_overrides_hint() {
    // The key question: explicit mode should win over hint
    let fv_strict = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_mode::<Strict>()
        .with_source_hint(SourceHint::Csv);

    let fv_csv_only = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);

    let strict_result: Result<i64, _> = fv_strict.extract("x");
    let csv_result: Result<i64, _> = fv_csv_only.extract("x");

    println!("Strict+CSV: {:?}", strict_result);
    println!("CSV only: {:?}", csv_result);

    // CSV only should succeed (StringCoercion)
    assert!(csv_result.is_ok(), "CSV-only should coerce string→i64");
}
