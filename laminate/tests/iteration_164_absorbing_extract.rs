// Iteration 164: with_mode::<Absorbing>() + extract — SafeWidening coercion level
// Target #135 — Absorbing mode uses SafeWidening, which allows int→float
// but rejects string→number. Verify extract() actually enforces this.

use laminate::{Absorbing, FlexValue};

#[test]
fn absorbing_allows_int_to_float() {
    // Integer 42 → f64 should succeed under SafeWidening
    let val = FlexValue::from_json(r#"{"count": 42}"#).unwrap();
    let result: f64 = val.with_mode::<Absorbing>().extract("count").unwrap();
    assert_eq!(result, 42.0);
}

#[test]
fn absorbing_rejects_string_to_number() {
    // String "42" → u32 should FAIL under SafeWidening (that's StringCoercion territory)
    let val = FlexValue::from_json(r#"{"port": "8080"}"#).unwrap();
    let result: Result<u32, _> = val.with_mode::<Absorbing>().extract("port");
    assert!(
        result.is_err(),
        "SafeWidening should reject string→number coercion"
    );
}

#[test]
fn absorbing_allows_bool_to_int() {
    // Bool true → i32 should succeed under SafeWidening
    let val = FlexValue::from_json(r#"{"flag": true}"#).unwrap();
    let result: i32 = val.with_mode::<Absorbing>().extract("flag").unwrap();
    assert_eq!(result, 1);
}

#[test]
fn absorbing_exact_number_passthrough() {
    // f64 → f64 should always succeed (no coercion needed)
    let val = FlexValue::from_json(r#"{"pi": 3.14}"#).unwrap();
    let result: f64 = val.with_mode::<Absorbing>().extract("pi").unwrap();
    assert!((result - 3.14).abs() < f64::EPSILON);
}

#[test]
fn absorbing_rejects_string_to_bool() {
    // String "true" → bool should fail under SafeWidening
    let val = FlexValue::from_json(r#"{"active": "true"}"#).unwrap();
    let result: Result<bool, _> = val.with_mode::<Absorbing>().extract("active");
    assert!(
        result.is_err(),
        "SafeWidening should reject string→bool coercion"
    );
}
