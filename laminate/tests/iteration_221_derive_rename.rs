/// Iteration 221: #[laminate(rename = "camelCase")] — JSON key lookup uses renamed key
///
/// Target #221: Verify that renamed fields look up the JSON key by the rename
/// value, not the Rust field name. Also test round-trip: to_value() should
/// use the renamed key too.
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct ApiResponse {
    #[laminate(rename = "statusCode")]
    status_code: i64,
    #[laminate(rename = "errorMessage")]
    error_message: String,
    data: String,
}

#[test]
fn rename_reads_from_camel_case_key() {
    let json = r#"{"statusCode": 200, "errorMessage": "none", "data": "ok"}"#;
    let (resp, diagnostics) = ApiResponse::from_json(json).unwrap();

    println!("resp = {:?}", resp);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.error_message, "none");
    assert_eq!(resp.data, "ok");
    assert!(diagnostics.is_empty(), "no diagnostics for clean parse");
}

#[test]
fn rename_rust_field_name_is_unknown() {
    // Using Rust field names instead of renamed JSON keys should fail
    let json = r#"{"status_code": 200, "error_message": "none", "data": "ok"}"#;
    let result = ApiResponse::from_json(json);

    println!("result = {:?}", result);

    // "statusCode" and "errorMessage" are missing → error on first required field
    assert!(
        result.is_err(),
        "Rust field names should NOT work when renamed"
    );
}

#[test]
fn rename_round_trip_preserves_renamed_keys() {
    let json = r#"{"statusCode": 200, "errorMessage": "none", "data": "ok"}"#;
    let (resp, _) = ApiResponse::from_json(json).unwrap();

    let value = resp.to_value();
    println!("round-trip value = {}", value);

    // to_value() should use the renamed keys, not Rust field names
    assert!(
        value.get("statusCode").is_some(),
        "to_value should use renamed key 'statusCode'"
    );
    assert!(
        value.get("errorMessage").is_some(),
        "to_value should use renamed key 'errorMessage'"
    );
    assert!(
        value.get("status_code").is_none(),
        "Rust field name should NOT appear"
    );
    assert!(
        value.get("error_message").is_none(),
        "Rust field name should NOT appear"
    );
}
