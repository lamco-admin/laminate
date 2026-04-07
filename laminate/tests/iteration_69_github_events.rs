//! Iteration 69 — Real Data: GitHub Events API
//!
//! Probe: Heterogeneous payload types, string-encoded large IDs,
//! null fields, varying field sets across event types.

use laminate::schema::{InferredSchema, JsonType};
use laminate::FlexValue;

fn events() -> FlexValue {
    let json = include_str!("../testdata/github-api/events.json");
    FlexValue::from_json(json).unwrap()
}

fn events_raw() -> serde_json::Value {
    let json = include_str!("../testdata/github-api/events.json");
    serde_json::from_str(json).unwrap()
}

// --- Schema inference on heterogeneous events ---

#[test]
fn schema_infers_top_level_fields() {
    let rows = events_raw();
    let rows = rows.as_array().unwrap();
    let schema = InferredSchema::from_values(rows);

    // All events share: id, type, actor, repo, payload, public, created_at
    assert_eq!(schema.total_records, 3);
    assert!(schema.fields.contains_key("id"));
    assert!(schema.fields.contains_key("type"));
    assert!(schema.fields.contains_key("payload"));
    assert!(schema.fields.contains_key("created_at"));

    // id is a STRING in GitHub API (not integer!)
    let id_field = &schema.fields["id"];
    assert_eq!(id_field.dominant_type, Some(JsonType::String));
    assert_eq!(id_field.fill_rate(), 1.0);

    // payload is Object for all events
    let payload = &schema.fields["payload"];
    assert_eq!(payload.dominant_type, Some(JsonType::Object));
}

// --- Path navigation across heterogeneous payloads ---

#[test]
fn navigate_create_event_null_description() {
    let fv = events();

    // CreateEvent (index 0) has null description
    let desc = fv.maybe::<String>("[0].payload.description").unwrap();
    assert!(desc.is_none(), "null description should produce None");

    // But ref exists
    let ref_val: String = fv.extract("[0].payload.ref").unwrap();
    assert!(!ref_val.is_empty());
}

#[test]
fn navigate_push_event_specific_fields() {
    let fv = events();

    // PushEvent (index 1) has push_id — large integer > u32::MAX
    let push_id: i64 = fv.extract("[1].payload.push_id").unwrap();
    assert!(push_id > i32::MAX as i64, "push_id should exceed i32::MAX");

    // push_id as u32 should overflow — test what happens
    let push_id_u32_result = fv.extract::<u32>("[1].payload.push_id");
    // Observe: does it error, silently truncate, or produce a diagnostic?
    println!("push_id as u32: {:?}", push_id_u32_result);
}

#[test]
fn navigate_field_missing_in_some_events() {
    let fv = events();

    // push_id only exists in PushEvent (index 1), not CreateEvent (index 0)
    let missing = fv.maybe::<i64>("[0].payload.push_id").unwrap();
    assert!(missing.is_none(), "push_id should not exist in CreateEvent");

    // description only in CreateEvent (index 0), not PushEvent (index 1)
    let no_desc = fv.maybe::<String>("[1].payload.description").unwrap();
    assert!(
        no_desc.is_none(),
        "description should not exist in PushEvent"
    );
}

// --- String-encoded large numeric IDs ---

#[test]
fn string_id_coerces_to_i64() {
    let fv = events();

    // id is string "10054663802" — extracts to String and i64
    let id_str: String = fv.extract("[0].id").unwrap();
    assert!(id_str.starts_with("100"));

    let id_i64: i64 = fv.extract("[0].id").unwrap();
    assert!(id_i64 > 10_000_000_000);
}

#[test]
fn string_id_overflow_produces_coercion_failed() {
    let fv = events();

    // String "10054663802" > u32::MAX — should produce CoercionFailed, not opaque serde error
    let err = fv.extract::<u32>("[0].id").unwrap_err();
    let msg = format!("{}", err);
    // Should mention overflow, not "invalid type: string"
    assert!(
        msg.contains("overflow"),
        "error should mention overflow, got: {msg}"
    );
}

#[test]
fn string_id_overflow_via_diagnostics_also_clear() {
    let fv = events();

    // extract_with_diagnostics should also produce the overflow error
    let err = fv.extract_with_diagnostics::<u32>("[0].id").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("overflow"),
        "diagnostics path should also mention overflow, got: {msg}"
    );
}

// --- Empty string fields ---

#[test]
fn empty_string_gravatar_id() {
    let fv = events();

    // gravatar_id is always "" (empty string, not null)
    let grav: String = fv.extract("[0].actor.gravatar_id").unwrap();
    assert_eq!(grav, "");

    // As Option<String> — empty string is NOT null, should be Some("")
    let grav_opt: Option<String> = fv.extract("[0].actor.gravatar_id").unwrap();
    assert_eq!(grav_opt, Some(String::new()));
}
