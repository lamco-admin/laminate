//! Iteration 238: E2E MultiPack — currencies + units + dates in same row
//!
//! Tests PackCoercion::All with mixed domain data. A single row containing:
//! - Currency values ("$1,234.56")
//! - Unit values ("72.5 kg")
//! - Date values ("2026-04-06")
//! - Plain strings ("Alice")
//!
//! The interesting question: when multiple packs could claim a value,
//! which one wins? And does extract::<String> correctly bypass pack coercion?

use laminate::value::PackCoercion;
use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn multipack_row_all_types() {
    let row = serde_json::json!({
        "name": "Alice",
        "salary": "$75,000.00",
        "weight": "68.5 kg",
        "hire_date": "2026-04-06",
        "score": "95.5"
    });

    let val = FlexValue::from(row)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All);

    // Name should extract as String without pack interference
    let name: String = val.extract("name").unwrap();
    assert_eq!(name, "Alice");

    // Salary should be stripped of currency symbol
    let salary: f64 = val.extract("salary").unwrap();
    println!("salary: {}", salary);
    assert!((salary - 75000.0).abs() < 0.01);

    // Weight should have unit stripped by units pack
    let weight: f64 = val.extract("weight").unwrap();
    println!("weight: {}", weight);
    assert!((weight - 68.5).abs() < 0.01);

    // Date should extract as String (no pack needed)
    let hire_date: String = val.extract("hire_date").unwrap();
    assert_eq!(hire_date, "2026-04-06");

    // Plain numeric string should coerce normally (not via packs)
    let score: f64 = val.extract("score").unwrap();
    assert!((score - 95.5).abs() < 0.01);
}

#[test]
fn multipack_string_extraction_preserves_everything() {
    // When extracting as String, pack coercion should NOT interfere
    let row = serde_json::json!({
        "price": "$12.99",
        "weight": "5 kg",
        "temp": "98.6 °F"
    });

    let val = FlexValue::from(row)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All);

    let price: String = val.extract("price").unwrap();
    let weight: String = val.extract("weight").unwrap();
    let temp: String = val.extract("temp").unwrap();

    assert_eq!(
        price, "$12.99",
        "String extraction should preserve currency"
    );
    assert_eq!(weight, "5 kg", "String extraction should preserve units");
    assert_eq!(
        temp, "98.6 °F",
        "String extraction should preserve temperature"
    );
}

#[test]
fn multipack_numeric_extraction_with_units() {
    let row = serde_json::json!({
        "distance": "42.195 km",
        "time": "2:03:59",
        "speed": "20.5 km/h"
    });

    let val = FlexValue::from(row)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All);

    // Distance: units pack should strip "km"
    let distance: f64 = val.extract("distance").unwrap();
    println!("distance: {}", distance);
    assert!((distance - 42.195).abs() < 0.001);
}

#[test]
fn multipack_with_safewidening_blocks_packs() {
    // PackCoercion::All at SafeWidening level should NOT fire packs
    let row = serde_json::json!({"price": "$12.99"});

    let val = FlexValue::from(row)
        .with_coercion(CoercionLevel::SafeWidening)
        .with_pack_coercion(PackCoercion::All);

    let result: Result<f64, _> = val.extract("price");
    println!("$12.99 at SafeWidening with All packs: {:?}", result);
    assert!(
        result.is_err(),
        "packs should not fire at SafeWidening even with All"
    );
}
