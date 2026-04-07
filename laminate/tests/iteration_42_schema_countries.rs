use laminate::schema::{InferredSchema, JsonType, ViolationKind};
use serde_json::Value;

/// Iteration 42 — Schema inference on nested Countries API data.
///
/// Probe: Rich nested data (5 countries, ~35 top-level fields) with:
/// - Nested objects (name, currencies, translations, maps, flags, etc.)
/// - Arrays of strings (tld, altSpellings, capital, timezones, continents)
/// - Optional field (borders: absent for Japan/island nations)
/// - Float values with fract()==0.0 (area: 377930.0)
/// - Deeply nested objects (translations with ~26 language sub-objects)
///
/// Key questions:
/// 1. Does it correctly detect `borders` as absent for Japan?
/// 2. How many top-level fields are discovered?
/// 3. Does `area` get Integer or Float? (all values end in .0)
/// 4. What happens when audit data has a type change in a nested field?

fn load_countries() -> Vec<Value> {
    let path = format!(
        "{}/testdata/countries/multi.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn probe_schema_from_nested_countries_data() {
    let rows = load_countries();
    assert_eq!(rows.len(), 5);

    let schema = InferredSchema::from_values(&rows);
    println!("Total fields: {}", schema.fields.len());
    println!("\n{}", schema.summary());

    // ── Observation 1: Field count ──
    // Countries data has ~35 top-level keys. How many does inference find?
    let field_count = schema.fields.len();
    println!("\nField count: {}", field_count);
    // Just observe — don't assert a specific number yet.

    // ── Observation 2: borders field (absent for Japan) ──
    let borders = &schema.fields["borders"];
    println!(
        "\nborders: dominant_type={:?}, present={}, absent={}, null={}, type_counts={:?}",
        borders.dominant_type,
        borders.present_count,
        borders.absent_count,
        borders.null_count,
        borders.type_counts
    );
    // Japan has no borders key — absent_count should be 1
    assert_eq!(
        borders.absent_count, 1,
        "Japan is an island — no borders key"
    );
    assert_eq!(borders.dominant_type, Some(JsonType::Array));

    // ── Observation 3: area field (all end in .0) ──
    let area = &schema.fields["area"];
    println!(
        "\narea: dominant_type={:?}, type_counts={:?}",
        area.dominant_type, area.type_counts
    );
    // 377930.0, 3287263.0, 357114.0, 8515767.0, 9525067.0 — all fract()==0.0
    // JsonType::of classifies these as Integer
    // Just observe — this is the existing behavior from iteration 18.

    // ── Observation 4: Nested object fields ──
    let name = &schema.fields["name"];
    assert_eq!(name.dominant_type, Some(JsonType::Object));
    assert!(name.appears_required());

    let translations = &schema.fields["translations"];
    assert_eq!(translations.dominant_type, Some(JsonType::Object));

    let currencies = &schema.fields["currencies"];
    assert_eq!(currencies.dominant_type, Some(JsonType::Object));

    // ── Observation 5: Array of strings fields ──
    let tld = &schema.fields["tld"];
    assert_eq!(tld.dominant_type, Some(JsonType::Array));
    assert!(tld.appears_required());

    let timezones = &schema.fields["timezones"];
    assert_eq!(timezones.dominant_type, Some(JsonType::Array));

    // ── Observation 6: Boolean fields ──
    let independent = &schema.fields["independent"];
    assert_eq!(independent.dominant_type, Some(JsonType::Bool));

    let landlocked = &schema.fields["landlocked"];
    assert_eq!(landlocked.dominant_type, Some(JsonType::Bool));

    // ── Observation 7: String fields ──
    let region = &schema.fields["region"];
    assert_eq!(region.dominant_type, Some(JsonType::String));
    // Sample values collected for string fields
    println!("\nregion sample_values: {:?}", region.sample_values);

    // ── Observation 8: Self-audit (clean data) ──
    let report = schema.audit(&rows);
    println!("\nSelf-audit: {} violations", report.total_violations);
    for v in &report.violations {
        println!("  {}", v);
    }
    // Self-audit should produce 0 violations (data matches its own schema)
    // BUT: borders is absent in Japan, and is_field_required checks fill_rate >= 1.0
    // fill_rate = (present - null) / total = (4 - 0) / 5 = 0.8 < 1.0
    // So borders is NOT required → no MissingRequired violation. Good.
    assert_eq!(report.total_violations, 0, "Self-audit should be clean");
}

#[test]
fn probe_audit_nested_type_change() {
    // Infer schema from the real countries data
    let rows = load_countries();
    let schema = InferredSchema::from_values(&rows);

    // Now audit with mutated data where "name" is a string instead of an object
    let mutated = vec![serde_json::json!({
        "tld": [".xx"],
        "cca2": "XX",
        "ccn3": "999",
        "cca3": "XXX",
        "cioc": "XXX",
        "independent": true,
        "status": "officially-assigned",
        "unMember": true,
        "idd": {"root": "+0", "suffixes": ["0"]},
        "capital": ["Testville"],
        "altSpellings": ["XX"],
        "region": "Testing",
        "subregion": "Unit Tests",
        "landlocked": false,
        "borders": ["AAA"],
        "area": 100.0,
        "maps": {"googleMaps": "https://example.com", "openStreetMaps": "https://example.com"},
        "population": 1000,
        "fifa": "XXX",
        "car": {"signs": ["X"], "side": "right"},
        "timezones": ["UTC+00:00"],
        "continents": ["Testing"],
        "flag": "🏳",
        // KEY MUTATION: name is a string, not an object
        "name": "Test Country",
        "currencies": {"XXX": {"symbol": "X", "name": "Test dollar"}},
        "languages": {"eng": "English"},
        "latlng": [0.0, 0.0],
        "demonyms": {"eng": {"f": "Tester", "m": "Tester"}},
        "translations": {},
        "gini": {},
        "flags": {"png": "https://example.com/flag.png", "svg": "https://example.com/flag.svg"},
        "coatOfArms": {},
        "startOfWeek": "monday",
        "capitalInfo": {"latlng": [0.0, 0.0]},
        "postalCode": {"format": "#####", "regex": "^(\\d{5})$"}
    })];

    let report = schema.audit(&mutated);
    println!("Mutated audit: {} violations", report.total_violations);
    for v in &report.violations {
        println!("  {}", v);
    }

    // The "name" field changed from Object to String.
    // Schema should detect this as a TypeMismatch (not coercible: String→Object).
    let name_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.field == "name" && v.kind == ViolationKind::TypeMismatch)
        .collect();
    println!("\nname violations: {:?}", name_violations);

    // Also check: missing fields from the mutated record
    let missing: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.kind == ViolationKind::MissingRequired)
        .collect();
    println!("Missing required: {:?}", missing);
}
