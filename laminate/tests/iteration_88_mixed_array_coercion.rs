/// Iteration 88: Extract Vec<i64> from [1, "2", 3] — element-level coercion
///
/// Vec<T> previously bypassed coercion entirely because coercion_hint=None.
/// Now Vec<T> propagates T's coercion hint to each array element via
/// the element_hint() trait method.
use laminate::coerce::CoercionLevel;
use laminate::FlexValue;

#[test]
fn mixed_array_vec_i64_best_effort() {
    // BestEffort should coerce "2" → 2
    let val =
        FlexValue::from(serde_json::json!([1, "2", 3])).with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(
        result,
        vec![1, 2, 3],
        "BestEffort should coerce string elements"
    );
}

#[test]
fn mixed_array_vec_i64_string_coercion() {
    // StringCoercion should also handle "2" → 2
    let val = FlexValue::from(serde_json::json!([1, "2", 3]))
        .with_coercion(CoercionLevel::StringCoercion);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn mixed_array_vec_i64_exact_rejects() {
    // Exact mode should reject the string element
    let val = FlexValue::from(serde_json::json!([1, "2", 3])).with_coercion(CoercionLevel::Exact);

    let result: Result<Vec<i64>, _> = val.extract_root();
    assert!(
        result.is_err(),
        "Exact mode should reject string in i64 array"
    );
}

#[test]
fn homogeneous_array_unaffected() {
    // All integers — should work at any coercion level
    let val = FlexValue::from(serde_json::json!([1, 2, 3])).with_coercion(CoercionLevel::Exact);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}
