//! Iteration 24: Encoding — Unicode field names (emoji, RTL, CJK, dot-in-key, null byte)
//! PASS: Path parser handles all Unicode correctly since it only splits on ASCII . and [

use laminate::FlexValue;

#[test]
fn iter24_emoji_rtl_cjk_keys_navigate_correctly() {
    let json = serde_json::json!({
        "🔑": "emoji value",
        "اسم": "Arabic name",
        "日本語": "Japanese",
        "a.b": "dot in key",
        "normal": {
            "🎯": "nested emoji",
            "спасибо": "Russian thanks"
        },
        "a\u{0000}b": "null byte key"
    });

    let flex = FlexValue::new(json);

    // Emoji, RTL, CJK all work as bare path segments
    assert_eq!(flex.extract::<String>("🔑").unwrap(), "emoji value");
    assert_eq!(flex.extract::<String>("اسم").unwrap(), "Arabic name");
    assert_eq!(flex.extract::<String>("日本語").unwrap(), "Japanese");

    // Dot-in-key: bare path splits (expected), quoted path works
    assert!(flex.extract::<String>("a.b").is_err());
    assert_eq!(flex.extract::<String>(r#"["a.b"]"#).unwrap(), "dot in key");

    // Nested Unicode keys
    assert_eq!(flex.extract::<String>("normal.🎯").unwrap(), "nested emoji");
    assert_eq!(
        flex.extract::<String>("normal.спасибо").unwrap(),
        "Russian thanks"
    );

    // Null byte in key passes through
    assert_eq!(
        flex.extract::<String>("a\u{0000}b").unwrap(),
        "null byte key"
    );
}
