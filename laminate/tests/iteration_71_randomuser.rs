//! Iteration 71 — Real Data: randomuser.me mixed nationalities
//!
//! Probe: Heterogeneous postcode types (int for AU/US, string for GB),
//! string-encoded coordinates, schema inference on mixed-type fields.

use laminate::schema::InferredSchema;
use laminate::FlexValue;

fn users() -> FlexValue {
    let json = include_str!("../testdata/randomuser_mixed.json");
    FlexValue::from_json(json).unwrap()
}

fn users_raw() -> Vec<serde_json::Value> {
    let json = include_str!("../testdata/randomuser_mixed.json");
    serde_json::from_str::<Vec<serde_json::Value>>(json).unwrap()
}

// --- Heterogeneous postcode field ---

#[test]
fn postcode_mixed_types_extract_as_string() {
    let fv = users();

    // AU user: postcode is integer 7190 — coerces to String
    let pc_au: String = fv.extract("[0].location.postcode").unwrap();
    assert_eq!(pc_au, "7190");

    // GB user: postcode is string "VO5 8EX" — passes through
    let pc_gb: String = fv.extract("[1].location.postcode").unwrap();
    assert!(
        pc_gb.contains(' '),
        "UK postcode should have space: {pc_gb}"
    );
}

// --- Schema inference on mixed-type postcode ---

#[test]
fn schema_detects_mixed_postcode_types() {
    let rows = users_raw();
    let locations: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|u| u.get("location").cloned())
        .collect();
    let schema = InferredSchema::from_values(&locations);

    let pc_field = &schema.fields["postcode"];
    // 3 Integer (AU/US) vs 2 String (GB) — Integer wins by count
    assert_eq!(
        pc_field.dominant_type,
        Some(laminate::schema::JsonType::Integer)
    );
    // Both types tracked in type_counts
    assert!(
        pc_field.type_counts.len() >= 2,
        "should have at least 2 types"
    );
}

// --- String coordinates → f64 coercion ---

#[test]
fn string_coordinates_coerce_to_f64() {
    let fv = users();

    // Coordinates are strings: "27.3731", "-134.8907"
    let lat: f64 = fv.extract("[0].location.coordinates.latitude").unwrap();
    assert!(lat > 27.0 && lat < 28.0);

    let lon: f64 = fv.extract("[0].location.coordinates.longitude").unwrap();
    assert!(lon < -134.0 && lon > -135.0);

    // Negative coordinates also coerce correctly
    let neg_lat: f64 = fv.extract("[2].location.coordinates.latitude").unwrap();
    assert!(neg_lat < 0.0, "negative latitude should coerce: {neg_lat}");
}

// --- Schema audit with type violation ---

#[test]
fn schema_audit_postcode_coercible_not_violated() {
    let rows = users_raw();
    let locations: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|u| u.get("location").cloned())
        .collect();
    let schema = InferredSchema::from_values(&locations);
    let report = schema.audit(&locations);

    // String postcodes ("VO5 8EX") cannot be coerced to Integer — they are
    // violations, not merely coercible. is_coercible_value now checks the actual
    // string value, not just the type pair.
    let pc_stats = &report.field_stats["postcode"];
    println!(
        "postcode stats: clean={}, coercible={}, violations={}",
        pc_stats.clean, pc_stats.coercible, pc_stats.violations
    );

    // Dominant type is Integer (3 vs 2) — Integer postcodes are clean
    assert_eq!(
        pc_stats.clean, 3,
        "integer postcodes should match dominant type"
    );
    // Non-numeric string postcodes cannot coerce to Integer → violations
    assert_eq!(
        pc_stats.coercible, 0,
        "non-numeric string postcodes are not coercible"
    );
    assert_eq!(
        pc_stats.violations, 2,
        "non-numeric string postcodes are hard violations"
    );
}
