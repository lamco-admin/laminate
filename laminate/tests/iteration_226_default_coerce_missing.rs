/// Iteration 226: #[laminate(default, coerce)] on missing field
///
/// Target #226: When default + coerce are combined:
/// - Missing field → Default (no coercion attempt)
/// - Present + coercible → coerce
/// - Present + uncoercible → ErrorDefaulted diagnostic + Default
/// - Null → Default
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(default, coerce)]
    port: i64, // default = 0
    #[laminate(default, coerce)]
    debug: bool, // default = false
    #[laminate(default, coerce)]
    label: String, // default = ""
}

#[test]
fn default_coerce_missing_field() {
    let json = r#"{"name": "app"}"#;
    let (config, diagnostics) = Config::from_json(json).unwrap();

    println!("config = {:?}", config);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(config.name, "app");
    assert_eq!(config.port, 0, "missing → default 0");
    assert_eq!(config.debug, false, "missing → default false");
    assert_eq!(config.label, "", "missing → default empty string");
    assert!(
        diagnostics.is_empty(),
        "no diagnostics for default on missing"
    );
}

#[test]
fn default_coerce_null_field() {
    let json = r#"{"name": "app", "port": null, "debug": null, "label": null}"#;
    let (config, diagnostics) = Config::from_json(json).unwrap();

    println!("config = {:?}", config);
    assert_eq!(config.port, 0, "null → default 0");
    assert_eq!(config.debug, false, "null → default false");
    assert!(diagnostics.is_empty(), "no diagnostics for default on null");
}

#[test]
fn default_coerce_coercible_value() {
    let json = r#"{"name": "app", "port": "8080", "debug": "true"}"#;
    let (config, diagnostics) = Config::from_json(json).unwrap();

    println!("config = {:?}", config);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(config.port, 8080, "string coerced to i64");
    assert_eq!(config.debug, true, "string coerced to bool");
    assert!(diagnostics.len() >= 2, "should have coercion diagnostics");
}

#[test]
fn default_coerce_uncoercible_value_falls_back() {
    // "not-a-number" can't be coerced to i64 → ErrorDefaulted → default 0
    let json = r#"{"name": "app", "port": "not-a-number"}"#;
    let (config, diagnostics) = Config::from_json(json).unwrap();

    println!("config = {:?}", config);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(config.port, 0, "uncoercible → default");

    // Should have ErrorDefaulted diagnostic
    let error_defaulted: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.kind,
                laminate::diagnostic::DiagnosticKind::ErrorDefaulted { .. }
            )
        })
        .collect();
    println!("error_defaulted = {:?}", error_defaulted);
    assert!(
        !error_defaulted.is_empty(),
        "should emit ErrorDefaulted diagnostic"
    );
}
