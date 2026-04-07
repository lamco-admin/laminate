//! Iteration 231: E2E CSV pipeline — CSV row as JSON strings → SourceHint::Csv → typed struct
//!
//! Simulates a CSV row where all values are strings. SourceHint::Csv should
//! upgrade coercion to StringCoercion, enabling string→number extraction.

use laminate::value::SourceHint;
use laminate::FlexValue;

#[test]
fn csv_row_all_strings_extract_typed() {
    // CSV row: all values are strings (as they arrive from a CSV parser)
    let row = serde_json::json!({
        "name": "Alice",
        "age": "30",
        "salary": "75000.50",
        "active": "true"
    });
    let val = FlexValue::from(row).with_source_hint(SourceHint::Csv);

    let name: String = val.extract("name").unwrap();
    let age: i64 = val.extract("age").unwrap();
    let salary: f64 = val.extract("salary").unwrap();
    let active: bool = val.extract("active").unwrap();

    assert_eq!(name, "Alice");
    assert_eq!(age, 30);
    assert!((salary - 75000.50).abs() < 0.01);
    assert_eq!(active, true);
}

#[test]
fn csv_row_with_diagnostics() {
    let row = serde_json::json!({"temp": "98.6", "unit": "F"});
    let val = FlexValue::from(row).with_source_hint(SourceHint::Csv);

    let temp: f64 = val.extract("temp").unwrap();
    assert!((temp - 98.6).abs() < 0.01);

    // Diagnostics should show coercion from string
    let diags = val.extract_with_diagnostics::<f64>("temp");
    println!("CSV extract diagnostics: {:?}", diags);
}

#[test]
fn csv_null_sentinel_in_csv_row() {
    // CSV often uses "N/A" or "" for missing values
    let row = serde_json::json!({"value": "N/A", "count": ""});
    let val = FlexValue::from(row).with_source_hint(SourceHint::Csv);

    // "N/A" should be treated as null sentinel
    let result: Result<i64, _> = val.extract("value");
    println!("N/A as i64 in CSV: {:?}", result);
    // With CSV hint (BestEffort), "N/A" → null → 0 (default for i64)
}
