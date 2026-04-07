//! Iteration 55: JSONPlaceholder /users — nested objects, schema infer + extract.

use laminate::schema::InferredSchema;
use laminate::FlexValue;

const USERS_SAMPLE: &str = r#"[
    {"id":1,"name":"Leanne","username":"Bret","email":"a@b.com",
     "address":{"street":"Kulas Light","city":"Gwenborough",
       "geo":{"lat":"-37.3159","lng":"81.1496"}},
     "phone":"1-770-736","website":"hildegard.org",
     "company":{"name":"Romaguera","catchPhrase":"Multi-layered","bs":"synergize"}},
    {"id":2,"name":"Ervin","username":"Antonette","email":"b@c.com",
     "address":{"street":"Victor Plains","city":"Wisokyburgh",
       "geo":{"lat":"-43.9509","lng":"-34.4618"}},
     "phone":"010-692","website":"anastasia.net",
     "company":{"name":"Deckow","catchPhrase":"Proactive","bs":"synergize"}}
]"#;

#[test]
fn infer_users_nested_schema() {
    let value: serde_json::Value = serde_json::from_str(USERS_SAMPLE).unwrap();
    let rows = value.as_array().unwrap();
    let schema = InferredSchema::from_values(rows);

    assert_eq!(schema.total_records, 2);
    assert_eq!(schema.fields.len(), 8);

    let addr = &schema.fields["address"];
    assert_eq!(addr.dominant_type, Some(laminate::schema::JsonType::Object));

    let company = &schema.fields["company"];
    assert_eq!(
        company.dominant_type,
        Some(laminate::schema::JsonType::Object)
    );
}

#[test]
fn extract_deeply_nested_geo() {
    let fv = FlexValue::from_json(USERS_SAMPLE).unwrap();

    let lat: String = fv.extract("[0].address.geo.lat").unwrap();
    assert_eq!(lat, "-37.3159");

    // Coerce string lat to float
    let lat_f: f64 = fv.extract("[0].address.geo.lat").unwrap();
    assert!((lat_f - (-37.3159)).abs() < 0.0001);
}

#[test]
fn audit_users_zero_violations() {
    let value: serde_json::Value = serde_json::from_str(USERS_SAMPLE).unwrap();
    let rows = value.as_array().unwrap();
    let schema = InferredSchema::from_values(rows);
    let report = schema.audit(rows);
    assert_eq!(report.violations.len(), 0);
}
