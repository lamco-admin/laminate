//! Iteration 196: Schema audit max_length with Unicode — byte-length vs char-length.
//!
//! "日本語" is 3 chars but 9 bytes in UTF-8. max_length should count CHARACTERS.

use laminate::schema::InferredSchema;

#[test]
fn max_length_uses_char_count_not_byte_count() {
    let training = vec![
        serde_json::json!({"label": "hello"}),
        serde_json::json!({"label": "world"}),
    ];

    let mut schema = InferredSchema::from_values(&training);

    // Set max_length = 5 (5 characters)
    schema.fields.get_mut("label").unwrap().max_length = Some(5);

    // "日本語" is 3 chars (well within 5) but 9 bytes
    let test = vec![serde_json::json!({"label": "日本語"})];
    let report = schema.audit(&test);

    println!("Violations: {}", report.violations.len());
    for v in &report.violations {
        println!("  {v}");
    }

    // Should NOT violate: 3 chars ≤ 5 max_length
    assert_eq!(
        report.violations.len(),
        0,
        "3-char Unicode string should not violate max_length=5 (byte len was used instead of char count)"
    );
}

#[test]
fn max_length_rejects_long_unicode_string() {
    let training = vec![serde_json::json!({"label": "hi"})];
    let mut schema = InferredSchema::from_values(&training);
    schema.fields.get_mut("label").unwrap().max_length = Some(3);

    // "日本語テスト" is 6 chars → should violate max_length=3
    let test = vec![serde_json::json!({"label": "日本語テスト"})];
    let report = schema.audit(&test);
    assert_eq!(
        report.violations.len(),
        1,
        "6-char Unicode string should violate max_length=3"
    );
}
