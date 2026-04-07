//! Iteration 328: FlexValue::clone() preserves coercion level, source hint, pack coercion.

use laminate::mode::Strict;
use laminate::value::{PackCoercion, SourceHint};
use laminate::FlexValue;

#[test]
fn clone_preserves_strict_mode() {
    let fv = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_mode::<Strict>();
    let cloned = fv.clone();

    let original_result: Result<i64, _> = fv.extract("x");
    let cloned_result: Result<i64, _> = cloned.extract("x");

    println!("original extract: {:?}", original_result);
    println!("cloned extract: {:?}", cloned_result);

    assert_eq!(
        original_result.is_ok(),
        cloned_result.is_ok(),
        "clone should preserve coercion behavior"
    );
}

#[test]
fn clone_preserves_source_hint() {
    let fv = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Csv);
    let cloned = fv.clone();

    let original_result: Result<i64, _> = fv.extract("x");
    let cloned_result: Result<i64, _> = cloned.extract("x");

    assert_eq!(original_result.is_ok(), cloned_result.is_ok());
    if let (Ok(a), Ok(b)) = (original_result, cloned_result) {
        assert_eq!(a, b);
    }
}

#[test]
fn clone_preserves_pack_coercion() {
    let fv = FlexValue::from_json(r#"{"x": "42"}"#)
        .unwrap()
        .with_pack_coercion(PackCoercion::All);
    let cloned = fv.clone();

    let original_result: Result<i64, _> = fv.extract("x");
    let cloned_result: Result<i64, _> = cloned.extract("x");

    assert_eq!(original_result.is_ok(), cloned_result.is_ok());
}
