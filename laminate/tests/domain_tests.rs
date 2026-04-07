#![allow(dead_code, unused_imports, unused_must_use)]
//! Domain-specific fixture tests covering:
//! - Webhooks (Stripe, GitHub)
//! - Ambiguous dates (ISO, US, EU, GEDCOM, approximate)
//! - Supply chain data (currency symbols, pack sizes, mixed units)
//! - Protocol data (RDP capabilities with hex strings, extension fields)
//! - Schema drift (evolving records across versions)

use laminate::schema::{InferredSchema, JsonType};
use laminate::FlexValue;
use laminate_derive::Laminate;
use std::collections::HashMap;

fn load_fixture(path: &str) -> FlexValue {
    let full_path = format!("{}/testdata/{}", env!("CARGO_MANIFEST_DIR"), path);
    let json = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {full_path}: {e}"));
    FlexValue::from_json(&json).unwrap()
}

// ═══════════════════════════════════════════════════════════════
// Webhooks — deeply nested, optional fields, metadata as strings
// ═══════════════════════════════════════════════════════════════

#[test]
fn stripe_webhook_deep_extraction() {
    let raw = load_fixture("webhooks/stripe_payment.json");

    let event_type: String = raw.extract("type").unwrap();
    assert_eq!(event_type, "payment_intent.succeeded");

    let amount: u64 = raw.extract("data.object.amount").unwrap();
    assert_eq!(amount, 2000);

    let currency: String = raw.extract("data.object.currency").unwrap();
    assert_eq!(currency, "usd");

    // Metadata values are strings — coerce to typed
    let order_id: u64 = raw.extract("data.object.metadata.order_id").unwrap();
    assert_eq!(order_id, 1234);

    let user_id: u64 = raw.extract("data.object.metadata.user_id").unwrap();
    assert_eq!(user_id, 5678);

    // Deeply nested charge data
    let paid: bool = raw.extract("data.object.charges.data[0].paid").unwrap();
    assert!(paid);

    // null field — has() returns true (field exists), value is null
    assert!(raw.has("data.object.charges.data[0].failure_code"));
    let failure_val = raw.at("data.object.charges.data[0].failure_code").unwrap();
    assert!(failure_val.is_null());
}

