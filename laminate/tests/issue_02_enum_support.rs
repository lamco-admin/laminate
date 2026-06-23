//! Tests for issue #2: `#[derive(Laminate)]` on string-valued enums, with an
//! optional `#[laminate(unknown)]` fallback variant that captures unrecognized
//! values gracefully.

use laminate::{DiagnosticKind, RiskLevel};
use laminate_derive::Laminate;

#[derive(Debug, PartialEq, Laminate)]
enum Status {
    Active,
    #[laminate(rename = "in_progress")]
    InProgress,
    #[laminate(unknown)]
    Other(String),
}

#[test]
fn known_variant_matches_cleanly() {
    let (s, diags) = Status::from_flex_value(&serde_json::json!("Active")).unwrap();
    assert_eq!(s, Status::Active);
    assert!(diags.is_empty());
}

#[test]
fn rename_is_respected() {
    let (s, diags) = Status::from_flex_value(&serde_json::json!("in_progress")).unwrap();
    assert_eq!(s, Status::InProgress);
    assert!(diags.is_empty());
}

#[test]
fn unknown_value_is_captured_with_diagnostic() {
    let (s, diags) = Status::from_flex_value(&serde_json::json!("paused")).unwrap();
    assert_eq!(s, Status::Other("paused".to_string()));
    assert_eq!(diags.len(), 1, "expected one diagnostic, got: {diags:?}");
    assert!(matches!(diags[0].kind, DiagnosticKind::Coerced { .. }));
    assert_eq!(diags[0].risk, RiskLevel::Warning);
}

#[test]
fn from_json_parses_a_json_string() {
    let (s, _) = Status::from_json(r#""Active""#).unwrap();
    assert_eq!(s, Status::Active);
}

#[test]
fn non_string_input_errors() {
    assert!(Status::from_flex_value(&serde_json::json!(42)).is_err());
}

#[test]
fn shape_strict_rejects_unknown_capture() {
    assert!(Status::shape_strict(&serde_json::json!("Active")).is_ok());
    assert!(Status::shape_strict(&serde_json::json!("paused")).is_err());
}

#[test]
fn shape_lenient_captures_unknown() {
    let r = Status::shape_lenient(&serde_json::json!("paused")).unwrap();
    assert_eq!(r.value, Status::Other("paused".to_string()));
    assert_eq!(r.diagnostics.len(), 1);
}

// An enum WITHOUT an unknown fallback: unrecognized values must error.
#[derive(Debug, PartialEq, Laminate)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn no_fallback_errors_on_unrecognized() {
    assert!(Color::from_flex_value(&serde_json::json!("Red")).is_ok());
    assert!(Color::from_flex_value(&serde_json::json!("purple")).is_err());
}
