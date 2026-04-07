/// Iteration 83: DynamicMode::from_str — whitespace tolerance in config parsing
///
/// from_str was rejecting strings with leading/trailing whitespace or
/// newlines, which is common in config files, env vars, and YAML.
/// Fixed by adding trim() before to_lowercase().
use laminate::mode::DynamicMode;

#[test]
fn from_str_trims_whitespace() {
    assert_eq!(
        " lenient ".parse::<DynamicMode>().unwrap(),
        DynamicMode::Lenient,
        "leading/trailing spaces should be trimmed"
    );
}

#[test]
fn from_str_trims_trailing_newline() {
    assert_eq!(
        "strict\n".parse::<DynamicMode>().unwrap(),
        DynamicMode::Strict,
        "trailing newline (common from env vars) should be trimmed"
    );
}

#[test]
fn from_str_trims_mixed_whitespace() {
    assert_eq!(
        "\t Absorbing \r\n".parse::<DynamicMode>().unwrap(),
        DynamicMode::Absorbing,
        "tabs, carriage returns, newlines should all be trimmed"
    );
}

#[test]
fn from_str_roundtrip_all_modes() {
    for mode in [
        DynamicMode::Lenient,
        DynamicMode::Absorbing,
        DynamicMode::Strict,
    ] {
        let s = mode.to_string();
        let parsed: DynamicMode = s.parse().unwrap();
        assert_eq!(parsed, mode);
    }
}
