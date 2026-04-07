/// Iteration 222: #[laminate(skip)] — field ignored, uses Default
///
/// Target #222: skip field never reads from JSON. What happens when JSON
/// contains a key matching the skip field's name? Is it "unknown"?
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Record {
    name: String,
    #[laminate(skip)]
    computed: String, // Never read from JSON, always Default::default()
}

#[test]
fn skip_field_gets_default() {
    let json = r#"{"name": "test"}"#;
    let (record, diagnostics) = Record::from_json(json).unwrap();

    println!("record = {:?}", record);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(record.name, "test");
    assert_eq!(
        record.computed, "",
        "skip field should be Default::default() = empty string"
    );
    assert!(diagnostics.is_empty(), "no diagnostics expected");
}

#[test]
fn skip_field_key_in_json_becomes_unknown() {
    // JSON has "computed" key, but field is skipped — what happens?
    let json = r#"{"name": "test", "computed": "should-be-ignored"}"#;
    let (record, diagnostics) = Record::from_json(json).unwrap();

    println!("record = {:?}", record);
    println!("diagnostics = {:?}", diagnostics);

    // The skip field should still be default
    assert_eq!(record.computed, "", "skip field ignores JSON value");

    // The "computed" key in JSON should be treated as unknown → Dropped diagnostic
    let dropped: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, laminate::diagnostic::DiagnosticKind::Dropped { .. }))
        .collect();
    println!("dropped = {:?}", dropped);
    // This is the interesting part — is "computed" flagged as unknown?
}

/// Skip with non-String Default type
#[derive(Debug, Laminate)]
struct WithCounter {
    label: String,
    #[laminate(skip)]
    count: u32, // Default::default() = 0
}

#[test]
fn skip_field_non_string_default() {
    let json = r#"{"label": "items"}"#;
    let (record, _) = WithCounter::from_json(json).unwrap();

    assert_eq!(record.label, "items");
    assert_eq!(record.count, 0, "u32 default is 0");
}
