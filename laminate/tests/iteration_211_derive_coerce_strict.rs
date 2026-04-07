#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration 211: #[laminate(coerce)] + shape_strict — does strict reject coerced fields?
//!
//! If a field has #[laminate(coerce)], from_flex_value uses BestEffort coercion.
//! But shape_strict uses Exact coercion. Does coerce attribute override strict mode?

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(coerce)]
    port: u16,
}

#[test]
fn shape_strict_rejects_coerced_field() {
    // "8080" requires coercion to u16. shape_strict should reject.
    let json = serde_json::json!({"name": "test", "port": "8080"});
    let result = Config::shape_strict(&json);

    assert!(
        result.is_err(),
        "shape_strict should reject coerced 'port' field"
    );
}

#[test]
fn shape_strict_accepts_exact_types() {
    // port = 8080 (number, not string) — no coercion needed
    let json = serde_json::json!({"name": "test", "port": 8080});
    let result = Config::shape_strict(&json);

    assert!(result.is_ok(), "shape_strict should accept exact types");
}

#[test]
fn shape_lenient_accepts_coerced_field() {
    let json = serde_json::json!({"name": "test", "port": "8080"});
    let result = Config::shape_lenient(&json);

    assert!(result.is_ok(), "shape_lenient should accept coerced fields");
    assert_eq!(result.unwrap().value.port, 8080);
}
