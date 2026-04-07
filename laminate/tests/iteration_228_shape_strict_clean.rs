//! Iteration 228: shape_strict with all-valid data — no diagnostics, returns Ok(Self)
//!
//! shape_strict requires exact types and no unknown fields.
//! When data perfectly matches, it should return Ok(Self) with no ShapingDiagnostics error.

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Config {
    host: String,
    port: i64,
    debug: bool,
}

#[test]
fn shape_strict_all_valid() {
    let json = serde_json::json!({
        "host": "localhost",
        "port": 8080,
        "debug": true
    });
    let result = Config::shape_strict(&json);
    println!("shape_strict all-valid: {:?}", result);
    let val = result.expect("shape_strict should succeed with exact types");
    assert_eq!(val.host, "localhost");
    assert_eq!(val.port, 8080);
    assert_eq!(val.debug, true);
}

#[test]
fn shape_strict_rejects_string_where_int_expected() {
    let json = serde_json::json!({
        "host": "localhost",
        "port": "8080",
        "debug": true
    });
    let result = Config::shape_strict(&json);
    println!("shape_strict string port: {:?}", result);
    assert!(
        result.is_err(),
        "shape_strict should reject string where i64 expected"
    );
}

#[test]
fn shape_strict_rejects_unknown_field() {
    let json = serde_json::json!({
        "host": "localhost",
        "port": 8080,
        "debug": true,
        "extra": "field"
    });
    let result = Config::shape_strict(&json);
    println!("shape_strict unknown field: {:?}", result);
    assert!(result.is_err(), "shape_strict should reject unknown fields");
}
