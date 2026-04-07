/// Iteration 219: #[laminate(flatten)] — which keys does it consume?
///
/// Target #219: flatten deserializes from remaining map after regular fields
/// are extracted. Test: does flatten only see leftover keys? What happens
/// if the flatten struct has a field with same name as an already-extracted field?
use laminate::Laminate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Inner {
    version: i64,
    author: String,
}

#[derive(Debug, Laminate)]
struct Document {
    title: String,
    version: i64, // Regular field — extracted first, removes "version" from map
    #[laminate(flatten)]
    meta: Inner, // Flatten sees remaining map — "version" already removed!
}

#[test]
fn flatten_after_regular_field_removes_key() {
    // Both Document.version and Inner.version want "version"
    // Regular field extraction happens first (removes "version" from map)
    // Then flatten sees the remaining map — without "version"
    let json = r#"{"title": "Hello", "version": 42, "author": "Alice"}"#;

    let result = Document::from_json(json);
    println!("result = {:?}", result);

    // Observed: Err(DeserializeError { path: "(flatten)", source: "missing field `version`" })
    // Regular field extraction consumed "version" before flatten saw the map.
    // This is correct behavior — overlapping keys between regular and flatten should error.
    assert!(
        result.is_err(),
        "overlapping key between regular field and flatten should error"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("version"),
        "error should mention the missing field: {}",
        err
    );
}

/// Separate test: flatten with no key overlap
#[derive(Debug, Deserialize, Serialize)]
struct Tags {
    color: String,
    priority: i64,
}

#[derive(Debug, Laminate)]
struct Item {
    name: String,
    #[laminate(flatten)]
    tags: Tags,
}

#[test]
fn flatten_no_overlap_clean() {
    let json = r#"{"name": "widget", "color": "red", "priority": 1}"#;
    let (item, diagnostics) = Item::from_json(json).unwrap();

    println!("item = {:?}", item);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(item.name, "widget");
    assert_eq!(item.tags.color, "red");
    assert_eq!(item.tags.priority, 1);

    // No dropped diagnostics expected
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, laminate::diagnostic::DiagnosticKind::Dropped { .. }))
        .collect();
    assert!(
        dropped.is_empty(),
        "no unknown fields, no drops: {:?}",
        dropped
    );
}
