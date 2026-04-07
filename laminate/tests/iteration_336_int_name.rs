//! Iteration 336: shape_lenient on {"name": 42, "port": 8080} — coerce int→string?

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    #[laminate(coerce)]
    name: String,
    #[laminate(default)]
    port: u16,
}

#[test]
fn shape_lenient_int_to_string_coerce() {
    let json = serde_json::json!({"name": 42, "port": 8080});
    let result = Config::shape_lenient(&json);
    println!("int→string coerce: {:?}", result);

    // With #[laminate(coerce)], int→String should work via Any→String
    assert!(result.is_ok(), "coerce should handle int→string");
    let lr = result.unwrap();
    assert_eq!(lr.value.name, "42", "int 42 should coerce to string \"42\"");
}

#[derive(Debug, Laminate)]
struct StrictConfig {
    name: String, // no coerce attribute
    #[laminate(default)]
    port: u16,
}

#[test]
fn shape_lenient_int_to_string_no_coerce() {
    let json = serde_json::json!({"name": 42, "port": 8080});
    let result = StrictConfig::shape_lenient(&json);
    println!("int→string NO coerce: {:?}", result);

    // Without #[laminate(coerce)], int→String should fail even in lenient
    assert!(
        result.is_err(),
        "without coerce attr, int→string should fail"
    );
}
