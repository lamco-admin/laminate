#![allow(dead_code)]
//! Iteration 188: shape::<i64, Lenient>() with diagnostics.
//!
//! Question: When a string is coerced to i64 via Lenient mode shape(),
//! does the diagnostics vec contain the coercion detail?
//! And what does the diagnostic look like — does it mention the path?

use laminate::{Diagnostic, FlexValue, Lenient};

#[test]
fn shape_lenient_string_to_i64_produces_diagnostic() {
    let val = FlexValue::from_json(r#"{"age": "30"}"#).unwrap();
    let (result, diagnostics): (i64, Vec<Diagnostic>) = val.shape::<i64, Lenient>("age").unwrap();

    println!("Result: {result}");
    println!("Diagnostics count: {}", diagnostics.len());
    for d in &diagnostics {
        println!("  {d:?}");
    }

    // The coercion "30" → 30 should produce a diagnostic
    assert_eq!(result, 30);
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic for string→i64 coercion"
    );
}

#[test]
fn shape_lenient_native_type_no_diagnostic() {
    // i64 extracting from a JSON integer — no coercion needed
    let val = FlexValue::from_json(r#"{"age": 30}"#).unwrap();
    let (result, diagnostics): (i64, Vec<Diagnostic>) = val.shape::<i64, Lenient>("age").unwrap();

    println!("Result: {result}");
    println!("Diagnostics count: {}", diagnostics.len());

    assert_eq!(result, 30);
    // No coercion needed — diagnostics should be empty
    assert!(
        diagnostics.is_empty(),
        "No diagnostic expected for native i64 extraction"
    );
}

#[test]
fn shape_lenient_diagnostic_contains_path() {
    let val = FlexValue::from_json(r#"{"user": {"age": "25"}}"#).unwrap();
    let (result, diagnostics): (i64, Vec<Diagnostic>) =
        val.shape::<i64, Lenient>("user.age").unwrap();

    println!("Result: {result}");
    for d in &diagnostics {
        println!("  path='{}' kind={:?}", d.path, d.kind);
    }

    assert_eq!(result, 25);
    // The diagnostic should reference the nested path
    if !diagnostics.is_empty() {
        let d = &diagnostics[0];
        assert!(
            d.path.contains("age"),
            "Diagnostic path should reference 'age', got: '{}'",
            d.path
        );
    }
}
