//! Iteration 282 — Coerce-ArrayUnwrapString
//! String extraction from arrays serializes to JSON (PASS — by design)

use laminate::FlexValue;

#[test]
fn single_element_array_serialized_for_string() {
    // String extraction serializes the array to JSON, does NOT unwrap
    let fv = FlexValue::from_json(r#"["hello"]"#).unwrap();
    assert_eq!(fv.extract_root::<String>().unwrap(), r#"["hello"]"#);
}

#[test]
fn single_element_array_unwraps_for_i64() {
    // Control: numeric extraction DOES unwrap
    let fv = FlexValue::from_json(r#"[42]"#).unwrap();
    assert_eq!(fv.extract_root::<i64>().unwrap(), 42);
}

#[test]
fn multi_element_array_serialized_for_string() {
    let fv = FlexValue::from_json(r#"["hello","world"]"#).unwrap();
    assert_eq!(fv.extract_root::<String>().unwrap(), r#"["hello","world"]"#);
}
