// Iteration 161: SourceHint::Env with value containing equals sign
// Target #99 — "KEY=VALUE" as env var value. Does the equals sign
// interfere with extraction or coercion?

use laminate::{value::SourceHint, FlexValue};

#[test]
fn env_value_with_equals_sign() {
    // An env var whose VALUE contains an equals sign (e.g., FOO="bar=baz")
    let val = FlexValue::from_json(r#"{"config": "bar=baz"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Env);

    // Should extract as plain string without mangling
    let result: String = val.extract("config").unwrap();
    assert_eq!(result, "bar=baz");
}

#[test]
fn env_value_with_multiple_equals() {
    // More complex: multiple equals signs, like a base64 string
    let val = FlexValue::from_json(r#"{"token": "eyJhbGciOiJIUzI1NiJ9.e30=.abc=="}"#)
        .unwrap()
        .with_source_hint(SourceHint::Env);

    let result: String = val.extract("token").unwrap();
    assert_eq!(result, "eyJhbGciOiJIUzI1NiJ9.e30=.abc==");
}

#[test]
fn env_numeric_value_with_equals_coercion() {
    // Env hint enables BestEffort coercion — but "8080" should still coerce to u16
    let val = FlexValue::from_json(r#"{"port": "8080"}"#)
        .unwrap()
        .with_source_hint(SourceHint::Env);

    let port: u16 = val.extract("port").unwrap();
    assert_eq!(port, 8080);
}
