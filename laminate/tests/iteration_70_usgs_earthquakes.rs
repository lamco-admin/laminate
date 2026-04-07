//! Iteration 70 — Real Data: USGS Earthquake GeoJSON
//!
//! Probe: Deeply nested GeoJSON, 15-digit precision coordinates,
//! millisecond epoch timestamps, 5 null fields per record,
//! integer used as boolean (tsunami: 0).

use laminate::schema::{InferredSchema, JsonType};
use laminate::FlexValue;

fn quakes() -> FlexValue {
    let json = include_str!("../testdata/usgs_earthquakes.json");
    FlexValue::from_json(json).unwrap()
}

// --- Deep path navigation into GeoJSON ---

#[test]
fn navigate_to_coordinates() {
    let fv = quakes();

    // 4-level deep: features[0].geometry.coordinates[0] for longitude
    let lon: f64 = fv.extract("features[0].geometry.coordinates[0]").unwrap();
    let lat: f64 = fv.extract("features[0].geometry.coordinates[1]").unwrap();
    let depth: f64 = fv.extract("features[0].geometry.coordinates[2]").unwrap();

    // Check precision — these are 15-digit values
    println!("lon={lon}, lat={lat}, depth={depth}");
    assert!(lon < -120.0 && lon > -121.0, "longitude should be ~-120.58");
    assert!(lat > 35.0 && lat < 37.0, "latitude should be ~36.02");
    assert!(depth > 0.0 && depth < 100.0, "depth should be reasonable");
}

#[test]
fn coordinate_precision_preserved() {
    let fv = quakes();

    // The source coordinate -120.581497192383 has 15 significant digits.
    // Does it survive FlexValue round-trip?
    let lon: f64 = fv.extract("features[0].geometry.coordinates[0]").unwrap();

    // IEEE 754 f64 has ~15.9 significant digits — this should be exact
    // But serde_json might truncate or round during Value→f64 conversion.
    // Let's check if at least 12 significant digits survive.
    let lon_str = format!("{lon:.12}");
    println!("lon precision: {lon_str}");
    assert!(
        lon_str.starts_with("-120.581497192"),
        "should preserve at least 12 decimal digits, got: {lon_str}"
    );
}

// --- Millisecond epoch timestamps ---

#[test]
fn millisecond_timestamp_as_i64() {
    let fv = quakes();

    // time is ~1.775 trillion — fits i64, overflows i32
    let time: i64 = fv.extract("features[0].properties.time").unwrap();
    assert!(time > 1_000_000_000_000, "should be millisecond epoch");
    println!("timestamp: {time}");
}

#[test]
fn millisecond_timestamp_overflows_i32() {
    let fv = quakes();

    // 1775006658650 > i32::MAX (2147483647) — what error do we get?
    let result = fv.extract::<i32>("features[0].properties.time");
    println!("timestamp as i32: {:?}", result);
    // Should be an error (overflow)
    assert!(result.is_err());
}

// --- Null field handling ---

#[test]
fn null_fields_extract_as_none() {
    let fv = quakes();

    // 5 fields are null: tz, felt, cdi, mmi, alert
    let tz: Option<String> = fv.extract("features[0].properties.tz").unwrap();
    assert!(tz.is_none());

    let felt: Option<i64> = fv.extract("features[0].properties.felt").unwrap();
    assert!(felt.is_none());

    let alert: Option<String> = fv.extract("features[0].properties.alert").unwrap();
    assert!(alert.is_none());
}

// --- Integer-as-boolean coercion ---

#[test]
fn tsunami_integer_to_bool() {
    let fv = quakes();

    // tsunami is 0 (integer) — coerce to bool should give false
    let tsunami_int: i64 = fv.extract("features[0].properties.tsunami").unwrap();
    assert_eq!(tsunami_int, 0);

    let tsunami_bool: bool = fv.extract("features[0].properties.tsunami").unwrap();
    assert!(!tsunami_bool, "tsunami:0 should coerce to false");
}

// --- Schema inference on features array ---

#[test]
fn schema_on_features_array() {
    // features is an array of objects — infer schema on it
    let features_raw: serde_json::Value =
        serde_json::from_str(include_str!("../testdata/usgs_earthquakes.json")).unwrap();
    let features = features_raw["features"].as_array().unwrap();

    let schema = InferredSchema::from_values(features);
    println!("fields: {:?}", schema.fields.keys().collect::<Vec<_>>());

    // Each feature has: type, properties, geometry, id
    assert!(schema.fields.contains_key("type"));
    assert!(schema.fields.contains_key("properties"));
    assert!(schema.fields.contains_key("geometry"));

    // properties and geometry are Objects
    let props = &schema.fields["properties"];
    assert_eq!(props.dominant_type, Some(JsonType::Object));
}
