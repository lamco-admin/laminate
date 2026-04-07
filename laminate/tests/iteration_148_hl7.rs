/// Iteration 148: parse_hl7_datetime edge cases (targets #122, #123)
use laminate::packs::time::{convert_to_iso8601, detect_format};

#[test]
fn hl7_far_future() {
    // Target #122: "99991231" — far future date
    let fmt = detect_format("99991231");
    println!("detect_format(\"99991231\"): {:?}", fmt);
    let result = convert_to_iso8601("99991231");
    println!("convert: {:?}", result);
    assert_eq!(
        result.as_deref(),
        Some("9999-12-31"),
        "far future HL7 date should convert"
    );
}

#[test]
fn hl7_with_fractional_seconds() {
    // Target #123: "20260402143022.1234-0500"
    let fmt = detect_format("20260402143022.1234-0500");
    println!("detect_format with fractional: {:?}", fmt);
    let result = convert_to_iso8601("20260402143022.1234-0500");
    println!("convert: {:?}", result);
    // Should at minimum extract the date portion
    assert!(
        result.is_some(),
        "HL7 with fractional seconds should convert"
    );
}
