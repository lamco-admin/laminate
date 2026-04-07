//! Iteration 314: BMI classification at exact boundaries.
//! 18.5 = normal (not mild thinness), 25.0 = overweight (not normal), 30.0 = obese I.

use laminate::packs::medical::classify_bmi;

#[test]
fn bmi_exactly_18_5() {
    let class = classify_bmi(18.5);
    println!("BMI 18.5 → {}", class);
    // 18.5 is >= 18.5 (not < 18.5) so should be "normal"
    assert_eq!(class, "normal", "18.5 should be normal, not mild thinness");
}

#[test]
fn bmi_just_below_18_5() {
    let class = classify_bmi(18.499999);
    println!("BMI 18.499999 → {}", class);
    assert_eq!(class, "mild thinness");
}

#[test]
fn bmi_exactly_25_0() {
    let class = classify_bmi(25.0);
    println!("BMI 25.0 → {}", class);
    // 25.0 is >= 25.0 (not < 25.0) so should be "overweight"
    assert_eq!(class, "overweight", "25.0 should be overweight, not normal");
}

#[test]
fn bmi_exactly_30_0() {
    let class = classify_bmi(30.0);
    println!("BMI 30.0 → {}", class);
    // 30.0 is >= 30.0 so should be "obese class I"
    assert_eq!(class, "obese class I", "30.0 should be obese class I");
}

#[test]
fn bmi_exactly_35_0() {
    let class = classify_bmi(35.0);
    println!("BMI 35.0 → {}", class);
    assert_eq!(class, "obese class II");
}

#[test]
fn bmi_exactly_40_0() {
    let class = classify_bmi(40.0);
    println!("BMI 40.0 → {}", class);
    assert_eq!(class, "obese class III");
}
