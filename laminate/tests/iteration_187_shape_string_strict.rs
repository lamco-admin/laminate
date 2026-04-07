//! Iteration 187: shape::<String, Strict>() on integer value
//!
//! Strict maps to Exact coercion. Number→String requires StringCoercion.
//! Does shape reject this? If so, what's the error/diagnostic quality?

use laminate::mode::Strict;
use laminate::FlexValue;

#[test]
fn shape_string_strict_on_integer() {
    let val = FlexValue::from_json(r#"{"count": 42}"#).unwrap();

    // shape::<String, Strict> on an integer field
    let result = val.shape::<String, Strict>("count");
    println!("shape::<String, Strict>(\"count\"): {:?}", result);

    // Strict rejects: int→String requires StringCoercion
    assert!(result.is_err(), "Strict should reject int→String");

    // Control: Lenient (BestEffort) should coerce 42 → "42"
    let result_lenient = val.shape::<String, laminate::mode::Lenient>("count");
    println!("shape::<String, Lenient>(\"count\"): {:?}", result_lenient);
    assert!(result_lenient.is_ok(), "Lenient should coerce int→String");
    let (value, _diagnostics) = result_lenient.unwrap();
    assert_eq!(value, "42");
}
