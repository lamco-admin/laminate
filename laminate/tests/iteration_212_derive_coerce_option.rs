#![allow(dead_code, unused_imports)]
//! Iteration 212: #[laminate(coerce)] on Option<T>.
//!
//! - JSON null → None (standard)
//! - String "null" → null sentinel → None? Or error?
//! - String "42" → Some(42)?
//! - String "abc" → error (can't coerce to i64)?

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Record {
    name: String,
    #[laminate(coerce)]
    score: Option<i64>,
}

#[test]
fn json_null_becomes_none() {
    let json = serde_json::json!({"name": "Alice", "score": null});
    let result = Record::from_flex_value(&json);
    println!("null → {:?}", result);
    assert!(result.is_ok());
    let (rec, _) = result.unwrap();
    assert_eq!(rec.score, None);
}

#[test]
fn string_number_coerces_to_some() {
    let json = serde_json::json!({"name": "Bob", "score": "42"});
    let result = Record::from_flex_value(&json);
    println!("\"42\" → {:?}", result);
    assert!(result.is_ok());
    let (rec, _) = result.unwrap();
    assert_eq!(rec.score, Some(42));
}

#[test]
fn string_null_sentinel_becomes_none() {
    // "null" as a string — BestEffort should recognize as null sentinel
    let json = serde_json::json!({"name": "Charlie", "score": "null"});
    let result = Record::from_flex_value(&json);
    println!("\"null\" string → {:?}", result);

    // Observe: does "null" string become None via sentinel detection?
    // Or does it try to parse "null" as i64 and fail?
    match &result {
        Ok((rec, diags)) => {
            println!("score = {:?}", rec.score);
            println!("diagnostics: {:?}", diags);
        }
        Err(e) => println!("Error: {e}"),
    }
}

#[test]
fn string_na_sentinel_becomes_none() {
    let json = serde_json::json!({"name": "Dana", "score": "N/A"});
    let result = Record::from_flex_value(&json);
    println!("\"N/A\" → {:?}", result);

    match &result {
        Ok((rec, diags)) => {
            println!("score = {:?}", rec.score);
            println!("diagnostics: {:?}", diags);
        }
        Err(e) => println!("Error: {e}"),
    }
}

#[test]
fn non_coercible_string_errors() {
    let json = serde_json::json!({"name": "Eve", "score": "abc"});
    let result = Record::from_flex_value(&json);
    println!("\"abc\" → {:?}", result);

    // "abc" can't coerce to i64 and isn't a null sentinel
    assert!(result.is_err(), "Non-coercible string should fail");
}
