//! Iteration 252: merge() with complex type conflicts
//!
//! What happens when merging objects with conflicting types at the same key?
//! - Object key exists as scalar in base, object in overlay → overlay wins
//! - Array in base, scalar in overlay → overlay replaces
//! - Null in overlay → should it null-out or preserve base?
//! - Deep merge with mixed depth objects

use laminate::FlexValue;

#[test]
fn merge_scalar_replaced_by_object() {
    let base = FlexValue::from_json(r#"{"config": 42}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"config": {"debug": true}}"#).unwrap();

    let merged = base.merge(&overlay);
    let debug: bool = merged.extract("config.debug").unwrap();
    assert_eq!(debug, true, "object should replace scalar");
}

#[test]
fn merge_object_replaced_by_scalar() {
    let base = FlexValue::from_json(r#"{"config": {"debug": true}}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"config": "simple"}"#).unwrap();

    let merged = base.merge(&overlay);
    let config: String = merged.extract("config").unwrap();
    assert_eq!(config, "simple", "scalar should replace object");
}

#[test]
fn merge_array_replaced_by_scalar() {
    let base = FlexValue::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"items": "none"}"#).unwrap();

    let merged = base.merge(&overlay);
    let items: String = merged.extract("items").unwrap();
    assert_eq!(items, "none");
}

#[test]
fn merge_null_overlay_replaces_base() {
    let base = FlexValue::from_json(r#"{"a": 42}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"a": null}"#).unwrap();

    let merged = base.merge(&overlay);
    println!("null overlay: {:?}", merged.raw());
    // null in overlay should replace the base value
    assert!(
        merged.at("a").unwrap().raw().is_null(),
        "null overlay should replace base"
    );
}

#[test]
fn merge_preserves_unmentioned_keys() {
    let base = FlexValue::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"b": 99}"#).unwrap();

    let merged = base.merge(&overlay);
    assert_eq!(
        merged.extract::<i64>("a").unwrap(),
        1,
        "unmentioned key 'a' preserved"
    );
    assert_eq!(
        merged.extract::<i64>("b").unwrap(),
        99,
        "mentioned key 'b' overridden"
    );
    assert_eq!(
        merged.extract::<i64>("c").unwrap(),
        3,
        "unmentioned key 'c' preserved"
    );
}

#[test]
fn merge_deep_recursive() {
    let base = FlexValue::from_json(
        r#"{
        "db": {"host": "localhost", "port": 5432, "options": {"pool_size": 5, "timeout": 30}}
    }"#,
    )
    .unwrap();
    let overlay = FlexValue::from_json(
        r#"{
        "db": {"port": 5433, "options": {"timeout": 60}}
    }"#,
    )
    .unwrap();

    let merged = base.merge(&overlay);

    assert_eq!(
        merged.extract::<String>("db.host").unwrap(),
        "localhost",
        "deep: host preserved"
    );
    assert_eq!(
        merged.extract::<i64>("db.port").unwrap(),
        5433,
        "deep: port overridden"
    );
    assert_eq!(
        merged.extract::<i64>("db.options.pool_size").unwrap(),
        5,
        "deep: pool_size preserved"
    );
    assert_eq!(
        merged.extract::<i64>("db.options.timeout").unwrap(),
        60,
        "deep: timeout overridden"
    );
}

#[test]
fn merge_shallow_replaces_objects() {
    let base = FlexValue::from_json(
        r#"{
        "db": {"host": "localhost", "port": 5432}
    }"#,
    )
    .unwrap();
    let overlay = FlexValue::from_json(
        r#"{
        "db": {"port": 5433}
    }"#,
    )
    .unwrap();

    let merged = base.merge_shallow(&overlay);

    // Shallow merge: entire "db" replaced, so "host" should be gone
    assert_eq!(merged.extract::<i64>("db.port").unwrap(), 5433);
    let host_result: Result<String, _> = merged.extract("db.host");
    assert!(
        host_result.is_err(),
        "shallow merge should replace entire object, losing 'host'"
    );
}

#[test]
fn merge_with_diagnostics_reports_changes() {
    let base = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"b": 99, "c": 3}"#).unwrap();

    let (merged, diags) = base.merge_with_diagnostics(&overlay);

    println!("Merge diagnostics: {:?}", diags);
    assert!(
        !diags.is_empty(),
        "merge should produce diagnostics for changes"
    );

    assert_eq!(merged.extract::<i64>("a").unwrap(), 1);
    assert_eq!(merged.extract::<i64>("b").unwrap(), 99);
    assert_eq!(merged.extract::<i64>("c").unwrap(), 3);
}

#[test]
fn merge_empty_objects() {
    let base = FlexValue::from_json(r#"{}"#).unwrap();
    let overlay = FlexValue::from_json(r#"{"a": 1}"#).unwrap();

    let merged = base.merge(&overlay);
    assert_eq!(merged.extract::<i64>("a").unwrap(), 1);

    // Reverse
    let merged2 = overlay.merge(&base);
    assert_eq!(
        merged2.extract::<i64>("a").unwrap(),
        1,
        "merge with empty preserves all"
    );
}
