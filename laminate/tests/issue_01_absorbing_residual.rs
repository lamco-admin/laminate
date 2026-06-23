//! Regression test for issue #1: `shape_absorbing()` must populate the
//! `LaminateResult<T, Absorbing>` overflow **residual**, not only the struct's
//! own `#[laminate(overflow)]` field. Prior to the fix the residual was always
//! an empty map, while the struct field was populated — so tests that checked
//! the struct field passed and masked the gap.

use laminate_derive::Laminate;
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Resp {
    id: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn absorbing_residual_reflects_unknown_fields() {
    let value = serde_json::json!({ "id": "abc", "foo": 1, "bar": "x" });
    let result = Resp::shape_absorbing(&value).unwrap();

    // The struct field continues to capture unknowns.
    assert_eq!(result.value.id, "abc");
    assert_eq!(result.value.extra.len(), 2);

    // The LaminateResult residual must ALSO reflect them (this was the bug).
    assert_eq!(
        result.residual.len(),
        2,
        "residual should reflect overflow, got: {:?}",
        result.residual
    );
    assert_eq!(result.residual["foo"], serde_json::json!(1));
    assert_eq!(result.residual["bar"], serde_json::json!("x"));
}

#[test]
fn absorbing_residual_empty_when_no_unknowns() {
    let value = serde_json::json!({ "id": "abc" });
    let result = Resp::shape_absorbing(&value).unwrap();
    assert!(result.value.extra.is_empty());
    assert!(result.residual.is_empty());
}
