//! Iteration 236: E2E Currency parsing pipeline
//!
//! "$1,234.56" → PackCoercion::Currency → extract::<f64> → verify precision.
//!
//! Adversarial angles:
//! - US format with comma thousands: "$1,234.56"
//! - European format: "€1.234,56"
//! - Negative/accounting format: "($500.00)" and "-$500.00"
//! - Code suffix format: "12.99 USD"
//! - Precision: does the pipeline preserve cents accurately?

use laminate::value::PackCoercion;
use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn us_currency_symbol_prefix() {
    let val = FlexValue::from(serde_json::json!("$1,234.56"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    println!("$1,234.56 → {}", result);
    assert!(
        (result - 1234.56).abs() < 0.001,
        "should parse US currency: got {}",
        result
    );
}

#[test]
fn european_currency_symbol_prefix() {
    let val = FlexValue::from(serde_json::json!("€1.234,56"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    println!("€1.234,56 → {}", result);
    assert!(
        (result - 1234.56).abs() < 0.001,
        "should parse European currency: got {}",
        result
    );
}

#[test]
fn negative_accounting_format() {
    // Accounting negative: ($500.00) — parentheses indicate negative
    let val = FlexValue::from(serde_json::json!("($500.00)"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    println!("($500.00) → {}", result);
    assert!(
        (result - (-500.0)).abs() < 0.001,
        "accounting negative should be -500: got {}",
        result
    );
}

#[test]
fn negative_dash_format() {
    // Dash negative: -$500.00
    let val = FlexValue::from(serde_json::json!("-$500.00"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    println!("-$500.00 → {}", result);
    assert!(
        (result - (-500.0)).abs() < 0.001,
        "dash negative should be -500: got {}",
        result
    );
}

#[test]
fn code_suffix_format() {
    let val = FlexValue::from(serde_json::json!("12.99 USD"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    println!("12.99 USD → {}", result);
    assert!(
        (result - 12.99).abs() < 0.001,
        "code suffix should parse: got {}",
        result
    );
}

#[test]
fn currency_blocked_at_safewidening() {
    // PackCoercion::Currency should NOT fire at SafeWidening level
    // (needs StringCoercion or higher)
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::SafeWidening)
        .with_pack_coercion(PackCoercion::Currency);

    let result: Result<f64, _> = val.extract_root();
    println!("$12.99 at SafeWidening: {:?}", result);
    assert!(
        result.is_err(),
        "currency pack should NOT fire at SafeWidening"
    );
}

#[test]
fn currency_extract_as_string_preserves_original() {
    // When extracting as String, currency should NOT be stripped
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: String = val.extract_root().unwrap();
    assert_eq!(
        result, "$12.99",
        "String extraction should preserve original currency string"
    );
}

#[test]
fn currency_precision_cents() {
    // Verify that currency amounts preserve cent precision
    let amounts = vec!["$0.01", "$0.99", "$999.99", "$1,000,000.01"];
    let expected = vec![0.01, 0.99, 999.99, 1_000_000.01];

    for (s, exp) in amounts.iter().zip(expected.iter()) {
        let val = FlexValue::from(serde_json::json!(s))
            .with_coercion(CoercionLevel::BestEffort)
            .with_pack_coercion(PackCoercion::Currency);

        let result: f64 = val.extract_root().unwrap();
        assert!(
            (result - exp).abs() < 0.001,
            "precision test: {} should be {}, got {}",
            s,
            exp,
            result
        );
    }
}
