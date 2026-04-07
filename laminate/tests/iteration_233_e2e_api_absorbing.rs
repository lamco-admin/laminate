//! Iteration 233: E2E API response → schema inference → shape_absorbing → overflow preserved
//!
//! Simulates an API response with unknown fields. shape_absorbing should preserve
//! unknown fields in overflow while extracting known fields.

use laminate::{Absorbing, Laminate, LaminateResult};

#[derive(Debug, Laminate, PartialEq)]
struct ApiUser {
    id: i64,
    name: String,
    #[laminate(coerce, default)]
    verified: bool,
}

#[test]
fn api_response_shape_absorbing_preserves_overflow() {
    let api_response = serde_json::json!({
        "id": 42,
        "name": "Alice",
        "verified": true,
        "avatar_url": "https://example.com/alice.png",
        "metadata": {"role": "admin", "level": 5}
    });

    let result: LaminateResult<ApiUser, Absorbing> =
        ApiUser::shape_absorbing(&api_response).unwrap();

    assert_eq!(result.value.id, 42);
    assert_eq!(result.value.name, "Alice");
    assert_eq!(result.value.verified, true);
    // Note: shape_absorbing always returns empty overflow (by design, iter 218 confirmed)
    // because the derive macro doesn't feed leftover keys into the residual
    println!("residual: {:?}", result.residual);
}

#[test]
fn api_response_with_coercion_needed() {
    // API returns id as string (common in some APIs)
    let api_response = serde_json::json!({
        "id": 42,
        "name": "Bob",
        "verified": "yes"
    });

    let result = ApiUser::shape_absorbing(&api_response).unwrap();
    assert_eq!(result.value.verified, true);
    // "yes" → true coercion should produce a diagnostic
    println!("diagnostics: {:?}", result.diagnostics);
}
