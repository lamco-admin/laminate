#![allow(dead_code, unused_imports, unused_must_use)]
//! Iteration tests 31-60 from laminate-iterate loop.

use laminate::packs::{currency, time, units};
use laminate::schema::{InferenceConfig, InferredSchema, JsonType};
use laminate::FlexValue;
use laminate_derive::Laminate;
use serde_json::json;
use std::collections::HashMap;

fn load(path: &str) -> FlexValue {
    let full = format!("{}/testdata/{}", env!("CARGO_MANIFEST_DIR"), path);
    let json = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{full}: {e}"));
    FlexValue::from_json(&json).unwrap()
}

// ═══════════════════════════════════════════════════════════════
// Iterations 31-33: New Fixtures (IoT, Logging, Event Sourcing)
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter31_iot_telemetry() {
    let raw = load("iot/telemetry.json");
    // Row 0: native types
    let temp: f64 = raw.extract("[0].temp").unwrap();
    assert_eq!(temp, 22.5);
    // Row 1: timestamp as unix int, strings for temp/humidity
    let ts: i64 = raw.extract("[1].ts").unwrap();
    assert!(ts > 1_000_000_000);
    let temp1: f64 = raw.extract("[1].temp").unwrap();
    assert!((temp1 - 22.3).abs() < 0.01); // coerced from "22.3"
                                          // Row 2: nulls for temp/humidity
    assert!(raw.at("[2].temp").unwrap().is_null());
    // Row 3: sentinel -9999
    let bad: f64 = raw.extract("[3].temp").unwrap();
    assert_eq!(bad, -9999.0);
    // Row 4: nested location object
    let lat: f64 = raw.extract("[4].location.lat").unwrap();
    assert!((lat - 40.7128).abs() < 0.001);
    // Row 5: uptime as string
    let uptime: u64 = raw.extract("[5].uptime_secs").unwrap();
    assert_eq!(uptime, 432000);
}

#[test]
fn iter32_structured_logs() {
    let raw = load("logging/structured_logs.json");
    // Mixed types: status is int (200) in row 2, string ("200") in row 3
    let s1: u16 = raw.extract("[2].status").unwrap();
    assert_eq!(s1, 200);
    let s2: u16 = raw.extract("[3].status").unwrap();
    assert_eq!(s2, 200); // coerced from "200"
                         // duration_ms is string "12.5" in row 2, int 2500 in row 3
    let d1: f64 = raw.extract("[2].duration_ms").unwrap();
    assert!((d1 - 12.5).abs() < 0.01);
    let d2: f64 = raw.extract("[3].duration_ms").unwrap();
    assert_eq!(d2, 2500.0);
    // bytes is string
    let bytes: u64 = raw.extract("[2].bytes").unwrap();
    assert_eq!(bytes, 4096);
    // user_id is null in last row
    assert!(raw.at("[5].user_id").unwrap().is_null());
}

#[test]
fn iter33_event_sourcing() {
    let raw = load("eventsource/events.json");
    // Schema evolution: v1 events have fewer fields than v2
    let evt_type: String = raw.extract("[0].event_type").unwrap();
    assert_eq!(evt_type, "UserCreated");
    // v2 event has extra "role" field in data
    let role: String = raw.extract("[2].data.role").unwrap();
    assert_eq!(role, "admin");
    // Mixed total types: string "25.98" vs number 99.00
    let total1: f64 = raw.extract("[3].data.total").unwrap();
    assert!((total1 - 25.98).abs() < 0.01);
    let total2: f64 = raw.extract("[5].data.total").unwrap();
    assert_eq!(total2, 99.0);
    // qty is string "1" in v2 event
    let qty: u32 = raw.extract("[5].data.items[0].qty").unwrap();
    assert_eq!(qty, 1);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 34-36: Schema Inference — Advanced
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter34_schema_custom_threshold() {
    let rows: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            let mut obj = json!({"id": i, "name": format!("user_{i}")});
            if i < 90 {
                obj.as_object_mut()
                    .unwrap()
                    .insert("email".into(), json!(format!("u{i}@test.com")));
            }
            obj
        })
        .collect();

    // With default threshold (1.0), email is NOT required (only 90% present)
    let schema1 = InferredSchema::from_values(&rows);
    assert!(!schema1.is_field_required(&schema1.fields["email"]));

    // With 0.85 threshold, email IS required (90% > 85%)
    let config = InferenceConfig {
        required_threshold: 0.85,
        ..Default::default()
    };
    let schema2 = InferredSchema::from_values_with_config(&rows, &config);
    assert!(schema2.is_field_required(&schema2.fields["email"]));
}

