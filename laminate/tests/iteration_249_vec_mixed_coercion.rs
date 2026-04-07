//! Iteration 249: Vec<T> with mixed coercible/non-coercible elements
//!
//! When extracting Vec<i64> from a JSON array with mixed types, the coercion
//! engine processes each element individually. What happens when:
//! - Some elements coerce and others don't?
//! - The array has nulls mixed in?
//! - The array has strings that are null sentinels?
//!
//! The question: does the Vec extraction fail on the first non-coercible
//! element, or does it collect all results?

use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn vec_all_coercible_strings() {
    let val = FlexValue::from(serde_json::json!(["1", "2", "3"]))
        .with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(
        result,
        vec![1, 2, 3],
        "all string elements should coerce to i64"
    );
}

#[test]
fn vec_mixed_types_coercible() {
    let val = FlexValue::from(serde_json::json!([1, "2", 3, "4"]))
        .with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(
        result,
        vec![1, 2, 3, 4],
        "mixed int/string elements should all coerce"
    );
}

#[test]
fn vec_with_non_coercible_element() {
    let val =
        FlexValue::from(serde_json::json!([1, "two", 3])).with_coercion(CoercionLevel::BestEffort);

    let result: Result<Vec<i64>, _> = val.extract_root();
    println!("Vec with non-coercible 'two': {:?}", result);
    // "two" can't be parsed as i64 — serde should fail
    assert!(
        result.is_err(),
        "Vec with non-coercible element should fail"
    );
}

#[test]
fn vec_with_null_sentinels() {
    // At BestEffort, "N/A" → null → 0 for i64 elements
    let val =
        FlexValue::from(serde_json::json!([1, "N/A", 3])).with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract_root().unwrap();
    println!("Vec with N/A: {:?}", result);
    // "N/A" → null sentinel → null → 0 (default for i64)
    assert_eq!(result, vec![1, 0, 3], "N/A should become 0 in Vec<i64>");
}

#[test]
fn vec_of_option_with_nulls() {
    let val =
        FlexValue::from(serde_json::json!([1, null, 3])).with_coercion(CoercionLevel::BestEffort);

    let result: Vec<Option<i64>> = val.extract_root().unwrap();
    println!("Vec<Option<i64>> with null: {:?}", result);
    assert_eq!(result, vec![Some(1), None, Some(3)]);
}

#[test]
fn vec_at_exact_no_coercion() {
    let val =
        FlexValue::from(serde_json::json!(["1", "2", "3"])).with_coercion(CoercionLevel::Exact);

    let result: Result<Vec<i64>, _> = val.extract_root();
    println!("Vec<i64> at Exact with strings: {:?}", result);
    // At Exact, strings can't coerce to i64
    assert!(
        result.is_err(),
        "Vec<i64> with strings should fail at Exact"
    );
}

#[test]
fn vec_of_strings_preserves_values() {
    let val = FlexValue::from(serde_json::json!(["hello", "world", "42"]))
        .with_coercion(CoercionLevel::BestEffort);

    let result: Vec<String> = val.extract_root().unwrap();
    assert_eq!(result, vec!["hello", "world", "42"]);
}

#[test]
fn nested_vec_extraction_via_path() {
    let val = FlexValue::from(serde_json::json!({
        "scores": ["90", "85", "95"]
    }))
    .with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract("scores").unwrap();
    assert_eq!(result, vec![90, 85, 95]);
}

#[test]
fn empty_vec() {
    let val = FlexValue::from(serde_json::json!([])).with_coercion(CoercionLevel::BestEffort);

    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(
        result,
        Vec::<i64>::new(),
        "empty array should produce empty Vec"
    );
}
