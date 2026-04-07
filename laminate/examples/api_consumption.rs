//! Example: Consuming a messy REST API response
//!
//! Demonstrates: type coercion, path navigation, derive macro with overflow,
//! and diagnostics for auditing what was coerced.

use laminate::{FlexValue, Laminate};
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct User {
    name: String,
    #[laminate(coerce)]
    age: u32,
    #[laminate(coerce)]
    verified: bool,
    #[laminate(coerce, default)]
    score: f64,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

fn main() {
    // Simulated API response — types are messy (strings where numbers expected)
    let api_response = r#"{
        "name": "Alice",
        "age": "28",
        "verified": "true",
        "score": "95.5",
        "department": "engineering",
        "start_date": "2024-01-15"
    }"#;

    // Parse and shape into typed struct
    let (user, diagnostics) = User::from_json(api_response).unwrap();

    println!("User: {} (age {})", user.name, user.age);
    println!("Verified: {}", user.verified);
    println!("Score: {}", user.score);
    println!(
        "Extra fields preserved: {:?}",
        user.extra.keys().collect::<Vec<_>>()
    );

    // Every coercion is recorded — nothing is silent
    println!("\nDiagnostics ({} total):", diagnostics.len());
    for d in &diagnostics {
        println!("  {}", d);
    }

    // Round-trip: extra fields survive serialization
    let value = user.to_value();
    println!("\nRound-trip JSON:");
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}
