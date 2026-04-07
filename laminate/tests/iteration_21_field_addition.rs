//! Iteration 21: Field Addition — GitHub repo.json with overflow struct
//! Mutation: Parse full GitHub API response (80+ fields) through a small struct.
//! Add an unknown field not in the original API response. Observe overflow behavior
//! with nested objects and large field counts.

use laminate_derive::Laminate;
use std::collections::HashMap;

/// Small struct extracting only 5 fields from the 80+ field GitHub response
#[derive(Debug, Laminate)]
struct GithubRepoSlim {
    id: u64,
    name: String,
    full_name: String,
    #[laminate(coerce)]
    private: bool,
    language: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn iter21_github_overflow_with_added_field() {
    // Load the real GitHub API fixture and add an unknown field
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("../testdata/github-api/repo.json")).unwrap();

    // Mutation: add a field that doesn't exist in the GitHub API
    json["custom_webhook_secret"] = serde_json::json!("s3cret_t0ken");

    let json_str = serde_json::to_string(&json).unwrap();
    let (repo, diags) = GithubRepoSlim::from_json(&json_str).unwrap();

    // Known fields should be extracted correctly
    assert_eq!(repo.id, 14367737);
    assert_eq!(repo.name, "serde");
    assert_eq!(repo.full_name, "serde-rs/serde");
    assert!(!repo.private);
    assert_eq!(repo.language, "Rust");

    // The added field should appear in overflow
    assert_eq!(
        repo.extra.get("custom_webhook_secret"),
        Some(&serde_json::json!("s3cret_t0ken")),
        "Added unknown field should be captured in overflow"
    );

    // Nested objects should be preserved as Values in overflow
    let owner = repo
        .extra
        .get("owner")
        .expect("owner should be in overflow");
    assert!(
        owner.is_object(),
        "owner should remain an object in overflow"
    );
    assert_eq!(owner["login"], "serde-rs");

    let license = repo
        .extra
        .get("license")
        .expect("license should be in overflow");
    assert!(license.is_object());
    assert_eq!(license["key"], "apache-2.0");

    // Count overflow fields — should be total JSON fields minus the 5 known ones
    // The JSON has ~80+ top-level fields; we extract 5 (id, name, full_name, private, language)
    let overflow_count = repo.extra.len();
    assert!(
        overflow_count > 70,
        "Expected 70+ overflow fields, got {overflow_count}"
    );

    // Diagnostics: each overflow field should produce a Preserved diagnostic
    let preserved_diags: Vec<_> = diags
        .iter()
        .filter(|d| matches!(d.kind, laminate::DiagnosticKind::Preserved { .. }))
        .collect();
    assert_eq!(
        preserved_diags.len(),
        overflow_count,
        "Each overflow field should produce exactly one Preserved diagnostic"
    );
}
