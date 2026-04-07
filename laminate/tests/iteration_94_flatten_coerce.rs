/// Iteration 94: #[laminate(flatten)] — false "Dropped" diagnostics
///
/// Flatten used map.iter() (read-only) to build the flat value, so consumed
/// keys remained in the map and were flagged as "unknown/dropped" by the
/// overflow handler. Fixed by serializing the flattened result back and
/// removing consumed keys from the map.
use laminate::Laminate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Metadata {
    version: i64,
    author: String,
}

#[derive(Debug, Laminate)]
struct Document {
    title: String,
    #[laminate(flatten)]
    meta: Metadata,
}

#[test]
fn flatten_no_false_dropped_diagnostics() {
    let json = r#"{"title": "Hello", "version": 1, "author": "Alice"}"#;
    let (doc, diagnostics) = Document::from_json(json).unwrap();

    assert_eq!(doc.title, "Hello");
    assert_eq!(doc.meta.version, 1);
    assert_eq!(doc.meta.author, "Alice");

    // No diagnostics should be emitted — all fields are accounted for
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, laminate::diagnostic::DiagnosticKind::Dropped { .. }))
        .collect();
    assert!(
        dropped.is_empty(),
        "flatten should not produce Dropped diagnostics for consumed fields, got: {:?}",
        dropped
    );
}

#[test]
fn flatten_with_extra_unknown_field() {
    // "debug" is not in title, version, or author — should be dropped
    let json = r#"{"title": "Hello", "version": 1, "author": "Alice", "debug": true}"#;
    let (doc, diagnostics) = Document::from_json(json).unwrap();

    assert_eq!(doc.title, "Hello");
    assert_eq!(doc.meta.version, 1);

    // Only "debug" should be dropped — version and author should NOT be
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, laminate::diagnostic::DiagnosticKind::Dropped { .. }))
        .collect();
    assert_eq!(
        dropped.len(),
        1,
        "only truly unknown fields should be dropped, got: {:?}",
        dropped
    );
}
