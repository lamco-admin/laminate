//! Iteration 311: Lab value round-trip precision.
//! Convert glucose 126 mg/dL → mmol/L → mg/dL and verify precision.

use laminate::packs::medical::convert_lab_value;

#[test]
fn glucose_roundtrip_precision() {
    let original = 126.0_f64; // mg/dL
    let si = convert_lab_value(original, "glucose", "mg/dL", "mmol/L")
        .expect("glucose mg/dL→mmol/L should work");
    println!("126 mg/dL → {} mmol/L", si);

    let back = convert_lab_value(si, "glucose", "mmol/L", "mg/dL")
        .expect("glucose mmol/L→mg/dL should work");
    println!("{} mmol/L → {} mg/dL", si, back);

    // Round-trip should be within floating-point epsilon
    assert!(
        (back - original).abs() < 1e-10,
        "round-trip mismatch: {} → {} → {} (delta={})",
        original,
        si,
        back,
        (back - original).abs()
    );
}

#[test]
fn hba1c_roundtrip_precision() {
    let original = 6.5_f64; // %
    let si = convert_lab_value(original, "hba1c", "%", "mmol/mol")
        .expect("hba1c %→mmol/mol should work");
    println!("6.5% → {} mmol/mol", si);

    let back =
        convert_lab_value(si, "hba1c", "mmol/mol", "%").expect("hba1c mmol/mol→% should work");
    println!("{} mmol/mol → {}%", si, back);

    // HbA1c uses affine formula so round-trip should still hold
    assert!(
        (back - original).abs() < 1e-10,
        "hba1c round-trip mismatch: {} → {} → {} (delta={})",
        original,
        si,
        back,
        (back - original).abs()
    );
}

#[test]
fn creatinine_roundtrip_precision() {
    let original = 1.2_f64; // mg/dL
    let si = convert_lab_value(original, "creatinine", "mg/dL", "µmol/L")
        .expect("creatinine conversion should work");
    let back = convert_lab_value(si, "creatinine", "µmol/L", "mg/dL")
        .expect("creatinine reverse should work");
    println!("creatinine: {} → {} → {}", original, si, back);

    assert!(
        (back - original).abs() < 1e-10,
        "creatinine round-trip: {} vs {} (delta={})",
        original,
        back,
        (back - original).abs()
    );
}
