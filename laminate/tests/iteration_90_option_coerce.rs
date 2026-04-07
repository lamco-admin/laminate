/// Iteration 90: Derive struct Option<String> + #[laminate(coerce)]
///
/// Fixed two issues: (1) rsplit("::") corrupted Option<String> to "String>",
/// preventing coercion. (2) Option fields need null→None handling before
/// coercion, otherwise type_hint_from_syn unwraps Option to inner type
/// and Null→Default produces Some("") instead of None.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    #[laminate(coerce)]
    label: Option<String>,
    #[laminate(coerce)]
    count: Option<i64>,
}

#[test]
fn option_string_coerced_from_number() {
    let json = r#"{"label": 42, "count": 42}"#;
    let (config, diags) = Config::from_json(json).unwrap();
    assert_eq!(
        config.label,
        Some("42".to_string()),
        "Number should coerce to String"
    );
    assert_eq!(config.count, Some(42));
    // Should produce a coercion diagnostic for label (number→string)
    assert!(diags
        .iter()
        .any(|d| format!("{:?}", d.kind).contains("Coerced")));
}

#[test]
fn option_null_produces_none() {
    let json = r#"{"label": null, "count": null}"#;
    let (config, _) = Config::from_json(json).unwrap();
    assert_eq!(
        config.label, None,
        "null should produce None, not Some(\"\")"
    );
    assert_eq!(config.count, None, "null should produce None, not Some(0)");
}

#[test]
fn option_absent_produces_none() {
    // Missing fields on Option with coerce should produce None
    let json = r#"{}"#;
    let (config, _) = Config::from_json(json).unwrap();
    assert_eq!(config.label, None);
    assert_eq!(config.count, None);
}
