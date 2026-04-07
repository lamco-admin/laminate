//! Iteration 249: Full BestEffort chain: "null" string → null sentinel → Null → default (0)

use laminate::FlexValue;
use laminate::Lenient;

#[test]
fn null_string_full_chain_to_default() {
    let val = FlexValue::from_json(r#"{"x": "null"}"#)
        .unwrap()
        .with_mode::<Lenient>();

    let result = val.extract::<i64>("x");
    match &result {
        Ok(v) => println!("OK: {v}"),
        Err(e) => println!("ERR: {e}"),
    }

    // BestEffort should: "null" → recognized as null sentinel → treat as null → default 0
    assert!(result.is_ok(), "BestEffort should handle 'null' string → 0");
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn various_null_sentinels_to_default() {
    // Note: "" (empty string) is NOT treated as null sentinel by the coercion engine.
    // guess_type("") → NullSentinel, but coerce doesn't use guess_type.
    // This is by design: empty string at BestEffort errors (no data to coerce).
    let sentinels = [
        "null", "NULL", "None", "none", "nil", "NIL", "NA", "N/A", "n/a",
    ];

    for sentinel in sentinels {
        let json = format!(r#"{{"x": "{}"}}"#, sentinel);
        let val = FlexValue::from_json(&json).unwrap().with_mode::<Lenient>();

        let result = val.extract::<i64>("x");
        assert!(
            result.is_ok(),
            "BestEffort should handle '{sentinel}' → 0, got: {result:?}"
        );
        assert_eq!(
            result.unwrap(),
            0,
            "Sentinel '{sentinel}' should yield default 0"
        );
    }
}
