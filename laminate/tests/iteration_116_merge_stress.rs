use laminate::diagnostic::DiagnosticKind;
/// Iteration 116: merge() stress — 500 overlapping + 500 unique keys each
///
/// Verifies correctness of deep_merge_values at scale:
/// - Result has exactly 1500 keys (500 shared from b + 500 unique-a + 500 unique-b)
/// - All overlapping keys take b's value (override semantics)
/// - All unique-to-a keys are preserved unchanged
/// - All unique-to-b keys are added
/// - merge_with_diagnostics emits exactly 500 Overridden diagnostics
use laminate::FlexValue;
use serde_json::{Map, Value};

fn build_objects() -> (FlexValue, FlexValue) {
    let mut a_map = Map::new();
    let mut b_map = Map::new();

    // 500 shared keys: a has value i*10, b has value i*10 + 1
    for i in 0..500usize {
        let key = format!("shared_{i:04}");
        a_map.insert(key.clone(), Value::Number((i as u64 * 10).into()));
        b_map.insert(key, Value::Number((i as u64 * 10 + 1).into()));
    }

    // 500 unique-to-a keys
    for i in 0..500usize {
        a_map.insert(
            format!("only_a_{i:04}"),
            Value::String(format!("a_val_{i}")),
        );
    }

    // 500 unique-to-b keys
    for i in 0..500usize {
        b_map.insert(
            format!("only_b_{i:04}"),
            Value::String(format!("b_val_{i}")),
        );
    }

    (
        FlexValue::from(Value::Object(a_map)),
        FlexValue::from(Value::Object(b_map)),
    )
}

#[test]
fn merge_stress_correct_key_count() {
    let (a, b) = build_objects();
    let merged = a.merge(&b);

    let key_count = merged.keys().map(|v| v.len()).unwrap_or(0);
    assert_eq!(
        key_count, 1500,
        "expected 1500 keys: 500 shared + 500 unique-a + 500 unique-b"
    );
}

#[test]
fn merge_stress_shared_keys_take_b_value() {
    let (a, b) = build_objects();
    let merged = a.merge(&b);

    // Spot-check first, middle, last shared keys
    for i in [0usize, 249, 499] {
        let key = format!("shared_{i:04}");
        let expected = (i as i64 * 10 + 1) as i64; // b's value
        let got: i64 = merged
            .extract(&key)
            .unwrap_or_else(|e| panic!("key {key} missing: {e}"));
        assert_eq!(got, expected, "shared key {key} should have b's value");
    }
}

#[test]
fn merge_stress_unique_a_preserved() {
    let (a, b) = build_objects();
    let merged = a.merge(&b);

    for i in [0usize, 249, 499] {
        let key = format!("only_a_{i:04}");
        let got: String = merged
            .extract(&key)
            .unwrap_or_else(|e| panic!("unique-a key {key} missing: {e}"));
        assert_eq!(
            got,
            format!("a_val_{i}"),
            "unique-a key {key} should be preserved"
        );
    }
}

#[test]
fn merge_stress_unique_b_added() {
    let (a, b) = build_objects();
    let merged = a.merge(&b);

    for i in [0usize, 249, 499] {
        let key = format!("only_b_{i:04}");
        let got: String = merged
            .extract(&key)
            .unwrap_or_else(|e| panic!("unique-b key {key} missing: {e}"));
        assert_eq!(
            got,
            format!("b_val_{i}"),
            "unique-b key {key} should be added from b"
        );
    }
}

#[test]
fn merge_stress_diagnostics_count() {
    let (a, b) = build_objects();
    let (merged, diagnostics) = a.merge_with_diagnostics(&b);

    // Merged result should still have 1500 keys
    let key_count = merged.keys().map(|v| v.len()).unwrap_or(0);
    assert_eq!(key_count, 1500, "diagnostic merge: expected 1500 keys");

    // Should have exactly 500 Overridden diagnostics (one per shared key)
    let override_count = diagnostics
        .iter()
        .filter(|d| matches!(d.kind, DiagnosticKind::Overridden { .. }))
        .count();
    assert_eq!(
        override_count, 500,
        "expected exactly 500 Overridden diagnostics for shared keys, got {}",
        override_count
    );
}
