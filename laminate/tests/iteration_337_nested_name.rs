//! Iteration 337: shape_lenient on {"name": {"first": "A", "last": "B"}} — object→string.

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Config {
    #[laminate(coerce)]
    name: String,
}

#[test]
fn shape_lenient_object_to_string() {
    let json = serde_json::json!({"name": {"first": "A", "last": "B"}});
    let result = Config::shape_lenient(&json);
    println!("object→string: {:?}", result);

    // Any→String coercion should JSON-serialize the object
    match result {
        Ok(lr) => {
            println!("name = {:?}", lr.value.name);
            // Should be a JSON string representation
            assert!(lr.value.name.contains("first"), "should serialize object");
            assert!(lr.value.name.contains("last"), "should contain 'last'");
        }
        Err(e) => {
            panic!("object→string with coerce should work: {:?}", e);
        }
    }
}

#[test]
fn shape_lenient_array_to_string() {
    let json = serde_json::json!({"name": [1, 2, 3]});
    let result = Config::shape_lenient(&json);
    println!("array→string: {:?}", result);

    match result {
        Ok(lr) => {
            println!("name = {:?}", lr.value.name);
            assert!(lr.value.name.contains("["), "should serialize array");
        }
        Err(e) => {
            panic!("array→string with coerce should work: {:?}", e);
        }
    }
}
