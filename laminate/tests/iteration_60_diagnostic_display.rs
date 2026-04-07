//! Iteration 60: Diagnostic Display format — human-readable, actionable messages.

use laminate::FlexValue;

#[test]
fn diagnostic_display_is_human_readable() {
    let json = r#"{"name":"Alice","age":"30","score":true,"count":null}"#;
    let fv = FlexValue::from_json(json).unwrap();

    // String→Integer coercion
    let (_val, diags) = fv.extract_with_diagnostics::<i64>("age").unwrap();
    let msg = diags[0].to_string();
    println!("String→Int: {msg}");
    assert!(msg.contains("[info]"), "should use lowercase risk level");
    assert!(
        msg.contains("coerced string → i64"),
        "should use arrow notation"
    );
    assert!(!msg.contains("Coerced {"), "should not use Debug format");

    // Bool→Integer coercion
    let (_val, diags) = fv.extract_with_diagnostics::<i64>("score").unwrap();
    let msg = diags[0].to_string();
    println!("Bool→Int: {msg}");
    assert!(msg.contains("coerced bool → i64"));

    // Null→Default
    let (_val, diags) = fv.extract_with_diagnostics::<i64>("count").unwrap();
    let msg = diags[0].to_string();
    println!("Null→Default: {msg}");
    assert!(msg.contains("[warning]"));
    assert!(msg.contains("defaulted field"));
    assert!(
        !msg.contains("Suggestion:"),
        "should not have redundant label"
    );

    // Hex string
    let hex_json = r#"{"code":"0xFF"}"#;
    let fv2 = FlexValue::from_json(hex_json).unwrap();
    let (_val, diags) = fv2.extract_with_diagnostics::<i64>("code").unwrap();
    let msg = diags[0].to_string();
    println!("Hex→Int: {msg}");
    assert!(msg.contains("[warning]"));
    assert!(msg.contains("coerced"));
}

#[test]
fn diagnostic_kind_display() {
    use laminate::DiagnosticKind;

    let coerced = DiagnosticKind::Coerced {
        from: "string".into(),
        to: "i64".into(),
    };
    assert_eq!(coerced.to_string(), "coerced string → i64");

    let defaulted = DiagnosticKind::Defaulted {
        field: "age".into(),
        value: "null → default".into(),
    };
    assert_eq!(
        defaulted.to_string(),
        "defaulted field 'age' (null → default)"
    );

    let dropped = DiagnosticKind::Dropped {
        field: "extra".into(),
    };
    assert_eq!(dropped.to_string(), "dropped unknown field 'extra'");

    let preserved = DiagnosticKind::Preserved {
        field: "meta".into(),
    };
    assert_eq!(
        preserved.to_string(),
        "preserved unknown field 'meta' in overflow"
    );

    let error_defaulted = DiagnosticKind::ErrorDefaulted {
        field: "x".into(),
        error: "parse failed".into(),
    };
    assert_eq!(
        error_defaulted.to_string(),
        "field 'x' failed (parse failed), used default"
    );
}
