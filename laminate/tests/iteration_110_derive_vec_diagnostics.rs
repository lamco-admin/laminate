/// Iteration 110: Derive Vec<i64> element diagnostics — same fix as iter 109
///
/// The string-based coerce_value path (used by derive macro) had the same
/// "first diagnostic only" bug as the trait-based path fixed in iter 109.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Data {
    #[laminate(coerce)]
    values: Vec<i64>,
}

#[test]
fn derive_vec_reports_all_coerced_elements() {
    let json = r#"{"values": [1, "2", "3", "4"]}"#;
    let (data, diagnostics) = Data::from_json(json).unwrap();

    assert_eq!(data.values, vec![1, 2, 3, 4]);

    // Find diagnostics related to the values field
    let value_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.path.contains("values"))
        .collect();

    println!("value diagnostics: {:?}", value_diags);

    // Should mention all 3 coerced elements, not just the first
    assert!(!value_diags.is_empty(), "should have coercion diagnostics");

    // Check that all coerced indices are reported
    let all_text = format!("{:?}", value_diags);
    assert!(
        all_text.contains("[1]") && all_text.contains("[2]") && all_text.contains("[3]"),
        "should report all 3 coerced elements: {}",
        all_text
    );
}
