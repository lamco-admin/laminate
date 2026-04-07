#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration 30: Boundary — Vec<String> field receiving a single string

use laminate_derive::Laminate;

#[derive(Debug, Laminate)]
struct TagsCoerced {
    name: String,
    #[laminate(coerce)]
    tags: Vec<String>,
}

#[derive(Debug, Laminate)]
struct TagsDefault {
    name: String,
    #[laminate(coerce, default)]
    tags: Vec<String>,
}

#[test]
fn iter30_baseline_array_works() {
    let (s, _) = TagsCoerced::from_json(r#"{"name": "test", "tags": ["rust", "serde"]}"#).unwrap();
    assert_eq!(s.tags, vec!["rust", "serde"]);
}

#[test]
fn iter30_coerce_single_string_to_vec() {
    let result = TagsCoerced::from_json(r#"{"name": "test", "tags": "rust"}"#);
    println!("coerce result: {:?}", result);
    let (s, _) = result.unwrap();
    assert_eq!(s.tags, vec!["rust"]);
}

#[test]
fn iter30_coerce_default_single_string_to_vec() {
    let (s, diags) = TagsDefault::from_json(r#"{"name": "test", "tags": "rust"}"#).unwrap();
    println!("coerce+default result: {:?}", s);
    println!("diagnostics: {:?}", diags);
    assert_eq!(s.tags, vec!["rust"]);
}
