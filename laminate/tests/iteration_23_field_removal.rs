//! Iteration 23: Field Removal — remove `name` array from FHIR patient[0]
//! PASS: PathNotFound correctly reports the exact missing segment ("[0].name"),
//! not the full requested path. maybe() returns None for mid-path missing fields.

use laminate::FlexValue;

#[test]
fn iter23_mid_path_field_removal_error_reports_exact_segment() {
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("../testdata/healthcare/hl7_fhir_patient.json")).unwrap();
    json[0].as_object_mut().unwrap().remove("name");

    let flex = FlexValue::new(json);

    // extract reports the exact failing segment, not the full path
    let err = flex.extract::<String>("[0].name[0].family").unwrap_err();
    match err {
        laminate::FlexError::PathNotFound { path } => {
            assert_eq!(path, "[0].name", "Should report exact missing segment");
        }
        other => panic!("Expected PathNotFound, got: {:?}", other),
    }

    // maybe() collapses mid-path missing into None
    assert_eq!(flex.maybe::<String>("[0].name[0].family").unwrap(), None);

    // has() returns false
    assert!(!flex.has("[0].name"));

    // Other patients unaffected
    assert_eq!(flex.extract::<String>("[1].name[0].family").unwrap(), "Doe");
}
