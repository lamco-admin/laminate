/// Mode system integration tests — verifying that modes actually control shaping behavior.
use laminate::{FlexValue, Laminate, Lenient, Strict};

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(coerce)]
    port: u16,
    #[laminate(coerce, default)]
    debug: bool,
}

#[test]
fn with_mode_sets_coercion_level() {
    let val = FlexValue::from_json(r#"{"x": "42"}"#).unwrap();

    // Lenient → BestEffort
    let lenient = val.clone().with_mode::<Lenient>();
    let result: i64 = lenient.extract("x").unwrap();
    assert_eq!(result, 42);

    // Strict → Exact (rejects string → int)
    let strict = val.clone().with_mode::<Strict>();
    let result: Result<i64, _> = strict.extract("x");
    assert!(
        result.is_err(),
        "Strict mode should reject string → int coercion"
    );
}

#[test]
fn shape_lenient_produces_laminate_result() {
    let json = serde_json::json!({"name": "test", "port": "8080", "debug": "yes", "extra": true});
    let result = Config::shape_lenient(&json).unwrap();

    assert_eq!(result.value.name, "test");
    assert_eq!(result.value.port, 8080);
    assert_eq!(result.value.debug, true);
    // Lenient residual is ()
    assert_eq!(result.residual, ());
    // Diagnostics should show coercions
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn shape_strict_rejects_unknown_fields() {
    let json = serde_json::json!({"name": "test", "port": 8080, "debug": true, "unknown": "field"});
    let result = Config::shape_strict(&json);

    assert!(result.is_err(), "Strict mode should reject unknown fields");
    if let Err(laminate::FlexError::ShapingDiagnostics { count, .. }) = result {
        assert!(count > 0);
    }
}

#[test]
fn shape_strict_accepts_clean_data() {
    let json = serde_json::json!({"name": "test", "port": 8080, "debug": true});
    let result = Config::shape_strict(&json);
    assert!(result.is_ok(), "Strict mode should accept matching types");
    let config = result.unwrap();
    assert_eq!(config.port, 8080);
}

#[test]
fn shape_strict_rejects_coerced_data() {
    // port is a string "8080" — requires coercion, strict should reject
    let json = serde_json::json!({"name": "test", "port": "8080", "debug": true});
    let result = Config::shape_strict(&json);
    assert!(result.is_err(), "Strict mode should reject coerced values");
}

#[test]
fn shape_absorbing_produces_result_with_overflow() {
    let json = serde_json::json!({"name": "test", "port": "8080", "debug": "yes"});
    let result = Config::shape_absorbing(&json).unwrap();
    assert_eq!(result.value.port, 8080);
}

#[test]
fn dynamic_mode_sets_coercion() {
    use laminate::DynamicMode;

    let val = FlexValue::from_json(r#"{"x": "42"}"#).unwrap();

    let lenient = val.clone().with_dynamic_mode(DynamicMode::Lenient);
    let result: i64 = lenient.extract("x").unwrap();
    assert_eq!(result, 42);

    let strict = val.with_dynamic_mode(DynamicMode::Strict);
    let result: Result<i64, _> = strict.extract("x");
    assert!(result.is_err());
}
