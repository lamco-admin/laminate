use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::{json, Value};

use laminate::coerce::{coerce_value, CoercionLevel};
use laminate::detect::guess_type;
use laminate::packs::currency::detect_currency_format;
use laminate::packs::identifiers::{detect as detect_id, validate, IdentifierType};
use laminate::packs::time::{convert_to_iso8601, detect_format};
use laminate::path::parse_path;
use laminate::streaming::sse::SseParser;
use laminate::value::FlexValue;

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

fn sample_json() -> &'static str {
    r#"{"user":{"name":"Alice","age":"30","scores":[98,85,92]},"debug":"true","port":"8080"}"#
}

fn schema_rows() -> Vec<Value> {
    (0..100)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("user_{i}"),
                "age": 20 + (i % 50),
                "score": 50.0 + (i as f64) * 0.5,
                "active": i % 3 != 0,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// FlexValue vs raw serde_json comparison
// ---------------------------------------------------------------------------

fn bench_parse(c: &mut Criterion) {
    let json_str = sample_json();

    let mut group = c.benchmark_group("parse");
    group.bench_function("serde_json::from_str", |b| {
        b.iter(|| {
            let _: Value = serde_json::from_str(black_box(json_str)).unwrap();
        });
    });
    group.bench_function("FlexValue::from_json", |b| {
        b.iter(|| {
            let _ = FlexValue::from_json(black_box(json_str)).unwrap();
        });
    });
    group.finish();
}

fn bench_extract_i64(c: &mut Criterion) {
    let json_str = sample_json();
    let flex = FlexValue::from_json(json_str).unwrap();
    let raw: Value = serde_json::from_str(json_str).unwrap();

    let mut group = c.benchmark_group("extract_i64");
    group.bench_function("serde_json (manual navigation)", |b| {
        b.iter(|| {
            let v = black_box(&raw)
                .get("user")
                .and_then(|u| u.get("scores"))
                .and_then(|s| s.get(0))
                .and_then(|v| v.as_i64())
                .unwrap();
            black_box(v);
        });
    });
    group.bench_function("FlexValue::extract", |b| {
        b.iter(|| {
            let v: i64 = black_box(&flex).extract("user.scores[0]").unwrap();
            black_box(v);
        });
    });
    group.finish();
}

fn bench_extract_with_coercion(c: &mut Criterion) {
    let json_str = r#"{"port": "8080"}"#;
    let flex = FlexValue::from_json(json_str).unwrap();

    c.bench_function("extract_coerced_string_to_u16", |b| {
        b.iter(|| {
            let v: u16 = black_box(&flex).extract("port").unwrap();
            black_box(v);
        });
    });
}

// ---------------------------------------------------------------------------
// Coercion levels
// ---------------------------------------------------------------------------

fn bench_coercion_levels(c: &mut Criterion) {
    let string_val = Value::String("42".into());
    let int_val = json!(42);

    let mut group = c.benchmark_group("coerce_value");
    for level in [
        CoercionLevel::Exact,
        CoercionLevel::SafeWidening,
        CoercionLevel::StringCoercion,
        CoercionLevel::BestEffort,
    ] {
        group.bench_function(format!("{level:?}/int_to_i64"), |b| {
            b.iter(|| {
                let r = coerce_value(black_box(&int_val), "i64", level, "bench");
                black_box(r);
            });
        });
        group.bench_function(format!("{level:?}/string_to_i64"), |b| {
            b.iter(|| {
                let r = coerce_value(black_box(&string_val), "i64", level, "bench");
                black_box(r);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// guess_type
// ---------------------------------------------------------------------------

fn bench_guess_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("guess_type");
    let cases = [
        ("integer", "42"),
        ("float", "3.14159"),
        ("date_iso", "2026-04-05"),
        ("currency", "$12.99"),
        ("uuid", "550e8400-e29b-41d4-a716-446655440000"),
        ("email", "alice@example.com"),
        ("url", "https://example.com/path"),
        ("plain_string", "hello world"),
        ("null_sentinel", "N/A"),
        ("european_number", "1.234,56"),
    ];
    for (name, input) in cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let g = guess_type(black_box(input));
                black_box(g);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Path parsing
// ---------------------------------------------------------------------------

fn bench_path_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_parse");
    group.bench_function("simple_key", |b| {
        b.iter(|| black_box(parse_path(black_box("name")).unwrap()));
    });
    group.bench_function("nested_dot", |b| {
        b.iter(|| black_box(parse_path(black_box("user.profile.name")).unwrap()));
    });
    group.bench_function("array_index", |b| {
        b.iter(|| black_box(parse_path(black_box("users[0].scores[2]")).unwrap()));
    });
    group.bench_function("quoted_key", |b| {
        b.iter(|| black_box(parse_path(black_box(r#"meta["content-type"]"#)).unwrap()));
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Schema inference
// ---------------------------------------------------------------------------

#[cfg(feature = "schema")]
fn bench_schema_inference(c: &mut Criterion) {
    let rows = schema_rows();

    let mut group = c.benchmark_group("schema");
    group.bench_function("infer_100_rows", |b| {
        b.iter(|| {
            let schema = laminate::schema::InferredSchema::from_values(black_box(&rows));
            black_box(schema);
        });
    });

    // Inference + audit cycle
    let schema = laminate::schema::InferredSchema::from_values(&rows);
    group.bench_function("audit_100_rows", |b| {
        b.iter(|| {
            let report = black_box(&schema).audit(black_box(&rows));
            black_box(report);
        });
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Date parsing
// ---------------------------------------------------------------------------

fn bench_date_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("date_parse");
    let cases = [
        ("iso8601", "2026-04-06T14:30:00Z"),
        ("iso_date", "2026-04-06"),
        ("us_date", "04/15/2026"),
        ("eu_date", "15/04/2026"),
        ("compact_hl7", "20260406"),
        ("unix_seconds", "1711900800"),
        ("long_date", "April 6, 2026"),
        ("not_a_date", "hello world"),
    ];
    for (name, input) in cases {
        group.bench_function(format!("detect/{name}"), |b| {
            b.iter(|| black_box(detect_format(black_box(input))));
        });
        group.bench_function(format!("convert/{name}"), |b| {
            b.iter(|| black_box(convert_to_iso8601(black_box(input))));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// SSE parsing
// ---------------------------------------------------------------------------

fn bench_sse_parsing(c: &mut Criterion) {
    let single_event =
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\n";
    let multi_event = "event: message_start\ndata: {\"type\":\"message_start\"}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hi\"}}\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";

    // Build a 100-event stream
    let large_stream: String = (0..100)
        .map(|i| format!("data: {{\"index\":{i}}}\n\n"))
        .collect();

    let mut group = c.benchmark_group("sse_parse");
    group.bench_function("single_event", |b| {
        b.iter(|| {
            let mut p = SseParser::new();
            let events = p.feed(black_box(single_event));
            black_box(events);
        });
    });
    group.bench_function("3_events", |b| {
        b.iter(|| {
            let mut p = SseParser::new();
            let events = p.feed(black_box(multi_event));
            black_box(events);
        });
    });
    group.bench_function("100_events", |b| {
        b.iter(|| {
            let mut p = SseParser::new();
            let events = p.feed(black_box(&large_stream));
            black_box(events);
        });
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Identifier detection
// ---------------------------------------------------------------------------

fn bench_identifiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("identifiers");
    group.bench_function("detect/credit_card", |b| {
        b.iter(|| black_box(detect_id(black_box("4111111111111111"))));
    });
    group.bench_function("detect/email", |b| {
        b.iter(|| black_box(detect_id(black_box("user@example.com"))));
    });
    group.bench_function("detect/plain_string", |b| {
        b.iter(|| black_box(detect_id(black_box("hello world"))));
    });
    group.bench_function("validate/iban", |b| {
        b.iter(|| {
            black_box(validate(
                black_box("GB29NWBK60161331926819"),
                IdentifierType::Iban,
            ))
        });
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Currency detection
// ---------------------------------------------------------------------------

fn bench_currency(c: &mut Criterion) {
    let mut group = c.benchmark_group("currency");
    let cases = [
        ("usd_symbol", "$1,234.56"),
        ("eur_code", "EUR 1.234,56"),
        ("aud_symbol", "A$ 1,234.56"),
        ("not_currency", "hello world"),
    ];
    for (name, input) in cases {
        group.bench_function(name, |b| {
            b.iter(|| black_box(detect_currency_format(black_box(input))));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

#[cfg(feature = "schema")]
criterion_group!(
    benches,
    bench_parse,
    bench_extract_i64,
    bench_extract_with_coercion,
    bench_coercion_levels,
    bench_guess_type,
    bench_path_parsing,
    bench_schema_inference,
    bench_date_parsing,
    bench_sse_parsing,
    bench_identifiers,
    bench_currency,
);

#[cfg(not(feature = "schema"))]
criterion_group!(
    benches,
    bench_parse,
    bench_extract_i64,
    bench_extract_with_coercion,
    bench_coercion_levels,
    bench_guess_type,
    bench_path_parsing,
    bench_date_parsing,
    bench_sse_parsing,
    bench_identifiers,
    bench_currency,
);

criterion_main!(benches);
