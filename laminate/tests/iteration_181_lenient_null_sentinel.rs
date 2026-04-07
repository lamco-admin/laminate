//! Iteration 181: Lenient mode + null sentinel "N/A" → extract::<i64>
//!
//! BestEffort should recognize "N/A" as a null sentinel, then handle
//! the null→i64 conversion (either default 0 or error).

use laminate::FlexValue;
use laminate::Lenient;

#[test]
fn lenient_null_sentinel_to_i64() {
    let val = FlexValue::from_json(r#"{"x": "N/A"}"#)
        .unwrap()
        .with_mode::<Lenient>();

    let result = val.extract::<i64>("x");
    // Observe: does BestEffort parse "N/A" as null sentinel → default 0?
    // Or does it fail trying to parse "N/A" as an integer?
    match &result {
        Ok(v) => println!("OK: {v}"),
        Err(e) => println!("ERR: {e}"),
    }
    // "N/A" is a recognized null sentinel in BestEffort mode.
    // null → i64 should give 0 (default) in BestEffort.
    assert!(
        result.is_ok(),
        "BestEffort should handle N/A → null → 0: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn lenient_null_sentinel_to_string() {
    let val = FlexValue::from_json(r#"{"x": "N/A"}"#)
        .unwrap()
        .with_mode::<Lenient>();

    // When extracting as String, "N/A" should stay as-is (it's already a string)
    let result: String = val.extract("x").unwrap();
    assert_eq!(result, "N/A");
}
