//! Iteration 291 — Coerce-VecNestedCoerce
//! Per-element coercion in Vec works correctly at all levels (PASS)

use laminate::{CoercionLevel, FlexValue};

#[test]
fn vec_mixed_besteffort_coerces_all() {
    let fv = FlexValue::from_json(r#"[1, "2", true]"#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let result: Vec<i64> = fv.extract("").unwrap();
    assert_eq!(result, vec![1, 2, 1]); // int, string→int, bool→int
}

#[test]
fn vec_mixed_exact_rejects_coercion() {
    let fv = FlexValue::from_json(r#"[1, "2", true]"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    assert!(fv.extract::<Vec<i64>>("").is_err());
}

#[test]
fn vec_homogeneous_exact_passes() {
    let fv = FlexValue::from_json(r#"[1, 2, 3]"#)
        .unwrap()
        .with_coercion(CoercionLevel::Exact);
    let result: Vec<i64> = fv.extract("").unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}
