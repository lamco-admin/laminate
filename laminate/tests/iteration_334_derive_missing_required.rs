//! Iteration 334: shape_lenient on object missing required "name" field.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(default)]
    port: u16,
}

#[test]
fn shape_lenient_missing_required_name() {
    // Missing "name" which is a required String (no #[laminate(default)])
    let json = serde_json::json!({"port": 8080});
    let result = Config::shape_lenient(&json);
    println!("shape_lenient missing name: {:?}", result);

    // Should this fail (name is required) or use String::default() (empty string)?
    match result {
        Ok(lr) => {
            println!("OK: name={:?}, port={}", lr.value.name, lr.value.port);
            println!("diagnostics: {:?}", lr.diagnostics);
            // Lenient mode — may default to empty string
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // Missing required field should error
        }
    }
}

#[test]
fn shape_strict_missing_required_name() {
    let json = serde_json::json!({"port": 8080});
    let result = Config::shape_strict(&json);
    println!("shape_strict missing name: {:?}", result);
    // Strict should definitely fail for missing required field
    assert!(
        result.is_err(),
        "strict should reject missing required field"
    );
}
