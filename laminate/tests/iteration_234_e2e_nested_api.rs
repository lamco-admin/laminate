//! Iteration 234: Nested API response → at().each_iter() → extract per element → collect
//!
//! Simulates a paginated API response where results are nested under "data".
//! Uses FlexValue path navigation + iteration to extract typed elements.

use laminate::FlexValue;

#[test]
fn nested_api_response_iterate_and_extract() {
    let api_response = serde_json::json!({
        "status": "ok",
        "data": [
            {"id": 1, "name": "Alice", "score": "95"},
            {"id": 2, "name": "Bob", "score": "88"},
            {"id": 3, "name": "Charlie", "score": "91"}
        ],
        "total": 3
    });

    let val = FlexValue::from(api_response);

    // Use each_iter("data") to iterate over the nested array
    let items: Vec<FlexValue> = val.each_iter("data").collect();

    assert_eq!(items.len(), 3);

    // Extract typed values from each item
    let names: Vec<String> = items
        .iter()
        .map(|item| item.extract::<String>("name").unwrap())
        .collect();
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);

    // Scores are strings — coercion should handle them
    let scores: Vec<i64> = items
        .iter()
        .map(|item| item.extract::<i64>("score").unwrap())
        .collect();
    assert_eq!(scores, vec![95, 88, 91]);
}

#[test]
fn nested_api_top_level_field_extraction() {
    let api_response = serde_json::json!({
        "status": "ok",
        "data": [{"id": 1}],
        "total": 1
    });

    let val = FlexValue::from(api_response);
    let status: String = val.extract("status").unwrap();
    let total: i64 = val.extract("total").unwrap();

    assert_eq!(status, "ok");
    assert_eq!(total, 1);
}

#[test]
fn nested_api_empty_data_array() {
    let api_response = serde_json::json!({
        "status": "ok",
        "data": [],
        "total": 0
    });

    let val = FlexValue::from(api_response);
    let items: Vec<FlexValue> = val.each_iter("data").collect();

    assert_eq!(
        items.len(),
        0,
        "empty data array should produce empty iterator"
    );
}
