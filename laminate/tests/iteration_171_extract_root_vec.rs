// Iteration 171: extract_root::<Vec<T>>() on non-array JSON
// Fresh target — what happens when root is an object/string/number but we ask for Vec?

use laminate::FlexValue;

#[test]
fn extract_root_vec_from_array() {
    // Normal case: root is an array
    let val = FlexValue::from_json(r#"[1, 2, 3]"#).unwrap();
    let result: Vec<i64> = val.extract_root().unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn extract_root_vec_from_object_fails() {
    // Root is an object — can't extract as Vec
    let val = FlexValue::from_json(r#"{"a": 1}"#).unwrap();
    let result: Result<Vec<i64>, _> = val.extract_root();
    assert!(result.is_err(), "Extracting Vec from object should fail");
}

#[test]
fn extract_root_vec_from_string_fails() {
    // Root is a string — can't extract as Vec
    let val = FlexValue::from_json(r#""hello""#).unwrap();
    let result: Result<Vec<String>, _> = val.extract_root();
    assert!(result.is_err(), "Extracting Vec from string should fail");
}

#[test]
fn extract_root_vec_from_number_fails() {
    // Root is a number — can't extract as Vec
    let val = FlexValue::from_json(r#"42"#).unwrap();
    let result: Result<Vec<i64>, _> = val.extract_root();
    assert!(result.is_err(), "Extracting Vec from number should fail");
}

#[test]
fn extract_root_vec_of_strings_from_mixed_array() {
    // Array of mixed types — extracting as Vec<String> with BestEffort
    let val = FlexValue::from_json(r#"[1, "two", true, null]"#)
        .unwrap()
        .with_coercion(laminate::coerce::CoercionLevel::BestEffort);
    let result: Vec<String> = val.extract_root().unwrap();
    // Numbers and bools should coerce to strings
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "1");
    assert_eq!(result[1], "two");
    assert_eq!(result[2], "true");
}

#[test]
fn extract_root_empty_array() {
    let val = FlexValue::from_json(r#"[]"#).unwrap();
    let result: Vec<i64> = val.extract_root().unwrap();
    assert!(result.is_empty());
}
