// Iteration 165: parse_pack_notation("0x100-count") — hex or pack?
// Target #144 — GAP: "0x100" was parsed as pack notation (0 times 100 = 0).
// Fixed by detecting hex prefix (0x + hex digit) and skipping pack match.

use laminate::packs::units::parse_pack_notation;

#[test]
fn hex_literal_not_parsed_as_pack() {
    // "0x100" is hex 256 — must NOT match pack notation
    let result = parse_pack_notation("0x100");
    assert!(
        result.is_none(),
        "Hex literal '0x100' should not be pack notation, got: {:?}",
        result
    );
}

#[test]
fn hex_with_count_suffix_not_parsed() {
    // "0x100-count" starts with hex prefix — should not match
    let result = parse_pack_notation("0x100-count");
    assert!(
        result.is_none(),
        "Hex-prefixed '0x100-count' should not be pack notation, got: {:?}",
        result
    );
}

#[test]
fn hex_upper_not_parsed() {
    // "0X1F" — uppercase hex
    let result = parse_pack_notation("0X1F");
    assert!(result.is_none(), "Hex '0X1F' should not be pack notation");
}

#[test]
fn valid_pack_with_multiplier() {
    // "3x100-count" — legitimate pack notation
    let result = parse_pack_notation("3x100-count");
    assert!(result.is_some());
    let pack = result.unwrap();
    assert_eq!(pack.total_units, 300);
    assert_eq!(pack.packs, Some(3));
}

#[test]
fn valid_1x100_count() {
    // "1x100-count" — legitimate, not hex
    let result = parse_pack_notation("1x100-count");
    assert!(result.is_some());
    let pack = result.unwrap();
    assert_eq!(pack.total_units, 100);
}

#[test]
fn hex_like_0xfff_not_parsed() {
    // "0xfff" — hex with letters, should not match
    let result = parse_pack_notation("0xfff");
    assert!(result.is_none(), "Hex '0xfff' should not be pack notation");
}
