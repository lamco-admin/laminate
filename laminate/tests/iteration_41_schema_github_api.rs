#![recursion_limit = "512"]
use laminate::schema::{InferredSchema, JsonType};
use serde_json::json;

/// Iteration 41 — Schema inference on wide, nullable, GitHub-API-like data.
///
/// Probe: 80+ field objects with nullable fields, nested objects, arrays.
/// Key question: What does schema inference + audit produce for:
/// 1. Fields that are always null across all records
/// 2. Fields that are null in some records, populated in others
/// 3. Nested objects (owner, license)
/// 4. Empty vs populated array fields

fn github_repo_row(
    id: u64,
    name: &str,
    description: Option<&str>,
    homepage: Option<&str>,
    language: Option<&str>,
    license: Option<serde_json::Value>,
    mirror_url: Option<&str>,
    has_wiki: bool,
    stargazers: u64,
    topics: Vec<&str>,
) -> serde_json::Value {
    json!({
        "id": id,
        "node_id": format!("MDEwOlJlcG9zaXRvcnk{}", id),
        "name": name,
        "full_name": format!("org/{}", name),
        "private": false,
        "owner": {
            "login": "org",
            "id": 1000,
            "node_id": "MDQ6VXNlcjEwMDA=",
            "avatar_url": "https://avatars.example.com/u/1000",
            "type": "Organization"
        },
        "html_url": format!("https://github.com/org/{}", name),
        "description": description,
        "fork": false,
        "url": format!("https://api.github.com/repos/org/{}", name),
        "forks_url": format!("https://api.github.com/repos/org/{}/forks", name),
        "keys_url": format!("https://api.github.com/repos/org/{}/keys{{/key_id}}", name),
        "collaborators_url": format!("https://api.github.com/repos/org/{}/collaborators{{/collaborator}}", name),
        "teams_url": format!("https://api.github.com/repos/org/{}/teams", name),
        "hooks_url": format!("https://api.github.com/repos/org/{}/hooks", name),
        "issue_events_url": format!("https://api.github.com/repos/org/{}/issues/events{{/number}}", name),
        "events_url": format!("https://api.github.com/repos/org/{}/events", name),
        "assignees_url": format!("https://api.github.com/repos/org/{}/assignees{{/user}}", name),
        "branches_url": format!("https://api.github.com/repos/org/{}/branches{{/branch}}", name),
        "tags_url": format!("https://api.github.com/repos/org/{}/tags", name),
        "blobs_url": format!("https://api.github.com/repos/org/{}/git/blobs{{/sha}}", name),
        "git_tags_url": format!("https://api.github.com/repos/org/{}/git/tags{{/sha}}", name),
        "git_refs_url": format!("https://api.github.com/repos/org/{}/git/refs{{/sha}}", name),
        "trees_url": format!("https://api.github.com/repos/org/{}/git/trees{{/sha}}", name),
        "statuses_url": format!("https://api.github.com/repos/org/{}/statuses/{{sha}}", name),
        "languages_url": format!("https://api.github.com/repos/org/{}/languages", name),
        "stargazers_url": format!("https://api.github.com/repos/org/{}/stargazers", name),
        "contributors_url": format!("https://api.github.com/repos/org/{}/contributors", name),
        "subscribers_url": format!("https://api.github.com/repos/org/{}/subscribers", name),
        "subscription_url": format!("https://api.github.com/repos/org/{}/subscription", name),
        "commits_url": format!("https://api.github.com/repos/org/{}/commits{{/sha}}", name),
        "git_commits_url": format!("https://api.github.com/repos/org/{}/git/commits{{/sha}}", name),
        "comments_url": format!("https://api.github.com/repos/org/{}/comments{{/number}}", name),
        "issue_comment_url": format!("https://api.github.com/repos/org/{}/issues/comments{{/number}}", name),
        "contents_url": format!("https://api.github.com/repos/org/{}/contents/{{+path}}", name),
        "compare_url": format!("https://api.github.com/repos/org/{}/compare/{{base}}...{{head}}", name),
        "merges_url": format!("https://api.github.com/repos/org/{}/merges", name),
        "archive_url": format!("https://api.github.com/repos/org/{}/{{archive_format}}{{/ref}}", name),
        "downloads_url": format!("https://api.github.com/repos/org/{}/downloads", name),
        "issues_url": format!("https://api.github.com/repos/org/{}/issues{{/number}}", name),
        "pulls_url": format!("https://api.github.com/repos/org/{}/pulls{{/number}}", name),
        "milestones_url": format!("https://api.github.com/repos/org/{}/milestones{{/number}}", name),
        "notifications_url": format!("https://api.github.com/repos/org/{}/notifications{{?since,all,participating}}", name),
        "labels_url": format!("https://api.github.com/repos/org/{}/labels{{/name}}", name),
        "releases_url": format!("https://api.github.com/repos/org/{}/releases{{/id}}", name),
        "deployments_url": format!("https://api.github.com/repos/org/{}/deployments", name),
        "created_at": "2020-01-15T10:30:00Z",
        "updated_at": "2026-03-30T14:22:00Z",
        "pushed_at": "2026-03-30T12:00:00Z",
        "git_url": format!("git://github.com/org/{}.git", name),
        "ssh_url": format!("git@github.com:org/{}.git", name),
        "clone_url": format!("https://github.com/org/{}.git", name),
        "svn_url": format!("https://github.com/org/{}", name),
        "homepage": homepage,
        "size": 1024 + id * 100,
        "stargazers_count": stargazers,
        "watchers_count": stargazers,
        "language": language,
        "has_issues": true,
        "has_projects": true,
        "has_downloads": true,
        "has_wiki": has_wiki,
        "has_pages": false,
        "has_discussions": false,
        "forks_count": stargazers / 10,
        "mirror_url": mirror_url,
        "archived": false,
        "disabled": false,
        "open_issues_count": id % 50,
        "license": license,
        "allow_forking": true,
        "is_template": false,
        "web_commit_signoff_required": false,
        "topics": topics,
        "visibility": "public",
        "forks": stargazers / 10,
        "open_issues": id % 50,
        "watchers": stargazers,
        "default_branch": "main",
        "permissions": {
            "admin": false,
            "maintain": false,
            "push": true,
            "triage": true,
            "pull": true
        },
        "temp_clone_token": serde_json::Value::Null,
        "organization": {
            "login": "org",
            "id": 1000,
            "type": "Organization"
        },
        "network_count": stargazers * 2,
        "subscribers_count": stargazers / 5
    })
}

