//! Iteration 185: Strict mode + SourceHint::Csv — explicit mode should win.

use laminate::value::SourceHint;
use laminate::FlexValue;
use laminate::Strict;

#[test]
fn strict_then_csv_hint_strict_wins() {
    // Set strict FIRST, then CSV hint. Strict sets coercion_explicit=true,
    // so CSV hint should NOT override to BestEffort.
    let val = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_mode::<Strict>()
        .with_source_hint(SourceHint::Csv);

    let result = val.extract::<i64>("x");
    assert!(result.is_err(), "Explicit Strict should win over CSV hint");
}

#[test]
fn csv_hint_then_strict_strict_wins() {
    // Set CSV hint FIRST (promotes to BestEffort), then Strict overrides.
    let val = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv)
        .with_mode::<Strict>();

    let result = val.extract::<i64>("x");
    assert!(
        result.is_err(),
        "Strict should override CSV hint regardless of order"
    );
}
