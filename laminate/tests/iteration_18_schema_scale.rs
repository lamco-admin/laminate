//! Iteration 18: Scale — Schema inference on Countries multi.json (PASS)
//! 5 countries with 50+ fields each. Inference handles wide data, tracks
//! absent fields correctly, classifies nested objects/arrays properly.

use laminate::schema::{InferredSchema, JsonType};

fn load_countries_rows() -> Vec<serde_json::Value> {
    let full = format!(
        "{}/testdata/countries/multi.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&full).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn iter18_wide_schema_field_count() {
    let rows = load_countries_rows();
    let schema = InferredSchema::from_values(&rows);
    assert_eq!(schema.total_records, 5);
    // Countries data has 35 distinct top-level fields across 5 objects
    assert!(
        schema.fields.len() >= 30,
        "expected 30+ fields, got {}",
        schema.fields.len()
    );
}

#[test]
fn iter18_absent_field_tracking() {
    // Japan has no `borders` field — should be absent=1
    let rows = load_countries_rows();
    let schema = InferredSchema::from_values(&rows);
    let borders = schema.fields.get("borders").expect("borders should exist");
    assert_eq!(borders.present_count, 4);
    assert_eq!(borders.absent_count, 1);
    assert_eq!(borders.dominant_type, Some(JsonType::Array));
}

#[test]
fn iter18_nested_objects_classified() {
    let rows = load_countries_rows();
    let schema = InferredSchema::from_values(&rows);
    // Complex nested fields should all be Object type
    for field_name in &["name", "currencies", "translations", "demonyms", "flags"] {
        let field = schema
            .fields
            .get(*field_name)
            .unwrap_or_else(|| panic!("{field_name} should exist"));
        assert_eq!(
            field.dominant_type,
            Some(JsonType::Object),
            "{field_name} should be Object, got {:?}",
            field.dominant_type
        );
        assert_eq!(
            field.present_count, 5,
            "{field_name} should be present in all 5"
        );
    }
}

#[test]
fn iter18_array_fields_classified() {
    let rows = load_countries_rows();
    let schema = InferredSchema::from_values(&rows);
    for field_name in &["tld", "altSpellings", "timezones", "continents", "capital"] {
        let field = schema
            .fields
            .get(*field_name)
            .unwrap_or_else(|| panic!("{field_name} should exist"));
        assert_eq!(
            field.dominant_type,
            Some(JsonType::Array),
            "{field_name} should be Array, got {:?}",
            field.dominant_type
        );
    }
}
