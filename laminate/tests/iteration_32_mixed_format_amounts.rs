//! Iteration 32: Format — mixed amount formats in financial fixture
//!
//! Verifies that extracting amounts as f64 from mixed-format financial data
//! handles each format correctly, and that European decimal formats ("24,51")
//! are NOT silently misinterpreted as US comma-thousands.

use laminate::{CoercionLevel, FlexValue};

#[test]
fn mixed_amount_formats_in_financial_fixture() {
    let json = include_str!("../testdata/financial/transactions.json");
    let fv = FlexValue::from_json(json)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);

    // TXN-001: "1250.00" → 1250.0 (string coercion)
    let amt: f64 = fv.extract("[0].amount").unwrap();
    assert_eq!(amt, 1250.0);

    // TXN-002: "-500.00" → -500.0 (negative string coercion)
    let amt: f64 = fv.extract("[1].amount").unwrap();
    assert_eq!(amt, -500.0);

    // TXN-003: "€2.450,75" → Error (European + currency symbol, needs currency pack)
    assert!(fv.extract::<f64>("[2].amount").is_err());

    // TXN-004: 99999 → 99999.0 (number→float widening)
    let amt: f64 = fv.extract("[3].amount").unwrap();
    assert_eq!(amt, 99999.0);

    // TXN-005: "10000" → 10000.0 (string coercion)
    let amt: f64 = fv.extract("[4].amount").unwrap();
    assert_eq!(amt, 10000.0);

    // TXN-006: "0.00000001" → 1e-8 (tiny decimal)
    let amt: f64 = fv.extract("[5].amount").unwrap();
    assert_eq!(amt, 1e-8);
}

#[test]
fn european_decimal_not_misread_as_us_thousands() {
    let fv = FlexValue::from_json(r#"{"v": "24,51"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);

    // "24,51" has only 2 digits after the comma — NOT valid US thousands.
    // Must NOT silently coerce to 2451.0 (which would be 100x data corruption).
    assert!(fv.extract::<f64>("v").is_err());
    assert!(fv.extract::<i64>("v").is_err());
}

#[test]
fn valid_us_thousands_still_work() {
    let fv = FlexValue::from_json(r#"{"v": "1,000"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);

    // "1,000" has exactly 3 digits after comma — valid US thousands
    let v: i64 = fv.extract("v").unwrap();
    assert_eq!(v, 1000);

    let fv = FlexValue::from_json(r#"{"v": "1,234,567"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let v: i64 = fv.extract("v").unwrap();
    assert_eq!(v, 1234567);

    // Negative with comma thousands
    let fv = FlexValue::from_json(r#"{"v": "-1,000"}"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let v: i64 = fv.extract("v").unwrap();
    assert_eq!(v, -1000);
}

#[test]
fn ambiguous_comma_formats_rejected() {
    let fv_fn = |s: &str| -> FlexValue {
        FlexValue::from_json(&format!(r#"{{"v": "{}"}}"#, s))
            .unwrap()
            .with_coercion(CoercionLevel::BestEffort)
    };

    // "1,00" — 2 digits after comma, not valid US thousands
    assert!(fv_fn("1,00").extract::<f64>("v").is_err());

    // "12,3456" — 4 digits after comma, not valid US thousands
    assert!(fv_fn("12,3456").extract::<f64>("v").is_err());

    // "1,23" — 2 digits, European-looking
    assert!(fv_fn("1,23").extract::<i64>("v").is_err());
}
