/// Iteration 82: SafeWidening integer → f64 precision loss detection
///
/// Large integers (beyond 2^53) can't be represented exactly as f64.
/// SafeWidening should still coerce but produce a Warning diagnostic
/// for precision loss, not silently convert at Info level.
use laminate::coerce::CoercionLevel;
use laminate::diagnostic::RiskLevel;
use laminate::FlexValue;

#[test]
fn safe_widening_small_int_to_f64_is_info() {
    // Small integer: exact f64 representation, should be Info risk
    let val = FlexValue::from(serde_json::json!(42)).with_coercion(CoercionLevel::SafeWidening);
    let (result, diags): (f64, _) = val.extract_root_with_diagnostics().unwrap();
    assert!((result - 42.0).abs() < f64::EPSILON);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].risk, RiskLevel::Info);
}

#[test]
fn safe_widening_i64_max_to_f64_warns_precision() {
    // i64::MAX = 9223372036854775807 exceeds 2^53 — precision loss
    let val =
        FlexValue::from(serde_json::json!(i64::MAX)).with_coercion(CoercionLevel::SafeWidening);
    let (result, diags): (f64, _) = val.extract_root_with_diagnostics().unwrap();
    // Coercion should succeed (it's still a valid f64)
    assert!(result > 0.0);
    // But the diagnostic should be Warning, not Info
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].risk,
        RiskLevel::Warning,
        "large integer → f64 should produce Warning for precision loss"
    );
    assert!(
        diags[0]
            .suggestion
            .as_deref()
            .unwrap()
            .contains("precision"),
        "diagnostic should mention precision"
    );
}

#[test]
fn safe_widening_u64_max_to_f64_warns_precision() {
    // u64::MAX should now go through coercion (not serde passthrough) with Warning
    let val =
        FlexValue::from(serde_json::json!(u64::MAX)).with_coercion(CoercionLevel::SafeWidening);
    let (result, diags): (f64, _) = val.extract_root_with_diagnostics().unwrap();
    assert!(result > 0.0);
    assert_eq!(
        diags.len(),
        1,
        "u64::MAX should produce a coercion diagnostic (not bypass coercion)"
    );
    assert_eq!(diags[0].risk, RiskLevel::Warning);
}

#[test]
fn safe_widening_2_pow_53_is_exact() {
    // 2^53 = 9007199254740992 — last integer exactly representable as f64
    let val = FlexValue::from(serde_json::json!(9007199254740992_i64))
        .with_coercion(CoercionLevel::SafeWidening);
    let (result, diags): (f64, _) = val.extract_root_with_diagnostics().unwrap();
    assert_eq!(result, 9007199254740992.0);
    assert_eq!(diags[0].risk, RiskLevel::Info, "2^53 fits exactly in f64");
}

#[test]
fn safe_widening_2_pow_53_plus_1_loses_precision() {
    // 2^53 + 1 = 9007199254740993 — first integer NOT exactly representable as f64
    let val = FlexValue::from(serde_json::json!(9007199254740993_i64))
        .with_coercion(CoercionLevel::SafeWidening);
    let (result, diags): (f64, _) = val.extract_root_with_diagnostics().unwrap();
    // f64 rounds to 9007199254740992.0 — off by 1
    assert_ne!(result as i64, 9007199254740993, "precision should be lost");
    assert_eq!(
        diags[0].risk,
        RiskLevel::Warning,
        "2^53 + 1 should warn about precision loss"
    );
}
