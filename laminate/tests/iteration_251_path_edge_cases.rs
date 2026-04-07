//! Iteration 251: Path parsing and navigation edge cases
//!
//! Tests adversarial path inputs and navigation through FlexValue.
//!
//! Adversarial:
//! - Trailing dot: "a.b." — parser should reject
//! - Empty segments: "a..b" — parser should reject
//! - Unicode keys: keys with emoji, CJK characters
//! - Numeric-looking keys: "42" as an object key (not array index)
//! - Very deep nesting
//! - Path to null intermediate

use laminate::FlexValue;

#[test]
fn trailing_dot_rejected() {
    let val = FlexValue::from(serde_json::json!({"a": {"b": 1}}));
    let result = val.at("a.b.");
    println!("trailing dot: {:?}", result);
    assert!(result.is_err(), "trailing dot should be rejected");
}

#[test]
fn empty_segment_rejected() {
    let val = FlexValue::from(serde_json::json!({"a": {"b": 1}}));
    let result = val.at("a..b");
    println!("empty segment: {:?}", result);
    assert!(result.is_err(), "empty segment should be rejected");
}

#[test]
fn unicode_key_navigation() {
    let val = FlexValue::from(serde_json::json!({"名前": "Alice", "年齢": 30}));

    let name: String = val.extract("名前").unwrap();
    assert_eq!(name, "Alice");

    let age: i64 = val.extract("年齢").unwrap();
    assert_eq!(age, 30);
}

#[test]
fn emoji_key_navigation() {
    let val = FlexValue::from(serde_json::json!({"🔑": "secret", "📊": [1, 2, 3]}));

    let secret: String = val.extract("🔑").unwrap();
    assert_eq!(secret, "secret");
}

#[test]
fn numeric_looking_object_key() {
    // Object key "42" should be treated as a key, not an array index
    let val = FlexValue::from(serde_json::json!({"42": "answer"}));

    let result: String = val.extract("42").unwrap();
    assert_eq!(
        result, "answer",
        "numeric-looking key should work as object key"
    );
}

#[test]
fn deep_nesting_navigation() {
    // 10 levels deep
    let val = FlexValue::from(serde_json::json!({
        "a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": {"j": 42}}}}}}}}}
    }));

    let result: i64 = val.extract("a.b.c.d.e.f.g.h.i.j").unwrap();
    assert_eq!(result, 42);
}

#[test]
fn path_through_null_intermediate() {
    let val = FlexValue::from(serde_json::json!({"a": null}));

    let result = val.at("a.b");
    println!("Path through null: {:?}", result);
    assert!(result.is_err(), "navigating through null should error");
}

#[test]
fn path_through_scalar() {
    let val = FlexValue::from(serde_json::json!({"a": 42}));

    let result = val.at("a.b");
    println!("Path through scalar: {:?}", result);
    assert!(result.is_err(), "navigating through scalar should error");
}

#[test]
fn array_index_out_of_bounds() {
    let val = FlexValue::from(serde_json::json!({"items": [1, 2, 3]}));

    let result = val.at("items[99]");
    println!("Index out of bounds: {:?}", result);
    assert!(result.is_err(), "out-of-bounds index should error");
}

#[test]
fn quoted_key_with_dots() {
    // Quoted key containing dots: meta["content.type"]
    let val = FlexValue::from(serde_json::json!({"meta": {"content.type": "text/html"}}));

    let result: String = val.extract("meta[\"content.type\"]").unwrap();
    assert_eq!(result, "text/html", "quoted key with dots should work");
}

#[test]
fn maybe_on_missing_path() {
    let val = FlexValue::from(serde_json::json!({"a": 1}));

    let result: Option<i64> = val.maybe("nonexistent.path").unwrap();
    println!("maybe on missing: {:?}", result);
    assert_eq!(
        result, None,
        "maybe should return Ok(None) for missing path"
    );
}

#[test]
fn maybe_on_existing_path() {
    let val = FlexValue::from(serde_json::json!({"a": {"b": 42}}));

    let result: Option<i64> = val.maybe("a.b").unwrap();
    assert_eq!(
        result,
        Some(42),
        "maybe should return Ok(Some(42)) for existing path"
    );
}

#[test]
fn maybe_on_null_value() {
    let val = FlexValue::from(serde_json::json!({"a": null}));

    let result: Option<i64> = val.maybe("a").unwrap();
    assert_eq!(result, None, "maybe should return Ok(None) for null value");
}

#[test]
fn maybe_through_null_intermediate() {
    // Path "a.b" where "a" is null — should return Ok(None) not an error
    let val = FlexValue::from(serde_json::json!({"a": null}));

    let result: Option<i64> = val.maybe("a.b").unwrap();
    println!("maybe through null: {:?}", result);
    assert_eq!(
        result, None,
        "maybe through null intermediate should return Ok(None)"
    );
}
