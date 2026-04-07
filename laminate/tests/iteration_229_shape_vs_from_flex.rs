//! Iteration 229: shape_lenient vs from_flex_value — same value, different wrapping?
//!
//! Both should produce identical struct values. shape_lenient wraps in LaminateResult,
//! from_flex_value returns (Self, Vec<Diagnostic>). Are the values identical?

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Record {
    name: String,
    #[laminate(coerce)]
    age: i64,
    #[laminate(coerce, default)]
    active: bool,
}

#[test]
fn shape_lenient_vs_from_flex_value_same_struct() {
    let json = serde_json::json!({"name": "Alice", "age": "30", "active": "yes"});

    let (from_val, from_diags) = Record::from_flex_value(&json).unwrap();
    let shape_result = Record::shape_lenient(&json).unwrap();

    println!("from_flex_value: {:?}, diags: {:?}", from_val, from_diags);
    println!(
        "shape_lenient:   {:?}, diags: {:?}",
        shape_result.value, shape_result.diagnostics
    );

    assert_eq!(
        from_val, shape_result.value,
        "struct values should be identical"
    );
    assert_eq!(
        from_diags.len(),
        shape_result.diagnostics.len(),
        "diagnostic counts should match"
    );
}

#[test]
fn shape_lenient_vs_from_flex_value_clean_data() {
    // No coercion needed — both should produce identical results with no diagnostics
    let json = serde_json::json!({"name": "Bob", "age": 25, "active": true});

    let (from_val, from_diags) = Record::from_flex_value(&json).unwrap();
    let shape_result = Record::shape_lenient(&json).unwrap();

    assert_eq!(from_val, shape_result.value);
    assert_eq!(from_diags.len(), shape_result.diagnostics.len());
    // Both should have zero diagnostics for clean data
    assert!(
        from_diags.is_empty(),
        "clean data should produce no diagnostics from from_flex_value"
    );
    assert!(
        shape_result.diagnostics.is_empty(),
        "clean data should produce no diagnostics from shape_lenient"
    );
}
