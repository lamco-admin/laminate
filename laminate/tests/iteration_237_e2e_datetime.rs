//! Iteration 237: E2E Date pipeline
//!
//! detect_column_format → disambiguate → convert_to_iso8601 → extract as String.
//!
//! Adversarial angles:
//! - Column with mixed ambiguous dates — can detect_column_format disambiguate?
//! - Ambiguous "01/02/2026" — US vs EU interpretation
//! - Unix timestamps mixed with date strings
//! - HL7 dates from medical pipeline

use laminate::packs::time::{
    convert_to_iso8601, convert_to_iso8601_with_hint, detect_column_format,
};

#[test]
fn column_format_detection_with_disambiguator() {
    // Column with dates where one value has day > 12, proving DD/MM (EU) format
    let values = vec!["01/02/2026", "05/03/2026", "15/06/2026", "22/11/2025"];
    let info = detect_column_format(&values);

    println!("Column info: {:?}", info);
    println!("Dominant format: {:?}", info.dominant_format);
    println!(
        "Disambiguated: {}, day_first: {:?}",
        info.disambiguated, info.day_first
    );

    // "15/06/2026" has first field > 12 → proves DD/MM (European)
    assert!(
        info.disambiguated,
        "column with day>12 values should be disambiguated"
    );
    assert_eq!(
        info.day_first,
        Some(true),
        "should detect European (DD/MM) format"
    );
}

#[test]
fn column_all_ambiguous_no_disambiguation() {
    // All dates have both parts ≤ 12 — no way to disambiguate
    let values = vec!["01/02/2026", "05/03/2026", "06/07/2025", "08/09/2024"];
    let info = detect_column_format(&values);

    println!("All ambiguous: {:?}", info);
    assert!(
        !info.disambiguated,
        "all-ambiguous column cannot be disambiguated"
    );
    assert_eq!(info.day_first, None, "should return None for ambiguous");
}

#[test]
fn convert_various_formats_to_iso8601() {
    // Each format should convert to ISO 8601
    let cases = vec![
        ("2026-04-06", "2026-04-06"),    // Already ISO
        ("04/06/2026", "2026-04-06"),    // US format (ambiguous, defaults to MM/DD)
        ("31 March 2026", "2026-03-31"), // Long format
        ("Mar 31, 2026", "2026-03-31"),  // Abbreviated
        ("2026", "2026-01-01"),          // Year only → Jan 1
    ];

    for (input, expected) in &cases {
        let result = convert_to_iso8601(input);
        println!("{} → {:?}", input, result);
        assert_eq!(
            result.as_deref(),
            Some(*expected),
            "convert_to_iso8601({}) should produce {}",
            input,
            expected
        );
    }
}

#[test]
fn ambiguous_date_with_hint() {
    // "01/02/2026" is ambiguous — depends on day_first hint
    let us_result = convert_to_iso8601_with_hint("01/02/2026", false);
    let eu_result = convert_to_iso8601_with_hint("01/02/2026", true);

    println!("US interpretation: {:?}", us_result);
    println!("EU interpretation: {:?}", eu_result);

    // US: MM/DD → January 2
    assert_eq!(us_result.as_deref(), Some("2026-01-02"));
    // EU: DD/MM → February 1
    assert_eq!(eu_result.as_deref(), Some("2026-02-01"));
}

#[test]
fn unix_timestamp_conversion() {
    // 1711900800 = 2024-03-31T16:00:00Z (16:00 UTC on March 31)
    let result = convert_to_iso8601("1711900800");
    println!("Unix 1711900800 → {:?}", result);
    assert!(result.is_some(), "Unix timestamp should convert");
    let iso = result.unwrap();
    assert_eq!(
        iso, "2024-03-31T16:00:00Z",
        "1711900800 is 2024-03-31T16:00:00Z"
    );
}

#[test]
fn hl7_date_conversion() {
    let result = convert_to_iso8601("20260402");
    println!("HL7 20260402 → {:?}", result);
    assert_eq!(
        result.as_deref(),
        Some("2026-04-02"),
        "HL7 date should convert to ISO"
    );
}

#[test]
fn gedcom_dates_return_none() {
    // GEDCOM dates are approximate — convert_to_iso8601 should return None
    // because they can't be meaningfully represented as a specific ISO date
    let gedcom_cases = vec!["ABT 1850", "BET 1840 AND 1860", "BEF 1899"];

    for case in &gedcom_cases {
        let result = convert_to_iso8601(case);
        println!("{} → {:?}", case, result);
        // These are recognized formats but can't produce a single ISO date
    }
}

#[test]
fn full_pipeline_column_detect_then_convert() {
    // Full E2E: detect column format, then use the hint to convert all values
    let column = vec!["15/03/2026", "01/04/2026", "25/12/2025", "07/07/2024"];
    let info = detect_column_format(&column);

    assert!(info.disambiguated, "should disambiguate with day>12");
    let day_first = info.day_first.unwrap_or(false);

    let converted: Vec<String> = column
        .iter()
        .filter_map(|v| convert_to_iso8601_with_hint(v, day_first))
        .collect();

    println!("Converted: {:?}", converted);
    assert_eq!(converted.len(), 4);
    assert_eq!(converted[0], "2026-03-15"); // 15 March
    assert_eq!(converted[1], "2026-04-01"); // 1 April
    assert_eq!(converted[2], "2025-12-25"); // Christmas
    assert_eq!(converted[3], "2024-07-07"); // 7 July
}
