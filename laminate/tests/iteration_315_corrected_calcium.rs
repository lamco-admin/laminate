//! Iteration 315: Corrected calcium when albumin is normal (4.0).
//! Formula: Corrected Ca = Total Ca + 0.8 × (4.0 - Albumin)
//! When albumin=4.0, correction should be zero.

use laminate::packs::medical::calculate_corrected_calcium;

#[test]
fn corrected_calcium_normal_albumin() {
    let total_ca = 9.5;
    let albumin = 4.0;
    let corrected = calculate_corrected_calcium(total_ca, albumin);
    println!("Ca {} + 0.8*(4.0-{}) = {}", total_ca, albumin, corrected);
    // 9.5 + 0.8*(4.0-4.0) = 9.5 + 0 = 9.5
    assert!(
        (corrected - 9.5).abs() < f64::EPSILON,
        "normal albumin should give zero correction: got {}",
        corrected
    );
}

#[test]
fn corrected_calcium_low_albumin() {
    let total_ca = 8.0;
    let albumin = 2.0;
    let corrected = calculate_corrected_calcium(total_ca, albumin);
    println!("Ca {} + 0.8*(4.0-{}) = {}", total_ca, albumin, corrected);
    // 8.0 + 0.8*(4.0-2.0) = 8.0 + 1.6 = 9.6
    assert!(
        (corrected - 9.6).abs() < 1e-10,
        "low albumin should increase corrected Ca: got {}",
        corrected
    );
}

#[test]
fn corrected_calcium_high_albumin() {
    let total_ca = 10.0;
    let albumin = 5.0;
    let corrected = calculate_corrected_calcium(total_ca, albumin);
    println!("Ca {} + 0.8*(4.0-{}) = {}", total_ca, albumin, corrected);
    // 10.0 + 0.8*(4.0-5.0) = 10.0 - 0.8 = 9.2
    assert!(
        (corrected - 9.2).abs() < 1e-10,
        "high albumin should decrease corrected Ca: got {}",
        corrected
    );
}
