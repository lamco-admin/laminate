//! Example: CSV ETL pipeline with source-aware coercion
//!
//! Demonstrates: SourceHint::Csv, pack coercion for currency/units,
//! guess_type() for unknown columns, batch date detection.

use laminate::detect::{guess_type, GuessedType};
use laminate::value::SourceHint;
use laminate::FlexValue;

fn main() {
    // Simulated CSV data (all values are strings — typical of CSV)
    let rows = vec![
        r#"{"price": "$12.99", "weight": "2.5 kg", "date": "03/15/2026", "active": "yes"}"#,
        r#"{"price": "$24.50", "weight": "1.0 kg", "date": "03/16/2026", "active": "no"}"#,
        r#"{"price": "$7.99", "weight": "0.5 kg", "date": "03/17/2026", "active": "true"}"#,
    ];

    println!("=== CSV ETL with source-aware coercion ===\n");

    for (i, row) in rows.iter().enumerate() {
        let val = FlexValue::from_json(row)
            .unwrap()
            .with_source_hint(SourceHint::Csv); // enables BestEffort + pack coercion

        let price: f64 = val.extract("price").unwrap(); // "$12.99" → 12.99 (currency pack)
        let weight: f64 = val.extract("weight").unwrap(); // "2.5 kg" → 2.5 (units pack)
        let active: bool = val.extract("active").unwrap(); // "yes" → true (bool coercion)

        println!(
            "Row {}: price={:.2}, weight={:.1}, active={}",
            i + 1,
            price,
            weight,
            active
        );
    }

    // Detect column types from a sample
    println!("\n=== Column type detection ===\n");
    let sample_values = vec!["$12.99", "$24.50", "$7.99"];
    for val in &sample_values {
        let guesses = guess_type(val);
        println!(
            "\"{}\" → {:?} ({:.0}% confidence)",
            val,
            guesses[0].kind,
            guesses[0].confidence * 100.0
        );
    }

    // Batch date format detection
    println!("\n=== Batch date disambiguation ===\n");
    let dates = vec!["03/15/2026", "03/16/2026", "13/04/2026"];
    let info =
        laminate::packs::time::detect_column_format(&dates.iter().map(|s| *s).collect::<Vec<_>>());
    println!("Dominant format: {:?}", info.dominant_format);
    println!("Date percentage: {:.0}%", info.date_percentage * 100.0);
    println!(
        "Disambiguated: {} (day_first: {:?})",
        info.disambiguated, info.day_first
    );
}
