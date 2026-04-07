//! Iteration 317: Negative lab values in conversion.
//! Medical values should be non-negative, but what happens with -1.0?

use laminate::packs::medical::{classify_lab_value, convert_lab_value};

#[test]
fn convert_negative_glucose() {
    // -1.0 mg/dL is physically meaningless but what does the converter do?
    let result = convert_lab_value(-1.0, "glucose", "mg/dL", "mmol/L");
    println!("convert -1.0 glucose: {:?}", result);
    // Observe: does it return Some(-negative) or None?
    // Currently: probably applies factor blindly
    match result {
        Some(val) => {
            println!("Got Some({})", val);
            // The converter applies a linear factor, so -1.0 * factor = negative
            // This is mathematically correct but medically meaningless
            assert!(val < 0.0, "negative input → negative output");
        }
        None => println!("Got None — converter rejects negative values"),
    }
}

#[test]
fn classify_negative_glucose() {
    let result = classify_lab_value(-1.0, "glucose", "mg/dL");
    println!("classify -1.0 glucose: {:?}", result);
    // Should this be CriticalLow or an error? Observe.
    assert!(
        result.is_some(),
        "classify should return something for -1.0"
    );
}
