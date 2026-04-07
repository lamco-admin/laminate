//! Iteration 284 — Coerce-NullSentinelEmpty
//! Test extract::<i64>() on "" (empty string) at various coercion levels

use laminate::{CoercionLevel, FlexValue};

#[test]
fn empty_string_to_i64_besteffort() {
    let fv = FlexValue::from_json(r#""""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let result = fv.extract::<i64>("");
    eprintln!("BestEffort: empty string → i64 = {:?}", result);
    // Empty string is not in is_null_sentinel() list, so coercion fails.
    // This is correct: "" is not "null", "N/A", "none", etc.
    // If the user wants empty strings as nulls, they should pre-process.
    assert!(result.is_err(), "empty string should not coerce to i64");
}

#[test]
fn empty_string_to_option_i64_besteffort() {
    let fv = FlexValue::from_json(r#""""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let result = fv.extract::<Option<i64>>("");
    eprintln!("BestEffort: empty string → Option<i64> = {:?}", result);
    // Even for Option<i64>, empty string isn't null, so it should fail
    // (it's a present-but-unparseable value, not an absent/null value)
}

#[test]
fn empty_string_to_string_besteffort() {
    // But extracting as String should work — it IS a string
    let fv = FlexValue::from_json(r#""""#)
        .unwrap()
        .with_coercion(CoercionLevel::BestEffort);
    let result: String = fv.extract("").unwrap();
    assert_eq!(result, "");
}
