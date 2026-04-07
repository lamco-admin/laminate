#![allow(dead_code, unused_imports, unused_must_use)]
//! Integration tests against real-world JSON fixtures.
//!
//! These tests exercise laminate against messy data from various domains:
//! - LLM API responses (Anthropic, OpenAI)
//! - REST API responses with type mismatches
//! - Config files with all-string values
//! - ETL data with inconsistent types

use laminate::provider::anthropic::parse_anthropic_response;
use laminate::provider::openai::parse_openai_response;
use laminate::{CoercionLevel, FlexValue};
use laminate_derive::Laminate;
use std::collections::HashMap;

fn load_fixture(path: &str) -> FlexValue {
    let full_path = format!("{}/testdata/{}", env!("CARGO_MANIFEST_DIR"), path);
    let json = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {full_path}: {e}"));
    FlexValue::from_json(&json).unwrap()
}

// ═══════════════════════════════════════════════════════════════
// Anthropic API fixtures
// ═══════════════════════════════════════════════════════════════

#[test]
fn anthropic_text_response_fixture() {
    let raw = load_fixture("api-responses/anthropic_text.json");
    let resp = parse_anthropic_response(&raw).unwrap();

    assert_eq!(resp.id, "msg_01XFDUDYJgAACzvnptvVoYEL");
    assert_eq!(resp.model, "claude-opus-4-6-20260301");
    assert_eq!(resp.text(), "Hello! How can I help you today?");
    assert!(!resp.has_tool_use());
    assert_eq!(resp.usage.input_tokens, 25);
    assert_eq!(resp.usage.output_tokens, 12);
    assert_eq!(resp.usage.cache_creation_tokens, Some(0));
    assert_eq!(resp.usage.cache_read_tokens, Some(0));
}

#[test]
fn anthropic_tool_use_fixture() {
    let raw = load_fixture("api-responses/anthropic_tool_use.json");
    let resp = parse_anthropic_response(&raw).unwrap();

    assert_eq!(resp.content.len(), 2);
    assert_eq!(resp.text(), "I'll look that up for you.");
    assert!(resp.has_tool_use());

    let (id, name, input) = resp.content[1].as_tool_use().unwrap();
    assert_eq!(id, "toolu_01A09q90qw90lq917835lq9");
    assert_eq!(name, "get_weather");
    let city: String = input.extract("city").unwrap();
    assert_eq!(city, "London");

    assert_eq!(resp.usage.cache_read_tokens, Some(120));
}

// ═══════════════════════════════════════════════════════════════
// OpenAI API fixtures
// ═══════════════════════════════════════════════════════════════

#[test]
fn openai_text_response_fixture() {
    let raw = load_fixture("api-responses/openai_text.json");
    let resp = parse_openai_response(&raw).unwrap();

    assert_eq!(resp.id, "chatcmpl-abc123def456");
    assert_eq!(resp.model, "gpt-4o-2024-08-06");
    assert_eq!(resp.text(), "Hello! How can I assist you today?");
    assert_eq!(resp.usage.input_tokens, 12);
    assert_eq!(resp.usage.output_tokens, 9);
}

#[test]
fn openai_tool_calls_fixture() {
    let raw = load_fixture("api-responses/openai_tool_calls.json");
    let resp = parse_openai_response(&raw).unwrap();

    assert!(resp.has_tool_use());
    assert_eq!(resp.tool_uses().len(), 2);

    let (id1, name1, input1) = resp.content[0].as_tool_use().unwrap();
    assert_eq!(id1, "call_abc123");
    assert_eq!(name1, "get_weather");
    let city: String = input1.extract("city").unwrap();
    assert_eq!(city, "London");

    let (_, name2, input2) = resp.content[1].as_tool_use().unwrap();
    assert_eq!(name2, "search_web");
    let max: u64 = input2.extract("max_results").unwrap();
    assert_eq!(max, 5);
}

// ═══════════════════════════════════════════════════════════════
// Messy REST API — the core laminate use case
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate)]
struct User {
    #[laminate(coerce)]
    id: u64,
    name: String,
    email: String,
    #[laminate(coerce)]
    age: u32,
    #[laminate(coerce)]
    verified: bool,
    score: f64,
    #[laminate(default)]
    tags: Vec<String>,
    #[laminate(default)]
    preferences: serde_json::Value,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Laminate)]
struct Pagination {
    #[laminate(coerce)]
    page: u32,
    #[laminate(coerce)]
    per_page: u32,
    #[laminate(coerce)]
    total: u64,
    #[laminate(coerce)]
    has_next: bool,
}

