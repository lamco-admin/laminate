use laminate::diagnostic::{DiagnosticKind, RiskLevel};
/// Iteration 112: merge — structural replacement gets Warning risk
///
/// When a merge replaces an object/array with a scalar, the risk is
/// now Warning (not Info), since it destroys nested data structure.
use laminate::FlexValue;

#[test]
fn merge_object_to_scalar_is_warning() {
    let base = FlexValue::from(serde_json::json!({"a": {"b": 1, "c": 2}}));
    let overlay = FlexValue::from(serde_json::json!({"a": 42}));

    let (_, diagnostics) = base.merge_with_diagnostics(&overlay);
    let d = diagnostics.iter().find(|d| d.path == "a").unwrap();

    assert_eq!(
        d.risk,
        RiskLevel::Warning,
        "structural replacement should be Warning"
    );
    assert!(d
        .suggestion
        .as_deref()
        .unwrap()
        .contains("nested data lost"));
    match &d.kind {
        DiagnosticKind::Overridden { from_type, to_type } => {
            assert_eq!(from_type, "object");
            assert_eq!(to_type, "number");
        }
        _ => panic!("expected Overridden"),
    }
}

#[test]
fn merge_scalar_to_scalar_stays_info() {
    let base = FlexValue::from(serde_json::json!({"a": 42}));
    let overlay = FlexValue::from(serde_json::json!({"a": "hello"}));

    let (_, diagnostics) = base.merge_with_diagnostics(&overlay);
    let d = diagnostics.iter().find(|d| d.path == "a").unwrap();

    assert_eq!(d.risk, RiskLevel::Info, "scalar→scalar should stay Info");
}
