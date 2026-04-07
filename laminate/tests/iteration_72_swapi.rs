//! Iteration 72 — Real Data: SWAPI (Star Wars API)
//!
//! Probe: String-encoded numbers ("172", "77"), "unknown" sentinel string
//! for missing values (not null, not empty), empty arrays, URL strings.

use laminate::FlexValue;

fn people() -> FlexValue {
    let json = include_str!("../testdata/swapi_people.json");
    FlexValue::from_json(json).unwrap()
}

// --- String-encoded numbers ---

#[test]
fn string_height_coerces_to_numeric() {
    let fv = people();

    // Luke: height="172" — string-encoded integer
    let height_str: String = fv.extract("[0].height").unwrap();
    assert_eq!(height_str, "172");

    let height_i64: i64 = fv.extract("[0].height").unwrap();
    assert_eq!(height_i64, 172);

    let height_f64: f64 = fv.extract("[0].height").unwrap();
    assert!((height_f64 - 172.0).abs() < f64::EPSILON);
}

// --- "unknown" sentinel string ---

#[test]
fn unknown_mass_as_option_f64_returns_none() {
    let fv = people();

    // Finis Valorum (index 3): mass="unknown" — now recognized as null sentinel
    let mass: Option<f64> = fv.extract("[3].mass").unwrap();
    assert!(
        mass.is_none(),
        "\"unknown\" should coerce to None for Option<f64>"
    );
}

#[test]
fn unknown_mass_as_bare_f64_gives_default() {
    let fv = people();

    // Bare f64 (not Option) with "unknown" → null sentinel → default 0.0
    let mass: f64 = fv.extract("[3].mass").unwrap();
    assert_eq!(mass, 0.0, "\"unknown\" → null → default 0.0 at BestEffort");
}

#[test]
fn unknown_mass_as_string_passes_through() {
    let fv = people();

    // As String target — "unknown" is preserved, no null coercion
    let mass: String = fv.extract("[3].mass").unwrap();
    assert_eq!(mass, "unknown");
}

#[test]
fn known_mass_still_coerces() {
    let fv = people();

    // Luke (index 0): mass="77" — still coerces normally
    let mass: f64 = fv.extract("[0].mass").unwrap();
    assert_eq!(mass, 77.0);

    // Watto (index 4): mass="unknown"
    let watto: Option<f64> = fv.extract("[4].mass").unwrap();
    assert!(watto.is_none());
}

// --- Empty species array ---

#[test]
fn empty_species_array() {
    let fv = people();

    // Luke has species: [] (empty)
    let species: Vec<String> = fv.extract("[0].species").unwrap();
    assert!(species.is_empty());

    // C-3PO has species: ["https://swapi.dev/api/species/2/"]
    let species2: Vec<String> = fv.extract("[1].species").unwrap();
    assert_eq!(species2.len(), 1);
}