#[test]
fn messy_rest_api_user_extraction() {
    let raw = load_fixture("api-responses/messy_rest_api.json");

    // Extract user from nested path
    let user_val = raw.at("data.user").unwrap();
    let (user, diags) = User::from_flex_value(user_val.raw()).unwrap();

    assert_eq!(user.id, 12345); // "12345" coerced to u64
    assert_eq!(user.name, "Alice Johnson");
    assert_eq!(user.age, 29); // "29" coerced to u32
    assert!(user.verified); // "true" coerced to bool
    assert_eq!(user.score, 98.0);
    assert_eq!(user.tags, vec!["admin", "beta"]);

    // Diagnostics should report coercions
    assert!(
        diags.len() >= 3,
        "Expected at least 3 coercion diagnostics, got {}",
        diags.len()
    );

    // Extra fields captured
    assert!(user.extra.contains_key("created_at"));
    assert!(user.extra.contains_key("metadata"));
}

#[test]
fn messy_rest_api_pagination() {
    let raw = load_fixture("api-responses/messy_rest_api.json");
    let pag_val = raw.at("data.pagination").unwrap();
    let (pag, diags) = Pagination::from_flex_value(pag_val.raw()).unwrap();

    assert_eq!(pag.page, 1);
    assert_eq!(pag.per_page, 25);
    assert_eq!(pag.total, 142);
    assert!(pag.has_next);

    // All 4 fields were string → number/bool coercions
    assert_eq!(diags.len(), 4);
}

// ═══════════════════════════════════════════════════════════════
// Config file with all-string values (env var / TOML style)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate)]
struct ServerConfig {
    host: String,
    #[laminate(coerce)]
    port: u16,
    #[laminate(coerce)]
    workers: u32,
    #[laminate(coerce)]
    debug: bool,
    #[laminate(coerce)]
    tls: bool,
    #[laminate(coerce)]
    max_connections: u32,
    #[laminate(coerce)]
    timeout_ms: u64,
}

#[derive(Debug, Laminate)]
struct DbConfig {
    url: String,
    #[laminate(coerce)]
    pool_size: u32,
    #[laminate(coerce)]
    max_idle: u32,
    #[laminate(coerce)]
    connect_timeout_secs: u32,
}

#[test]
fn config_server_section() {
    let raw = load_fixture("config/app_config_messy.json");
    let server_val = raw.at("server").unwrap();
    let (cfg, diags) = ServerConfig::from_flex_value(server_val.raw()).unwrap();

    assert_eq!(cfg.host, "0.0.0.0");
    assert_eq!(cfg.port, 8443);
    assert_eq!(cfg.workers, 4);
    assert!(!cfg.debug);
    assert!(cfg.tls);
    assert_eq!(cfg.max_connections, 1000);
    assert_eq!(cfg.timeout_ms, 30000);

    // 6 string-to-number/bool coercions (host stays String, no coercion)
    assert_eq!(diags.len(), 6);
}

#[test]
fn config_database_section() {
    let raw = load_fixture("config/app_config_messy.json");
    let db_val = raw.at("database").unwrap();
    let (cfg, _) = DbConfig::from_flex_value(db_val.raw()).unwrap();

    assert_eq!(cfg.url, "postgresql://user:pass@localhost:5432/mydb");
    assert_eq!(cfg.pool_size, 10);
    assert_eq!(cfg.max_idle, 5);
    assert_eq!(cfg.connect_timeout_secs, 5);
}

#[test]
fn config_unknown_sections_accessible_via_flexvalue() {
    let raw = load_fixture("config/app_config_messy.json");

    // Can navigate to unknown future sections without a struct
    assert!(raw.has("unknown_future_section"));
    let foo: String = raw.extract("unknown_future_section.foo").unwrap();
    assert_eq!(foo, "bar");
    let ver: u64 = raw.extract("unknown_future_section.version").unwrap();
    assert_eq!(ver, 2);
}

// ═══════════════════════════════════════════════════════════════
// ETL — CSV-as-JSON (all strings)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate)]
struct Product {
    #[laminate(coerce)]
    id: u64,
    name: String,
    #[laminate(coerce, default)]
    price: f64,
    #[laminate(coerce)]
    quantity: u32,
    #[laminate(coerce)]
    in_stock: bool,
    #[laminate(coerce, default)]
    weight_kg: f64,
    #[laminate(default)]
    category: String,
}