#[derive(Debug, Laminate)]
struct StripeMetadata {
    #[laminate(coerce)]
    order_id: u64,
    #[laminate(coerce)]
    user_id: u64,
    plan: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn stripe_metadata_derive() {
    let raw = load_fixture("webhooks/stripe_payment.json");
    let meta_val = raw.at("data.object.metadata").unwrap();
    let (meta, diags) = StripeMetadata::from_flex_value(meta_val.raw()).unwrap();

    assert_eq!(meta.order_id, 1234);
    assert_eq!(meta.user_id, 5678);
    assert_eq!(meta.plan, "pro");
    assert!(meta.extra.is_empty());
    assert!(diags.len() >= 2); // two string→u64 coercions
}

#[test]
fn github_webhook_extraction() {
    let raw = load_fixture("webhooks/github_push.json");

    let repo_name: String = raw.extract("repository.full_name").unwrap();
    assert_eq!(repo_name, "lamco-admin/laminate");

    let is_private: bool = raw.extract("repository.private").unwrap();
    assert!(is_private);

    let commit_msg: String = raw.extract("commits[0].message").unwrap();
    assert_eq!(commit_msg, "Phase 1: FlexValue core");

    let added_files = raw.each("commits[0].added");
    assert_eq!(added_files.len(), 1);

    let stars: u64 = raw.extract("repository.stargazers_count").unwrap();
    assert_eq!(stars, 0);
}

// ═══════════════════════════════════════════════════════════════
// Ambiguous dates — the world's most common data parsing bug
// ═══════════════════════════════════════════════════════════════

#[test]
fn date_formats_all_extractable_as_strings() {
    let raw = load_fixture("dates/ambiguous_dates.json");
    let rows = raw.raw().as_array().unwrap();

    let mut extracted = 0;
    for (i, row) in rows.iter().enumerate() {
        let fv = FlexValue::new(row.clone());
        match fv.extract::<String>("created") {
            Ok(s) => {
                extracted += 1;
                eprintln!("Row {i}: '{s}'");
            }
            Err(e) => {
                // null and empty string may fail depending on mode
                eprintln!("Row {i}: {e}");
            }
        }
    }
    // Most should be extractable as strings
    assert!(extracted >= 12, "Expected 12+ extractable, got {extracted}");
}

#[test]
fn date_schema_inference_shows_mixed_types() {
    let raw = load_fixture("dates/ambiguous_dates.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = InferredSchema::from_values(&rows);

    let created = &schema.fields["created"];
    assert_eq!(created.dominant_type, Some(JsonType::String));
    // Has nulls and an integer (unix timestamp)
    assert!(created.null_count > 0);
    assert!(
        created.is_mixed_type(),
        "Expected mixed types (string + integer + null)"
    );

    eprintln!("\n{}", schema.summary());
}

// ═══════════════════════════════════════════════════════════════
// Supply chain — currency, pack sizes, mixed units
// ═══════════════════════════════════════════════════════════════

#[test]
fn supply_chain_schema_inference() {
    let raw = load_fixture("supply-chain/product_catalog.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = InferredSchema::from_values(&rows);

    // price is mostly string but one row has a number
    let price = &schema.fields["price"];
    assert!(price.is_mixed_type() || price.dominant_type == Some(JsonType::String));
    assert!(price.null_count > 0); // one null price

    // active is wildly mixed: "Y", "Yes", true, 1, "TRUE", "N", "y"
    let active = &schema.fields["active"];
    assert!(active.is_mixed_type());

    // hazmat only appears in 2 of 8 rows
    let hazmat = &schema.fields["hazmat"];
    assert_eq!(hazmat.present_count, 2);
    assert_eq!(hazmat.absent_count, 6);

    // thread_pitch only in 1 row
    let thread = &schema.fields["thread_pitch"];
    assert_eq!(thread.present_count, 1);
    assert_eq!(thread.absent_count, 7);

    eprintln!("\n{}", schema.summary());
}

#[test]
fn supply_chain_audit_self() {
    let raw = load_fixture("supply-chain/product_catalog.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = InferredSchema::from_values(&rows);
    let report = schema.audit(&rows);

    // Should find type mismatches (numeric price in mostly-string field)
    // and missing fields (hazmat, thread_pitch absent from most rows)
    eprintln!("\n{}", report.summary());
    for v in &report.violations {
        eprintln!("  {v}");
    }

    // Verify we found something
    assert!(
        report.total_violations > 0 || report.field_stats.values().any(|s| s.coercible > 0),
        "Expected some violations or coercible mismatches"
    );
}

#[test]
fn supply_chain_extract_with_coercion() {
    let raw = load_fixture("supply-chain/product_catalog.json");

    // Row 3 (index 3) has native typed values — no coercion needed
    let row3 = raw.at("[3]").unwrap();
    let price: f64 = row3.extract("price").unwrap();
    assert_eq!(price, 99.0);
    let qty: u32 = row3.extract("qty").unwrap();
    assert_eq!(qty, 12);

    // Row 0 has "$12.99" — this can't be coerced to f64 by the current table
    // (currency symbols need a domain coercion pack)
    let row0 = raw.at("[0]").unwrap();
    let price_result: Result<f64, _> = row0.extract("price");
    assert!(
        price_result.is_err(),
        "Currency string '$12.99' should not coerce to f64 without domain pack"
    );

    // But it extracts fine as a string
    let price_str: String = row0.extract("price").unwrap();
    assert_eq!(price_str, "$12.99");
}

// ═══════════════════════════════════════════════════════════════
// Protocol data — hex strings, bitmasks, extension fields
// ═══════════════════════════════════════════════════════════════

#[test]
fn rdp_capabilities_navigation() {
    let raw = load_fixture("protocols/rdp_capabilities.json");

    let width: u32 = raw.extract("bitmap.desktop_width").unwrap();
    assert_eq!(width, 1920);

    let height: u32 = raw.extract("bitmap.desktop_height").unwrap();
    assert_eq!(height, 1080);

    let bpp: u32 = raw.extract("bitmap.preferred_bits_per_pixel").unwrap();
    assert_eq!(bpp, 32);

    // Hex strings stay as strings — protocol-specific
    let proto_ver: String = raw.extract("general.protocol_version").unwrap();
    assert_eq!(proto_ver, "0x0200");

    // Boolean capabilities
    let resize: bool = raw.extract("bitmap.desktop_resize_flag").unwrap();
    assert!(resize);

    // Unknown extension capability set preserved
    assert!(raw.has("extended_unknown_cap_set"));
    let cap_type: String = raw.extract("extended_unknown_cap_set.cap_type").unwrap();
    assert_eq!(cap_type, "0xFFFF");
}

#[derive(Debug, Laminate)]
struct BitmapCapability {
    preferred_bits_per_pixel: u32,
    desktop_width: u32,
    desktop_height: u32,
    desktop_resize_flag: bool,
    multiple_rectangle_support: bool,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn rdp_bitmap_derive() {
    let raw = load_fixture("protocols/rdp_capabilities.json");
    let bitmap_val = raw.at("bitmap").unwrap();
    let (cap, _) = BitmapCapability::from_flex_value(bitmap_val.raw()).unwrap();

    assert_eq!(cap.preferred_bits_per_pixel, 32);
    assert_eq!(cap.desktop_width, 1920);
    assert_eq!(cap.desktop_height, 1080);
    assert!(cap.desktop_resize_flag);
    assert!(cap.multiple_rectangle_support);
    // remaining fields captured in overflow
    assert!(cap.extra.contains_key("drawing_flags"));
    assert!(cap.extra.contains_key("receive_1_bit_per_pixel"));
}

// ═══════════════════════════════════════════════════════════════
// Schema drift — records evolving across versions
// ═══════════════════════════════════════════════════════════════

#[test]
fn schema_drift_inference() {
    let raw = load_fixture("etl/schema_drift.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = InferredSchema::from_values(&rows);

    // Core fields present in all versions
    let uid = &schema.fields["user_id"];
    assert!(uid.present_count == 8);
    assert!(uid.is_mixed_type()); // one row has string "1007"

    // phone added in v2, absent from v1 rows
    let phone = &schema.fields["phone"];
    assert!(phone.absent_count > 0);
    assert!(phone.null_count > 0); // Frank has null phone

    // role and mfa_enabled added in v3, absent from v1/v2 rows
    let role = &schema.fields["role"];
    assert!(role.absent_count > 0);
    assert_eq!(role.dominant_type, Some(JsonType::String));

    let mfa = &schema.fields["mfa_enabled"];
    assert!(mfa.is_mixed_type()); // Grace has "true" (string), others have bool

    eprintln!("\n{}", schema.summary());
}

#[test]
fn schema_drift_audit() {
    let raw = load_fixture("etl/schema_drift.json");
    let rows = raw.raw().as_array().unwrap().clone();

    let schema = InferredSchema::from_values(&rows);
    let report = schema.audit(&rows);

    eprintln!("\n{}", report.summary());
    for v in &report.violations {
        eprintln!("  {v}");
    }

    // String "1007" and string "true" are coercible to integer/bool (not violations)
    // But we should see coercible mismatches in the stats
    let uid_stats = &report.field_stats["user_id"];
    assert!(
        uid_stats.coercible > 0,
        "String user_id '1007' should be coercible to integer"
    );

    let mfa_stats = &report.field_stats["mfa_enabled"];
    assert!(
        mfa_stats.coercible > 0,
        "String mfa_enabled 'true' should be coercible to bool"
    );
}

#[derive(Debug, Laminate)]
struct UserV3 {
    #[laminate(coerce)]
    user_id: u64,
    name: String,
    email: String,
    #[laminate(default)]
    phone: String,
    #[laminate(default)]
    role: String,
    #[laminate(coerce, default)]
    mfa_enabled: bool,
    version: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn schema_drift_derive_all_versions() {
    let raw = load_fixture("etl/schema_drift.json");
    let rows = raw.raw().as_array().unwrap();

    let mut successes = 0;
    for (i, row) in rows.iter().enumerate() {
        match UserV3::from_flex_value(row) {
            Ok((user, diags)) => {
                successes += 1;
                eprintln!(
                    "Row {i} ({}): uid={}, phone='{}', role='{}', mfa={} [{}d]",
                    user.version,
                    user.user_id,
                    user.phone,
                    user.role,
                    user.mfa_enabled,
                    diags.len()
                );
            }
            Err(e) => {
                eprintln!("Row {i}: FAILED: {e}");
            }
        }
    }

    // All rows should parse with coerce+default handling the version differences
    assert_eq!(successes, 8, "All 8 rows should parse with coerce+default");
}
