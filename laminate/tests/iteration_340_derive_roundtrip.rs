//! Iteration 340: from_flex_value → to_value → from_flex_value — verify idempotent.

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Record {
    name: String,
    #[laminate(coerce)]
    age: i64,
    #[laminate(default)]
    active: bool,
    tags: Vec<String>,
}

#[test]
fn derive_roundtrip_idempotent() {
    let json = serde_json::json!({
        "name": "Alice",
        "age": 30,
        "active": true,
        "tags": ["admin", "user"]
    });

    // First extraction
    let (first, first_diags) = Record::from_flex_value(&json).unwrap();
    println!("first: {:?}, diags: {:?}", first, first_diags);

    // Serialize back to Value
    let value = first.to_value();
    println!(
        "serialized: {}",
        serde_json::to_string_pretty(&value).unwrap()
    );

    // Second extraction from serialized Value
    let (second, second_diags) = Record::from_flex_value(&value).unwrap();
    println!("second: {:?}, diags: {:?}", second, second_diags);

    // Should be identical
    assert_eq!(first, second, "round-trip should produce identical struct");
    // Second pass should have no coercion diagnostics (types already correct)
    assert!(
        second_diags.is_empty(),
        "second pass should have no diagnostics: {:?}",
        second_diags
    );
}
