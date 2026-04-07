use laminate::Laminate;
/// Iteration 218: shape_absorbing overflow residual is always empty
///
/// Target #218: shape_absorbing() sets mode residual to HashMap::new().
/// The struct's #[laminate(overflow)] captures unknowns, but the mode-level
/// residual is disconnected. Is this intentional? Observe actual behavior.
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct Config {
    name: String,
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,
}

#[test]
fn shape_absorbing_residual_vs_overflow_field() {
    let json_val: serde_json::Value = serde_json::from_str(
        r#"{
        "name": "myapp",
        "debug": true,
        "log_level": "info"
    }"#,
    )
    .unwrap();

    let result = Config::shape_absorbing(&json_val).unwrap();

    // Struct overflow field should capture unknown fields
    println!("result.value.extra = {:?}", result.value.extra);
    println!("result.residual = {:?}", result.residual);
    println!("diagnostics = {:?}", result.diagnostics);

    // Verify unknowns are in the struct's overflow
    assert_eq!(
        result.value.extra.len(),
        2,
        "derive overflow should capture 2 unknowns"
    );
    assert_eq!(result.value.extra["debug"], serde_json::json!(true));

    // The design question: is mode residual empty or populated?
    // Current code: shape_absorbing() passes HashMap::new() always.
    // This means unknowns live in the struct only, not in the mode wrapper.
    println!("residual len = {}", result.residual.len());
}
