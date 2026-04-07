#![allow(dead_code)]
//! Iteration 205: external_type overrides inferred dominant type.
//!
//! Question: If data is all strings but external_type says Integer,
//! does audit use Integer as the expected type and flag mismatches?

use laminate::schema::{ExternalConstraint, InferredSchema, JsonType};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn external_type_overrides_inferred() {
    // All strings in training
    let rows = vec![
        json!({"code": "abc"}),
        json!({"code": "def"}),
        json!({"code": "ghi"}),
    ];
    let schema = InferredSchema::from_values(&rows);

    // Verify inferred type is String
    let code = schema.fields.get("code").unwrap();
    assert_eq!(code.dominant_type, Some(JsonType::String));

    // Override with external_type = Integer
    let mut constraints = HashMap::new();
    constraints.insert(
        "code".into(),
        ExternalConstraint {
            expected_type: Some(JsonType::Integer),
            ..ExternalConstraint::default()
        },
    );
    let schema = schema.with_constraints(constraints);

    // Effective type should now be Integer
    let code = schema.fields.get("code").unwrap();
    let effective = schema.effective_type(code);
    println!("effective_type: {:?}", effective);
    assert_eq!(
        effective,
        Some(JsonType::Integer),
        "External type should override inferred"
    );

    // Audit with a string value — should now flag TypeMismatch
    let test_data = vec![json!({"code": "xyz"})];
    let report = schema.audit(&test_data);
    println!("Violations: {:?}", report.violations);

    let type_mismatches: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "code")
        .collect();
    assert!(
        !type_mismatches.is_empty(),
        "String value should violate Integer constraint"
    );
}
