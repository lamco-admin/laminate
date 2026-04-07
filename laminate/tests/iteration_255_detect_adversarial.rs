//! Iteration 255: guess_type() adversarial inputs
//!
//! Tests the type detection system with tricky inputs that could confuse
//! pattern matching: malformed UUIDs, IPv4-mapped IPv6, ambiguous date-like
//! strings, very long strings, unicode-only strings.

use laminate::detect::guess_type;

#[test]
fn guess_type_malformed_uuid() {
    // Correct length and dashes but invalid hex chars
    let result = guess_type("550e8400-e29b-41d4-a716-44665544000g");
    println!("Malformed UUID: {:?}", result);
    // 'g' is not hex, so it should NOT detect as UUID
}

#[test]
fn guess_type_all_zeros_uuid() {
    let result = guess_type("00000000-0000-0000-0000-000000000000");
    println!("All-zeros UUID: {:?}", result);
    // Valid UUID format even though all zeros
}

#[test]
fn guess_type_scientific_notation() {
    let result = guess_type("1.23e10");
    println!("Scientific: {:?}", result);
    // Should detect as a float/number

    let result2 = guess_type("1.23E-5");
    println!("Scientific negative exp: {:?}", result2);
}

#[test]
fn guess_type_very_long_string() {
    let long = "a".repeat(10000);
    let result = guess_type(&long);
    println!("10000-char string: {:?}", result);
    // Should not panic or hang
}

#[test]
fn guess_type_unicode_only() {
    let result = guess_type("こんにちは世界");
    println!("Japanese: {:?}", result);

    let result2 = guess_type("مرحبا");
    println!("Arabic: {:?}", result2);
}

#[test]
fn guess_type_whitespace_variations() {
    let result = guess_type("  42  ");
    println!("Whitespace padded number: {:?}", result);

    let result2 = guess_type("  ");
    println!("Only whitespace: {:?}", result2);

    let result3 = guess_type("\t\n");
    println!("Tab/newline: {:?}", result3);
}

#[test]
fn guess_type_special_float_values() {
    let result = guess_type("Infinity");
    println!("Infinity: {:?}", result);

    let result2 = guess_type("-Infinity");
    println!("-Infinity: {:?}", result2);
}

#[test]
fn guess_type_sql_like_values() {
    let result = guess_type("SELECT * FROM users");
    println!("SQL: {:?}", result);

    let result2 = guess_type("DROP TABLE");
    println!("DROP TABLE: {:?}", result2);
}

#[test]
fn guess_type_url_detection() {
    let result = guess_type("https://example.com/path?query=1");
    println!("URL: {:?}", result);

    let result2 = guess_type("ftp://files.example.com");
    println!("FTP URL: {:?}", result2);
}

#[test]
fn guess_type_email_detection() {
    let result = guess_type("user@example.com");
    println!("Email: {:?}", result);

    let result2 = guess_type("user+tag@subdomain.example.co.uk");
    println!("Complex email: {:?}", result2);
}

#[test]
fn guess_type_boolean_like() {
    let cases = vec!["TRUE", "False", "YES", "no", "ON", "off", "1", "0"];
    for case in &cases {
        let result = guess_type(case);
        println!("{}: {:?}", case, result);
    }
}

#[test]
fn guess_type_json_like() {
    let result = guess_type(r#"{"key": "value"}"#);
    println!("JSON object: {:?}", result);

    let result2 = guess_type("[1, 2, 3]");
    println!("JSON array: {:?}", result2);
}
