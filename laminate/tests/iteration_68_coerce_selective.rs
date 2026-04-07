//! Iteration 68 — Derive: coerce fires only on marked fields
//!
//! PASS: #[laminate(coerce)] is strictly opt-in per field.
//! Unmarked fields reject mismatched types via serde.

use laminate::Laminate;

#[derive(Laminate, Debug)]
struct MixedCoerce {
    #[laminate(coerce)]
    port: i64,
    name: String,
    active: bool,
}

#[test]
fn coerce_fires_on_marked_field() {
    let (s, diags) =
        MixedCoerce::from_json(r#"{"port": "8080", "name": "test", "active": true}"#).unwrap();
    assert_eq!(s.port, 8080);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].path, "port");
}

#[test]
fn unmarked_field_rejects_string_for_bool() {
    let result = MixedCoerce::from_json(r#"{"port": 8080, "name": "test", "active": "true"}"#);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("active"),
        "error should mention the field name"
    );
}

#[test]
fn unmarked_field_rejects_number_for_string() {
    let result = MixedCoerce::from_json(r#"{"port": 8080, "name": 42, "active": true}"#);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("name"));
}

#[test]
fn all_correct_types_no_coercion_needed() {
    let (s, diags) =
        MixedCoerce::from_json(r#"{"port": 8080, "name": "test", "active": true}"#).unwrap();
    assert_eq!(s.port, 8080);
    assert_eq!(s.name, "test");
    assert!(s.active);
    assert!(diags.is_empty(), "no diagnostics when types match exactly");
}
