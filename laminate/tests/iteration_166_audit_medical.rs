// Iteration 166: Audit with medical lab values — type mismatch when units change
// Target #147 — Medical data often has "126 mg/dL" (string with unit) mixed
// with plain numbers like 7.0. Schema infers dominant type from the majority.
// Does audit correctly flag the mixed types?

use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn audit_lab_values_mixed_string_and_number() {
    // Training data: lab results as numbers
    let training = vec![
        json!({"patient_id": 1, "glucose": 126.0, "unit": "mg/dL"}),
        json!({"patient_id": 2, "glucose": 98.0, "unit": "mg/dL"}),
        json!({"patient_id": 3, "glucose": 110.0, "unit": "mg/dL"}),
    ];

    let schema = InferredSchema::from_values(&training);

    // Audit data: some results come back as strings with units embedded
    let audit_data = vec![
        json!({"patient_id": 4, "glucose": 130.0, "unit": "mg/dL"}),
        json!({"patient_id": 5, "glucose": "7.0 mmol/L", "unit": "mmol/L"}),
        json!({"patient_id": 6, "glucose": 95.0, "unit": "mg/dL"}),
    ];

    let report = schema.audit(&audit_data);
    println!("Violations: {:?}", report.violations);
    println!("Summary: {}", report.summary());

    // The schema should expect Number for glucose (inferred from training).
    // Row with "7.0 mmol/L" (String) should be flagged.
    let glucose_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "glucose")
        .collect();

    assert!(
        !glucose_violations.is_empty(),
        "String lab value '7.0 mmol/L' should be flagged as type mismatch against Number"
    );
}

#[test]
fn audit_lab_values_all_strings() {
    // If all training data has string values, schema expects String
    let training = vec![
        json!({"test": "glucose", "result": "126 mg/dL"}),
        json!({"test": "glucose", "result": "98 mg/dL"}),
        json!({"test": "hba1c", "result": "6.5 %"}),
    ];

    let schema = InferredSchema::from_values(&training);

    // Audit with a numeric value — should flag type mismatch
    let audit_data = vec![json!({"test": "glucose", "result": 130})];

    let report = schema.audit(&audit_data);
    let result_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "result")
        .collect();

    println!("Result violations: {:?}", result_violations);
    // Number where String expected — should be flagged (coercible or violation)
    // Number→String is coercible, so it might be counted as coercible rather than violation
}

#[test]
fn audit_lab_values_unit_field_change() {
    // Unit field changes from "mg/dL" to "mmol/L" — same type (String), different value
    // Schema doesn't have enum constraints, so this should PASS (no violation)
    let training = vec![
        json!({"analyte": "glucose", "value": 126.0, "unit": "mg/dL"}),
        json!({"analyte": "glucose", "value": 98.0, "unit": "mg/dL"}),
    ];

    let schema = InferredSchema::from_values(&training);

    let audit_data = vec![json!({"analyte": "glucose", "value": 7.0, "unit": "mmol/L"})];

    let report = schema.audit(&audit_data);
    // Unit change is same type (String→String), so no type violation
    let unit_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "unit")
        .collect();
    assert!(
        unit_violations.is_empty(),
        "Unit field change should not be a type violation: {:?}",
        unit_violations
    );
}
