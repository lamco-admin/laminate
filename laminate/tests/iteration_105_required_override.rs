#![cfg(feature = "schema")]
/// Iteration 105: external_required overrides inference in both directions
///
/// is_field_required only checked Some(true), ignoring Some(false).
/// Now any external_required value overrides the inference threshold.
use laminate::schema::InferredSchema;
use serde_json::json;

#[test]
fn external_required_false_overrides_inference() {
    let training = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"name": "Bob", "age": 25}),
    ];
    let mut schema = InferredSchema::from_values(&training);
    schema.fields.get_mut("name").unwrap().external_required = Some(false);

    // Missing "name" should NOT be a violation with external_required=false
    let report = schema.audit(&[json!({"age": 40})]);
    let name_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "name")
        .collect();
    assert!(
        name_violations.is_empty(),
        "external_required=false should suppress MissingRequired, got {:?}",
        name_violations
    );
}

#[test]
fn external_required_true_overrides_inference() {
    // Infer from data where "email" is only present 50% of the time
    let training = vec![
        json!({"name": "Alice", "email": "a@b.com"}),
        json!({"name": "Bob"}),
    ];
    let mut schema = InferredSchema::from_values(&training);
    // email would be inferred as optional (50% fill), but external says required
    schema.fields.get_mut("email").unwrap().external_required = Some(true);

    let report = schema.audit(&[json!({"name": "Charlie"})]);
    let email_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "email")
        .collect();
    assert_eq!(
        email_violations.len(),
        1,
        "external_required=true should enforce MissingRequired"
    );
}
