//! Iteration 292 — Coerce-MapExtract
//! Dotted path resolution: nested paths win, literal dotted keys unreachable (by design)

use laminate::FlexValue;

#[test]
fn nested_path_resolved() {
    let fv = FlexValue::from_json(r#"{"meta": {"content-type": "application/json"}}"#).unwrap();
    assert_eq!(
        fv.extract::<String>("meta.content-type").unwrap(),
        "application/json"
    );
}

#[test]
fn literal_dotted_key_unreachable() {
    // Known limitation: literal keys containing dots cannot be addressed via dot-path
    let fv = FlexValue::from_json(r#"{"a.b": "literal"}"#).unwrap();
    assert!(
        fv.extract::<String>("a.b").is_err(),
        "literal dotted key is unreachable via dot-path"
    );
}

#[test]
fn nested_wins_over_literal() {
    let fv = FlexValue::from_json(r#"{"a.b": "literal", "a": {"b": "nested"}}"#).unwrap();
    assert_eq!(fv.extract::<String>("a.b").unwrap(), "nested");
}
