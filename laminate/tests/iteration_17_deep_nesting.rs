//! Iteration 17: Nesting — Countries multi.json deep path navigation
//! Result: PASS — 5-level deep paths, Unicode values, and missing intermediate keys all handled correctly.

use laminate::FlexValue;

fn load_countries() -> FlexValue {
    let full = format!(
        "{}/testdata/countries/multi.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&full).unwrap();
    FlexValue::from_json(&json).unwrap()
}

#[test]
fn iter17_5_level_deep_path_with_unicode() {
    // [0] → name → nativeName → jpn → common: 5 navigation levels
    let flex = load_countries();
    let result: String = flex.extract("[0].name.nativeName.jpn.common").unwrap();
    assert_eq!(result, "日本");
}

#[test]
fn iter17_missing_intermediate_key_error() {
    // Japan has nativeName.jpn but NOT nativeName.spa
    let flex = load_countries();
    let result = flex.extract::<String>("[0].name.nativeName.spa.common");
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("nativeName.spa"),
        "error should point to missing segment: {err}"
    );
}

#[test]
fn iter17_multiple_native_names_with_unicode() {
    let flex = load_countries();
    let hin: String = flex.extract("[1].name.nativeName.hin.common").unwrap();
    assert_eq!(hin, "भारत");
    let tam: String = flex.extract("[1].name.nativeName.tam.common").unwrap();
    assert_eq!(tam, "இந\u{bcd}திய\u{bbe}");
}

#[test]
fn iter17_unicode_currency_symbols() {
    let flex = load_countries();
    let yen: String = flex.extract("[0].currencies.JPY.symbol").unwrap();
    assert_eq!(yen, "¥");
    let inr: String = flex.extract("[1].currencies.INR.symbol").unwrap();
    assert_eq!(inr, "₹");
}

#[test]
fn iter17_unicode_tld_via_array_index() {
    // Japan's second TLD is ".みんな" (Unicode internationalized TLD)
    let flex = load_countries();
    let tld: String = flex.extract("[0].tld[1]").unwrap();
    assert_eq!(tld, ".みんな");
}