#[test]
fn csv_clean_rows_parse() {
    let raw = load_fixture("etl/csv_as_json.json");

    // First row — all clean string values that need coercion
    let row0 = raw.at("[0]").unwrap();
    let (product, diags) = Product::from_flex_value(row0.raw()).unwrap();

    assert_eq!(product.id, 1);
    assert_eq!(product.name, "Widget A");
    assert_eq!(product.price, 12.99);
    assert_eq!(product.quantity, 100);
    assert!(product.in_stock);
    assert_eq!(product.weight_kg, 0.5);
    assert_eq!(product.category, "hardware");

    // All numeric/bool fields were coerced from strings
    assert!(
        diags.len() >= 5,
        "Expected 5+ coercions, got {}",
        diags.len()
    );
}

#[test]
fn csv_null_category_defaults() {
    let raw = load_fixture("etl/csv_as_json.json");

    // Row 6 (index 5) — has null category, "yes" for in_stock, non-numeric price
    let row5 = raw.at("[5]").unwrap();
    let (product, _) = Product::from_flex_value(row5.raw()).unwrap();

    assert_eq!(product.name, "Thingamajig F");
    assert_eq!(product.price, 0.0); // "not_a_price" → coercion fails → default 0.0
    assert!(product.in_stock); // "yes" → true via coercion
    assert_eq!(product.category, ""); // null → default empty string
}

#[test]
fn csv_batch_processing() {
    let raw = load_fixture("etl/csv_as_json.json");

    // Process all rows, track successes and failures
    let arr = raw.raw().as_array().unwrap();
    let mut successes = 0;
    let mut failures = 0;
    let mut total_diags = 0;

    for (i, row_val) in arr.iter().enumerate() {
        match Product::from_flex_value(row_val) {
            Ok((_, diags)) => {
                successes += 1;
                total_diags += diags.len();
            }
            Err(e) => {
                failures += 1;
                // Expected failures: row 4 (empty weight), row 5 (non-numeric price)
                eprintln!("Row {i} failed: {e}");
            }
        }
    }

    // At least some rows should succeed
    assert!(
        successes >= 4,
        "Expected at least 4 successes, got {successes}"
    );
    assert!(total_diags > 0, "Expected coercion diagnostics");
    eprintln!("CSV batch: {successes} ok, {failures} failed, {total_diags} diagnostics");
}

// ═══════════════════════════════════════════════════════════════
// ETL — inconsistent types across rows
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Laminate)]
struct UserRecord {
    #[laminate(coerce)]
    user_id: u64,
    #[laminate(coerce)]
    score: f64,
    #[laminate(coerce)]
    active: bool,
    #[laminate(default)]
    joined: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn inconsistent_types_batch() {
    let raw = load_fixture("etl/inconsistent_types.json");
    let arr = raw.raw().as_array().unwrap();

    let mut results: Vec<(
        usize,
        std::result::Result<(UserRecord, Vec<laminate::Diagnostic>), laminate::FlexError>,
    )> = Vec::new();

    for (i, row_val) in arr.iter().enumerate() {
        results.push((i, UserRecord::from_flex_value(row_val)));
    }

    // Row 0: all correct types, no coercion needed
    let (r0, d0) = results[0].1.as_ref().unwrap();
    assert_eq!(r0.user_id, 1001);
    assert_eq!(r0.score, 95.5);
    assert!(r0.active);
    assert!(d0.is_empty() || d0.len() <= 1); // score 95.5 is already f64

    // Row 1: all strings — coercion should handle
    let (r1, d1) = results[1].1.as_ref().unwrap();
    assert_eq!(r1.user_id, 1002);
    assert_eq!(r1.score, 87.3);
    assert!(r1.active);
    assert!(d1.len() >= 3, "Expected 3+ coercions for all-string row");

    // Row 2: integer 92 for score (should coerce to f64), integer 1 for active
    let (r2, _) = results[2].1.as_ref().unwrap();
    assert_eq!(r2.user_id, 1003);
    assert_eq!(r2.score, 92.0);

    // Row 3: null score with coerce — should fail or coerce to 0.0
    // This tests null handling under coercion
    let r3_result = &results[3].1;
    match r3_result {
        Ok((r3, _)) => assert_eq!(r3.score, 0.0), // null coerced to 0.0
        Err(e) => eprintln!("Row 3 (null score): {e}"),
    }

    // Row 5: extra fields should be in overflow
    let (r5, _) = results[5].1.as_ref().unwrap();
    assert_eq!(r5.user_id, 1006);
    assert!(r5.extra.contains_key("extra_field"));
    assert!(r5.extra.contains_key("another"));

    // Summary
    let ok_count = results.iter().filter(|(_, r)| r.is_ok()).count();
    let fail_count = results.iter().filter(|(_, r)| r.is_err()).count();
    eprintln!(
        "Inconsistent types: {ok_count} ok, {fail_count} failed out of {}",
        arr.len()
    );
}

// ═══════════════════════════════════════════════════════════════
// FlexValue ad-hoc extraction (no derive macro)
// ═══════════════════════════════════════════════════════════════

#[test]
fn flexvalue_ad_hoc_messy_api() {
    let raw = load_fixture("api-responses/messy_rest_api.json");

    // Extract with coercion — FlexValue handles string→number
    let user_id: u64 = raw.extract("data.user.id").unwrap();
    let age: u32 = raw.extract("data.user.age").unwrap();
    let verified: bool = raw.extract("data.user.verified").unwrap();
    let page: u32 = raw.extract("data.pagination.page").unwrap();
    let total: u64 = raw.extract("data.pagination.total").unwrap();

    assert_eq!(user_id, 12345);
    assert_eq!(age, 29);
    assert!(verified);
    assert_eq!(page, 1);
    assert_eq!(total, 142);
}

#[test]
fn flexvalue_exact_mode_rejects_string_numbers() {
    let raw = load_fixture("api-responses/messy_rest_api.json").with_coercion(CoercionLevel::Exact);

    // In exact mode, "12345" cannot become u64
    let result: Result<u64, _> = raw.extract("data.user.id");
    assert!(result.is_err(), "Exact mode should reject string→u64");

    // But string→string still works
    let name: String = raw.extract("data.user.name").unwrap();
    assert_eq!(name, "Alice Johnson");
}

#[test]
fn flexvalue_with_diagnostics_tracks_coercions() {
    let raw = load_fixture("api-responses/messy_rest_api.json");

    let (age, diags): (u32, _) = raw.extract_with_diagnostics("data.user.age").unwrap();
    assert_eq!(age, 29);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].risk, laminate::RiskLevel::Info);
}

