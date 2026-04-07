/// guess_type() tests — detect what type an unknown string represents.
use laminate::detect::{guess_type, GuessedType};

#[test]
fn detect_integer() {
    let guesses = guess_type("42");
    assert_eq!(guesses[0].kind, GuessedType::Integer);
    assert!(guesses[0].confidence > 0.9);
}

#[test]
fn detect_float() {
    let guesses = guess_type("3.14");
    assert_eq!(guesses[0].kind, GuessedType::Float);
}

#[test]
fn detect_boolean() {
    let guesses = guess_type("true");
    assert_eq!(guesses[0].kind, GuessedType::Boolean);

    let guesses = guess_type("Yes");
    assert_eq!(guesses[0].kind, GuessedType::Boolean);
}

#[test]
fn detect_date() {
    let guesses = guess_type("2026-04-02");
    assert!(matches!(guesses[0].kind, GuessedType::Date(_)));
}

#[test]
fn detect_currency() {
    let guesses = guess_type("$12.99");
    assert_eq!(guesses[0].kind, GuessedType::Currency);
}

#[test]
fn detect_unit_value() {
    let guesses = guess_type("5.2 kg");
    assert!(matches!(guesses[0].kind, GuessedType::UnitValue(_)));
}

#[test]
fn detect_uuid() {
    let guesses = guess_type("550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(guesses[0].kind, GuessedType::Uuid);
}

#[test]
fn detect_email() {
    let guesses = guess_type("alice@example.com");
    assert_eq!(guesses[0].kind, GuessedType::Email);
}

#[test]
fn detect_url() {
    let guesses = guess_type("https://example.com/path");
    assert_eq!(guesses[0].kind, GuessedType::Url);
}

#[test]
fn detect_ip_v4() {
    let guesses = guess_type("192.168.1.1");
    assert!(guesses.iter().any(|g| g.kind == GuessedType::IpAddress));
}

#[test]
fn detect_json_object() {
    let guesses = guess_type(r#"{"key": "value"}"#);
    assert_eq!(guesses[0].kind, GuessedType::Json);
}

#[test]
fn detect_null_sentinel() {
    let guesses = guess_type("N/A");
    assert_eq!(guesses[0].kind, GuessedType::NullSentinel);

    let guesses = guess_type("null");
    assert_eq!(guesses[0].kind, GuessedType::NullSentinel);
}

#[test]
fn detect_plain_string() {
    let guesses = guess_type("hello world");
    assert_eq!(guesses[0].kind, GuessedType::PlainString);
}

#[test]
fn multiple_guesses_sorted_by_confidence() {
    // "1" is both integer and boolean
    let guesses = guess_type("1");
    assert!(guesses.len() >= 2);
    assert!(guesses[0].confidence >= guesses[1].confidence);
}

#[test]
fn ambiguous_date() {
    let guesses = guess_type("01/02/2026");
    let date_guess = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::Date(_)));
    assert!(date_guess.is_some());
    // Ambiguous dates should have lower confidence
    assert!(date_guess.unwrap().confidence < 0.8);
}

#[test]
fn iter118_empty_string_is_null_sentinel() {
    // Empty string is the #1 most common null sentinel in CSV/DB exports.
    // pandas treats "" as NaN by default. guess_type("") should return
    // NullSentinel, not PlainString — the previous early-return was wrong.
    let guesses = guess_type("");
    assert_eq!(
        guesses[0].kind,
        GuessedType::NullSentinel,
        "empty string should be NullSentinel"
    );
    assert!(guesses[0].confidence >= 0.9, "confidence should be high");

    // Whitespace-only string: after trim it's also empty → NullSentinel
    let guesses_ws = guess_type("   ");
    assert_eq!(
        guesses_ws[0].kind,
        GuessedType::NullSentinel,
        "whitespace-only should be NullSentinel"
    );

    // Tab-only string
    let guesses_tab = guess_type("\t\t");
    assert_eq!(
        guesses_tab[0].kind,
        GuessedType::NullSentinel,
        "tab-only should be NullSentinel"
    );

    // Non-empty strings should NOT be affected by this change
    let guesses_hello = guess_type("hello");
    assert_eq!(
        guesses_hello[0].kind,
        GuessedType::PlainString,
        "normal string still PlainString"
    );
}
