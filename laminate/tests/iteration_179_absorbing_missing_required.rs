//! Iteration 179: shape_absorbing on missing required field — require_all_fields enforced?
//!
//! Current design: from_flex_value() always errors on missing non-default fields.
//! shape_lenient delegates to from_flex_value(), so it also errors on missing fields
//! unless #[laminate(default)] is annotated. This is a known design gap:
//! Mode::require_all_fields() is not wired into field extraction yet.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct User {
    name: String,
    age: i64,
}

#[derive(Debug, Laminate)]
struct UserWithDefaults {
    name: String,
    #[laminate(default)]
    age: i64,
}

#[test]
fn shape_absorbing_rejects_missing_required_field() {
    let json = serde_json::json!({"name": "Alice"});
    let result = User::shape_absorbing(&json);
    assert!(
        result.is_err(),
        "Absorbing should reject missing required field 'age'"
    );
}

#[test]
fn shape_absorbing_accepts_complete_data() {
    let json = serde_json::json!({"name": "Alice", "age": 30});
    let result = User::shape_absorbing(&json);
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.value.name, "Alice");
    assert_eq!(user.value.age, 30);
}

#[test]
fn shape_lenient_with_default_attr_defaults_missing_field() {
    // With #[laminate(default)], lenient mode correctly defaults missing fields
    let json = serde_json::json!({"name": "Alice"});
    let result = UserWithDefaults::shape_lenient(&json);
    assert!(
        result.is_ok(),
        "Lenient + #[laminate(default)] should default missing fields"
    );
    let user = result.unwrap();
    assert_eq!(user.value.name, "Alice");
    assert_eq!(user.value.age, 0);
}

#[test]
fn shape_lenient_without_default_attr_still_errors_on_missing() {
    // GAP: shape_lenient currently errors on missing non-default fields
    // because from_flex_value doesn't consult Mode::require_all_fields()
    let json = serde_json::json!({"name": "Alice"});
    let result = User::shape_lenient(&json);
    assert!(
        result.is_err(),
        "shape_lenient currently errors on missing non-default fields (known gap)"
    );
}
