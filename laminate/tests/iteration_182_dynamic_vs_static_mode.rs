//! Iteration 182: DynamicMode vs static Mode — should be identical coercion behavior.

use laminate::FlexValue;
use laminate::{Absorbing, DynamicMode};

#[test]
fn dynamic_absorbing_matches_static_absorbing() {
    let json = r#"{"x": "42", "y": 3.14, "z": true}"#;

    let static_val = FlexValue::from_json(json).unwrap().with_mode::<Absorbing>();
    let dynamic_val = FlexValue::from_json(json)
        .unwrap()
        .with_dynamic_mode(DynamicMode::Absorbing);

    // SafeWidening: int-as-string "42" → i64 should work at SafeWidening
    // (BestEffort would also work, but SafeWidening is Absorbing's level)
    let s_x = static_val.extract::<i64>("x");
    let d_x = dynamic_val.extract::<i64>("x");

    // Both should have identical results
    match (&s_x, &d_x) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "Values should match"),
        (Err(_), Err(_)) => {} // both error — fine
        _ => panic!("Mismatched results: static={s_x:?}, dynamic={d_x:?}"),
    }

    // Float extraction
    let s_y: f64 = static_val.extract("y").unwrap();
    let d_y: f64 = dynamic_val.extract("y").unwrap();
    assert_eq!(s_y, d_y);

    // Bool extraction
    let s_z: bool = static_val.extract("z").unwrap();
    let d_z: bool = dynamic_val.extract("z").unwrap();
    assert_eq!(s_z, d_z);
}
