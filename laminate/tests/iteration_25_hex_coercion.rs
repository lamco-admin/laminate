//! Iteration 25: GAP — hex/octal/binary string coercion to integer
//! "0x1F", "0o77", "0b1010" failed with opaque serde error.
//! Fix: Added try_parse_radix_int() with from_str_radix for 0x/0o/0b prefixes.

use laminate::FlexValue;

#[test]
fn iter25_hex_string_coerces_to_integer() {
    let json = serde_json::json!({"color": "0x1F"});
    let flex = FlexValue::new(json);

    let val: i64 = flex.extract("color").unwrap();
    assert_eq!(val, 31);

    let val_u8: u8 = flex.extract("color").unwrap();
    assert_eq!(val_u8, 31);
}

#[test]
fn iter25_hex_0xff_fits_u8() {
    let json = serde_json::json!({"mask": "0xFF"});
    let flex = FlexValue::new(json);

    let val: u8 = flex.extract("mask").unwrap();
    assert_eq!(val, 255);
}

#[test]
fn iter25_octal_binary_coerce() {
    let json = serde_json::json!({"perm": "0o77", "flags": "0b1010"});
    let flex = FlexValue::new(json);

    assert_eq!(flex.extract::<i64>("perm").unwrap(), 63);
    assert_eq!(flex.extract::<i64>("flags").unwrap(), 10);
}

#[test]
fn iter25_hex_produces_warning_diagnostic() {
    let json = serde_json::json!({"val": "0x1F"});
    let flex = FlexValue::new(json);

    let (val, diags) = flex.extract_with_diagnostics::<i64>("val").unwrap();
    assert_eq!(val, 31);
    assert_eq!(diags.len(), 1);
    assert!(matches!(
        diags[0].kind,
        laminate::DiagnosticKind::Coerced { .. }
    ));
    assert_eq!(diags[0].risk, laminate::RiskLevel::Warning);
}

#[test]
fn iter25_hex_overflow_detected() {
    // 0x100 = 256, overflows u8
    let json = serde_json::json!({"val": "0x100"});
    let flex = FlexValue::new(json);

    let result = flex.extract::<u8>("val");
    assert!(result.is_err(), "0x100 should overflow u8");
}

#[test]
fn iter25_hex_as_string_preserved() {
    let json = serde_json::json!({"val": "0x1F"});
    let flex = FlexValue::new(json);

    let s: String = flex.extract("val").unwrap();
    assert_eq!(s, "0x1F");
}
