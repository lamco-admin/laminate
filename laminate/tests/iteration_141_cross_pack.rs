/// Iteration 141: Cross-pack interaction — currency vs unit
///
/// GAP: "126 mg/dL" and "$5.00 kg" not detected.
/// mg/dL not in unit patterns (medical concentration units missing).
/// "$5.00 kg" — composite value not handled by either pack.
/// Fix requires adding medical units to UNIT_PATTERNS.
use laminate::detect::{guess_type, GuessedType};

#[test]
fn medical_mg_dl_not_yet_detected() {
    // GAP: mg/dL not in unit patterns — needs medical concentration units
    let guesses = guess_type("126 mg/dL");
    // Currently returns PlainString — test documents current behavior
    assert!(
        guesses.iter().any(|g| g.kind == GuessedType::PlainString),
        "126 mg/dL currently detected as PlainString (gap documented)"
    );
    // TODO: When medical concentration units are added to UNIT_PATTERNS,
    // this test should be updated to assert UnitValue detection.
}

#[test]
fn pure_currency_still_works() {
    let guesses = guess_type("$12.99");
    assert!(
        guesses.iter().any(|g| g.kind == GuessedType::Currency),
        "$12.99 should be Currency"
    );
    assert_eq!(
        guesses[0].kind,
        GuessedType::Currency,
        "Currency should be top candidate for $12.99"
    );
}
