//! Iteration 246: Single-element array [42] → extract::<i64> at BestEffort.
//!
//! Some coercion engines unwrap single-element arrays automatically.
//! Does laminate's BestEffort do this?

use laminate::FlexValue;
use laminate::Lenient;

#[test]
fn single_element_array_to_scalar() {
    let val = FlexValue::from_json(r#"{"x": [42]}"#)
        .unwrap()
        .with_mode::<Lenient>();

    let result = val.extract::<i64>("x");
    match &result {
        Ok(v) => println!("OK: {v}"),
        Err(e) => println!("ERR: {e}"),
    }

    // BestEffort unwraps [42] → 42 automatically.
    assert!(
        result.is_ok(),
        "BestEffort should unwrap single-element array"
    );
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn array_index_still_works() {
    // Regardless of unwrap behavior, explicit indexing should always work
    let val = FlexValue::from_json(r#"{"x": [42]}"#)
        .unwrap()
        .with_mode::<Lenient>();

    let result: i64 = val.extract("x[0]").unwrap();
    assert_eq!(result, 42);
}
