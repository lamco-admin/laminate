#![allow(dead_code, unused_imports, unused_must_use)]
/// Iteration 95: Flatten with #[serde(skip_serializing)] — known limitation
///
/// The round-trip probe (to_value) used to discover consumed flatten keys
/// misses fields with #[serde(skip_serializing)]. These fields are consumed
/// during deserialization but absent from serialization, causing false
/// "Dropped" diagnostics. This is a known limitation documented in the
/// derive macro code.
use laminate::Laminate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct SecretMeta {
    visible: String,
    #[serde(skip_serializing)]
    secret: String,
}

#[derive(Debug, Laminate)]
struct Wrapper {
    name: String,
    #[laminate(flatten)]
    meta: SecretMeta,
}

#[test]
fn flatten_skip_serializing_known_limitation() {
    let json = r#"{"name": "test", "visible": "yes", "secret": "password123"}"#;
    let (wrapper, diagnostics) = Wrapper::from_json(json).unwrap();

    // The struct correctly captures the secret field
    assert_eq!(wrapper.meta.secret, "password123");
    assert_eq!(wrapper.meta.visible, "yes");

    // Known limitation: "secret" appears as Dropped because to_value()
    // doesn't include skip_serializing fields
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, laminate::diagnostic::DiagnosticKind::Dropped { .. }))
        .collect();
    // This documents the known limitation — secret is falsely dropped
    assert_eq!(
        dropped.len(),
        1,
        "known limitation: skip_serializing fields produce false Dropped"
    );
}
