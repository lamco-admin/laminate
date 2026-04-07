// Iteration 174: parse_coordinate with trailing garbage
// Fresh target — what if coordinate strings have extra text appended?

use laminate::packs::geo::parse_coordinate;

#[test]
fn coordinate_with_trailing_text() {
    // "40.7128, -74.0060 (New York)" — trailing context
    let result = parse_coordinate("40.7128, -74.0060 (New York)");
    println!("With trailing text: {:?}", result);
    // Should fail — extra tokens after the two numbers
    assert!(result.is_none(), "Trailing text should prevent parsing");
}

#[test]
fn coordinate_with_unit_suffix() {
    // "40.7128°N, 74.0060°W extra" — valid DMS with trailing
    let result = parse_coordinate("40.7128, -74.0060m");
    println!("With 'm' suffix: {:?}", result);
    // "-74.0060m" won't parse as f64
}

#[test]
fn coordinate_with_leading_label() {
    // "lat: 40.7128, lng: -74.0060" — labeled coordinates
    let result = parse_coordinate("lat: 40.7128, lng: -74.0060");
    println!("With labels: {:?}", result);
    // "lat: 40.7128" won't parse as f64
    assert!(result.is_none(), "Labels should prevent parsing");
}

#[test]
fn coordinate_clean_passes() {
    // Baseline — clean coordinates should work
    let result = parse_coordinate("40.7128, -74.0060");
    assert!(result.is_some());
    let coord = result.unwrap();
    assert!((coord.latitude - 40.7128).abs() < 0.0001);
}

#[test]
fn coordinate_with_extra_number() {
    // Three numbers — should reject
    let result = parse_coordinate("40.7128, -74.0060, 100");
    assert!(result.is_none(), "Three numbers should reject");
}

#[test]
fn coordinate_with_whitespace_only() {
    let result = parse_coordinate("   ");
    assert!(result.is_none());
}

#[test]
fn coordinate_semicolon_separated() {
    // Some systems use semicolons — does this parse?
    let result = parse_coordinate("40.7128; -74.0060");
    println!("Semicolon separated: {:?}", result);
    // The parser splits on comma or whitespace, not semicolons
}
