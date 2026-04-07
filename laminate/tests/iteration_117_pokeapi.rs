//! Iteration 117 — Real Data: PokeAPI Pikachu
//!
//! Probe: Deeply nested arrays of objects containing more arrays.
//! Structure: moves[i].version_group_details[j].move_learn_method.name (4 levels)
//! Key adversarial case: moves[0].version_group_details[0].order is null (1006/1075 null)
//!
//! Questions:
//! 1. Does 4-level path navigation through arrays work?
//! 2. Does `extract::<Option<i64>>` on deep-null-field work?
//! 3. Does `maybe()` short-circuit at null correctly inside deeply nested path?
//! 4. Does schema inference on `version_group_details` classify `order` as nullable int?

use laminate::schema::{InferredSchema, JsonType};
use laminate::FlexValue;

fn pikachu() -> FlexValue {
    let json = include_str!("../testdata/gaming/pikachu.json");
    FlexValue::from_json(json).unwrap()
}

// --- 4-level deep path navigation ---

#[test]
fn four_level_nested_path() {
    let fv = pikachu();

    // moves[0].version_group_details[0].move_learn_method.name
    let method: String = fv
        .extract("moves[0].version_group_details[0].move_learn_method.name")
        .unwrap();
    println!("move_learn_method.name = {method}");
    // pikachu's first move (thunderbolt or similar) learned by machine
    assert!(!method.is_empty(), "should have a method name");
}

#[test]
fn nested_array_index_access() {
    let fv = pikachu();

    // Confirm array index access works at each depth
    let move_name: String = fv.extract("moves[0].move.name").unwrap();
    let vg_name: String = fv
        .extract("moves[0].version_group_details[0].version_group.name")
        .unwrap();
    println!("move={move_name}, version_group={vg_name}");
    assert!(!move_name.is_empty());
    assert!(!vg_name.is_empty());
}

// --- null `order` field at deepest nesting level ---

#[test]
fn null_order_extract_as_option() {
    let fv = pikachu();

    // moves[0].version_group_details[0].order is null (1006/1075 are null)
    let order: Option<i64> = fv
        .extract("moves[0].version_group_details[0].order")
        .unwrap();
    assert!(
        order.is_none(),
        "null order should extract as None for Option<i64>"
    );
}

#[test]
fn null_order_maybe_returns_none() {
    let fv = pikachu();

    // maybe() should return None when the value is null (not PathNotFound)
    let order: Option<i64> = fv.maybe("moves[0].version_group_details[0].order").unwrap();
    assert!(
        order.is_none(),
        "maybe() on null order should be None, not Some(0)"
    );
}

#[test]
fn non_null_order_extract_works() {
    let fv = pikachu();

    // moves[10].version_group_details[13].order = 1 (one of 69 non-null)
    let order: Option<i64> = fv
        .extract("moves[10].version_group_details[13].order")
        .unwrap();
    assert_eq!(order, Some(1), "non-null order should be Some(1)");

    let order_direct: i64 = fv
        .extract("moves[10].version_group_details[13].order")
        .unwrap();
    assert_eq!(order_direct, 1);
}

// --- Level-2 array: version_group_details across all moves ---

#[test]
fn schema_inference_on_version_group_details() {
    // Collect all version_group_details entries across all moves
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../testdata/gaming/pikachu.json")).unwrap();

    let moves = raw["moves"].as_array().unwrap();
    let mut all_vgd: Vec<serde_json::Value> = Vec::new();
    for mv in moves {
        if let Some(details) = mv["version_group_details"].as_array() {
            all_vgd.extend(details.iter().cloned());
        }
    }
    println!("total version_group_details entries: {}", all_vgd.len());

    let schema = InferredSchema::from_values(&all_vgd);
    println!("vgd fields: {:?}", schema.fields.keys().collect::<Vec<_>>());

    // All entries have: level_learned_at, move_learn_method, order, version_group
    assert!(schema.fields.contains_key("level_learned_at"));
    assert!(schema.fields.contains_key("order"));
    assert!(schema.fields.contains_key("move_learn_method"));
    assert!(schema.fields.contains_key("version_group"));

    // level_learned_at should be Integer (always a number)
    let lla = &schema.fields["level_learned_at"];
    assert_eq!(
        lla.dominant_type,
        Some(JsonType::Integer),
        "level_learned_at should be Integer"
    );

    // order is 93% null — should show up as Null or mixed with Integer
    // Either way, null_count should be high
    let order = &schema.fields["order"];
    println!("order field: {:?}", order);
    // The null count should dominate
    assert!(
        order.null_count > 0,
        "order field should have null_count > 0 (1006 nulls)"
    );
}

// --- Top-level stats array ---

#[test]
fn stats_array_navigation() {
    let fv = pikachu();

    // stats is [{base_stat, effort, stat: {name, url}}, ...]
    let hp: i64 = fv.extract("stats[0].base_stat").unwrap();
    let stat_name: String = fv.extract("stats[0].stat.name").unwrap();
    println!("stat={stat_name}, base_stat={hp}");
    assert_eq!(stat_name, "hp");
    assert!(hp > 0 && hp < 256, "hp should be a valid stat value");
}

#[test]
fn all_six_stats_accessible() {
    let fv = pikachu();

    let stat_names = [
        "hp",
        "attack",
        "defense",
        "special-attack",
        "special-defense",
        "speed",
    ];
    for (i, &expected_name) in stat_names.iter().enumerate() {
        let name: String = fv.extract(&format!("stats[{i}].stat.name")).unwrap();
        assert_eq!(name, expected_name, "stats[{i}] should be {expected_name}");
    }
}
