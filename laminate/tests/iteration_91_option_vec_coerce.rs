/// Iteration 91: Option<Vec<i64>> + #[laminate(coerce)] — nested generic coercion
///
/// Option<Vec<T>> needs both Option null guard AND Vec element coercion.
/// The derive macro now detects this pattern and generates combined handling.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Data {
    #[laminate(coerce)]
    scores: Option<Vec<i64>>,
}

#[test]
fn option_vec_null_produces_none() {
    let json = r#"{"scores": null}"#;
    let (data, _) = Data::from_json(json).unwrap();
    assert_eq!(data.scores, None);
}

#[test]
fn option_vec_absent_produces_none() {
    let json = r#"{}"#;
    let (data, _) = Data::from_json(json).unwrap();
    assert_eq!(data.scores, None);
}

#[test]
fn option_vec_array_produces_some() {
    let json = r#"{"scores": [1, 2, 3]}"#;
    let (data, _) = Data::from_json(json).unwrap();
    assert_eq!(data.scores, Some(vec![1, 2, 3]));
}

#[test]
fn option_vec_mixed_array_coerced() {
    let json = r#"{"scores": [1, "2", 3]}"#;
    let (data, diags) = Data::from_json(json).unwrap();
    assert_eq!(
        data.scores,
        Some(vec![1, 2, 3]),
        "string element should be coerced"
    );
    assert!(
        diags
            .iter()
            .any(|d| format!("{:?}", d.kind).contains("Coerced")),
        "should produce coercion diagnostic for element"
    );
}
