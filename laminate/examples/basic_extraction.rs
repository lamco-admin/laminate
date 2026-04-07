//! The simplest laminate example: parse JSON and extract typed values.
//!
//! Run with: cargo run --example basic_extraction --features full

use laminate::FlexValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse messy JSON — strings where numbers should be, missing fields, etc.
    let data = FlexValue::from_json(
        r#"{
        "name": "Alice",
        "port": "8080",
        "debug": "true",
        "score": "95.5",
        "workers": 4
    }"#,
    )?;

    // Extract with automatic type coercion — strings become numbers/bools
    let name: String = data.extract("name")?;
    let port: u16 = data.extract("port")?; // "8080" → 8080
    let debug: bool = data.extract("debug")?; // "true" → true
    let score: f64 = data.extract("score")?; // "95.5" → 95.5
    let workers: i32 = data.extract("workers")?; // 4 → 4

    println!("name={name}, port={port}, debug={debug}, score={score}, workers={workers}");

    // Navigate nested paths
    let nested = FlexValue::from_json(
        r#"{
        "user": {
            "profile": {
                "email": "alice@example.com"
            },
            "scores": [98, 85, 92]
        }
    }"#,
    )?;

    let email: String = nested.extract("user.profile.email")?;
    let first_score: i64 = nested.extract("user.scores[0]")?;
    println!("email={email}, first_score={first_score}");

    // Use maybe() for optional fields — returns Ok(None) instead of Err
    let missing: Option<String> = nested.maybe("user.profile.phone")?;
    println!("phone={missing:?}"); // None — no error, no panic

    Ok(())
}
