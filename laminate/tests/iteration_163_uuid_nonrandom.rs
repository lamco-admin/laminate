// Iteration 163: UUID v4 with non-random bits — still valid format?
// Target #108 — UUID v4 spec says bits 48-51 = 0100 (version) and bits 64-65 = 10 (variant).
// A UUID that violates these constraints is still a valid UUID *format* but not a valid v4.
// Question: does validate() accept UUIDs with wrong version/variant bits?

use laminate::packs::identifiers::{detect, validate, IdentifierType};

#[test]
fn uuid_v4_standard() {
    // A proper v4 UUID: version nibble = 4, variant bits = 10xx
    let result = validate("550e8400-e29b-41d4-a716-446655440000", IdentifierType::Uuid);
    assert!(result.is_valid);
}

#[test]
fn uuid_all_zeros() {
    // "Nil UUID" — all zeros. Valid format but not v4.
    let result = validate("00000000-0000-0000-0000-000000000000", IdentifierType::Uuid);
    assert!(
        result.is_valid,
        "Nil UUID should be valid format: {:?}",
        result.error
    );
}

#[test]
fn uuid_all_f() {
    // All F's — "max UUID". Valid format.
    let result = validate("ffffffff-ffff-ffff-ffff-ffffffffffff", IdentifierType::Uuid);
    assert!(
        result.is_valid,
        "Max UUID should be valid format: {:?}",
        result.error
    );
}

#[test]
fn uuid_wrong_version_nibble() {
    // Version nibble = 9 (position 13, third group first char)
    // This would be an invalid v4 but valid format
    let result = validate("550e8400-e29b-91d4-a716-446655440000", IdentifierType::Uuid);
    assert!(
        result.is_valid,
        "UUID with wrong version nibble should still be valid format: {:?}",
        result.error
    );
}

#[test]
fn uuid_detect_ranking() {
    // Does detect() identify a UUID-shaped string?
    let candidates = detect("550e8400-e29b-41d4-a716-446655440000");
    let uuid_match = candidates.iter().find(|(t, _)| *t == IdentifierType::Uuid);
    assert!(
        uuid_match.is_some(),
        "detect() should identify UUID: {:?}",
        candidates
    );
}
