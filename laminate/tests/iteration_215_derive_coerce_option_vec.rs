#![allow(dead_code)]
//! Iteration 215: #[laminate(coerce)] on Option<Vec<String>>.
//!
//! Tests the interaction of Option + Vec + coerce:
//! - null → None
//! - [] → Some(vec![])
//! - ["42"] → Some(["42"])
//! - [42, true] → Some(["42", "true"]) via coercion?
//! - "not-array" → error?

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Tags {
    name: String,
    #[laminate(coerce)]
    labels: Option<Vec<String>>,
}

#[test]
fn null_becomes_none() {
    let json = serde_json::json!({"name": "test", "labels": null});
    let result = Tags::from_flex_value(&json);
    println!("null → {:?}", result);
    assert!(result.is_ok());
    let (t, _) = result.unwrap();
    assert_eq!(t.labels, None);
}

#[test]
fn missing_becomes_none() {
    let json = serde_json::json!({"name": "test"});
    let result = Tags::from_flex_value(&json);
    println!("missing → {:?}", result);
    // Option field: missing → None
    assert!(result.is_ok());
    let (t, _) = result.unwrap();
    assert_eq!(t.labels, None);
}

#[test]
fn empty_array_becomes_some_empty() {
    let json = serde_json::json!({"name": "test", "labels": []});
    let result = Tags::from_flex_value(&json);
    println!("[] → {:?}", result);
    assert!(result.is_ok());
    let (t, _) = result.unwrap();
    assert_eq!(t.labels, Some(vec![]));
}

#[test]
fn string_array_works() {
    let json = serde_json::json!({"name": "test", "labels": ["alpha", "beta"]});
    let result = Tags::from_flex_value(&json);
    println!("[strings] → {:?}", result);
    assert!(result.is_ok());
    let (t, _) = result.unwrap();
    assert_eq!(
        t.labels,
        Some(vec!["alpha".to_string(), "beta".to_string()])
    );
}

#[test]
fn mixed_types_coerced_to_strings() {
    // [42, true, "text"] — can coercion stringify the non-strings?
    let json = serde_json::json!({"name": "test", "labels": [42, true, "text"]});
    let result = Tags::from_flex_value(&json);
    println!("[42, true, \"text\"] → {:?}", result);

    // Observe: does Vec<String> element-level coercion convert 42→"42", true→"true"?
    match &result {
        Ok((t, diags)) => {
            println!("labels = {:?}", t.labels);
            println!("diagnostics: {:?}", diags);
        }
        Err(e) => println!("Error: {e}"),
    }
}

#[test]
fn scalar_string_not_array() {
    // What about a scalar string where array expected?
    let json = serde_json::json!({"name": "test", "labels": "single"});
    let result = Tags::from_flex_value(&json);
    println!("\"single\" → {:?}", result);

    match &result {
        Ok((t, diags)) => {
            println!("labels = {:?}", t.labels);
            println!("diagnostics: {:?}", diags);
        }
        Err(e) => println!("Error: {e}"),
    }
}
