//! Iteration 54: JSONPlaceholder /posts — 100 records, schema infer + audit.

use laminate::schema::InferredSchema;
use laminate::FlexValue;

const POSTS_SAMPLE: &str = r#"[
    {"userId":1,"id":1,"title":"post 1","body":"content 1"},
    {"userId":1,"id":2,"title":"post 2","body":"content 2"},
    {"userId":2,"id":3,"title":"post 3","body":"content\nwith newlines"},
    {"userId":2,"id":4,"title":"post 4","body":"content 4"},
    {"userId":3,"id":5,"title":"post 5","body":"content 5"}
]"#;

#[test]
fn infer_posts_schema() {
    let value: serde_json::Value = serde_json::from_str(POSTS_SAMPLE).unwrap();
    let rows = value.as_array().unwrap();
    let schema = InferredSchema::from_values(rows);

    assert_eq!(schema.total_records, 5);
    assert_eq!(schema.fields.len(), 4);

    let id = &schema.fields["id"];
    assert_eq!(id.dominant_type, Some(laminate::schema::JsonType::Integer));
    assert_eq!(id.fill_rate(), 1.0);

    let body = &schema.fields["body"];
    assert_eq!(body.dominant_type, Some(laminate::schema::JsonType::String));
}

#[test]
fn audit_posts_zero_violations() {
    let value: serde_json::Value = serde_json::from_str(POSTS_SAMPLE).unwrap();
    let rows = value.as_array().unwrap();
    let schema = InferredSchema::from_values(rows);
    let report = schema.audit(rows);

    assert_eq!(report.violations.len(), 0);
}

#[test]
fn extract_post_fields() {
    let fv = FlexValue::from_json(POSTS_SAMPLE).unwrap();

    let title: String = fv.extract("[2].title").unwrap();
    assert_eq!(title, "post 3");

    let user_id: i64 = fv.extract("[2].userId").unwrap();
    assert_eq!(user_id, 2);

    // Body with embedded newlines
    let body: String = fv.extract("[2].body").unwrap();
    assert!(body.contains('\n'));
}
