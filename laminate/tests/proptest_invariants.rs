//! Property-based tests for laminate round-trip invariants.
//!
//! These tests use proptest to generate arbitrary inputs and verify
//! that certain properties always hold, regardless of input.

use proptest::prelude::*;

use laminate::FlexValue;

/// Compare JSON values with floating-point tolerance.
fn json_values_approx_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::Number(an), serde_json::Value::Number(bn)) => {
            match (an.as_f64(), bn.as_f64()) {
                (Some(af), Some(bf)) => (af - bf).abs() < 1e-14 * af.abs().max(1.0),
                _ => an == bn,
            }
        }
        (serde_json::Value::Array(aa), serde_json::Value::Array(ba)) => {
            aa.len() == ba.len() && aa.iter().zip(ba).all(|(x, y)| json_values_approx_eq(x, y))
        }
        (serde_json::Value::Object(ao), serde_json::Value::Object(bo)) => {
            ao.len() == bo.len()
                && ao
                    .iter()
                    .all(|(k, v)| bo.get(k).map_or(false, |bv| json_values_approx_eq(v, bv)))
        }
        _ => a == b,
    }
}
use laminate::coerce::{coerce_value, CoercionLevel};
use laminate::detect::guess_type;
use laminate::packs::currency::detect_currency_format;
use laminate::packs::identifiers::{detect as detect_id, validate, IdentifierType};
use laminate::packs::time::{convert_to_iso8601, detect_format};
use laminate::packs::units::parse_unit_value;

// ── FlexValue Round-Trip ────────────────────────────────────────

fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    let leaf = prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i64>().prop_map(|i| serde_json::json!(i)),
        any::<f64>()
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(|f| serde_json::json!(f)),
        "[a-zA-Z0-9 _.-]{0,50}".prop_map(|s| serde_json::json!(s)),
    ];
    leaf.prop_recursive(
        3,  // depth
        64, // max nodes
        8,  // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..5).prop_map(serde_json::Value::Array),
                prop::collection::hash_map("[a-z]{1,8}", inner, 0..5)
                    .prop_map(|m| { serde_json::Value::Object(m.into_iter().collect()) }),
            ]
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// FlexValue from JSON → to_json string → from JSON round-trip preserves structure.
    /// Floating-point values may lose the last ULP due to serialization.
    #[test]
    fn flexvalue_json_roundtrip(value in arb_json_value()) {
        let fv = FlexValue::from(value.clone());
        let json_str = format!("{}", fv);
        let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        prop_assert!(
            json_values_approx_eq(&value, &reparsed),
            "Round-trip mismatch:\n  original: {}\n  reparsed: {}",
            serde_json::to_string(&value).unwrap(),
            serde_json::to_string(&reparsed).unwrap(),
        );
    }

    /// FlexValue::from → raw() returns the original value.
    #[test]
    fn flexvalue_raw_identity(value in arb_json_value()) {
        let fv = FlexValue::from(value.clone());
        prop_assert_eq!(fv.raw(), &value);
    }
}

// ── Provider Round-Trip ─────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Anthropic parse → emit → re-parse preserves text content.
    #[test]
    fn anthropic_roundtrip_text(text in "[a-zA-Z0-9 .,!?]{1,200}") {
        use laminate::provider::anthropic::{parse_anthropic_response, emit_anthropic_response};

        let input = serde_json::json!({
            "id": "msg_test",
            "type": "message",
            "model": "test-model",
            "content": [{"type": "text", "text": text}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 1, "output_tokens": 1}
        });
        let body = FlexValue::from(input);
        let parsed = parse_anthropic_response(&body).unwrap();
        let emitted = emit_anthropic_response(&parsed);
        let body2 = FlexValue::from(emitted);
        let reparsed = parse_anthropic_response(&body2).unwrap();

        prop_assert_eq!(parsed.text(), reparsed.text());
        prop_assert_eq!(parsed.id, reparsed.id);
    }

    /// OpenAI parse → emit → re-parse preserves text content.
    #[test]
    fn openai_roundtrip_text(text in "[a-zA-Z0-9 .,!?]{1,200}") {
        use laminate::provider::openai::{parse_openai_response, emit_openai_response};

        let input = serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": text},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        });
        let body = FlexValue::from(input);
        let parsed = parse_openai_response(&body).unwrap();
        let emitted = emit_openai_response(&parsed);
        let body2 = FlexValue::from(emitted);
        let reparsed = parse_openai_response(&body2).unwrap();

        prop_assert_eq!(parsed.text(), reparsed.text());
    }
}

