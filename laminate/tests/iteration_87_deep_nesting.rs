/// Iteration 87: Deeply nested JSON (50 levels) — path navigation & merge
///
/// Verified at depth 50: serde_json parsing, FlexValue::at() navigation,
/// and deep_merge all work correctly. serde_json's 128-level recursion
/// limit kicks in at 130 levels, and FlexValue surfaces the error cleanly.
use laminate::FlexValue;

fn nested_json(depth: usize, leaf_value: &str) -> String {
    let mut s = String::new();
    for _ in 0..depth {
        s.push_str(r#"{"a":"#);
    }
    s.push_str(leaf_value);
    for _ in 0..depth {
        s.push('}');
    }
    s
}

fn dotted_path(depth: usize) -> String {
    (0..depth).map(|_| "a").collect::<Vec<_>>().join(".")
}

#[test]
fn parse_and_navigate_50_levels() {
    let json = nested_json(50, "42");
    let fv = FlexValue::from_json(&json).unwrap();
    let path = dotted_path(50);
    let result: i64 = fv.extract(&path).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn merge_50_level_objects() {
    let a = FlexValue::from_json(&nested_json(50, r#""a_val""#)).unwrap();
    let b = FlexValue::from_json(&nested_json(50, r#""b_val""#)).unwrap();
    let merged = a.merge(&b);
    let result: String = merged.extract(&dotted_path(50)).unwrap();
    assert_eq!(result, "b_val");
}

#[test]
fn serde_recursion_limit_error_surfaced() {
    // 130 levels exceeds serde_json's 128-level limit
    let json = nested_json(130, "1");
    let result = FlexValue::from_json(&json);
    assert!(
        result.is_err(),
        "130-level nesting should fail at serde limit"
    );
}
