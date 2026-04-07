//! Example: Medical lab value conversion between US and EU units
//!
//! Demonstrates: analyte-aware conversion, pharmaceutical notation
//! normalization, and HL7 v2 date parsing.

use laminate::packs::medical::{convert_lab_value, normalize_pharma_unit, parse_hl7_datetime};

fn main() {
    println!("=== Lab Value Conversion (US ↔ EU) ===\n");

    let conversions = vec![
        ("Glucose", 126.0, "mg/dL", "mmol/L"),
        ("Cholesterol", 200.0, "mg/dL", "mmol/L"),
        ("Creatinine", 1.2, "mg/dL", "µmol/L"),
        ("Hemoglobin", 14.0, "g/dL", "g/L"),
        ("Calcium", 10.0, "mg/dL", "mmol/L"),
    ];

    for (analyte, value, from, to) in &conversions {
        match convert_lab_value(*value, analyte, from, to) {
            Some(result) => println!("{}: {:.1} {} → {:.1} {}", analyte, value, from, result, to),
            None => println!("{}: conversion not found", analyte),
        }
    }

    // Reverse conversion
    println!("\n=== Reverse (EU → US) ===\n");
    let eu_glucose = 7.0; // mmol/L
    let us_glucose = convert_lab_value(eu_glucose, "glucose", "mmol/L", "mg/dL").unwrap();
    println!(
        "Glucose: {:.1} mmol/L → {:.0} mg/dL",
        eu_glucose, us_glucose
    );

    // Pharmaceutical notation
    println!("\n=== Pharmaceutical Notation ===\n");
    for unit in &["mcg", "ug", "microgram", "cc", "IU", "ml"] {
        println!("\"{}\" → \"{}\"", unit, normalize_pharma_unit(unit));
    }

    // HL7 v2 dates
    println!("\n=== HL7 v2 Date Parsing ===\n");
    let hl7_dates = vec!["20260402", "20260402143022", "20260402143022-0500"];
    for d in &hl7_dates {
        println!(
            "\"{}\" → {}",
            d,
            parse_hl7_datetime(d).unwrap_or("(invalid)".into())
        );
    }
}
