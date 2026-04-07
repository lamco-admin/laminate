//! Iteration 312: classify_lab_value via alias "blood sugar" → canonical "glucose".

use laminate::packs::medical::{classify_lab_value, LabClassification};

#[test]
fn classify_via_alias_blood_sugar() {
    // 126 mg/dL glucose is high (normal range typically 70-100 fasting)
    let result = classify_lab_value(126.0, "blood sugar", "mg/dL");
    println!("blood sugar 126: {:?}", result);
    assert!(
        result.is_some(),
        "alias 'blood sugar' should resolve to glucose"
    );
    assert_eq!(
        result.unwrap(),
        LabClassification::High,
        "126 mg/dL should be High for fasting glucose"
    );
}

#[test]
fn classify_via_alias_glu() {
    let result = classify_lab_value(90.0, "glu", "mg/dL");
    println!("glu 90: {:?}", result);
    assert!(result.is_some(), "alias 'glu' should resolve to glucose");
    assert_eq!(result.unwrap(), LabClassification::Normal);
}

#[test]
fn classify_via_alias_fbg() {
    let result = classify_lab_value(50.0, "fbg", "mg/dL");
    println!("fbg 50: {:?}", result);
    assert!(result.is_some(), "alias 'fbg' should resolve to glucose");
    assert_eq!(result.unwrap(), LabClassification::Low);
}