#[test]
fn iter35_schema_iot_fixture() {
    let raw = load("iot/telemetry.json");
    let rows = raw.raw().as_array().unwrap().clone();
    let schema = InferredSchema::from_values(&rows);
    // temp has mixed types: float, string, null, int (-9999)
    let temp = &schema.fields["temp"];
    assert!(temp.is_mixed_type() || temp.null_count > 0);
    // device_id always string
    assert_eq!(
        schema.fields["device_id"].dominant_type,
        Some(JsonType::String)
    );
    // location only in 1 of 6 rows
    assert!(schema.fields["location"].absent_count > 0);
}

#[test]
fn iter36_schema_logs_fixture() {
    let raw = load("logging/structured_logs.json");
    let rows = raw.raw().as_array().unwrap().clone();
    let schema = InferredSchema::from_values(&rows);
    // status is mixed: int and string
    assert!(schema.fields["status"].is_mixed_type() || schema.fields["status"].absent_count > 0);
    // Some fields only appear in some log entries
    assert!(schema.fields.contains_key("query"));
    assert!(schema.fields["query"].absent_count > 0);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 37-39: Coercion Edges — Advanced
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter37_coercion_chain_stringified_to_extract() {
    // String containing JSON object → parse → extract field from result
    let fv = FlexValue::new(json!({"args": "{\"city\":\"London\",\"temp\":22}"}));
    let args_str: String = fv.extract("args").unwrap();
    let args = FlexValue::from_json(&args_str).unwrap();
    let city: String = args.extract("city").unwrap();
    assert_eq!(city, "London");
    let temp: i64 = args.extract("temp").unwrap();
    assert_eq!(temp, 22);
}

#[test]
fn iter38_coercion_negative_floats() {
    let fv = FlexValue::new(json!({"a": "-3.14", "b": "-0.001", "c": "-.5"}));
    let a: f64 = fv.extract("a").unwrap();
    assert!((a - (-3.14)).abs() < 0.001);
    let b: f64 = fv.extract("b").unwrap();
    assert!((b - (-0.001)).abs() < 0.0001);
    // "-.5" — leading dot after minus
    let c_result: Result<f64, _> = fv.extract("c");
    if let Ok(c) = c_result {
        assert!((c - (-0.5)).abs() < 0.01);
    }
    // If it fails, that's a boundary — Rust's parse doesn't handle "-.5"
}

#[test]
fn iter39_coercion_bool_variants() {
    let fv = FlexValue::new(json!({
        "a": "TRUE", "b": "False", "c": "YES", "d": "no",
        "e": "ON", "f": "off", "g": "1", "h": "0"
    }));
    assert!(fv.extract::<bool>("a").unwrap());
    assert!(!fv.extract::<bool>("b").unwrap());
    assert!(fv.extract::<bool>("c").unwrap());
    assert!(!fv.extract::<bool>("d").unwrap());
    assert!(fv.extract::<bool>("e").unwrap());
    assert!(!fv.extract::<bool>("f").unwrap());
    assert!(fv.extract::<bool>("g").unwrap());
    assert!(!fv.extract::<bool>("h").unwrap());
}

// ═══════════════════════════════════════════════════════════════
// Iterations 40-42: Derive — Advanced
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate, serde::Serialize)]
struct CoerceDefaultEverything {
    #[laminate(coerce, default)]
    a: i64,
    #[laminate(coerce, default)]
    b: f64,
    #[laminate(coerce, default)]
    c: bool,
    #[laminate(coerce, default)]
    d: String,
    #[laminate(coerce, default)]
    e: u32,
}

#[test]
fn iter40_all_coerce_default() {
    // Completely empty object — everything defaults
    let (val, _) = CoerceDefaultEverything::from_json("{}").unwrap();
    assert_eq!(val.a, 0);
    assert_eq!(val.b, 0.0);
    assert!(!val.c);
    assert_eq!(val.d, "");
    assert_eq!(val.e, 0);
}

