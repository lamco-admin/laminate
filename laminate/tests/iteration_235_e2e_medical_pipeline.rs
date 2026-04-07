//! Iteration 235: E2E Medical lab results pipeline
//!
//! Full pipeline: string lab values (as from CSV/HL7) → SourceHint::Csv →
//! extract numeric values → convert between US/SI units → verify precision.
//!
//! The adversarial angle: lab values arrive as strings with units embedded.
//! Can we reliably go from "126 mg/dL" → extract the number → convert to mmol/L?

use laminate::packs::medical::{convert_lab_value, normalize_pharma_unit, parse_hl7_datetime};
use laminate::value::SourceHint;
use laminate::FlexValue;

#[test]
fn full_lab_results_pipeline() {
    // Simulated lab panel as CSV strings (everything is a string)
    let lab_panel = serde_json::json!({
        "patient_id": "12345",
        "glucose_mg_dl": "126",
        "cholesterol_mg_dl": "200",
        "hemoglobin_g_dl": "14.5",
        "creatinine_mg_dl": "1.2",
        "collection_date": "20260402143000"
    });

    let val = FlexValue::from(lab_panel).with_source_hint(SourceHint::Csv);

    // Step 1: Extract numeric values from string CSV data
    let glucose: f64 = val.extract("glucose_mg_dl").unwrap();
    let cholesterol: f64 = val.extract("cholesterol_mg_dl").unwrap();
    let hemoglobin: f64 = val.extract("hemoglobin_g_dl").unwrap();
    let creatinine: f64 = val.extract("creatinine_mg_dl").unwrap();

    assert!((glucose - 126.0).abs() < 0.01);
    assert!((cholesterol - 200.0).abs() < 0.01);
    assert!((hemoglobin - 14.5).abs() < 0.01);
    assert!((creatinine - 1.2).abs() < 0.01);

    // Step 2: Convert to SI units
    let glucose_si = convert_lab_value(glucose, "glucose", "mg/dL", "mmol/L").unwrap();
    let cholesterol_si = convert_lab_value(cholesterol, "cholesterol", "mg/dL", "mmol/L").unwrap();
    let hemoglobin_si = convert_lab_value(hemoglobin, "hemoglobin", "g/dL", "g/L").unwrap();
    let creatinine_si = convert_lab_value(creatinine, "creatinine", "mg/dL", "µmol/L").unwrap();

    println!("Glucose: {} mg/dL → {} mmol/L", glucose, glucose_si);
    println!(
        "Cholesterol: {} mg/dL → {} mmol/L",
        cholesterol, cholesterol_si
    );
    println!("Hemoglobin: {} g/dL → {} g/L", hemoglobin, hemoglobin_si);
    println!(
        "Creatinine: {} mg/dL → {} µmol/L",
        creatinine, creatinine_si
    );

    // Verify expected ranges (clinical reference)
    assert!(
        (glucose_si - 7.0).abs() < 0.1,
        "126 mg/dL glucose ≈ 7.0 mmol/L"
    );
    assert!(
        (cholesterol_si - 5.18).abs() < 0.1,
        "200 mg/dL cholesterol ≈ 5.18 mmol/L"
    );
    assert!(
        (hemoglobin_si - 145.0).abs() < 1.0,
        "14.5 g/dL hemoglobin ≈ 145 g/L"
    );
    assert!(
        (creatinine_si - 106.08).abs() < 1.0,
        "1.2 mg/dL creatinine ≈ 106 µmol/L"
    );

    // Step 3: Parse the HL7 collection date
    let date_str: String = val.extract("collection_date").unwrap();
    let parsed = parse_hl7_datetime(&date_str);
    println!("HL7 date parsed: {:?}", parsed);
    assert!(parsed.is_some(), "HL7 datetime should parse successfully");
}

#[test]
fn lab_value_bidirectional_roundtrip_precision() {
    // Convert glucose 126 mg/dL → mmol/L → back to mg/dL
    // Verify precision is maintained through the round-trip
    let original = 126.0_f64;
    let si = convert_lab_value(original, "glucose", "mg/dL", "mmol/L").unwrap();
    let roundtrip = convert_lab_value(si, "glucose", "mmol/L", "mg/dL").unwrap();

    let error = (roundtrip - original).abs();
    println!(
        "Glucose roundtrip: {} → {} → {}, error = {}",
        original, si, roundtrip, error
    );
    // The factor is 0.0555, so roundtrip = original * 0.0555 / 0.0555 = original
    // Floating point error should be extremely small
    assert!(
        error < 1e-10,
        "roundtrip error should be negligible, got {}",
        error
    );
}

#[test]
fn pharma_unit_normalization_in_pipeline() {
    // Different notations for microgram should all normalize the same
    let variants = ["mcg", "ug", "µg", "microgram", "micrograms"];
    let normalized: Vec<String> = variants.iter().map(|u| normalize_pharma_unit(u)).collect();

    println!("Normalized units: {:?}", normalized);
    for (i, n) in normalized.iter().enumerate() {
        assert_eq!(n, "µg", "variant '{}' should normalize to µg", variants[i]);
    }
}

#[test]
fn lab_values_via_alias_names() {
    // Use alias names instead of canonical names
    let result = convert_lab_value(126.0, "blood sugar", "mg/dL", "mmol/L");
    assert!(
        result.is_some(),
        "alias 'blood sugar' should resolve to glucose"
    );
    assert!((result.unwrap() - 7.0).abs() < 0.1);

    let result = convert_lab_value(200.0, "chol", "mg/dL", "mmol/L");
    assert!(
        result.is_some(),
        "alias 'chol' should resolve to cholesterol"
    );

    let result = convert_lab_value(14.5, "hgb", "g/dL", "g/L");
    assert!(result.is_some(), "alias 'hgb' should resolve to hemoglobin");
}
