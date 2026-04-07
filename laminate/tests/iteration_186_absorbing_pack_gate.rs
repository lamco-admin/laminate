//! Iteration 186: Absorbing mode + PackCoercion::All
//!
//! Absorbing maps to SafeWidening coercion, which is below StringCoercion.
//! Pack coercion requires >= StringCoercion to fire, so packs should be
//! silently blocked even when PackCoercion::All is set.

use laminate::mode::Absorbing;
use laminate::value::PackCoercion;
use laminate::FlexValue;

#[test]
fn absorbing_mode_blocks_pack_coercion() {
    // "$12.99" is a valid currency string that the currency pack would parse to 12.99
    let val = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_mode::<Absorbing>() // SafeWidening
        .with_pack_coercion(PackCoercion::All); // All packs enabled

    // Extract as f64 — at SafeWidening, packs should NOT fire,
    // so this should fail (string "$12.99" can't be parsed without packs)
    let result = val.extract::<f64>("price");
    println!(
        "absorbing + PackCoercion::All, extract::<f64>(\"price\"): {:?}",
        result
    );

    // Packs blocked by SafeWidening coercion level
    assert!(result.is_err(), "packs should be blocked at SafeWidening");

    // Control: same input with BestEffort (Lenient) + packs SHOULD succeed
    let val_lenient = FlexValue::from_json(r#"{"price": "$12.99"}"#)
        .unwrap()
        .with_mode::<laminate::mode::Lenient>() // BestEffort >= StringCoercion
        .with_pack_coercion(PackCoercion::All);
    let result_lenient = val_lenient.extract::<f64>("price");
    println!(
        "lenient + PackCoercion::All, extract::<f64>(\"price\"): {:?}",
        result_lenient
    );
    assert!(result_lenient.is_ok(), "packs should fire at BestEffort");
    assert!((result_lenient.unwrap() - 12.99).abs() < 0.001);
}
