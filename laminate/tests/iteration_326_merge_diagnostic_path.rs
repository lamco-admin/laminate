//! Iteration 326: merge_with_diagnostics deeply nested — verify diagnostic paths.

use laminate::FlexValue;

#[test]
fn merge_diagnostic_paths_deeply_nested() {
    let base = FlexValue::from_json(
        r#"{
        "a": {
            "b": {
                "c": {
                    "value": 1
                }
            }
        }
    }"#,
    )
    .unwrap();

    let overlay = FlexValue::from_json(
        r#"{
        "a": {
            "b": {
                "c": {
                    "value": 2,
                    "new_field": "hello"
                }
            }
        }
    }"#,
    )
    .unwrap();

    let (merged, diagnostics) = base.merge_with_diagnostics(&overlay);

    println!("Diagnostics:");
    for d in &diagnostics {
        println!("  path={}, kind={:?}", d.path, d.kind);
    }

    // Should have diagnostic for value change at "a.b.c.value"
    let value_diag = diagnostics.iter().find(|d| d.path == "a.b.c.value");
    assert!(
        value_diag.is_some(),
        "should have diagnostic at a.b.c.value, got: {:?}",
        diagnostics.iter().map(|d| &d.path).collect::<Vec<_>>()
    );

    // Should have diagnostic for new field at "a.b.c.new_field"
    let new_field_diag = diagnostics.iter().find(|d| d.path == "a.b.c.new_field");
    assert!(
        new_field_diag.is_some(),
        "should have diagnostic at a.b.c.new_field"
    );

    // Verify merged value
    let val: i64 = merged.extract("a.b.c.value").unwrap();
    assert_eq!(val, 2, "overlay value should win");
}
