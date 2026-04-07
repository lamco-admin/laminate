//! Iteration 56: Open-Meteo weather API — numeric precision, timestamps, nested arrays.

use laminate::FlexValue;

const WEATHER_SAMPLE: &str = r#"{
    "latitude": 40.710335,
    "longitude": -73.99308,
    "generationtime_ms": 0.03921985626220703,
    "utc_offset_seconds": 0,
    "timezone": "GMT",
    "timezone_abbreviation": "GMT",
    "elevation": 27.0,
    "hourly_units": {
        "time": "iso8601",
        "temperature_2m": "°C"
    },
    "hourly": {
        "time": [
            "2026-03-31T00:00",
            "2026-03-31T01:00",
            "2026-03-31T02:00"
        ],
        "temperature_2m": [18.7, 18.0, 17.2]
    }
}"#;

#[test]
fn probe_weather_data() {
    let fv = FlexValue::from_json(WEATHER_SAMPLE).unwrap();

    // High-precision float
    let gen_time: f64 = fv.extract("generationtime_ms").unwrap();
    println!("generationtime_ms = {gen_time}");
    assert!((gen_time - 0.03921985626220703).abs() < 1e-15);

    // Integer zero
    let offset: i64 = fv.extract("utc_offset_seconds").unwrap();
    assert_eq!(offset, 0);

    // Float with .0 (27.0 — serde might store as integer)
    let elevation: f64 = fv.extract("elevation").unwrap();
    println!("elevation = {elevation}");
    assert!((elevation - 27.0).abs() < f64::EPSILON);

    // Nested array access
    let temp: f64 = fv.extract("hourly.temperature_2m[0]").unwrap();
    println!("temp[0] = {temp}");
    assert!((temp - 18.7).abs() < 0.01);

    // Timestamp string extraction
    let time: String = fv.extract("hourly.time[0]").unwrap();
    println!("time[0] = {time}");
    assert_eq!(time, "2026-03-31T00:00");

    // Unicode unit string
    let unit: String = fv.extract("hourly_units.temperature_2m").unwrap();
    println!("temp unit = {unit}");
    assert_eq!(unit, "°C");

    // Array length
    let temps = fv.at("hourly.temperature_2m").unwrap();
    assert_eq!(temps.len(), Some(3));
}
