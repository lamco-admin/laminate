//! Iteration 247: Integer 2^53+1 → f64 at SafeWidening — precision loss.
//!
//! 2^53+1 = 9007199254740993 can't be exactly represented as f64.
//! SafeWidening should either reject this or warn about precision loss.

use laminate::Absorbing;
use laminate::FlexValue;

#[test]
fn large_int_to_float_precision_loss() {
    // 2^53 + 1 = 9007199254740993 — NOT exactly representable in f64
    let val = FlexValue::from_json(r#"{"big": 9007199254740993}"#)
        .unwrap()
        .with_mode::<Absorbing>(); // SafeWidening

    let result = val.extract::<f64>("big");
    match &result {
        Ok(v) => println!("OK: {v} (expected 9007199254740993)"),
        Err(e) => println!("ERR: {e}"),
    }

    // SafeWidening should be cautious about precision loss.
    // If it allows this conversion, verify if there's a diagnostic warning.
}

#[test]
fn safe_int_to_float_within_precision() {
    // 2^53 = 9007199254740992 — exactly representable in f64
    let val = FlexValue::from_json(r#"{"safe": 9007199254740992}"#)
        .unwrap()
        .with_mode::<Absorbing>();

    // This should be fine — no precision loss
    let result = val.extract::<f64>("safe");
    // Note: serde_json may already have stored this as f64 internally
    match &result {
        Ok(v) => println!("OK: {v}"),
        Err(e) => println!("ERR: {e}"),
    }
}
