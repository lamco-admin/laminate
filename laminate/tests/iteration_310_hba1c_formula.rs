//! Iteration 310 — Medical-HbA1cFormula
//! HbA1c uses affine IFCC formula: mmol/mol = (NGSP% - 2.15) × 10.929

use laminate::packs::medical::convert_lab_value;

#[test]
fn hba1c_6_5_percent() {
    // 6.5% NGSP = (6.5 - 2.15) * 10.929 ≈ 47.5 mmol/mol (diabetes diagnosis threshold)
    let result = convert_lab_value(6.5, "hba1c", "%", "mmol/mol").unwrap();
    assert!(
        (result - 47.5).abs() < 0.5,
        "6.5% should be ~47.5, got {}",
        result
    );
}

#[test]
fn hba1c_5_7_percent() {
    // 5.7% = (5.7 - 2.15) * 10.929 ≈ 38.8 mmol/mol (pre-diabetes)
    let result = convert_lab_value(5.7, "hba1c", "%", "mmol/mol").unwrap();
    assert!(
        (result - 38.8).abs() < 0.5,
        "5.7% should be ~39, got {}",
        result
    );
}

#[test]
fn hba1c_7_0_percent() {
    // 7.0% = (7.0 - 2.15) * 10.929 ≈ 53.0 mmol/mol
    let result = convert_lab_value(7.0, "hba1c", "%", "mmol/mol").unwrap();
    assert!(
        (result - 53.0).abs() < 0.5,
        "7.0% should be ~53, got {}",
        result
    );
}

#[test]
fn hba1c_roundtrip() {
    let forward = convert_lab_value(6.5, "hba1c", "%", "mmol/mol").unwrap();
    let back = convert_lab_value(forward, "hba1c", "mmol/mol", "%").unwrap();
    assert!(
        (back - 6.5).abs() < 0.001,
        "round-trip error: {} → {} → {}",
        6.5,
        forward,
        back
    );
}
