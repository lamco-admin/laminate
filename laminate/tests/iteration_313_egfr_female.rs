//! Iteration 313: eGFR CKD-EPI formula with female sex factor.
//! Verify the 1.012 sex factor is correctly applied.

use laminate::packs::medical::calculate_egfr_ckd_epi;

#[test]
fn egfr_female_vs_male_same_inputs() {
    let creatinine = 0.9; // mg/dL
    let age = 50;

    let female = calculate_egfr_ckd_epi(creatinine, age, true);
    let male = calculate_egfr_ckd_epi(creatinine, age, false);

    println!("Female eGFR: {:.2}", female);
    println!("Male eGFR: {:.2}", male);

    // Female uses kappa=0.7, alpha=-0.241, sex_factor=1.012
    // Male uses kappa=0.9, alpha=-0.302, sex_factor=1.0
    // With same creatinine, these will differ due to both kappa/alpha AND sex factor
    assert!(female != male, "female and male eGFR should differ");
}

#[test]
fn egfr_female_low_creatinine() {
    // Female, creatinine below kappa (0.7): scr_ratio < 1.0
    let egfr = calculate_egfr_ckd_epi(0.5, 30, true);
    println!("Female low creatinine eGFR: {:.2}", egfr);

    // Expected: 142 * (0.5/0.7)^(-0.241) * 1.0^(-1.2) * 0.9938^30 * 1.012
    // scr_ratio = 0.714, min(0.714,1)=0.714, 0.714^(-0.241)
    // max(0.714,1)=1.0, 1.0^(-1.2)=1.0
    let kappa: f64 = 0.7;
    let alpha: f64 = -0.241;
    let scr_ratio: f64 = 0.5 / kappa;
    let min_term = scr_ratio.min(1.0).powf(alpha);
    let max_term = scr_ratio.max(1.0).powf(-1.200);
    let expected = 142.0 * min_term * max_term * 0.9938_f64.powf(30.0) * 1.012;
    assert!(
        (egfr - expected).abs() < 1e-6,
        "got {}, expected {}",
        egfr,
        expected
    );
}

#[test]
fn egfr_female_high_creatinine() {
    // Female, creatinine above kappa (0.7): scr_ratio > 1.0
    let egfr = calculate_egfr_ckd_epi(1.5, 60, true);
    println!("Female high creatinine eGFR: {:.2}", egfr);

    let kappa: f64 = 0.7;
    let alpha: f64 = -0.241;
    let scr_ratio: f64 = 1.5 / kappa;
    let min_term = scr_ratio.min(1.0).powf(alpha); // 1.0^alpha = 1.0
    let max_term = scr_ratio.max(1.0).powf(-1.200);
    let expected = 142.0 * min_term * max_term * 0.9938_f64.powf(60.0) * 1.012;
    assert!(
        (egfr - expected).abs() < 1e-6,
        "got {}, expected {}",
        egfr,
        expected
    );

    // Should be lower due to high creatinine
    assert!(
        egfr < 60.0,
        "high creatinine should give low eGFR: {}",
        egfr
    );
}
