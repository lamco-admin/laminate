#![allow(dead_code)]
//! Iteration 210: guess_type on "1.2.3.4" vs "192.168.1.1".
//!
//! Both are valid IPv4. But "1.2.3.4" also looks like a version string.
//! Does guess_type distinguish them? Or rank them identically?
//! Also test edge cases: "999.999.999.999" (invalid IP, could be version),
//! "1.0.0" (semantic version, not IP), "10.0.0.256" (invalid octet).

use laminate::detect::{guess_type, GuessedType};

#[test]
fn private_ip_detected() {
    let guesses = guess_type("192.168.1.1");
    println!("guess_type(\"192.168.1.1\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(ip.is_some(), "192.168.1.1 should be detected as IP address");
    assert!(ip.unwrap().confidence >= 0.8, "Should be high confidence");
}

#[test]
fn low_octet_ip_detected() {
    // "1.2.3.4" — all octets are valid u8, so it's a valid IP
    let guesses = guess_type("1.2.3.4");
    println!("guess_type(\"1.2.3.4\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(ip.is_some(), "1.2.3.4 should be detected as IP address");
}

#[test]
fn invalid_octet_not_ip() {
    // "10.0.0.256" — 256 exceeds u8, NOT a valid IP
    let guesses = guess_type("10.0.0.256");
    println!("guess_type(\"10.0.0.256\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(
        ip.is_none(),
        "10.0.0.256 should NOT be detected as IP (256 > 255)"
    );
}

#[test]
fn three_part_version_not_ip() {
    // "1.0.0" — 3 parts, not 4 — should NOT be IP
    let guesses = guess_type("1.0.0");
    println!("guess_type(\"1.0.0\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(ip.is_none(), "3-part version string should not be IP");

    // Might parse as float? "1.0.0" is not a valid float
    let float = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::Float));
    println!("Float guess: {:?}", float);
}

#[test]
fn all_999_not_ip() {
    // "999.999.999.999" — 4 parts but 999 exceeds u8
    let guesses = guess_type("999.999.999.999");
    println!("guess_type(\"999.999.999.999\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(
        ip.is_none(),
        "999.999.999.999 should NOT be IP (octets > 255)"
    );
}

#[test]
fn loopback_ip() {
    let guesses = guess_type("127.0.0.1");
    println!("guess_type(\"127.0.0.1\"): {:?}", guesses);

    let ip = guesses
        .iter()
        .find(|g| matches!(g.kind, GuessedType::IpAddress));
    assert!(ip.is_some(), "127.0.0.1 should be detected as IP");
}
