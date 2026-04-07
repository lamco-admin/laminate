#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration 34: Type Swap — Derive with coerce on String field receiving object
//!
//! Validates that the Object/Array→String coercion works through the derive
//! macro's coerce_value() path, not just through extract().

use laminate_derive::Laminate;

#[derive(Laminate, Debug)]
struct Config {
    name: String,
    #[laminate(coerce)]
    settings: String,
}

#[derive(Laminate, Debug)]
struct ConfigNoCoerce {
    name: String,
    settings: String,
}

#[test]
fn derive_coerce_object_to_string() {
    let json = serde_json::json!({"name": "app", "settings": {"level": 3, "debug": true}});
    let (cfg, diags) = Config::from_flex_value(&json).unwrap();
    assert_eq!(cfg.name, "app");
    // Object serialized to compact JSON string
    assert!(cfg.settings.contains("\"level\":3") || cfg.settings.contains("\"level\": 3"));
    assert!(cfg.settings.contains("\"debug\":true") || cfg.settings.contains("\"debug\": true"));
    // Diagnostic warns about the coercion
    assert_eq!(diags.len(), 1);
}

#[test]
fn derive_coerce_array_to_string() {
    let json = serde_json::json!({"name": "app", "settings": [1, 2, 3]});
    let (cfg, _) = Config::from_flex_value(&json).unwrap();
    assert_eq!(cfg.settings, "[1,2,3]");
}

#[test]
fn derive_no_coerce_object_errors() {
    let json = serde_json::json!({"name": "app", "settings": {"level": 3}});
    // Without coerce, object→String is a type error
    assert!(ConfigNoCoerce::from_flex_value(&json).is_err());
}