// ── Schema Invariants ───────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Schema inferred from rows should produce zero violations when auditing the same rows.
    #[test]
    fn schema_audit_training_zero_violations(
        n_rows in 3..20usize,
        n_fields in 1..5usize,
    ) {
        use laminate::schema::InferredSchema;

        // Build uniform rows
        let mut rows = Vec::new();
        for i in 0..n_rows {
            let mut obj = serde_json::Map::new();
            for f in 0..n_fields {
                obj.insert(format!("field_{}", f), serde_json::json!(i * 10 + f));
            }
            rows.push(serde_json::Value::Object(obj));
        }

        let schema = InferredSchema::from_values(&rows);
        let report = schema.audit(&rows);

        prop_assert!(
            report.violations.is_empty(),
            "Auditing training data should produce zero violations, got {}",
            report.violations.len()
        );
    }
}

// ── Date Parser Invariants ──────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// detect_format never panics on any ASCII string.
    #[test]
    fn detect_format_never_panics(s in "\\PC{0,100}") {
        let _ = detect_format(&s);
    }

    /// convert_to_iso8601 never panics on any string.
    #[test]
    fn convert_to_iso8601_never_panics(s in "\\PC{0,100}") {
        let _ = convert_to_iso8601(&s);
    }

    /// If convert_to_iso8601 returns Some, the output should look like a date.
    #[test]
    fn iso8601_output_is_date_shaped(s in "\\PC{0,50}") {
        if let Some(result) = convert_to_iso8601(&s) {
            // ISO output should contain at least YYYY-MM-DD (10 chars with dashes)
            prop_assert!(
                result.len() >= 10,
                "ISO output too short: {:?} → {:?}", s, result
            );
        }
    }
}

// ── Coercion Invariants ─────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Coercion never panics for any string input at any level.
    #[test]
    fn coerce_string_never_panics(
        s in "\\PC{0,100}",
        level_idx in 0..4usize,
        target_idx in 0..6usize,
    ) {
        let levels = [
            CoercionLevel::Exact,
            CoercionLevel::SafeWidening,
            CoercionLevel::StringCoercion,
            CoercionLevel::BestEffort,
        ];
        let targets = ["i64", "f64", "bool", "String", "i32", "u64"];

        let value = serde_json::Value::String(s);
        let _ = coerce_value(&value, targets[target_idx], levels[level_idx], "proptest");
    }

    /// Coercion at Exact level on a matching type should preserve the value.
    #[test]
    fn exact_coercion_preserves_matching_type(i in any::<i64>()) {
        let value = serde_json::json!(i);
        let result = coerce_value(&value, "i64", CoercionLevel::Exact, "proptest");
        prop_assert_eq!(&result.value, &value, "Exact coercion on i64 should preserve value");
    }

    /// BestEffort String→i64 on valid integer strings should produce the integer.
    #[test]
    fn besteffort_string_to_i64(i in -1000000i64..1000000) {
        let value = serde_json::Value::String(i.to_string());
        let result = coerce_value(&value, "i64", CoercionLevel::BestEffort, "proptest");
        let expected = serde_json::json!(i);
        prop_assert_eq!(result.value, expected);
    }
}

// ── Type Detection Invariants ───────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// guess_type never panics and always returns at least one guess.
    #[test]
    fn guess_type_never_empty(s in "\\PC{1,100}") {
        let guesses = guess_type(&s);
        prop_assert!(!guesses.is_empty(), "guess_type should always return at least one guess");
    }

    /// guess_type results are sorted by confidence (highest first).
    #[test]
    fn guess_type_sorted_by_confidence(s in "\\PC{1,50}") {
        let guesses = guess_type(&s);
        for window in guesses.windows(2) {
            prop_assert!(
                window[0].confidence >= window[1].confidence,
                "Guesses not sorted: {} >= {} failed",
                window[0].confidence, window[1].confidence
            );
        }
    }
}

// ── Identifier Invariants ───────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// detect() never panics on any input.
    #[test]
    fn identifier_detect_never_panics(s in "\\PC{0,50}") {
        let _ = detect_id(&s);
    }

    /// All identifier validators never panic.
    #[test]
    fn identifier_validate_never_panics(s in "\\PC{0,50}") {
        let types = [
            IdentifierType::Uuid, IdentifierType::Iban,
            IdentifierType::CreditCard, IdentifierType::Isbn13,
            IdentifierType::Isbn10, IdentifierType::Email,
            IdentifierType::EuVat, IdentifierType::UsNpi,
            IdentifierType::UkNhs, IdentifierType::UsSsn,
            IdentifierType::UsEin, IdentifierType::Phone,
        ];
        for t in types {
            let _ = validate(&s, t);
        }
    }
}

// ── Currency & Unit Invariants ──────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn currency_detect_never_panics(s in "\\PC{0,50}") {
        let _ = detect_currency_format(&s);
    }

    #[test]
    fn unit_parse_never_panics(s in "\\PC{0,50}") {
        let _ = parse_unit_value(&s);
    }
}
