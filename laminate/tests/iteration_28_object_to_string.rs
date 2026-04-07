//! Iteration 28: GAP — Object/Array to String coercion missing
//! Fix: Added Object/Array→String arm at BestEffort via JSON serialization.

use laminate::FlexValue;

#[test]
fn iter28_object_to_string_serializes_json() {
    let json = serde_json::json!({"config": {"debug": true, "level": 3}});
    let flex = FlexValue::new(json);

    let s: String = flex.extract("config").unwrap();
    assert_eq!(s, r#"{"debug":true,"level":3}"#);
}

#[test]
fn iter28_array_to_string_serializes_json() {
    let json = serde_json::json!({"tags": ["rust", "serde"]});
    let flex = FlexValue::new(json);

    let s: String = flex.extract("tags").unwrap();
    assert_eq!(s, r#"["rust","serde"]"#);
}

#[test]
fn iter28_empty_object_and_array_to_string() {
    let json = serde_json::json!({"obj": {}, "arr": []});
    let flex = FlexValue::new(json);

    assert_eq!(flex.extract::<String>("obj").unwrap(), "{}");
    assert_eq!(flex.extract::<String>("arr").unwrap(), "[]");
}

#[test]
fn iter28_object_to_string_produces_warning() {
    let json = serde_json::json!({"val": {"a": 1}});
    let flex = FlexValue::new(json);

    let (s, diags) = flex.extract_with_diagnostics::<String>("val").unwrap();
    assert_eq!(s, r#"{"a":1}"#);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].risk, laminate::RiskLevel::Warning);
    assert!(matches!(
        diags[0].kind,
        laminate::DiagnosticKind::Coerced { .. }
    ));
}
