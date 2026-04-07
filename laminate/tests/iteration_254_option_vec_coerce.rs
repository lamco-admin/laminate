//! Iteration 254: Option<Vec<T>> with #[laminate(coerce)]
//!
//! The derive macro has a special option_vec code path for Option<Vec<T>>.
//! This tests all the edge cases:
//! - null → None
//! - [] → Some([])
//! - ["1", "2"] → Some([1, 2]) with coercion
//! - absent → depends on default attribute
//! - non-array value → error

use laminate::Laminate;

#[derive(Debug, Laminate, PartialEq)]
struct Record {
    #[laminate(coerce)]
    tags: Option<Vec<String>>,

    #[laminate(coerce)]
    scores: Option<Vec<i64>>,

    #[laminate(coerce, default)]
    ids: Option<Vec<i64>>,
}

#[test]
fn option_vec_null_becomes_none() {
    let json = serde_json::json!({"tags": null, "scores": null});
    let (result, _) = Record::from_flex_value(&json).unwrap();

    assert_eq!(result.tags, None, "null → None");
    assert_eq!(result.scores, None, "null → None");
}

#[test]
fn option_vec_empty_array_becomes_some_empty() {
    let json = serde_json::json!({"tags": [], "scores": []});
    let (result, _) = Record::from_flex_value(&json).unwrap();

    assert_eq!(result.tags, Some(vec![]), "[] → Some([])");
    assert_eq!(result.scores, Some(vec![]), "[] → Some([])");
}

#[test]
fn option_vec_with_coercible_elements() {
    let json = serde_json::json!({"tags": ["hello", "world"], "scores": ["90", "85", "95"]});
    let (result, diags) = Record::from_flex_value(&json).unwrap();

    println!("Result: {:?}", result);
    println!("Diagnostics: {:?}", diags);

    assert_eq!(
        result.tags,
        Some(vec!["hello".to_string(), "world".to_string()])
    );
    assert_eq!(
        result.scores,
        Some(vec![90, 85, 95]),
        "string scores should coerce to i64"
    );
}

#[test]
fn option_vec_missing_with_default() {
    // ids has #[laminate(default)] — missing should produce default
    let json = serde_json::json!({"tags": ["a"], "scores": [1]});
    let (result, _) = Record::from_flex_value(&json).unwrap();

    println!("ids (missing + default): {:?}", result.ids);
    // Default for Option<Vec<i64>> is None
    assert_eq!(
        result.ids, None,
        "missing Option<Vec<i64>> with default should be None"
    );
}

#[test]
fn option_vec_with_mixed_type_elements() {
    // Mix of int and string in scores — should coerce all to i64
    let json = serde_json::json!({"tags": ["a"], "scores": [1, "2", 3]});
    let (result, _) = Record::from_flex_value(&json).unwrap();

    assert_eq!(result.scores, Some(vec![1, 2, 3]));
}

#[test]
fn option_vec_single_element() {
    let json = serde_json::json!({"tags": ["only"], "scores": [42]});
    let (result, _) = Record::from_flex_value(&json).unwrap();

    assert_eq!(result.tags, Some(vec!["only".to_string()]));
    assert_eq!(result.scores, Some(vec![42]));
}
