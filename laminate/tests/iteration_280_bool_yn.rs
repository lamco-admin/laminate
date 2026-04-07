//! Iteration 280 — Coerce-BoolYN
//! Single-char "y"/"n"/"t"/"f" coerce to bool at BestEffort

use laminate::FlexValue;

#[test]
fn y_coerces_to_true() {
    let fv = FlexValue::from_json(r#""y""#).unwrap();
    assert!(fv.extract_root::<bool>().unwrap());
}

#[test]
fn n_coerces_to_false() {
    let fv = FlexValue::from_json(r#""n""#).unwrap();
    assert!(!(fv.extract_root::<bool>().unwrap()));
}

#[test]
fn t_coerces_to_true() {
    let fv = FlexValue::from_json(r#""t""#).unwrap();
    assert!(fv.extract_root::<bool>().unwrap());
}

#[test]
fn f_coerces_to_false() {
    let fv = FlexValue::from_json(r#""f""#).unwrap();
    assert!(!(fv.extract_root::<bool>().unwrap()));
}

#[test]
fn uppercase_y_coerces_to_true() {
    // to_lowercase() handles case
    let fv = FlexValue::from_json(r#""Y""#).unwrap();
    assert!(fv.extract_root::<bool>().unwrap());
}

#[test]
fn uppercase_n_coerces_to_false() {
    let fv = FlexValue::from_json(r#""N""#).unwrap();
    assert!(!(fv.extract_root::<bool>().unwrap()));
}
