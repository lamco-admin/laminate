//! Iteration 214: #[laminate(coerce)] on Vec<i64> with mixed-type array.
//!
//! Input: ["1", 2, "three"] — "1" coerces, 2 is already i64, "three" fails.
//! What happens? Does the whole Vec fail? Or do we get partial results?

use laminate::Laminate;

#[derive(Debug, Laminate)]
struct Data {
    #[laminate(coerce)]
    values: Vec<i64>,
}

#[test]
fn coerce_vec_with_mixed_types() {
    let json = serde_json::json!({"values": ["1", 2, "three"]});
    let result = Data::from_flex_value(&json);

    match &result {
        Ok((data, diags)) => {
            println!("OK: values = {:?}", data.values);
            println!("Diagnostics: {}", diags.len());
            for d in diags {
                println!("  {d:?}");
            }
        }
        Err(e) => println!("ERR: {e}"),
    }

    // "three" can't coerce to i64. The question is whether it errors
    // or silently produces a bad value. Either way, observe first.
    // Most likely: serde_json::from_value fails because "three" → i64 fails.
    assert!(
        result.is_err(),
        "Non-numeric 'three' should cause Vec extraction to fail"
    );
}

#[test]
fn coerce_vec_with_all_coercible() {
    let json = serde_json::json!({"values": ["1", "2", "3"]});
    let result = Data::from_flex_value(&json);
    assert!(result.is_ok());
    let (data, _) = result.unwrap();
    assert_eq!(data.values, vec![1, 2, 3]);
}

#[test]
fn coerce_vec_with_native_types() {
    let json = serde_json::json!({"values": [1, 2, 3]});
    let result = Data::from_flex_value(&json);
    assert!(result.is_ok());
    let (data, _) = result.unwrap();
    assert_eq!(data.values, vec![1, 2, 3]);
}
