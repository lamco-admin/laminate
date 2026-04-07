//! Iteration 306: detect() on "12-3456789" — SSN vs EIN ranking.
//! With strict formatting: "12-3456789" matches only EIN (NN-NNNNNNN),
//! "123-45-6789" matches only SSN (NNN-NN-NNNN).

use laminate::packs::identifiers::{detect, IdentifierType};

#[test]
fn detect_ein_format_only() {
    // "12-3456789" is EIN format (NN-NNNNNNN), NOT SSN format (NNN-NN-NNNN)
    let results = detect("12-3456789");
    println!("detect results: {:?}", results);

    let ein = results.iter().find(|(t, _)| *t == IdentifierType::UsEin);
    let ssn = results.iter().find(|(t, _)| *t == IdentifierType::UsSsn);

    assert!(ein.is_some(), "EIN should be detected");
    assert!(ssn.is_none(), "SSN should NOT match EIN format");
}

#[test]
fn detect_ssn_format_only() {
    // "123-45-6789" is SSN format (NNN-NN-NNNN), NOT EIN format (NN-NNNNNNN)
    let results = detect("123-45-6789");
    println!("detect results: {:?}", results);

    let ssn = results.iter().find(|(t, _)| *t == IdentifierType::UsSsn);
    let ein = results.iter().find(|(t, _)| *t == IdentifierType::UsEin);

    assert!(ssn.is_some(), "SSN should be detected");
    assert!(ein.is_none(), "EIN should NOT match SSN format");
}
