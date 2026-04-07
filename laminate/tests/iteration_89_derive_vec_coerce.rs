/// Iteration 89: Derive struct with Vec<i64> field + #[laminate(coerce)] on mixed array
///
/// The derive macro's coerce path used coerce_value() with target "Vec<i64>",
/// which didn't match any arm. Now coerce_value handles Vec<T> target types
/// by extracting the inner type and coercing each element individually.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Scores {
    #[laminate(coerce)]
    values: Vec<i64>,
}

#[test]
fn derive_vec_i64_mixed_array_coerced() {
    // "2" should be coerced to 2 at the element level
    let json = r#"{"values": [1, "2", 3]}"#;
    let (scores, diagnostics) = Scores::from_json(json).unwrap();

    assert_eq!(
        scores.values,
        vec![1, 2, 3],
        "string element should be coerced to i64"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| format!("{:?}", d.kind).contains("Coerced")),
        "should produce a coercion diagnostic for the string element"
    );
}

#[test]
fn derive_vec_i64_homogeneous_array() {
    let json = r#"{"values": [10, 20, 30]}"#;
    let (scores, _) = Scores::from_json(json).unwrap();
    assert_eq!(scores.values, vec![10, 20, 30]);
}
