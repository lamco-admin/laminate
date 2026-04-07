/// Iteration 114: Temperature conversion — formula-based ConversionRule
///
/// The units pack now supports formula-based conversions (scale + offset)
/// for temperature, alongside the existing factor-based conversions for
/// weight, length, and volume.
use laminate::packs::units::{convert, parse_unit_value};

#[test]
fn celsius_to_fahrenheit() {
    let result = convert(100.0, "c", "f").unwrap();
    assert!(
        (result - 212.0).abs() < 0.01,
        "100°C should be 212°F, got {}",
        result
    );

    let result = convert(0.0, "c", "f").unwrap();
    assert!(
        (result - 32.0).abs() < 0.01,
        "0°C should be 32°F, got {}",
        result
    );

    let result = convert(37.0, "c", "f").unwrap();
    assert!(
        (result - 98.6).abs() < 0.1,
        "37°C should be ~98.6°F, got {}",
        result
    );
}

#[test]
fn fahrenheit_to_celsius() {
    let result = convert(212.0, "f", "c").unwrap();
    assert!(
        (result - 100.0).abs() < 0.01,
        "212°F should be 100°C, got {}",
        result
    );

    let result = convert(32.0, "f", "c").unwrap();
    assert!(result.abs() < 0.01, "32°F should be 0°C, got {}", result);

    let result = convert(98.6, "f", "c").unwrap();
    assert!(
        (result - 37.0).abs() < 0.1,
        "98.6°F should be ~37°C, got {}",
        result
    );
}

#[test]
fn celsius_to_kelvin() {
    let result = convert(0.0, "c", "k").unwrap();
    assert!(
        (result - 273.15).abs() < 0.01,
        "0°C should be 273.15K, got {}",
        result
    );

    let result = convert(-273.15, "c", "k").unwrap();
    assert!(
        result.abs() < 0.01,
        "-273.15°C should be 0K, got {}",
        result
    );
}

#[test]
fn weight_conversion_still_works() {
    let result = convert(1.0, "kg", "lb").unwrap();
    assert!((result - 2.20462).abs() < 0.001);

    let result = convert(1.0, "lb", "kg").unwrap();
    assert!((result - 0.453592).abs() < 0.001);
}

#[test]
fn time_unit_parsing() {
    let uv = parse_unit_value("500 ms").unwrap();
    assert_eq!(uv.unit, "ms");
    assert!((uv.amount - 500.0).abs() < f64::EPSILON);

    let uv = parse_unit_value("30 seconds").unwrap();
    assert_eq!(uv.unit, "s");

    let uv = parse_unit_value("2.5 hours").unwrap();
    assert_eq!(uv.unit, "h");
}

#[test]
fn time_conversion() {
    let result = convert(120.0, "s", "min").unwrap();
    assert!(
        (result - 2.0).abs() < 0.01,
        "120s should be 2min, got {}",
        result
    );

    let result = convert(1.5, "h", "min").unwrap();
    assert!(
        (result - 90.0).abs() < 0.01,
        "1.5h should be 90min, got {}",
        result
    );
}

#[test]
fn nautical_mile_conversion() {
    let result = convert(1.0, "nmi", "km").unwrap();
    assert!((result - 1.852).abs() < 0.001);
}

#[test]
fn data_conversion() {
    let result = convert(1024.0, "mb", "gb").unwrap();
    assert!((result - 1.024).abs() < 0.001);
}
