use laminate::diagnostic::DiagnosticKind;
/// Iteration 102: merge_with_diagnostics — Overridden diagnostic kind
///
/// Merge value replacement now uses DiagnosticKind::Overridden instead of
/// Coerced. The from_type/to_type use type names ("number", "string")
/// instead of raw JSON representations.
use laminate::FlexValue;

#[test]
fn merge_override_uses_correct_diagnostic_kind() {
    let base = FlexValue::from(serde_json::json!({"count": 42, "name": "original"}));
    let overlay = FlexValue::from(serde_json::json!({"count": "forty-two", "extra": true}));

    let (merged, diagnostics) = base.merge_with_diagnostics(&overlay);

    // count should be overridden
    let count: String = merged.extract("count").unwrap();
    assert_eq!(count, "forty-two");

    // Diagnostic should use Overridden, not Coerced
    let count_diag = diagnostics.iter().find(|d| d.path == "count").unwrap();
    match &count_diag.kind {
        DiagnosticKind::Overridden { from_type, to_type } => {
            assert_eq!(from_type, "number", "should report source type");
            assert_eq!(to_type, "string", "should report target type");
        }
        other => panic!("expected Overridden, got {:?}", other),
    }

    // "extra" should be Preserved (new field)
    let extra_diag = diagnostics.iter().find(|d| d.path == "extra").unwrap();
    assert!(matches!(&extra_diag.kind, DiagnosticKind::Preserved { .. }));

    // "name" should not produce a diagnostic (unchanged)
    assert!(diagnostics.iter().find(|d| d.path == "name").is_none());
}
