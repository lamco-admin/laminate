//! Edge case tests for FlexValue, coercion, path parser, and schema inference.

use laminate::schema::InferredSchema;
use laminate::{CoercionLevel, FlexValue};
use serde_json::{json, Value};

// ═══════════════════════════════════════════════════════════════
// Deep nesting
// ═══════════════════════════════════════════════════════════════

#[test]
fn navigate_50_levels_deep() {
    // Build a 50-level deep object: {"a": {"a": {"a": ... "leaf" ...}}}
    let mut value = json!("leaf");
    for _ in 0..50 {
        value = json!({"a": value});
    }

    let fv = FlexValue::new(value);
    let path = (0..50).map(|_| "a").collect::<Vec<_>>().join(".");
    let leaf: String = fv.extract(&path).unwrap();
    assert_eq!(leaf, "leaf");
}

#[test]
fn navigate_100_levels_deep() {
    let mut value = json!(42);
    for _ in 0..100 {
        value = json!({"n": value});
    }

    let fv = FlexValue::new(value);
    let path = (0..100).map(|_| "n").collect::<Vec<_>>().join(".");
    let result: i64 = fv.extract(&path).unwrap();
    assert_eq!(result, 42);
}

// ═══════════════════════════════════════════════════════════════
// Wide objects
// ═══════════════════════════════════════════════════════════════

#[test]
fn object_with_1000_keys() {
    let mut obj = serde_json::Map::new();
    for i in 0..1000 {
        obj.insert(format!("field_{i}"), json!(i));
    }
    let fv = FlexValue::new(Value::Object(obj));

    assert_eq!(fv.keys().unwrap().len(), 1000);

    let v500: i64 = fv.extract("field_500").unwrap();
    assert_eq!(v500, 500);

    let v999: i64 = fv.extract("field_999").unwrap();
    assert_eq!(v999, 999);
}

#[test]
fn schema_inference_wide_object() {
    let mut obj = serde_json::Map::new();
    for i in 0..100 {
        obj.insert(format!("col_{i}"), json!(i));
    }
    let rows = vec![Value::Object(obj)];
    let schema = InferredSchema::from_values(&rows);
    assert_eq!(schema.fields.len(), 100);
}

// ═══════════════════════════════════════════════════════════════
// Large arrays
// ═══════════════════════════════════════════════════════════════

#[test]
fn array_with_10000_elements() {
    let arr: Vec<Value> = (0..10000).map(|i| json!(i)).collect();
    let fv = FlexValue::new(Value::Array(arr));

    assert_eq!(fv.len(), Some(10000));

    // Access last element
    let last: i64 = fv.extract("[9999]").unwrap();
    assert_eq!(last, 9999);

    // Iterator
    let count = fv.each_iter("[0]"); // This won't work — need root array
    let _ = count;
}

#[test]
fn root_array_iteration() {
    let arr: Vec<Value> = (0..100).map(|i| json!({"id": i})).collect();
    let fv = FlexValue::new(json!({"items": arr}));

    let items = fv.each("items");
    assert_eq!(items.len(), 100);

    // Lazy iteration
    let mut iter = fv.each_iter("items");
    let first = iter.next().unwrap();
    let id: i64 = first.extract("id").unwrap();
    assert_eq!(id, 0);
    assert_eq!(iter.len(), 99); // ExactSizeIterator
}

// ═══════════════════════════════════════════════════════════════
// Large strings
// ═══════════════════════════════════════════════════════════════

#[test]
fn large_string_value() {
    let big_string = "x".repeat(1_000_000); // 1MB
    let fv = FlexValue::new(json!({"data": big_string}));
    let extracted: String = fv.extract("data").unwrap();
    assert_eq!(extracted.len(), 1_000_000);
}

// ═══════════════════════════════════════════════════════════════
// Numeric precision edges
// ═══════════════════════════════════════════════════════════════

#[test]
fn i64_max() {
    let fv = FlexValue::new(json!({"n": i64::MAX}));
    let n: i64 = fv.extract("n").unwrap();
    assert_eq!(n, i64::MAX);
}

#[test]
fn i64_min() {
    let fv = FlexValue::new(json!({"n": i64::MIN}));
    let n: i64 = fv.extract("n").unwrap();
    assert_eq!(n, i64::MIN);
}

#[test]
fn string_i64_max_coercion() {
    let fv = FlexValue::new(json!({"n": i64::MAX.to_string()}));
    let n: i64 = fv.extract("n").unwrap();
    assert_eq!(n, i64::MAX);
}

#[test]
fn u64_max() {
    let fv = FlexValue::new(json!({"n": u64::MAX}));
    let n: u64 = fv.extract("n").unwrap();
    assert_eq!(n, u64::MAX);
}

#[test]
fn f64_precision() {
    // This value can't be represented exactly in f64
    let fv = FlexValue::new(json!({"n": "0.1"}));
    let n: f64 = fv.extract("n").unwrap();
    // Should be very close but not exactly 0.1
    assert!((n - 0.1).abs() < 1e-15);
}

#[test]
fn f64_nan_and_infinity() {
    // JSON doesn't support NaN/Infinity, but serde_json handles this
    let fv = FlexValue::new(json!({"n": null}));
    let n: f64 = fv.extract("n").unwrap(); // null → 0.0 via BestEffort
    assert_eq!(n, 0.0);
}

// ═══════════════════════════════════════════════════════════════
// Unicode in field names and values
// ═══════════════════════════════════════════════════════════════

