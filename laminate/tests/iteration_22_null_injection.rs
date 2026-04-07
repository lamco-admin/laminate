//! Iteration 22: Null Injection — set items[0].unit_price to null in ecommerce order
//! PASS: Null in array element correctly coerces to 0.0 (f64), None (Option<f64>),
//! and None (maybe), with appropriate Defaulted diagnostic.

use laminate::FlexValue;

#[test]
fn iter22_null_in_array_element_coerces_to_default() {
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("../testdata/ecommerce/order.json")).unwrap();
    json["items"][0]["unit_price"] = serde_json::Value::Null;

    let flex = FlexValue::new(json);

    // Bare f64: null → 0.0 with diagnostic
    let (val, diags) = flex
        .extract_with_diagnostics::<f64>("items[0].unit_price")
        .unwrap();
    assert_eq!(val, 0.0);
    assert_eq!(diags.len(), 1);
    assert!(matches!(
        diags[0].kind,
        laminate::DiagnosticKind::Defaulted { .. }
    ));

    // Option<f64>: null → None (no coercion, serde handles it)
    let opt: Option<f64> = flex.extract("items[0].unit_price").unwrap();
    assert_eq!(opt, None);

    // maybe(): null → None
    let maybe: Option<f64> = flex.maybe("items[0].unit_price").unwrap();
    assert_eq!(maybe, None);

    // Non-null items still work (string coercion + number pass-through)
    let price1: f64 = flex.extract("items[1].unit_price").unwrap();
    assert_eq!(price1, 99.0);
    let price2: f64 = flex.extract("items[2].unit_price").unwrap();
    assert_eq!(price2, 0.12);
}