#[test]
fn probe_schema_inference_wide_github_data() {
    // 3 repos with varying nullable fields
    let rows = vec![
        github_repo_row(
            14367737,
            "laminate",
            Some("A coercion library"),
            Some("https://laminate.dev"),
            Some("Rust"),
            Some(json!({"key": "MIT", "name": "MIT License", "spdx_id": "MIT"})),
            None,
            true,
            1200,
            vec!["rust", "coercion", "serde"],
        ),
        github_repo_row(
            28901234,
            "toolbox",
            None,
            None,
            Some("Python"),
            None,
            None,
            false,
            340,
            vec![],
        ),
        github_repo_row(
            55123456,
            "docs",
            Some("Documentation site"),
            None,
            None,
            Some(
                json!({"key": "Apache-2.0", "name": "Apache License 2.0", "spdx_id": "Apache-2.0"}),
            ),
            None,
            true,
            50,
            vec!["docs"],
        ),
    ];

    let schema = InferredSchema::from_values(&rows);

    // ── Observation 1: Field count ──
    // GitHub API repos have 80+ fields. How many does the schema find?
    let field_count = schema.fields.len();
    println!("Fields discovered: {}", field_count);
    assert!(
        field_count >= 75,
        "Expected 75+ fields, got {}",
        field_count
    );

    // ── Observation 2: Always-present string fields ──
    let name_field = &schema.fields["name"];
    assert_eq!(name_field.dominant_type, Some(JsonType::String));
    assert!(name_field.appears_required());
    assert_eq!(name_field.null_count, 0);

    // ── Observation 3: Sometimes-null string fields ──
    // "description" is null in row 2, string in rows 1 and 3
    let desc = &schema.fields["description"];
    println!(
        "description: dominant_type={:?}, null_count={}, present_count={}, absent_count={}",
        desc.dominant_type, desc.null_count, desc.present_count, desc.absent_count
    );
    assert_eq!(desc.dominant_type, Some(JsonType::String));
    assert_eq!(desc.null_count, 1);
    assert!(!desc.appears_required()); // has nulls

    // ── Observation 4: Always-null fields ──
    // "mirror_url" is null in all 3 rows, "temp_clone_token" is null in all 3
    let mirror = &schema.fields["mirror_url"];
    println!(
        "mirror_url: dominant_type={:?}, null_count={}, type_counts={:?}",
        mirror.dominant_type, mirror.null_count, mirror.type_counts
    );
    // KEY OBSERVATION: What is the dominant_type for an always-null field?
    // Expected: None (no non-null values observed)
    assert_eq!(mirror.null_count, 3);
    assert_eq!(
        mirror.dominant_type, None,
        "Always-null field should have no dominant type"
    );

    let token = &schema.fields["temp_clone_token"];
    assert_eq!(token.null_count, 3);
    assert_eq!(token.dominant_type, None);

    // ── Observation 5: Sometimes-null object fields ──
    // "license" is null in row 2, object in rows 1 and 3
    let license = &schema.fields["license"];
    println!(
        "license: dominant_type={:?}, null_count={}, type_counts={:?}",
        license.dominant_type, license.null_count, license.type_counts
    );
    assert_eq!(license.dominant_type, Some(JsonType::Object));
    assert_eq!(license.null_count, 1);

    // ── Observation 6: Nested object fields (always present) ──
    // "owner" and "permissions" are always objects
    let owner = &schema.fields["owner"];
    assert_eq!(owner.dominant_type, Some(JsonType::Object));
    assert!(owner.appears_required());

    // ── Observation 7: Array fields (varying content) ──
    // "topics" is [] in row 2, populated in rows 1 and 3
    let topics = &schema.fields["topics"];
    println!(
        "topics: dominant_type={:?}, type_counts={:?}",
        topics.dominant_type, topics.type_counts
    );
    assert_eq!(topics.dominant_type, Some(JsonType::Array));
    // Empty array [] still counts as Array type
    assert_eq!(topics.null_count, 0);
    assert!(topics.appears_required());

    // ── Observation 8: Integer count fields ──
    let stars = &schema.fields["stargazers_count"];
    assert_eq!(stars.dominant_type, Some(JsonType::Integer));

    // ── Observation 9: Boolean fields ──
    let wiki = &schema.fields["has_wiki"];
    assert_eq!(wiki.dominant_type, Some(JsonType::Bool));

    // ── Observation 10: Schema summary ──
    let summary = schema.summary();
    println!("\n{}", summary);
}