// ═══════════════════════════════════════════════════════════════
// Schema inference from fixture data
// ═══════════════════════════════════════════════════════════════

#[test]
fn infer_schema_from_csv_fixture() {
    let raw = load_fixture("etl/csv_as_json.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = laminate::InferredSchema::from_values(&rows);

    // All CSV data is strings — schema should reflect that
    assert_eq!(
        schema.fields["id"].dominant_type,
        Some(laminate::schema::JsonType::String)
    );
    assert_eq!(
        schema.fields["price"].dominant_type,
        Some(laminate::schema::JsonType::String)
    );
    assert_eq!(
        schema.fields["name"].dominant_type,
        Some(laminate::schema::JsonType::String)
    );

    // category has one null — not fully required
    assert!(!schema.fields["category"].appears_required());
    assert_eq!(schema.fields["category"].null_count, 1);

    eprintln!("\n{}", schema.summary());
}

#[test]
fn infer_schema_from_inconsistent_types() {
    let raw = load_fixture("etl/inconsistent_types.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = laminate::InferredSchema::from_values(&rows);

    // user_id: mix of integer and string — integer should dominate
    let uid = &schema.fields["user_id"];
    assert!(uid.is_mixed_type());

    // score: mix of float, integer, string, null
    let score = &schema.fields["score"];
    assert!(score.is_mixed_type());
    assert!(score.null_count > 0);

    // extra_field only in 1 of 6 rows
    let extra = &schema.fields["extra_field"];
    assert_eq!(extra.present_count, 1);
    assert_eq!(extra.absent_count, 5);

    eprintln!("\n{}", schema.summary());
}

#[test]
fn audit_inconsistent_types_against_inferred_schema() {
    let raw = load_fixture("etl/inconsistent_types.json");
    let rows = raw.raw().as_array().unwrap().clone();

    // Infer from the data itself, then audit the same data
    let schema = laminate::InferredSchema::from_values(&rows);
    let report = schema.audit(&rows);

    // The data should have some coercible mismatches (string "1002" in integer field)
    // and possibly some violations (array [1005] in integer field)
    let uid_stats = &report.field_stats["user_id"];
    assert!(
        uid_stats.coercible > 0 || uid_stats.violations > 0,
        "Expected mismatches in user_id: {uid_stats:?}"
    );

    eprintln!("\n{}", report.summary());

    // Print individual violations for inspection
    for v in &report.violations {
        eprintln!("  {v}");
    }
}
