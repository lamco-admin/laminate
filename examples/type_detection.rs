//! Type detection with guess_type() — identify what kind of data a string contains.
//!
//! Run with: cargo run --example type_detection --features full

use laminate::detect::guess_type;

fn main() {
    let samples = [
        "42",
        "3.14159",
        "true",
        "2026-04-06",
        "2026-04-06T14:30:00Z",
        "$1,234.56",
        "€ 1.234,56",
        "4111111111111111",
        "alice@example.com",
        "https://example.com",
        "550e8400-e29b-41d4-a716-446655440000",
        "192.168.1.1",
        "GB29NWBK60161331926819",
        "N/A",
        "15.5 kg",
        "hello world",
    ];

    println!("{:<45} {:<20} {}", "Input", "Top Type", "Confidence");
    println!("{}", "-".repeat(75));

    for sample in &samples {
        let guesses = guess_type(sample);
        if let Some(top) = guesses.first() {
            println!("{:<45} {:<20} {:.2}", sample, format!("{:?}", top.kind), top.confidence);
        }
    }
}