#[test]
fn probe_audit_always_null_field_gets_value() {
    // Infer schema where mirror_url is always null
    let training = vec![
        github_repo_row(
            1,
            "a",
            Some("desc"),
            None,
            Some("Rust"),
            None,
            None,
            true,
            10,
            vec![],
        ),
        github_repo_row(
            2,
            "b",
            Some("desc"),
            None,
            Some("Go"),
            None,
            None,
            true,
            20,
            vec![],
        ),
    ];
    let schema = InferredSchema::from_values(&training);

    // Confirm mirror_url has no type
    assert_eq!(schema.fields["mirror_url"].dominant_type, None);

    // Now audit with data where mirror_url has a string value
    let audit_data = vec![github_repo_row(
        3,
        "c",
        Some("desc"),
        None,
        Some("Rust"),
        None,
        Some("https://mirror.example.com"),
        true,
        30,
        vec![],
    )];

    let report = schema.audit(&audit_data);

    // KEY QUESTION: Does audit flag the always-null field now having a value?
    let mirror_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "mirror_url")
        .collect();
    println!("mirror_url violations: {:?}", mirror_violations);

    // Also check if it's counted as "clean" or something else
    let mirror_stats = &report.field_stats["mirror_url"];
    println!(
        "mirror_url audit stats: clean={}, coercible={}, violations={}, missing={}",
        mirror_stats.clean, mirror_stats.coercible, mirror_stats.violations, mirror_stats.missing
    );

    // OBSERVE: The field goes from always-null in training to having a string value.
    // effective_type is None (no dominant type). The type check is skipped.
    // is_nullable is true (null_count > 0). The null check passes.
    // So the value is marked as "clean" — no violation!
    // This means always-null fields are invisible to audit.
}

#[test]
fn probe_audit_new_data_with_type_change() {
    // Infer from data where "language" is always String
    let training = vec![
        github_repo_row(
            1,
            "a",
            Some("desc"),
            None,
            Some("Rust"),
            None,
            None,
            true,
            10,
            vec![],
        ),
        github_repo_row(
            2,
            "b",
            Some("desc"),
            None,
            Some("Go"),
            None,
            None,
            true,
            20,
            vec![],
        ),
        github_repo_row(
            3,
            "c",
            Some("desc"),
            None,
            Some("Python"),
            None,
            None,
            true,
            30,
            vec![],
        ),
    ];
    let schema = InferredSchema::from_values(&training);
    assert_eq!(
        schema.fields["language"].dominant_type,
        Some(JsonType::String)
    );

    // Audit with data where language is null in one record (not seen in training)
    let audit_data = vec![github_repo_row(
        4,
        "d",
        Some("desc"),
        None,
        None,
        None,
        None,
        true,
        5,
        vec![],
    )];

    let report = schema.audit(&audit_data);

    // language was never null in training, so is_nullable should be false
    let lang = &schema.fields["language"];
    println!(
        "language: null_count={}, nullable={}",
        lang.null_count,
        lang.null_count > 0
    );

    let lang_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "language")
        .collect();
    println!("language violations: {:?}", lang_violations);
    let lang_stats = &report.field_stats["language"];
    println!(
        "language stats: clean={}, coercible={}, violations={}, missing={}",
        lang_stats.clean, lang_stats.coercible, lang_stats.violations, lang_stats.missing
    );

    // OBSERVE: language was never null in training data.
    // is_nullable defaults to null_count > 0 = false.
    // So null value should trigger UnexpectedNull violation!
}
