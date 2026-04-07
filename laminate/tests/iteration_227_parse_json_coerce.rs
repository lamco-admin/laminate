/// Iteration 227: #[laminate(parse_json_string, coerce)] — parse then coerce
///
/// Target #227: When parse_json_string and coerce are combined, the macro
/// should parse the JSON string first, THEN coerce the parsed result.
/// Test: stringified integer "42" → parse as JSON → Value::Number(42) → coerce to String
use laminate::Laminate;

/// Case 1: Stringified number → parse → coerce to String
#[derive(Debug, Laminate)]
struct CoerceAfterParse {
    name: String,
    #[laminate(parse_json_string, coerce)]
    code: String, // JSON value "42" → parses to Number(42) → coerce to String "42"
}

#[test]
fn parse_json_then_coerce_number_to_string() {
    // "code" is the string "42" — parse_json_string parses it to Number(42)
    // Then coerce tries to make it a String
    let json = r#"{"name": "test", "code": "42"}"#;
    let (w, diagnostics) = CoerceAfterParse::from_json(json).unwrap();

    println!("w = {:?}", w);
    println!("diagnostics = {:?}", diagnostics);

    // parse_json_string: "42" → Value::Number(42)
    // coerce to String: Number(42) → "42"
    // OR: parse_json_string sees "42" as just a JSON number literal, not string
    // Then coerce converts it
    println!("code = {:?}", w.code);
}

/// Case 2: Stringified object → parse → coerce i64 from inner field
#[derive(Debug, Laminate)]
struct ParsedConfig {
    label: String,
    #[laminate(parse_json_string, coerce)]
    port: i64, // If value is "{\"port\": 8080}" → parse → ??? port is extracted by key, not from object
}

#[test]
fn parse_json_coerce_stringified_number() {
    // Here "port" value is "8080" (a string of a number)
    // parse_json_string: "8080" → Value::Number(8080)
    // coerce to i64: already i64, pass-through
    let json = r#"{"label": "app", "port": "8080"}"#;
    let (w, diagnostics) = ParsedConfig::from_json(json).unwrap();

    println!("w = {:?}", w);
    println!("diagnostics = {:?}", diagnostics);

    assert_eq!(w.port, 8080);
}

/// Case 3: Already a number (not stringified) with parse_json_string + coerce
#[test]
fn parse_json_coerce_already_number() {
    let json = r#"{"label": "app", "port": 8080}"#;
    let (w, diagnostics) = ParsedConfig::from_json(json).unwrap();

    println!("w = {:?}", w);
    assert_eq!(w.port, 8080);
    assert!(
        diagnostics.is_empty(),
        "no coercion needed for native number"
    );
}

/// Case 4: Stringified boolean → parse → coerce to i64
#[derive(Debug, Laminate)]
struct BoolToInt {
    #[laminate(parse_json_string, coerce)]
    flag: i64,
}

#[test]
fn parse_json_bool_string_coerce_to_int() {
    // "true" → parse_json_string → Value::Bool(true) → coerce to i64 → 1
    let json = r#"{"flag": "true"}"#;
    let (w, diagnostics) = BoolToInt::from_json(json).unwrap();

    println!("w = {:?}", w);
    println!("diagnostics = {:?}", diagnostics);

    // Bool(true) coerced to i64: should be 1
    assert_eq!(w.flag, 1, "true should coerce to 1");
}
