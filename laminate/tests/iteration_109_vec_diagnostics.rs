/// Iteration 109: Vec element coercion — all coerced elements in diagnostic
///
/// When multiple array elements are coerced, the diagnostic now lists all
/// affected indices instead of only reporting the first.
use laminate::coerce::CoercionLevel;
use laminate::FlexValue;

#[test]
fn vec_coercion_reports_all_elements() {
    let val =
        FlexValue::from(serde_json::json!([1, "2", "3"])).with_coercion(CoercionLevel::BestEffort);

    let (result, diagnostics): (Vec<i64>, _) = val.extract_root_with_diagnostics().unwrap();
    assert_eq!(result, vec![1, 2, 3]);

    // Should have 1 diagnostic that mentions both coerced elements
    assert_eq!(diagnostics.len(), 1);
    let suggestion = diagnostics[0].suggestion.as_deref().unwrap();
    assert!(
        suggestion.contains("[1]") && suggestion.contains("[2]"),
        "diagnostic should list all coerced element indices: {}",
        suggestion
    );
    assert!(
        suggestion.contains("2 elements"),
        "diagnostic should count coerced elements: {}",
        suggestion
    );
}

#[test]
fn vec_single_coercion_reports_element() {
    let val =
        FlexValue::from(serde_json::json!([1, "2", 3])).with_coercion(CoercionLevel::BestEffort);

    let (result, diagnostics): (Vec<i64>, _) = val.extract_root_with_diagnostics().unwrap();
    assert_eq!(result, vec![1, 2, 3]);

    // Single coercion uses the element-specific diagnostic
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].path.contains("[1]"));
}
