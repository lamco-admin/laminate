use laminate::Laminate;
use serde::{Deserialize, Serialize};
/// Iteration 220: #[laminate(flatten)] + #[laminate(overflow)] interaction
///
/// Target #220: When both flatten and overflow are present, flatten should
/// consume its keys first, then overflow should capture the true remainder.
use std::collections::HashMap;

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
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn flatten_then_overflow() {
    let json = r#"{
        "title": "Hello",
        "version": 1,
        "author": "Alice",
        "debug": true,
        "tags": ["rust", "test"]
    }"#;

    let result = Document::from_json(json);
    println!("result = {:?}", result);

    match result {
        Ok((doc, diagnostics)) => {
            println!("doc.title = {}", doc.title);
            println!("doc.meta = {:?}", doc.meta);
            println!("doc.extra = {:?}", doc.extra);
            println!("diagnostics = {:?}", diagnostics);

            assert_eq!(doc.title, "Hello");
            assert_eq!(doc.meta.version, 1);
            assert_eq!(doc.meta.author, "Alice");

            // Overflow should only have "debug" and "tags" — not version/author
            assert!(
                !doc.extra.contains_key("version"),
                "version consumed by flatten"
            );
            assert!(
                !doc.extra.contains_key("author"),
                "author consumed by flatten"
            );
            assert!(doc.extra.contains_key("debug"), "debug is truly unknown");
            assert!(doc.extra.contains_key("tags"), "tags is truly unknown");
            assert_eq!(doc.extra.len(), 2);
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
            panic!("Should succeed — flatten + overflow should cooperate");
        }
    }
}

/// Edge case: flatten consumes ALL leftover keys, nothing for overflow
#[test]
fn flatten_consumes_everything_overflow_empty() {
    let json = r#"{"title": "Hello", "version": 1, "author": "Alice"}"#;
    let (doc, diagnostics) = Document::from_json(json).unwrap();

    println!("extra = {:?}", doc.extra);
    println!("diagnostics = {:?}", diagnostics);

    assert!(doc.extra.is_empty(), "no unknown fields for overflow");
    // No Preserved or Dropped diagnostics expected
    assert!(
        diagnostics.is_empty(),
        "no diagnostics expected: {:?}",
        diagnostics
    );
}
