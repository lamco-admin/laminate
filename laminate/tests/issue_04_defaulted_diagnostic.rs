//! Regression test for issue #4: a plain `#[laminate(default)]` field (no
//! `coerce`) must emit a `Defaulted` diagnostic when it fills a missing or null
//! field. Previously this path was silent — only the `coerce` path recorded it.

use laminate::DiagnosticKind;
use laminate_derive::Laminate;

#[derive(Debug, Laminate)]
struct Cfg {
    name: String,
    #[laminate(default)]
    retries: u32,
}

fn defaulted_count(diags: &[laminate::Diagnostic], field: &str) -> usize {
    diags
        .iter()
        .filter(|d| matches!(&d.kind, DiagnosticKind::Defaulted { field: f, .. } if f == field))
        .count()
}

#[test]
fn plain_default_missing_emits_defaulted() {
    let (cfg, diags) = Cfg::from_json(r#"{"name": "svc"}"#).unwrap();
    assert_eq!(cfg.name, "svc");
    assert_eq!(cfg.retries, 0);
    assert_eq!(
        defaulted_count(&diags, "retries"),
        1,
        "expected one Defaulted diagnostic for `retries`, got: {diags:?}"
    );
}

#[test]
fn plain_default_null_emits_defaulted() {
    let (_cfg, diags) = Cfg::from_json(r#"{"name": "svc", "retries": null}"#).unwrap();
    assert_eq!(defaulted_count(&diags, "retries"), 1, "got: {diags:?}");
}

#[test]
fn present_value_emits_no_defaulted() {
    let (cfg, diags) = Cfg::from_json(r#"{"name": "svc", "retries": 5}"#).unwrap();
    assert_eq!(cfg.retries, 5);
    assert_eq!(defaulted_count(&diags, "retries"), 0, "got: {diags:?}");
}
