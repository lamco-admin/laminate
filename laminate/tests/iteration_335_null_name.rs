//! Iteration 335: shape_lenient on {"name": null} — null required field.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(default)]
    port: u16,
}

#[test]
fn shape_lenient_null_required_field() {
    let json = serde_json::json!({"name": null, "port": 8080});
    let result = Config::shape_lenient(&json);
    println!("shape_lenient null name: {:?}", result);

    // name is String (not Option<String>), and value is null
    // Should this fail or coerce null→""?
    match result {
        Ok(lr) => println!("OK: name={:?}, port={}", lr.value.name, lr.value.port),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[test]
fn shape_lenient_null_optional_field() {
    #[derive(Debug, Laminate)]
    struct OptConfig {
        name: Option<String>,
        #[laminate(default)]
        port: u16,
    }

    let json = serde_json::json!({"name": null, "port": 8080});
    let result = OptConfig::shape_lenient(&json);
    println!("shape_lenient null optional name: {:?}", result);

    assert!(result.is_ok(), "null on Option<String> should be Ok(None)");
    let lr = result.unwrap();
    assert_eq!(lr.value.name, None, "null should map to None");
}
