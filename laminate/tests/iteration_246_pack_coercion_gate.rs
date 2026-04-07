//! Iteration 246: PackCoercion::Currency at SafeWidening — gate verification
//!
//! Pack coercion is gated: it only fires when coercion level >= StringCoercion.
//! SafeWidening and Exact should silently skip pack coercion.
//!
//! This thoroughly tests the gate at all 4 coercion levels for both Currency and Units packs.

use laminate::value::PackCoercion;
use laminate::CoercionLevel;
use laminate::FlexValue;

// === Currency pack gate ===

#[test]
fn currency_at_exact_blocked() {
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::Exact)
        .with_pack_coercion(PackCoercion::Currency);

    let result: Result<f64, _> = val.extract_root();
    assert!(result.is_err(), "Currency pack should NOT fire at Exact");
}

#[test]
fn currency_at_safewidening_blocked() {
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::SafeWidening)
        .with_pack_coercion(PackCoercion::Currency);

    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_err(),
        "Currency pack should NOT fire at SafeWidening"
    );
}

#[test]
fn currency_at_stringcoercion_fires() {
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::StringCoercion)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    assert!(
        (result - 12.99).abs() < 0.01,
        "Currency pack should fire at StringCoercion"
    );
}

#[test]
fn currency_at_besteffort_fires() {
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    let result: f64 = val.extract_root().unwrap();
    assert!(
        (result - 12.99).abs() < 0.01,
        "Currency pack should fire at BestEffort"
    );
}

// === Units pack gate ===

#[test]
fn units_at_exact_blocked() {
    let val = FlexValue::from(serde_json::json!("5 kg"))
        .with_coercion(CoercionLevel::Exact)
        .with_pack_coercion(PackCoercion::Units);

    let result: Result<f64, _> = val.extract_root();
    assert!(result.is_err(), "Units pack should NOT fire at Exact");
}

#[test]
fn units_at_safewidening_blocked() {
    let val = FlexValue::from(serde_json::json!("5 kg"))
        .with_coercion(CoercionLevel::SafeWidening)
        .with_pack_coercion(PackCoercion::Units);

    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_err(),
        "Units pack should NOT fire at SafeWidening"
    );
}

#[test]
fn units_at_stringcoercion_fires() {
    let val = FlexValue::from(serde_json::json!("5 kg"))
        .with_coercion(CoercionLevel::StringCoercion)
        .with_pack_coercion(PackCoercion::Units);

    let result: f64 = val.extract_root().unwrap();
    assert!(
        (result - 5.0).abs() < 0.01,
        "Units pack should fire at StringCoercion"
    );
}

#[test]
fn units_at_besteffort_fires() {
    let val = FlexValue::from(serde_json::json!("5 kg"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Units);

    let result: f64 = val.extract_root().unwrap();
    assert!(
        (result - 5.0).abs() < 0.01,
        "Units pack should fire at BestEffort"
    );
}

// === PackCoercion::None should NEVER fire ===

#[test]
fn pack_none_at_besteffort_does_not_fire() {
    let val = FlexValue::from(serde_json::json!("$12.99"))
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::None);

    let result: Result<f64, _> = val.extract_root();
    assert!(
        result.is_err(),
        "PackCoercion::None should prevent all pack coercion"
    );
}

// === PackCoercion::All at StringCoercion ===

#[test]
fn all_packs_at_stringcoercion_currency() {
    let val = FlexValue::from(serde_json::json!("€24.50"))
        .with_coercion(CoercionLevel::StringCoercion)
        .with_pack_coercion(PackCoercion::All);

    let result: f64 = val.extract_root().unwrap();
    assert!((result - 24.50).abs() < 0.01);
}

#[test]
fn all_packs_at_stringcoercion_units() {
    let val = FlexValue::from(serde_json::json!("72 km"))
        .with_coercion(CoercionLevel::StringCoercion)
        .with_pack_coercion(PackCoercion::All);

    let result: f64 = val.extract_root().unwrap();
    assert!((result - 72.0).abs() < 0.01);
}
