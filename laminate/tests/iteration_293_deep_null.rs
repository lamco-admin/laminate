//! Iteration 293 — Coerce-DeepNull
//! null mid-path returns Ok(None), not panic or error (PASS)

use laminate::FlexValue;

#[test]
fn deep_null_mid_path_returns_none() {
    let fv = FlexValue::from_json(r#"{"a": null}"#).unwrap();
    let result = fv.maybe::<String>("a.b.c.d.e").unwrap();
    assert_eq!(result, None);
}

#[test]
fn deep_null_first_level() {
    let fv = FlexValue::from_json(r#"{"a": {"b": null}}"#).unwrap();
    let result = fv.maybe::<i64>("a.b.c").unwrap();
    assert_eq!(result, None);
}
