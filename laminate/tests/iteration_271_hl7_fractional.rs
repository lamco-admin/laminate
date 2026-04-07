//! Iteration 271 — Date-HL7FractionalSeconds
//! Test convert_to_iso8601 on HL7 datetime with fractional seconds "20260406143022.1234"

use laminate::packs::time::convert_to_iso8601;

#[test]
fn hl7_fractional_seconds() {
    let result = convert_to_iso8601("20260406143022.1234");
    // Observe: does HL7 fractional seconds get preserved in the ISO output?
    eprintln!("HL7 fractional result: {:?}", result);
    // We expect: "2026-04-06T14:30:22.1234" — date + time + fractional
    assert!(result.is_some(), "HL7 datetime should parse");
    let iso = result.unwrap();
    assert!(
        iso.contains(".1234"),
        "fractional seconds should be preserved, got: {}",
        iso
    );
}
