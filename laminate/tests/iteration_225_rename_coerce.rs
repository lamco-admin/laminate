/// Iteration 225: #[laminate(coerce, rename = "X")] — renamed + coerced
///
/// Target #225: When coerce and rename are combined, does coercion look up
/// the renamed key? And do diagnostics report the renamed path?
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Measurement {
    #[laminate(rename = "sampleId", coerce)]
    sample_id: i64,
    #[laminate(rename = "readingValue", coerce)]
    reading_value: f64,
    unit: String,
}

#[test]
fn rename_coerce_reads_correct_key() {
    // JSON uses camelCase, values are strings that need coercion
    let json = r#"{"sampleId": "42", "readingValue": "98.6", "unit": "F"}"#;
    let (m, diagnostics) = Measurement::from_json(json).unwrap();

    println!("m = {:?}", m);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(m.sample_id, 42);
    assert!((m.reading_value - 98.6).abs() < 0.01);
    assert_eq!(m.unit, "F");

    // Diagnostics should reference the RENAMED key, not the Rust field name
    assert!(diagnostics.len() >= 2, "should have coercion diagnostics");
    let paths: Vec<&str> = diagnostics.iter().map(|d| d.path.as_str()).collect();
    println!("diagnostic paths = {:?}", paths);
    assert!(
        paths.contains(&"sampleId"),
        "should use renamed key in diagnostic path"
    );
    assert!(
        paths.contains(&"readingValue"),
        "should use renamed key in diagnostic path"
    );
    assert!(
        !paths.contains(&"sample_id"),
        "should NOT use Rust field name"
    );
}

#[test]
fn rename_coerce_rust_name_is_unknown() {
    // Using Rust field names — should fail because renamed keys are required
    let json = r#"{"sample_id": "42", "reading_value": "98.6", "unit": "F"}"#;
    let result = Measurement::from_json(json);
    assert!(
        result.is_err(),
        "Rust field names should not work with rename"
    );
}