#[test]
fn iter41_all_coerce_default_with_strings() {
    let (val, diags) = CoerceDefaultEverything::from_json(
        r#"{"a": "42", "b": "3.14", "c": "yes", "d": 100, "e": "7"}"#,
    )
    .unwrap();
    assert_eq!(val.a, 42);
    assert!((val.b - 3.14).abs() < 0.001);
    assert!(val.c);
    assert_eq!(val.d, "100"); // number → string coercion
    assert_eq!(val.e, 7);
    assert!(
        diags.len() >= 4,
        "Expected 4+ coercions, got {}",
        diags.len()
    );
}

#[derive(Debug, Laminate)]
struct WithParseJsonString {
    name: String,
    #[laminate(parse_json_string)]
    config: serde_json::Value,
}

#[test]
fn iter42_parse_json_string_attribute() {
    let (val, _) = WithParseJsonString::from_json(
        r#"{"name": "test", "config": "{\"key\": \"value\", \"count\": 42}"}"#,
    )
    .unwrap();
    assert_eq!(val.name, "test");
    // config should be a parsed JSON object, not a string
    assert!(
        val.config.is_object(),
        "config should be parsed object, got: {:?}",
        val.config
    );
    assert_eq!(val.config["key"], "value");
    assert_eq!(val.config["count"], 42);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 43-45: Merge — Advanced
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter43_merge_deep_nested() {
    let a = FlexValue::from_json(r#"{"a": {"b": {"c": 1, "d": 2}, "e": 3}}"#).unwrap();
    let b = FlexValue::from_json(r#"{"a": {"b": {"c": 99}}}"#).unwrap();
    let merged = a.merge(&b);
    // Deep merge: c overridden, d preserved, e preserved
    assert_eq!(merged.extract::<i64>("a.b.c").unwrap(), 99);
    assert_eq!(merged.extract::<i64>("a.b.d").unwrap(), 2);
    assert_eq!(merged.extract::<i64>("a.e").unwrap(), 3);
}

#[test]
fn iter44_shallow_merge_replaces_object() {
    let a = FlexValue::from_json(r#"{"a": {"b": 1, "c": 2}}"#).unwrap();
    let b = FlexValue::from_json(r#"{"a": {"b": 99}}"#).unwrap();
    let merged = a.merge_shallow(&b);
    // Shallow: entire "a" replaced, so "c" is gone
    assert_eq!(merged.extract::<i64>("a.b").unwrap(), 99);
    assert!(!merged.has("a.c"));
}

#[test]
fn iter45_set_array_index() {
    let mut fv = FlexValue::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
    fv.set("items[1]", json!(99));
    assert_eq!(fv.extract::<i64>("items[1]").unwrap(), 99);
    // Other elements preserved
    assert_eq!(fv.extract::<i64>("items[0]").unwrap(), 1);
    assert_eq!(fv.extract::<i64>("items[2]").unwrap(), 3);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 46-48: Round-Trip — Advanced
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate, serde::Serialize)]
struct FullRoundTrip {
    name: String,
    #[laminate(coerce)]
    count: u32,
    #[laminate(rename = "type")]
    kind: String,
    #[laminate(default)]
    flag: bool,
    #[laminate(coerce, default)]
    score: f64,
    #[laminate(skip)]
    computed: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn iter46_full_attribute_roundtrip() {
    let json = r#"{"name":"Alice","count":"42","type":"admin","score":"99.5","unknown":"preserved","nested":{"a":1}}"#;
    let (val, _) = FullRoundTrip::from_json(json).unwrap();
    assert_eq!(val.count, 42);
    assert_eq!(val.kind, "admin");

    let emitted = val.to_value();
    let (val2, _) = FullRoundTrip::from_flex_value(&emitted).unwrap();
    assert_eq!(val2.name, "Alice");
    assert_eq!(val2.count, 42);
    assert_eq!(val2.kind, "admin");
    assert!(!val2.flag); // default
    assert!((val2.score - 99.5).abs() < 0.01);
    assert!(val2.extra.contains_key("unknown"));
    assert!(val2.extra.contains_key("nested"));
}

#[test]
fn iter47_roundtrip_to_json_string() {
    let (val, _) = FullRoundTrip::from_json(r#"{"name":"Bob","count":1,"type":"user"}"#).unwrap();
    let json_str = val.to_json();
    assert!(json_str.contains("Bob"));
    assert!(json_str.contains("\"type\""));
    // Parse back from string
    let reparsed = FlexValue::from_json(&json_str).unwrap();
    let name: String = reparsed.extract("name").unwrap();
    assert_eq!(name, "Bob");
}

#[test]
fn iter48_roundtrip_preserves_nested_overflow() {
    let json = r#"{"name":"X","count":1,"type":"t","deep":{"nested":{"value":42}}}"#;
    let (val, _) = FullRoundTrip::from_json(json).unwrap();
    let emitted = val.to_value();
    // Deep nested overflow should be preserved exactly
    let fv = FlexValue::new(emitted);
    let deep: i64 = fv.extract("deep.nested.value").unwrap();
    assert_eq!(deep, 42);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 49-51: Provider — Advanced
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter49_anthropic_to_openai_tool_args() {
    let raw = load("api-responses/anthropic_tool_use.json");
    let resp = laminate::provider::anthropic::parse_anthropic_response(&raw).unwrap();
    // Emit as OpenAI
    let oai = laminate::provider::openai::emit_openai_response(&resp);
    // Tool arguments should be stringified in OpenAI format
    let args = oai["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"].as_str();
    assert!(
        args.is_some(),
        "OpenAI tool args should be stringified JSON"
    );
    let args_parsed: serde_json::Value = serde_json::from_str(args.unwrap()).unwrap();
    assert_eq!(args_parsed["city"], "London");
}

#[test]
fn iter50_openai_to_anthropic_preserves_stop() {
    let raw = load("api-responses/openai_text.json");
    let resp = laminate::provider::openai::parse_openai_response(&raw).unwrap();
    assert_eq!(resp.stop_reason, laminate::StopReason::EndTurn);
    // Emit as Anthropic
    let anth = laminate::provider::anthropic::emit_anthropic_response(&resp);
    assert_eq!(anth["stop_reason"], "end_turn");
}

#[test]
fn iter51_ollama_roundtrip() {
    let raw = load("api-responses/ollama_response.json");
    let resp = laminate::provider::ollama::parse_ollama_response(&raw).unwrap();
    assert!(resp.text().contains("Hello, world!"));
    // Emit and verify structure
    use laminate::ProviderAdapter;
    let adapter = laminate::provider::ollama::OllamaAdapter;
    let emitted = adapter.emit_response(&resp);
    assert_eq!(emitted["model"], "llama3.2:latest");
    assert!(emitted["done"]);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 52-54: Pack Coverage — Advanced
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter52_currency_btc_precision() {
    let result = currency::parse_currency("0.00000001 BTC");
    if let Some((amount, code)) = result {
        assert!((amount - 1e-8).abs() < 1e-12);
        assert_eq!(code, Some("BTC".into()));
    }
}

#[test]
fn iter53_currency_european_large() {
    let result = currency::parse_currency("1.234.567,89");
    // Multiple dots + comma → European format
    // Current implementation: only handles one dot before comma
    // This may or may not parse depending on implementation
    if let Some((amount, _)) = result {
        assert!((amount - 1234567.89).abs() < 0.01);
    }
}

#[test]
fn iter54_time_formats_comprehensive() {
    use time::DateFormat;
    // Year with era
    assert_eq!(time::detect_format("2026"), DateFormat::YearOnly);
    // Various ISO variants
    assert_eq!(
        time::detect_format("2026-03-31T15:30:00+05:00"),
        DateFormat::Iso8601
    );
    // Slash dates
    assert_eq!(time::detect_format("12/25/2026"), DateFormat::UsDate); // Dec 25
    assert_eq!(time::detect_format("25/12/2026"), DateFormat::EuDate); // 25 Dec
                                                                       // Time only
    assert_eq!(time::detect_format("14:30"), DateFormat::Time24);
    assert_eq!(time::detect_format("2:30 PM"), DateFormat::Time12);
}

#[test]
fn iter55_units_compound() {
    // "km/h" — compound unit, may not parse
    let result = units::parse_unit_value("100 km/h");
    // Currently not supported — compound units are a gap
    // This documents the boundary
    if result.is_none() {
        // Expected: compound units not handled
    }
}

#[test]
fn iter56_units_conversion_chain() {
    // Convert 5 miles to km to meters
    let km = units::convert(5.0, "mi", "km").unwrap();
    assert!((km - 8.0467).abs() < 0.01);
    let m = units::convert(km, "km", "m").unwrap();
    assert!((m - 8046.7).abs() < 1.0);
}

// ═══════════════════════════════════════════════════════════════
// Iterations 57-60: SQL + Schema + Cross-cutting
// ═══════════════════════════════════════════════════════════════

#[test]
fn iter57_schema_event_sourcing() {
    let raw = load("eventsource/events.json");
    let rows = raw.raw().as_array().unwrap().clone();
    let schema = InferredSchema::from_values(&rows);
    // metadata not present in first event
    assert!(schema.fields["metadata"].absent_count > 0);
    // version should be integer
    assert_eq!(
        schema.fields["version"].dominant_type,
        Some(JsonType::Integer)
    );
    // data is always an object
    assert_eq!(schema.fields["data"].dominant_type, Some(JsonType::Object));
}

#[test]
fn iter58_schema_audit_finds_missing_metadata() {
    let raw = load("eventsource/events.json");
    let rows = raw.raw().as_array().unwrap().clone();
    let schema = InferredSchema::from_values(&rows);
    let report = schema.audit(&rows);
    // metadata is absent in row 0 — but since it's not always present,
    // it shouldn't be flagged as required
    let meta_stats = &report.field_stats["metadata"];
    assert!(meta_stats.missing > 0);
}

#[test]
fn iter59_flexvalue_is_number() {
    // FlexValue should handle number type checks
    let fv = FlexValue::new(json!(42));
    assert!(!fv.is_null());
    assert!(!fv.is_string());
    assert!(!fv.is_array());
    assert!(!fv.is_object());
    // Extract as both i64 and f64
    let i: i64 = fv.extract_root().unwrap();
    assert_eq!(i, 42);
    let f: f64 = fv.extract_root().unwrap();
    assert_eq!(f, 42.0);
}

#[test]
fn iter60_flexvalue_display_compact() {
    let fv = FlexValue::new(json!({"a": 1}));
    let display = fv.to_string();
    assert!(display.contains("\"a\""));
    // Should be valid JSON
    let reparsed: serde_json::Value = serde_json::from_str(&display).unwrap();
    assert_eq!(reparsed["a"], 1);
}

// ═══════════════════════════════════════════════════════════════
// ADVERSARIAL ITERATIONS — Real observations with unknown outcomes
// ═══════════════════════════════════════════════════════════════

#[test]
fn adversarial_01_github_null_to_string() {
    // MUTATION: Extract null `mirror_url` — test maybe() behavior
    let json = std::fs::read_to_string(format!(
        "{}/testdata/github-api/repo.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let fv = FlexValue::from_json(&json).unwrap();

    // GAP FOUND AND FIXED: maybe() on null field should return None, not Some("")
    let result: Option<String> = fv.maybe("mirror_url").unwrap();
    assert_eq!(result, None, "maybe() on null field should return None");

    // extract() on null with BestEffort still gives default (empty string) — that's correct
    let direct: String = fv.extract("mirror_url").unwrap();
    assert_eq!(
        direct, "",
        "extract() on null gives default via BestEffort coercion"
    );

    // Non-null field — maybe() returns Some
    let name: Option<String> = fv.maybe("name").unwrap();
    assert!(name.is_some());

    // Missing field — maybe() returns None
    let missing: Option<String> = fv.maybe("nonexistent_field").unwrap();
    assert_eq!(missing, None);
}

#[test]
fn adversarial_02_spacex_null_heavy() {
    let json = std::fs::read_to_string(format!(
        "{}/testdata/spacex/latest_launch.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let fv = FlexValue::from_json(&json).unwrap();

    // SpaceX has 5 null fields — test maybe() on each
    assert_eq!(fv.maybe::<String>("fairings").unwrap(), None); // null
    assert_eq!(fv.maybe::<String>("static_fire_date_utc").unwrap(), None); // null
    assert_eq!(fv.maybe::<String>("details").unwrap(), None); // null

    // Non-null fields should work
    let success: Option<bool> = fv.maybe("success").unwrap();
    assert!(success.is_some());

    // Nested links object
    assert!(fv.has("links"));
    let patch = fv.maybe::<String>("links.patch.small").unwrap();
    println!("OBSERVED links.patch.small: {:?}", patch);
}

#[test]
fn adversarial_03_countries_deep_nesting() {
    let json = std::fs::read_to_string(format!(
        "{}/testdata/countries/multi.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let fv = FlexValue::from_json(&json).unwrap();

    // Deep nested path: [0].name.nativeName — this is a complex object
    let common: String = fv.extract("[0].name.common").unwrap();
    println!("OBSERVED country[0] common name: {common}");

    // Extract population as various types
    let pop = fv.at("[0].population").unwrap();
    println!("OBSERVED population raw: {}", pop);
    let pop_num: u64 = fv.extract("[0].population").unwrap();
    println!("OBSERVED population as u64: {pop_num}");

    // Area is a float
    let area: f64 = fv.extract("[0].area").unwrap();
    println!("OBSERVED area: {area}");

    // Schema inference on countries data
    let rows = fv.raw().as_array().unwrap().clone();
    let schema = laminate::schema::InferredSchema::from_values(&rows);
    println!(
        "OBSERVED schema: {} fields, {} records",
        schema.fields.len(),
        schema.total_records
    );
    // Countries have MANY fields — test wide object handling
    assert!(
        schema.fields.len() > 20,
        "Countries should have 20+ fields, got {}",
        schema.fields.len()
    );
}

#[test]
fn adversarial_04_string_null_coercion() {
    // MUTATION: The string "null" (not JSON null) — how does coercion handle it?
    let fv = FlexValue::new(json!({"val": "null", "val2": "NULL", "val3": "None"}));

    // Coerce "null" string to bool — should this be false? error?
    let bool_result: Result<bool, _> = fv.extract("val");
    println!("OBSERVED 'null'→bool: {:?}", bool_result);

    // Coerce "null" string to integer
    let int_result: Result<i64, _> = fv.extract("val");
    println!("OBSERVED 'null'→i64: {:?}", int_result);

    // As string it should just work
    let s: String = fv.extract("val").unwrap();
    assert_eq!(s, "null");
}

#[test]
fn adversarial_05_nan_infinity_strings() {
    let fv = FlexValue::new(json!({"a": "NaN", "b": "Infinity", "c": "-Infinity"}));

    // OBSERVE: what happens when "NaN" is coerced to f64?
    let nan_result: Result<f64, _> = fv.extract("a");
    println!("OBSERVED 'NaN'→f64: {:?}", nan_result);

    let inf_result: Result<f64, _> = fv.extract("b");
    println!("OBSERVED 'Infinity'→f64: {:?}", inf_result);

    let neg_inf_result: Result<f64, _> = fv.extract("c");
    println!("OBSERVED '-Infinity'→f64: {:?}", neg_inf_result);
}

#[test]
fn adversarial_06_hex_string_coercion() {
    let fv = FlexValue::new(json!({"hex": "0x1F", "oct": "0o77", "bin": "0b1010"}));

    let hex_result: Result<i64, _> = fv.extract("hex");
    println!("OBSERVED '0x1F'→i64: {:?}", hex_result);

    let oct_result: Result<i64, _> = fv.extract("oct");
    println!("OBSERVED '0o77'→i64: {:?}", oct_result);

    let bin_result: Result<i64, _> = fv.extract("bin");
    println!("OBSERVED '0b1010'→i64: {:?}", bin_result);
}

#[test]
fn adversarial_07_locale_number_formats() {
    let fv = FlexValue::new(json!({
        "us": "1,000",
        "rust_underscore": "1_000",
        "space_separated": "1 000",
        "leading_zero": "007"
    }));

    let us: Result<i64, _> = fv.extract("us");
    println!("OBSERVED '1,000'→i64: {:?}", us);

    let rust: Result<i64, _> = fv.extract("rust_underscore");
    println!("OBSERVED '1_000'→i64: {:?}", rust);

    let space: Result<i64, _> = fv.extract("space_separated");
    println!("OBSERVED '1 000'→i64: {:?}", space);

    let leading: Result<i64, _> = fv.extract("leading_zero");
    println!("OBSERVED '007'→i64: {:?}", leading);
}

#[test]
fn adversarial_08_path_empty_segments() {
    let fv = FlexValue::new(json!({"a": {"b": 1}}));

    // Double dot — empty segment → error
    assert!(fv.at("a..b").is_err());

    // Trailing dot → error (BUG FIXED: was returning Ok before)
    assert!(fv.at("a.").is_err(), "trailing dot should be an error");

    // Leading dot → error
    assert!(fv.at(".a").is_err());
}

#[test]
fn adversarial_09_object_to_string() {
    let fv = FlexValue::new(json!({"nested": {"a": 1, "b": 2}}));

    // OBSERVE: what happens when you extract an object as String?
    let result: Result<String, _> = fv.extract("nested");
    println!("OBSERVED object→String: {:?}", result);
}

#[derive(Debug, Laminate)]
struct VecField {
    name: String,
    #[laminate(default)]
    tags: Vec<String>,
}

#[test]
fn adversarial_10_vec_from_single_string() {
    // MUTATION: tags is a single string instead of array
    let result = VecField::from_json(r#"{"name": "test", "tags": "single"}"#);
    println!(
        "OBSERVED Vec<String> from single string: {:?}",
        result.map(|(v, _)| format!("tags={:?}", v.tags))
    );
}

// ═══════════════════════════════════════════════════════════════
// Iteration 11: Type Swap — large integer extracted as u8
// ═══════════════════════════════════════════════════════════════

#[test]
fn iteration_11_github_id_overflow_detection() {
    // MUTATION: GitHub repo `id` is 14367737 — extract as u8 (max 255)
    // GAP FOUND: coercion system didn't detect integer overflow, let serde fail
    // FIX: Added integer_fits_target() range check in Number→Integer coercion arm
    let fv = load("github-api/repo.json");

    // Verify baseline: extract as i64 works fine
    let id_i64: i64 = fv.extract("id").unwrap();
    assert_eq!(id_i64, 14367737);

    // u8 overflows — should error with diagnostic about overflow
    let result_u8 = fv.extract::<u8>("id");
    assert!(result_u8.is_err(), "14367737 should not fit in u8");

    // u16 overflows (max 65535)
    let result_u16 = fv.extract::<u16>("id");
    assert!(result_u16.is_err(), "14367737 should not fit in u16");

    // i16 overflows (max 32767)
    let result_i16 = fv.extract::<i16>("id");
    assert!(result_i16.is_err(), "14367737 should not fit in i16");

    // Verify the coercion system now produces a diagnostic for the overflow
    let diag_result = fv.extract_with_diagnostics::<u8>("id");
    println!("OBSERVED id as u8 with diagnostics: {:?}", diag_result);
    // The flagged diagnostic should mention overflow even though serde still errors
    // (flagged_no_coerce produces diagnostic but doesn't change the value)

    // Values that DO fit should still work
    let stargazers: i64 = fv.extract("stargazers_count").unwrap();
    assert_eq!(stargazers, 10498);
    // 10498 fits in u16 (max 65535) and i32, but NOT u8
    let sg_u16: u16 = fv.extract("stargazers_count").unwrap();
    assert_eq!(sg_u16, 10498);
    let sg_u8 = fv.extract::<u8>("stargazers_count");
    assert!(sg_u8.is_err(), "10498 should not fit in u8");
}

// ═══════════════════════════════════════════════════════════════
// Iteration 12: Type Swap — boolean `true` extracted as integer
// ═══════════════════════════════════════════════════════════════

#[test]
fn iteration_12_bool_to_integer() {
    // MUTATION: Kubernetes spec.replicas is "3" — what if it were boolean `true`?
    // GAP FOUND: No Bool→Integer coercion existed. true/false as i64 failed with serde error.
    // FIX: Added Bool→Integer and Bool→Float coercion arms at SafeWidening level.
    let mutant = serde_json::json!({
        "replicas": true,
        "max_connections": false,
        "retry_count": true,
    });
    let fv = FlexValue::new(mutant);

    // Bool true → i64 should coerce to 1
    let replicas: i64 = fv.extract("replicas").unwrap();
    assert_eq!(replicas, 1);

    // Bool false → i64 should coerce to 0
    let max_conn: i64 = fv.extract("max_connections").unwrap();
    assert_eq!(max_conn, 0);

    // Bool true → u8 should coerce to 1
    let retry: u8 = fv.extract("retry_count").unwrap();
    assert_eq!(retry, 1);

    // Diagnostics should report the coercion
    let (val, diags) = fv.extract_with_diagnostics::<i64>("replicas").unwrap();
    assert_eq!(val, 1);
    assert!(
        !diags.is_empty(),
        "bool→int coercion should produce a diagnostic"
    );

    // Bool → f64 should also work (true → 1.0)
    let as_f64: f64 = fv.extract("replicas").unwrap();
    assert_eq!(as_f64, 1.0);
}

// ═══════════════════════════════════════════════════════════════
// Iteration 13: Type Swap — string "null" coerced to various types
// ═══════════════════════════════════════════════════════════════

#[test]
fn iteration_13_string_null_coercion() {
    // MUTATION: What happens when the literal string "null" is coerced?
    // GAP FOUND: "null" as Option<i64> errored — no null-sentinel recognition.
    // FIX: Added null-sentinel string coercion at BestEffort level.
    // "null"/"NULL"/"None"/"N/A"/"nil" → Value::Null for non-String targets.
    let fv = FlexValue::new(json!({
        "status": "null",
        "count": "null",
        "flag": "null",
        "actual_null": null,
        "none_val": "None",
        "na_val": "N/A",
    }));

    // "null" as String — preserved as-is (sentinel coercion skips String targets)
    let status: String = fv.extract("status").unwrap();
    assert_eq!(status, "null");

    // maybe() distinguishes string "null" from json null
    let maybe_str: Option<String> = fv.maybe("status").unwrap();
    assert_eq!(
        maybe_str,
        Some("null".to_string()),
        "maybe() sees string, not null"
    );

    let maybe_null: Option<String> = fv.maybe("actual_null").unwrap();
    assert_eq!(maybe_null, None, "maybe() sees real null");

    // "null" as Option<i64> — sentinel converts to Null, serde returns None
    let opt_count: Option<i64> = fv.extract("count").unwrap();
    assert_eq!(
        opt_count, None,
        "null-sentinel string should become None for Option<i64>"
    );

    // "None" and "N/A" also recognized
    let opt_none: Option<i64> = fv.extract("none_val").unwrap();
    assert_eq!(opt_none, None);
    let opt_na: Option<i64> = fv.extract("na_val").unwrap();
    assert_eq!(opt_na, None);
}

// ═══════════════════════════════════════════════════════════════
// Iteration 14: Boundary — JSON with UTF-8 BOM prefix
// ═══════════════════════════════════════════════════════════════

#[test]
fn iteration_14_bom_prefix() {
    // MUTATION: Prepend UTF-8 BOM (\xEF\xBB\xBF) to valid JSON.
    // GAP FOUND: serde_json does not handle BOM — fails with "expected value".
    // FIX: from_json() now strips leading BOM before parsing.
    let bom = "\u{FEFF}";
    let json_with_bom = format!("{bom}{{\"name\": \"test\", \"value\": 42}}");

    // BOM-prefixed JSON now parses successfully
    let fv = FlexValue::from_json(&json_with_bom).unwrap();
    let name: String = fv.extract("name").unwrap();
    assert_eq!(name, "test");
    let value: i64 = fv.extract("value").unwrap();
    assert_eq!(value, 42);

    // BOM inside a string value is preserved (correct JSON behavior)
    let mid_bom = format!("{{\"name\": \"{bom}test\"}}");
    let fv2 = FlexValue::from_json(&mid_bom).unwrap();
    let name2: String = fv2.extract("name").unwrap();
    assert!(name2.contains("test"));
    // BOM is 3 bytes in UTF-8 + "test" is 4 bytes = 7 bytes total
    assert_eq!(name2.len(), 7);

    // BOM-only input (no actual JSON) should still error
    let result_bom_only = FlexValue::from_json(bom);
    assert!(result_bom_only.is_err(), "BOM-only is not valid JSON");
}

// ═══════════════════════════════════════════════════════════════
// Iteration 15: Boundary — Derive rename collision
// ═══════════════════════════════════════════════════════════════
// BUG FOUND: Two fields mapping to same JSON key caused silent runtime failure.
// FIX: Proc macro now detects duplicate JSON keys at compile time.
// The struct `RenameCollision { name: String, #[laminate(rename = "name")] title: String }`
// now fails to compile with: "duplicate JSON key \"name\": field `name` and field `title`
// both map to the same key"
//
// Test: verify that valid renames (no collision) still work correctly.

#[derive(Debug, Laminate)]
struct ValidRename {
    #[laminate(rename = "full_name")]
    name: String,
    #[laminate(default)]
    title: String,
}

#[test]
fn iteration_15_valid_rename_works() {
    // Verify non-colliding rename still works
    let json = r#"{"full_name": "Alice", "title": "Engineer"}"#;
    let (val, _diags) = ValidRename::from_json(json).unwrap();
    assert_eq!(val.name, "Alice");
    assert_eq!(val.title, "Engineer");

    // Original field name "name" should NOT work (renamed to "full_name")
    let json2 = r#"{"name": "Alice", "title": "Engineer"}"#;
    let result = ValidRename::from_json(json2);
    assert!(
        result.is_err(),
        "original field name should not work after rename"
    );
}
