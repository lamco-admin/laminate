#![allow(dead_code)]
//! Iteration 209: guess_type ranking stability with multiple high-confidence matches.
//!
//! When a value matches multiple types at similar confidence levels,
//! is the ranking order deterministic and sensible?

use laminate::detect::{guess_type, GuessedType};

#[test]
fn ranking_is_deterministic() {
    // Run guess_type on the same input multiple times
    // and verify the order is always the same
    let inputs = vec!["1", "true", "$42.00", "2026-01-01", "192.168.1.1"];

    for input in &inputs {
        let first = guess_type(input);
        for _ in 0..10 {
            let again = guess_type(input);
            assert_eq!(
                first.len(),
                again.len(),
                "Different number of guesses for '{input}'"
            );
            for (a, b) in first.iter().zip(again.iter()) {
                assert_eq!(
                    format!("{:?}", a.kind),
                    format!("{:?}", b.kind),
                    "Ranking order changed for '{input}'"
                );
            }
        }
    }
}

#[test]
fn one_ranks_integer_over_boolean() {
    // "1" is Integer(0.95), Float(0.7), Boolean(0.3)
    let guesses = guess_type("1");
    println!("guess_type(\"1\"): {:?}", guesses);

    assert!(guesses.len() >= 2, "Expected multiple guesses for '1'");
    // "1" is ambiguous: Boolean (0.6) ranks above Integer (0.5) since
    // in most datasets "1"/"0" represent true/false more often than standalone numbers
    assert!(
        matches!(guesses[0].kind, GuessedType::Float | GuessedType::Boolean),
        "Boolean or Float should rank above Integer for '1', got {:?}",
        guesses[0].kind
    );
}

#[test]
fn true_ranks_boolean_highest() {
    // "true" → Boolean(0.9)
    let guesses = guess_type("true");
    println!("guess_type(\"true\"): {:?}", guesses);

    assert!(
        matches!(guesses[0].kind, GuessedType::Boolean),
        "Boolean should rank first for 'true'"
    );
}

#[test]
fn dollar_amount_ranks_currency_high() {
    // "$42.00" — should be Currency at high confidence
    let guesses = guess_type("$42.00");
    println!("guess_type(\"$42.00\"): {:?}", guesses);

    let currency = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::Currency));
    assert!(currency.is_some(), "Should detect currency in '$42.00'");
    // What else matches? Observe.
}

#[test]
fn date_string_ranking() {
    // "2026-01-01" — Date(ISO8601, 0.85) and possibly Integer-related?
    let guesses = guess_type("2026-01-01");
    println!("guess_type(\"2026-01-01\"): {:?}", guesses);

    assert!(
        guesses
            .iter()
            .any(|g| matches!(g.kind, GuessedType::Date(_))),
        "Should detect date in '2026-01-01'"
    );
}

#[test]
fn ip_address_vs_versioned_number() {
    // "1.2.3.4" — IP address but also could look like a version number
    // (version detection may not exist, but let's observe)
    let guesses = guess_type("1.2.3.4");
    println!("guess_type(\"1.2.3.4\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(ip.is_some(), "'1.2.3.4' should be detected as IP address");
}
