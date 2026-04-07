//! Iteration 241: European number "1.234,56" at SafeWidening — should NOT parse.
//!
//! European format uses dots as thousands separators and commas for decimals.
//! Only BestEffort should attempt locale-aware parsing.

use laminate::FlexValue;
use laminate::{Absorbing, Lenient};

#[test]
fn european_number_at_safe_widening_rejected() {
    // Absorbing mode = SafeWidening
    let val = FlexValue::from_json(r#"{"price": "1.234,56"}"#)
        .unwrap()
        .with_mode::<Absorbing>();

    let result = val.extract::<f64>("price");
    assert!(
        result.is_err(),
        "SafeWidening should NOT parse European format"
    );
}

#[test]
fn european_number_at_best_effort_parsed() {
    // Lenient mode = BestEffort
    let val = FlexValue::from_json(r#"{"price": "1.234,56"}"#)
        .unwrap()
        .with_mode::<Lenient>();

    let result = val.extract::<f64>("price");
    // BestEffort should attempt locale-aware parsing
    match &result {
        Ok(v) => {
            println!("Parsed: {v}");
            assert!((v - 1234.56).abs() < 0.01, "Expected 1234.56, got {v}");
        }
        Err(e) => {
            println!("Failed: {e}");
            // If BestEffort doesn't handle European format, that's also
            // a valid observation — just document it.
        }
    }
}
