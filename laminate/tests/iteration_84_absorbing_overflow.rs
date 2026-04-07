use laminate::mode::{Absorbing, LaminateResult};
use laminate::Laminate;
/// Iteration 84: LaminateResult<T, Absorbing> — overflow captures unknown fields from derive
///
/// Tests the derive macro's #[laminate(overflow)] field with JSON that has
/// unknown fields. Verifies that unknown fields are captured, diagnostics are
/// emitted, and the result can be wrapped in LaminateResult<T, Absorbing>.
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    port: u16,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn overflow_captures_unknown_fields() {
    let json = r#"{
        "name": "myapp",
        "port": 8080,
        "debug": true,
        "log_level": "info",
        "max_retries": 3
    }"#;

    let (config, diagnostics) = Config::from_json(json).unwrap();

    println!("config.name = {}", config.name);
    println!("config.port = {}", config.port);
    println!("config.extra = {:?}", config.extra);
    println!("diagnostics = {:?}", diagnostics);

    // Known fields should be extracted
    assert_eq!(config.name, "myapp");
    assert_eq!(config.port, 8080);

    // Unknown fields should be captured in overflow
    assert_eq!(
        config.extra.len(),
        3,
        "expected 3 unknown fields in overflow"
    );
    assert_eq!(config.extra["debug"], serde_json::json!(true));
    assert_eq!(config.extra["log_level"], serde_json::json!("info"));
    assert_eq!(config.extra["max_retries"], serde_json::json!(3));

    // Diagnostics should list preserved fields
    let preserved_fields: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.kind,
                laminate::diagnostic::DiagnosticKind::Preserved { .. }
            )
        })
        .collect();
    assert_eq!(
        preserved_fields.len(),
        3,
        "each overflow field should produce a Preserved diagnostic"
    );

    // Wrap in LaminateResult<Config, Absorbing> using the overflow as residual
    let result = LaminateResult::<Config, Absorbing>::absorbing(
        config,
        HashMap::new(), // The mode-level overflow (separate from derive overflow)
        diagnostics,
    );

    // The struct's overflow is in result.value.extra (derive-level)
    // The mode's residual is in result.residual (mode-level) — empty here
    assert_eq!(result.value.extra.len(), 3);
    assert!(result.residual.is_empty());
}

#[test]
fn overflow_round_trip_to_value() {
    let json = r#"{"name":"myapp","port":8080,"custom_field":"preserved"}"#;

    let (config, _) = Config::from_json(json).unwrap();
    assert_eq!(config.extra.len(), 1);
    assert_eq!(config.extra["custom_field"], serde_json::json!("preserved"));

    // Round-trip: to_value should preserve overflow fields
    let value = config.to_value();
    println!("round-trip value = {}", value);

    // The custom_field should survive the round-trip
    assert_eq!(
        value["custom_field"],
        serde_json::json!("preserved"),
        "overflow field should survive round-trip via to_value()"
    );
    assert_eq!(value["name"], serde_json::json!("myapp"));
    assert_eq!(value["port"], serde_json::json!(8080));
}
