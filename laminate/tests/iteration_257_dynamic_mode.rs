//! Iteration 257: DynamicMode integration with FlexValue
//!
//! Tests the runtime mode selection via with_dynamic_mode() and verifies
//! that it correctly maps to the same coercion levels as static modes.

use laminate::{CoercionLevel, DynamicMode, FlexValue};

#[test]
fn dynamic_lenient_enables_best_effort() {
    let val =
        FlexValue::from(serde_json::json!({"count": "42"})).with_dynamic_mode(DynamicMode::Lenient);

    let result: i64 = val.extract("count").unwrap();
    assert_eq!(
        result, 42,
        "Lenient dynamic mode should enable BestEffort coercion"
    );
}

#[test]
fn dynamic_strict_uses_exact() {
    let val =
        FlexValue::from(serde_json::json!({"count": "42"})).with_dynamic_mode(DynamicMode::Strict);

    let result: Result<i64, _> = val.extract("count");
    assert!(
        result.is_err(),
        "Strict dynamic mode should use Exact (no string coercion)"
    );
}

#[test]
fn dynamic_absorbing_uses_safewidening() {
    // SafeWidening allows int→float but not string→int
    let val = FlexValue::from(serde_json::json!({"count": "42"}))
        .with_dynamic_mode(DynamicMode::Absorbing);

    let result: Result<i64, _> = val.extract("count");
    assert!(
        result.is_err(),
        "Absorbing dynamic mode should use SafeWidening (no string coercion)"
    );

    // But int→float should work
    let val2 =
        FlexValue::from(serde_json::json!({"score": 42})).with_dynamic_mode(DynamicMode::Absorbing);
    let result2: f64 = val2.extract("score").unwrap();
    assert!(
        (result2 - 42.0).abs() < 0.01,
        "Absorbing should allow int→float"
    );
}

#[test]
fn dynamic_mode_from_string() {
    // Parse mode from config-like strings
    let mode: DynamicMode = "lenient".parse().unwrap();
    assert_eq!(mode, DynamicMode::Lenient);

    let mode: DynamicMode = "STRICT".parse().unwrap();
    assert_eq!(mode, DynamicMode::Strict);

    let mode: DynamicMode = "Absorbing".parse().unwrap();
    assert_eq!(mode, DynamicMode::Absorbing);

    let mode: Result<DynamicMode, _> = "invalid".parse();
    assert!(mode.is_err(), "invalid mode string should error");
}

#[test]
fn dynamic_mode_display_roundtrip() {
    let modes = vec![
        DynamicMode::Lenient,
        DynamicMode::Absorbing,
        DynamicMode::Strict,
    ];
    for mode in &modes {
        let s = mode.to_string();
        let parsed: DynamicMode = s.parse().unwrap();
        assert_eq!(*mode, parsed, "display/parse roundtrip for {:?}", mode);
    }
}

#[test]
fn dynamic_mode_coercion_matches_static() {
    assert_eq!(
        DynamicMode::Lenient.default_coercion(),
        CoercionLevel::BestEffort
    );
    assert_eq!(
        DynamicMode::Absorbing.default_coercion(),
        CoercionLevel::SafeWidening
    );
    assert_eq!(DynamicMode::Strict.default_coercion(), CoercionLevel::Exact);
}

#[test]
fn dynamic_mode_field_policies() {
    // Verify all mode policies match
    assert!(!DynamicMode::Lenient.reject_unknown_fields());
    assert!(!DynamicMode::Absorbing.reject_unknown_fields());
    assert!(DynamicMode::Strict.reject_unknown_fields());

    assert!(!DynamicMode::Lenient.require_all_fields());
    assert!(DynamicMode::Absorbing.require_all_fields());
    assert!(DynamicMode::Strict.require_all_fields());

    assert!(!DynamicMode::Lenient.fail_fast());
    assert!(!DynamicMode::Absorbing.fail_fast());
    assert!(DynamicMode::Strict.fail_fast());
}

#[test]
fn dynamic_mode_config_pattern() {
    // Real-world use: mode comes from config, applies to extraction
    let config_mode = "lenient"; // from config file
    let mode: DynamicMode = config_mode.parse().unwrap();

    let data = FlexValue::from(serde_json::json!({"port": "8080", "debug": "yes"}))
        .with_dynamic_mode(mode);

    let port: i64 = data.extract("port").unwrap();
    let debug: bool = data.extract("debug").unwrap();

    assert_eq!(port, 8080);
    assert!(debug);
}
