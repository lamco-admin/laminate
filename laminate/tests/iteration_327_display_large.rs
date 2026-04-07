//! Iteration 327: Display for FlexValue wrapping large JSON.
//! Does it truncate, or render the full thing?

use laminate::FlexValue;

#[test]
fn display_large_json_no_panic() {
    // Build a 10K-element array
    let items: Vec<serde_json::Value> = (0..10_000)
        .map(|i| serde_json::json!({"id": i, "name": format!("item_{}", i)}))
        .collect();
    let json = serde_json::Value::Array(items);
    let fv = FlexValue::from(json);

    let display = format!("{}", fv);
    println!("Display length: {} chars", display.len());

    // Should not panic, should produce valid output
    assert!(!display.is_empty(), "display should produce output");
    // Should contain first and last items
    assert!(display.contains("\"id\": 0"), "should contain first item");
    assert!(display.contains("\"id\": 9999"), "should contain last item");
}

#[test]
fn debug_large_json_no_panic() {
    let items: Vec<serde_json::Value> = (0..1_000).map(|i| serde_json::json!(i)).collect();
    let json = serde_json::Value::Array(items);
    let fv = FlexValue::from(json);

    let debug = format!("{:?}", fv);
    assert!(!debug.is_empty());
}