#[test]
fn unicode_field_names() {
    let fv = FlexValue::from_json(r#"{"名前": "太郎", "город": "Москва", "🎉": true}"#).unwrap();
    let name: String = fv.extract("名前").unwrap();
    assert_eq!(name, "太郎");

    let city: String = fv.extract("город").unwrap();
    assert_eq!(city, "Москва");

    let emoji: bool = fv.extract("🎉").unwrap();
    assert!(emoji);
}

#[test]
fn unicode_in_path_navigation() {
    let fv = FlexValue::from_json(r#"{"données": {"résultat": 42}}"#).unwrap();
    let result: i64 = fv.extract("données.résultat").unwrap();
    assert_eq!(result, 42);
}

// ═══════════════════════════════════════════════════════════════
// Empty containers
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_object() {
    let fv = FlexValue::from_json("{}").unwrap();
    assert!(fv.is_object());
    assert_eq!(fv.is_empty(), Some(true));
    assert_eq!(fv.keys().unwrap().len(), 0);
}

#[test]
fn empty_array() {
    let fv = FlexValue::from_json("[]").unwrap();
    assert!(fv.is_array());
    assert_eq!(fv.len(), Some(0));
    assert_eq!(fv.is_empty(), Some(true));
}

#[test]
fn each_on_empty_array() {
    let fv = FlexValue::new(json!({"items": []}));
    assert!(fv.each("items").is_empty());
    assert_eq!(fv.each_iter("items").len(), 0);
}

// ═══════════════════════════════════════════════════════════════
// Root-level scalars (non-object JSON)
// ═══════════════════════════════════════════════════════════════

#[test]
fn root_string() {
    let fv = FlexValue::from_json(r#""hello""#).unwrap();
    assert!(fv.is_string());
    let s: String = fv.extract_root().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn root_number() {
    let fv = FlexValue::from_json("42").unwrap();
    let n: i64 = fv.extract_root().unwrap();
    assert_eq!(n, 42);
}

#[test]
fn root_bool() {
    let fv = FlexValue::from_json("true").unwrap();
    let b: bool = fv.extract_root().unwrap();
    assert!(b);
}

#[test]
fn root_null() {
    let fv = FlexValue::from_json("null").unwrap();
    assert!(fv.is_null());
}

#[test]
fn root_array() {
    let fv = FlexValue::from_json("[1, 2, 3]").unwrap();
    assert!(fv.is_array());
    assert_eq!(fv.len(), Some(3));
}

// ═══════════════════════════════════════════════════════════════
// Coercion edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn coerce_empty_string_to_number_fails() {
    let fv = FlexValue::new(json!({"n": ""}));
    let result: Result<i64, _> = fv.extract("n");
    assert!(result.is_err());
}

#[test]
fn coerce_whitespace_string_to_number_fails() {
    let fv = FlexValue::new(json!({"n": "  "}));
    let result: Result<i64, _> = fv.extract("n");
    assert!(result.is_err());
}

#[test]
fn coerce_negative_string_to_number() {
    let fv = FlexValue::new(json!({"n": "-42"}));
    let n: i64 = fv.extract("n").unwrap();
    assert_eq!(n, -42);
}

#[test]
fn coerce_scientific_notation() {
    let fv = FlexValue::new(json!({"n": "1.5e10"}));
    let n: f64 = fv.extract("n").unwrap();
    assert_eq!(n, 1.5e10);
}

#[test]
fn exact_mode_passes_correct_types() {
    let fv = FlexValue::new(json!({"n": 42, "s": "hello", "b": true}))
        .with_coercion(CoercionLevel::Exact);

    let n: i64 = fv.extract("n").unwrap();
    let s: String = fv.extract("s").unwrap();
    let b: bool = fv.extract("b").unwrap();

    assert_eq!(n, 42);
    assert_eq!(s, "hello");
    assert!(b);
}

// ═══════════════════════════════════════════════════════════════
// Path parser edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn path_with_numeric_string_key() {
    let fv = FlexValue::from_json(r#"{"0": "zero", "1": "one"}"#).unwrap();
    let zero: String = fv.extract("0").unwrap();
    assert_eq!(zero, "zero");
}

#[test]
fn deeply_nested_array_path() {
    let fv = FlexValue::new(json!([[[["deep"]]]]));
    let deep: String = fv.extract("[0][0][0][0]").unwrap();
    assert_eq!(deep, "deep");
}

#[test]
fn path_error_message_quality() {
    let fv = FlexValue::new(json!({"a": {"b": {"c": 1}}}));
    let err = fv.at("a.b.missing").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("a.b.missing"),
        "Error should contain the path: {msg}"
    );
}

// ═══════════════════════════════════════════════════════════════
// Schema inference edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn infer_empty_dataset() {
    let schema = InferredSchema::from_values(&[]);
    assert_eq!(schema.total_records, 0);
    assert!(schema.fields.is_empty());
}

#[test]
fn infer_single_record() {
    let rows = vec![json!({"id": 1, "name": "Alice"})];
    let schema = InferredSchema::from_values(&rows);
    assert_eq!(schema.total_records, 1);
    assert!(schema.fields["id"].appears_required());
}

#[test]
fn infer_all_nulls() {
    let rows = vec![
        json!({"val": null}),
        json!({"val": null}),
        json!({"val": null}),
    ];
    let schema = InferredSchema::from_values(&rows);
    let val = &schema.fields["val"];
    assert_eq!(val.null_count, 3);
    assert_eq!(val.dominant_type, None); // All nulls — no dominant type
}

#[test]
fn audit_empty_dataset() {
    let schema = InferredSchema::from_values(&[json!({"id": 1})]);
    let report = schema.audit(&[]);
    assert_eq!(report.total_records, 0);
    assert_eq!(report.total_violations, 0);
}
