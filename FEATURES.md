# Laminate: Comprehensive Feature and Use Case Reference

> Progressive data shaping for Rust. Bonds layers of structure onto raw data:
> progressively, configurably, without breaking.

This reference is verified against the source. Signatures, counts, and behaviors
below reflect the actual public API; see [docs.rs](https://docs.rs/laminate) for
full rustdoc.

---

## Table of Contents

1. [Core Concept](#core-concept)
2. [FlexValue: The Central API](#flexvalue-the-central-api)
3. [Coercion Engine](#coercion-engine)
4. [Operational Modes](#operational-modes)
5. [Derive Macro](#derive-macro)
6. [Domain Packs](#domain-packs)
7. [Schema Inference and Data Auditing](#schema-inference-and-data-auditing)
8. [Type Detection](#type-detection)
9. [Streaming and Providers](#streaming-and-providers)
10. [Handler Registry](#handler-registry)
11. [SQL and File Data Sources](#sql-and-file-data-sources)
12. [Diagnostic System](#diagnostic-system)
13. [Use Cases by Domain](#use-cases-by-domain)
14. [Feature Flags and Crates](#feature-flags-and-crates)

---

## Core Concept

Laminate fills the gap between Rust's two extremes for handling external data:

| Approach | Strength | Weakness |
|----------|----------|----------|
| `#[derive(Deserialize)]` | compile-time type safety | fails on the first unexpected field or type mismatch |
| `serde_json::Value` | accepts anything | no compile-time guarantees; everything is `.get()?.as_str()?` |
| **`FlexValue`** (laminate) | navigable, coercible, auditable | the middle ground |

The progressive pipeline:

```
Raw bytes -> Parsed JSON -> FlexValue (navigable, coercible)
    -> Shaped struct (partially typed) -> Strict struct (fully typed)
```

Every layer is an on-ramp, not a gate. Stop wherever your use case requires.

---

## FlexValue: The Central API

`FlexValue` wraps `serde_json::Value` with path-based navigation, type coercion, and diagnostics. A fresh `FlexValue` defaults to `CoercionLevel::BestEffort`.

### Construction

| Method | Description |
|--------|-------------|
| `FlexValue::from_json(json_str)` | Parse a JSON string (strips a leading UTF-8 BOM first) |
| `FlexValue::new(serde_json::Value)` | Wrap an existing `Value` |
| `FlexValue::from_llm_response(text)` | Extract a JSON payload from LLM text (Markdown code fences, then the first balanced JSON span, then the trimmed whole), then parse it. `from_json` stays strict. |

### Configuration (builder pattern)

| Method | Effect |
|--------|--------|
| `.with_coercion(CoercionLevel)` | Set coercion aggressiveness (`Exact` .. `BestEffort`) |
| `.with_source_hint(SourceHint)` | Apply origin-based defaults (see Source Hints) |
| `.with_pack_coercion(PackCoercion)` | Enable domain-pack coercion (`Currency`, `Units`, `All`) |
| `.with_mode::<M>()` | Set mode at compile time (`Lenient`, `Absorbing`, `Strict`); sets the coercion level |
| `.with_dynamic_mode(DynamicMode)` | Set mode at runtime (from config/env) |
| `.with_data_source(impl CoercionDataSource + 'static)` | Attach external data (exchange rates, conversion factors) |
| `.data_source()` | Getter: `Option<&dyn CoercionDataSource>` |

### Navigation

| Method | Returns | Behavior |
|--------|---------|----------|
| `.at("path")` | `Result<FlexValue>` | Navigate to a nested value |
| `.extract::<T>("path")` | `Result<T>` | Navigate + coerce + deserialize |
| `.maybe::<T>("path")` | `Result<Option<T>>` | Like `extract`, but `None` for missing/null/out-of-bounds |
| `.extract_root::<T>()` | `Result<T>` | Coerce + deserialize the root value |
| `.has("path")` | `bool` | Whether a path exists |

Path syntax supports dot notation, array indices (`users[0].address.city`), and bracketed quoted keys for special characters (`meta["content-type"]`). Navigation transparently crosses stringified-JSON boundaries (e.g. a `"data"` field whose string value is `'{"inner": 42}'`). The `path` module exposes `parse_path()` and the `Segment` enum (`Key`/`Index`) publicly.

### Iteration

| Method | Returns | Behavior |
|--------|---------|----------|
| `.each("path")` | `Vec<FlexValue>` | Collect all elements of an array at path |
| `.each_iter("path")` | `FlexIter` | Lazy iterator over array elements |

Both transparently parse stringified JSON arrays (e.g. a string `"[1,2,3]"`).

### Introspection

| Method | Returns |
|--------|---------|
| `.is_null()`, `.is_string()`, `.is_array()`, `.is_object()` | `bool` |
| `.keys()` | `Option<Vec<&str>>` (object keys) |
| `.len()` | `Option<usize>` (array length or object key count) |
| `.is_empty()` | `Option<bool>` |
| `.raw()` / `.into_raw()` | `&Value` / `Value` |

### Mutation

| Method | Description |
|--------|-------------|
| `.merge(&other)` | Deep merge: objects recursively merged, scalars replaced |
| `.merge_shallow(&other)` | Shallow merge: top-level keys replaced wholesale |
| `.merge_with_diagnostics(&other)` | Deep merge with a diagnostic trail (what was overridden/added) |
| `.set("path", value)` | Set a value at a path, creating intermediates |

### Diagnostics and mode shaping

| Method | Returns |
|--------|---------|
| `.extract_with_diagnostics::<T>("path")` | `Result<(T, Vec<Diagnostic>)>` |
| `.extract_root_with_diagnostics::<T>()` | `Result<(T, Vec<Diagnostic>)>` |
| `.shape::<T, M: Mode>("path")` | `Result<(T, Vec<Diagnostic>)>` (apply a mode, then extract with diagnostics) |

### Source Hints

`with_source_hint` only changes behavior for `Csv`, `Env`, and `FormData`, and only if `with_coercion` was not already set explicitly. For those three it sets `BestEffort` coercion **and** `PackCoercion::All`. `Json`, `Database`, and `Unknown` are no-ops (they keep current settings).

| Hint | Effect |
|------|--------|
| `SourceHint::Csv` / `Env` / `FormData` | BestEffort coercion + PackCoercion::All (unless coercion was set explicitly) |
| `SourceHint::Json` / `Database` / `Unknown` | no change (keep current settings) |

---

## Coercion Engine

Four levels of aggressiveness, strictest to most permissive. `CoercionLevel` derives `Ord`, so `level >=` gates work.

### Coercion Levels

| Level | Behavior | Use for |
|-------|----------|---------|
| `Exact` | no coercion; types must match; rejects int->float | output, validation |
| `SafeWidening` | safe numeric widening (int->float), range-checked | round-tripping, proxying |
| `StringCoercion` | parse strings to targets (string->number, string->bool) | config, env vars |
| `BestEffort` | everything: string coercion, null->default, null-sentinels, stringified JSON, locale numbers, single-element-array unwrap | external APIs, scraping, CSV |

### Public coercion API

The engine is reachable directly:

```rust
pub trait Coercible: DeserializeOwned {
    fn coercion_hint() -> &'static str;
    fn is_optional() -> bool;
    fn element_hint() -> Option<&'static str>;
    fn is_element_optional() -> bool;
}

pub fn coerce_for<T: Coercible>(value: &Value, level: CoercionLevel, path: &str) -> CoercionResult;
pub fn coerce_value(value: &Value, target_type: &str, level: CoercionLevel, path: &str) -> CoercionResult;

pub struct CoercionResult { pub value: Value, pub coerced: bool, pub diagnostic: Option<Diagnostic> }
```

`Coercible` has blanket impls for all primitives, `Option<T>`, `Vec<T>`, and `serde_json::Value`. `coerce_for` is the preferred trait-based entry point; `coerce_value` is the string-hint-based core used by the derive macro.

### Built-in Coercion Table

| From -> To | Level required | Example |
|------------|----------------|---------|
| String -> Integer | StringCoercion | `"42"` -> `42` |
| String -> Float | StringCoercion | `"3.14"` -> `3.14` |
| String -> Bool | StringCoercion | `true`/`1`/`yes`/`on`/`y`/`t` -> `true` (and falsy mirror) |
| Hex/Octal/Binary string -> Integer | StringCoercion | `"0xFF"` -> `255` |
| Integer -> Float | SafeWidening | `42` -> `42.0` |
| Float -> Integer (lossless) | SafeWidening | `3.0` -> `3` (rejects `3.5`) |
| Bool -> Integer | SafeWidening | `true` -> `1` |
| Bool -> String | StringCoercion | `true` -> `"true"` |
| Integer -> Bool | SafeWidening | `0`/`1` only |
| Null -> Default | BestEffort | `null` -> `0`/`""`/`false` per target |
| Null sentinel -> Null | BestEffort | see sentinel list below |
| Stringified JSON -> Parsed | BestEffort | `'{"a":1}'` -> object |
| Single-element array -> Scalar | BestEffort | `[42]` -> `42` |
| Object/Array -> String | BestEffort | `{"a":1}` -> `'{"a":1}'` |
| Comma / European / Swiss-French thousands -> Number | BestEffort | `"1,234.56"`, `"1.234,56"`, `"1'234.56"` -> `1234.56` |
| Underscore-separated digits -> Number | BestEffort | `"1_000"` -> `1000` (Rust/Python style) |

**Null sentinels** (case-insensitive): `null`, `none`, `nil`, `n/a`, `na`, `nan`, `unknown`, `undefined`, `-`. The empty string `""` is **not** a sentinel.

**Special float handling:** `"NaN"` coerces to null at BestEffort; `"Infinity"`/`"-Infinity"` are flagged `Risky` (cannot be represented in JSON) and not coerced.

### Range and Precision Guards

- Narrowing is checked at SafeWidening and above: `256 -> u8` emits "value 256 overflows u8 range" (Risky, not coerced); `-1 -> u32` errors.
- `2^53` precision: `|i| > 2^53` targeting `f64` emits a `Warning` ("precision may be lost").
- `f32` overflow (`1e308`) and underflow (a non-zero value rounding to `0.0`) are both guarded.

### Pack Coercion

When enabled (it fires at `StringCoercion` and above), domain packs participate in extraction:

| `PackCoercion` | Effect |
|----------------|--------|
| `None` | packs called explicitly only (default for Exact/SafeWidening) |
| `Currency` | `extract::<f64>("price")` on `"$12.99"` -> `12.99` |
| `Units` | `extract::<f64>("weight")` on `"5.2 kg"` -> `5.2` |
| `All` | currency, units, and time/date detection participate |

### External Data Sources

```rust
pub trait CoercionDataSource: Send + Sync + std::fmt::Debug {
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64> { None }
    fn conversion_factor(&self, from: &str, to: &str) -> Option<f64> { None }
    fn lookup(&self, domain: &str, key: &str) -> Option<serde_json::Value> { None }
}
```

All three methods default to `None`, so an implementor supplies only what it needs. Provided implementations:

- `NoDataSource`: returns `None` for everything (the default).
- `StaticDataSource`: HashMap-backed exchange rates and conversion factors.
- `BuiltinRates` (currency pack): a built-in USD-relative rate table covering ~39 currencies, available opt-in via `with_data_source`.

---

## Operational Modes

A mode answers one question: what happens to data that does not fit? The answer is encoded in the type.

### Three Preset Modes

| Mode | Unknown fields | Coercion | Missing fields | Residual type |
|------|:-:|:-:|:-:|:-:|
| **Lenient** | dropped | BestEffort | defaulted | `()` (zero cost) |
| **Absorbing** | preserved | SafeWidening | error | `Overflow` = `HashMap<String, Value>` |
| **Strict** | error | Exact | error | `Infallible` (compile-time proof) |

These are zero-sized structs implementing the sealed `Mode` trait (`default_coercion`, `reject_unknown_fields`, `require_all_fields`, `fail_fast`, plus an associated `Residual` type). `DynamicMode` is a 3-variant runtime enum with the same query methods, `FromStr`, and `Display`.

### Residuals and Return Shapes

`LaminateResult<T, M: Mode>` bundles the shaped `value`, the mode-typed `residual`, and `diagnostics`. Constructors: `LaminateResult::lenient(value, diagnostics)` and `LaminateResult::absorbing(value, overflow, diagnostics)`.

Important: only `shape_lenient` and `shape_absorbing` return a `LaminateResult`. **`shape_strict` returns the bare value (`Result<Self>`)**: its success is itself the proof that nothing was dropped or coerced, and the `Strict` residual type (`Infallible`) is uninhabitable.

```rust
// Lenient: LaminateResult with residual ()
let result = MyStruct::shape_lenient(&json)?;
assert_eq!(result.residual, ());

// Absorbing: LaminateResult; residual carries every unknown field
let result = MyStruct::shape_absorbing(&json)?;
for (key, val) in &result.residual { println!("preserved: {key} = {val}"); }

// Strict: returns the value itself (NOT a LaminateResult)
let value = MyStruct::shape_strict(&json)?;
```

`shape_strict` rejects on any `Coerced` diagnostic and on a non-empty leftover map. (More generally, `RiskLevel::Warning`/`Risky` diagnostics become errors in strict contexts.)

### Compile-Time vs Runtime Mode Selection

```rust
let value = MyStruct::shape_strict(&json_value)?;          // mode is a type parameter
let mode = DynamicMode::from_str("strict")?;               // mode from config/env
let fv = FlexValue::from_json(json)?.with_dynamic_mode(mode);
```

### Use Cases by Mode

- **Lenient**: external APIs that change without notice, scraping, logs, CSV/config, prototyping.
- **Absorbing**: protocol proxying, config editing (preserve user keys), API-gateway middleware, schema-migration round-trips.
- **Strict**: constructing output, pre-insert validation, test assertions, compliance proof.

---

## Derive Macro

`#[derive(Laminate)]` generates progressive deserialization for **structs and string-valued enums** (`features = ["derive"]`).

### Generated Methods

| Method | Returns |
|--------|---------|
| `T::from_flex_value(&Value)` | `Result<(T, Vec<Diagnostic>)>` |
| `T::from_json(json_str)` | `Result<(T, Vec<Diagnostic>)>` |
| `T::from_llm_response(text)` | `Result<(T, Vec<Diagnostic>)>` (extract JSON from LLM text, then shape) |
| `T::shape_lenient(&Value)` | `Result<LaminateResult<T, Lenient>>` |
| `T::shape_absorbing(&Value)` | `Result<LaminateResult<T, Absorbing>>` |
| `T::shape_strict(&Value)` | `Result<T>` (bare value; see Modes) |
| `t.to_value()` | `Value` |
| `t.to_json()` / `t.to_json_pretty()` | `String` (preserves overflow) |

### Struct Field Attributes (seven)

| Attribute | Effect |
|-----------|--------|
| `#[laminate(coerce)]` | apply coercion to this field (BestEffort) |
| `#[laminate(default)]` | use `Default::default()` if missing or null (records a `Defaulted` diagnostic) |
| `#[laminate(rename = "x")]` | read JSON key `"x"` |
| `#[laminate(skip)]` | never read; always `Default::default()` |
| `#[laminate(overflow)]` | capture unrecognized fields into `HashMap<String, Value>` or `Option<HashMap<...>>` (max one per struct) |
| `#[laminate(flatten)]` | merge a nested object's fields into the parent |
| `#[laminate(parse_json_string)]` | if the value is a string, parse it as JSON first |

`coerce` + `default` combine to fall back to default with an `ErrorDefaulted` diagnostic on coercion failure. All attributes compose. Two fields mapping to the same JSON key (including via `rename`) is a compile error.

### Enum Derive

`#[derive(Laminate)]` on a string-valued enum matches the input string to a unit variant by name (or `#[laminate(rename = "...")]` on the variant). An optional single `#[laminate(unknown)]` newtype variant (e.g. `Unknown(String)`) captures an unrecognized value with a `Coerced`/`Warning` diagnostic instead of erroring. The same generated methods (`from_json`, `from_llm_response`, `shape_*`) apply.

### ToolDefinition Derive

`#[derive(ToolDefinition)]` generates `tool_definition() -> serde_json::Value` shaped as:

```json
{ "name": "<snake_case struct name>", "description": "<struct doc comment>",
  "input_schema": { "type": "object", "properties": { ... }, "required": [ ... ] } }
```

The tool name defaults to the snake_case of the struct name; the description falls back to the struct's doc comment.

---

## Domain Packs

Six packs, always compiled, no feature flags. (The `chrono-integration` and `uom-integration` features add typed-conversion helpers to the time and units packs.)

### Time Pack

| Function | Description |
|----------|-------------|
| `detect_format(s)` | identify a date/time format |
| `convert_to_iso8601(s)` | convert a recognized format to ISO 8601 |
| `convert_to_iso8601_with_hint(s, day_first)` | convert with DD/MM vs MM/DD disambiguation |
| `detect_column_format(values)` | batch: returns `ColumnDateInfo { dominant_format, date_percentage, ambiguous_count, disambiguated, day_first, total }` |
| `coerce_datetime(value, path)` | coercion-engine entry point |
| `to_naive_date(s)` / `to_naive_datetime(s)` | parse to `chrono` types (**requires `chrono-integration`**) |

`DateFormat` has 18 format variants (excluding the `Ambiguous`/`Unknown` markers): ISO 8601, ISO date, US, European, long, abbreviated month, year-only, Unix seconds, Unix millis, 12-hour time, 24-hour time, ISO week date, GEDCOM approximate/range/period/before-after/interpreted, and HL7 v2. Space-separated database datetimes (MySQL/Postgres style) are recognized.

### Currency Pack

| Function | Description |
|----------|-------------|
| `detect_currency_format(s)` | identify symbol/code/locale |
| `parse_currency(s)` | extract amount and currency code |
| `coerce_currency(value, path)` | coercion-engine integration |

**Coverage:** 29 currency codes, 19 symbol entries; US/European/Swiss/Japanese/Indian locales; accounting-negative `(1,234.56)`; crypto codes (BTC, ETH). The pack also exports `BuiltinRates` (`new`, `rate(from, to)`, `convert(amount, from, to)`), a `CoercionDataSource` with a built-in USD-relative table covering ~39 currencies, usable opt-in via `with_data_source`.

### Units Pack

| Function | Description |
|----------|-------------|
| `parse_unit_value(s)` | `"5.2 kg"` -> `UnitValue { amount: f64, unit: String, category: UnitCategory }` |
| `resolve_standard_code(code)` | UNECE/X12/DOD standard-code lookup |
| `conversion_factor(from, to)` / `convert(amount, from, to)` | unit conversion |
| `parse_pack_notation(s)` | `"1x100-count"` -> `PackSize` |
| `parse_qualified_weight(s)` | `"G.W. 15.5kg"` -> `QualifiedWeight` (with `WeightQualifier`: Gross/Net/Tare/Unspecified) |
| `coerce_unit_value(value, path)` | coercion-engine integration |
| `uom_convert::{to_mass, to_length, to_temperature}` | typed conversion (**requires `uom-integration`**) |

`UnitCategory` covers weight, length, temperature (C/F/K formulas), volume, time, data, frequency, speed, pressure, energy, and more. SI prefixes (kHz/MHz/GHz) are supported.

### Identifiers Pack

| Function | Description |
|----------|-------------|
| `validate(s, IdentifierType)` | -> `ValidationResult { is_valid, normalized, detail, error }` |
| `detect(s)` | -> `Vec<(IdentifierType, f64)>` ranked candidates |

**12 types with checksums:** IBAN (mod-97), credit card (Luhn + brand: Visa/Mastercard/Amex/Discover/Maestro/JCB/Diners), ISBN-10, ISBN-13, US SSN, US EIN, US NPI (Luhn), UK NHS (mod-11), EU VAT, UUID, email, phone.

### Geospatial Pack

| Function | Description |
|----------|-------------|
| `parse_coordinate(s)` | -> `Coordinate { latitude, longitude, format, datum }` |
| `detect_coordinate_order(pairs)` | disambiguate lat,lng vs lng,lat |

**Formats:** decimal degrees, DMS (with Unicode primes), DDM (degrees-decimal-minutes), ISO 6709, signed and compass-suffixed (`40.7128°N`). `Datum` is detected: WGS84, JGD2011, CGCS2000, PZ-90, KTRF, Unknown. (UTM/MGRS/Plus Codes are not implemented.)

### Medical Pack

| Function | Description |
|----------|-------------|
| `convert_lab_value(value, analyte, from, to)` | US conventional <-> SI |
| `convert_lab_value_with_config(..., MedicalConfig)` | with case-insensitivity/aliases |
| `known_analytes()` | list supported analytes |
| `classify_lab_value(...)` / `reference_range(...)` | -> `LabClassification` (Low/Normal/High/CriticalLow/CriticalHigh) against `ReferenceRange` |
| `calculate_bmi`, `classify_bmi`, `calculate_bsa` (Du Bois), `calculate_egfr_ckd_epi` (CKD-EPI 2021), `calculate_corrected_calcium`, `calculate_anion_gap`, `calculate_creatinine_clearance` (Cockcroft-Gault) | clinical calculators |
| `normalize_pharma_unit(unit)` | drug-unit normalization (mcg/ug/Unicode mu) |
| `normalize_pharma_abbreviation(abbrev)` | routes (PO/IV/IM), frequencies (QD/BID/PRN/STAT), dosage forms (TAB/CAP/ER) |
| `parse_hl7_datetime(s)` / `parse_hl7_segment(segment)` | HL7 v2 datetime and `|`/`^` segment parsing |
| `extract_fhir_observation(...)` | -> `FhirObservation` (code, display, value, unit, reference range, status, effective datetime) |

**Coverage:** 44 lab-value conversions across 36 analytes; a 33-entry adult reference-range table; clinical calculators; pharmaceutical normalization; HL7 v2 and FHIR parsing.

---

## Schema Inference and Data Auditing

`features = ["schema"]`.

### Inference

```rust
let schema = InferredSchema::from_values(&rows);
let schema = InferredSchema::from_values_with_config(&rows, InferenceConfig {
    required_threshold: 1.0, consistency_threshold: 0.9, max_enum_cardinality: 12,
});
```

Each field becomes a public `FieldDefinition` with `dominant_type`, `present_count`/`absent_count`/`null_count`, `type_counts`, `sample_values`, and query methods `fill_rate()`, `appears_required()`, `is_mixed_type()`, `type_consistency()`. `JsonType` (`Null`, `Bool`, `Integer`, `Float`, `String`, `Array`, `Object`) plus `JsonType::of(&Value)` are public.

### External Constraints

```rust
let mut constraints = HashMap::new();
constraints.insert("status".into(), ExternalConstraint {
    expected_type: Some(JsonType::String), required: true, nullable: false,
    max_length: Some(50), min_value: Some(0.0), max_value: None,
    allowed_values: Some(vec!["active".into(), "inactive".into()]),
});
let schema = schema.with_constraints(constraints);   // takes a HashMap<String, ExternalConstraint>
```

`InferredSchema` also exposes `is_field_required()`, `effective_type()` (external overrides inferred), and `summary()` (a Field/Type/Fill%/Null%/Consistency/Mixed table).

### Audit

```rust
let report = schema.audit(&rows);     // -> AuditReport
println!("{}", report.summary());
```

`AuditReport` has `total_records`, `total_violations`, `field_stats`, `violations`. Per field, `FieldAuditStats` tracks `clean`, **`coercible`** (salvageable type mismatches such as `"42"` in an integer field, distinct from a violation), `violations`, and `missing`.

`ViolationKind` has **5** variants: `TypeMismatch`, `UnexpectedNull`, `MissingRequired`, `UnknownField`, and `ConstraintViolation` (the single kind emitted for out-of-range, max-length, and not-in-allowed-values failures, distinguished by its expected/actual strings). Max-length checks are Unicode-aware (count chars, not bytes).

---

## Type Detection

```rust
use laminate::detect::{guess_type, GuessedType};

let guesses = guess_type("$12.99");                 // Vec<TypeGuess>
assert_eq!(guesses[0].kind, GuessedType::Currency); // ~0.90 confidence
```

`guess_type` returns `Vec<TypeGuess>` where `TypeGuess { kind: GuessedType, confidence: f64 }`, ranked by confidence. `GuessedType` has 20 variants (Integer, Float, Boolean, Date, Currency, UnitValue, Json, Uuid, Email, Url, IpAddress, NullSentinel, Iban, CreditCard, Isbn, Ssn, Ein, VatNumber, Phone, PlainString). A single string can match multiple types.

### Priority and Confidence (representative)

| Detector | Confidence | Notes |
|----------|:---:|---|
| Null sentinels | 0.95 | `null`, `none`, `nil`, `n/a`, `na`, `nan`, `unknown`, `undefined`, `-`, `""` |
| Email | 0.90 | early return (skips date detection) |
| Integer | 0.95 | any valid `i64` |
| Float | 0.95 / 0.70 | non-integer / also-integer |
| Boolean | 0.90 words / 0.60 for `1`/`0` | with Integer demoted to 0.5 on `0`/`1` |
| Date/Time | 0.85 / 0.60 | recognized / ambiguous; 18 formats incl. GEDCOM and HL7 |
| Currency | 0.90 / 0.70 | symbol / European locale; parenthesized accounting at 0.85 |
| Unit value | 0.85 | with category metadata |
| UUID / URL / JSON | 0.95 | |
| IP address | 0.90 / 0.80 | v4 / v6 |
| Identifiers | varies | IBAN, credit card, ISBN, SSN, EIN, VAT, phone |
| PlainString | 1.0 | fallback |

A high-confidence credit card / ISBN, or a specific date format on an all-digit string, demotes Integer/Float confidence so the more specific type ranks first. The identifiers pack detects US NPI and UK NHS, but `guess_type` deliberately suppresses those two as niche.

### Column-Level Detection

```rust
use laminate::packs::time::detect_column_format;
let info = detect_column_format(&["01/02/2026", "03/04/2026", "12/25/2025"]);
// info.dominant_format, info.ambiguous_count, info.day_first
```

---

## Streaming and Providers

`features = ["providers"]` (normalization) and `["streaming"]` (SSE).

### Provider Normalization

| Provider | Module | Parse function |
|----------|--------|----------------|
| Anthropic | `provider::anthropic` | `parse_anthropic_response(&FlexValue)` |
| OpenAI | `provider::openai` | `parse_openai_response(&FlexValue)` |
| Ollama | `provider::ollama` | `parse_ollama_response(&FlexValue)` (native Ollama chat format) |

```rust
let response = laminate::provider::anthropic::parse_anthropic_response(&body)?;
// -> NormalizedResponse { id, model, content: Vec<ContentBlock>, stop_reason, usage, raw }
```

`NormalizedResponse` has six fields (including `raw: FlexValue`, the original response) and methods `text() -> String`, `tool_uses() -> Vec<&ContentBlock>`, `has_tool_use() -> bool`. `ContentBlock` is `Text` / `ToolUse { input: FlexValue }` / `Unknown` (forward-compatible), with `is_text()`/`is_tool_use()`/`as_text()`/`as_tool_use()`. `Usage` has `input_tokens`, `output_tokens`, `cache_read_tokens`, `cache_creation_tokens`, and `extra: HashMap<String, Value>`.

The `ProviderAdapter` trait (`parse_response`, `emit_response`, `stream_parser`) is the extension point, implemented by `AnthropicAdapter`/`OpenAiAdapter`/`OllamaAdapter`. Each provider can also **emit** a `NormalizedResponse` back to its wire format (`emit_anthropic_response`, `emit_openai_response`, `OllamaAdapter::emit_response`).

### SSE Streaming

`FlexStream` is push-style: feed bytes, get back all events for that chunk.

```rust
let mut stream = FlexStream::new(StreamConfig {
    provider: Provider::Anthropic,        // Provider enum: Anthropic | OpenAI
    max_buffer_bytes: 1_048_576,
});
for event in stream.feed(chunk) {         // feed(&[u8]) / feed_str(&str) -> Vec<StreamEvent>
    match event {
        StreamEvent::TextDelta(text) => print!("{text}"),
        StreamEvent::BlockComplete { name, content, .. } => { /* assembled tool call */ }
        StreamEvent::Stop(reason) => {}
        _ => {}
    }
}
let trailing = stream.finish();           // flush
```

`StreamEvent` has 8 variants: `TextDelta(String)`, `BlockStart { .. }`, `BlockDelta { index, fragment }`, `BlockComplete { index, id, block_type, name, content }`, `Metadata`, `Stop(StopReason)`, `Unknown`, `ParseError`. `FlexStream::current_message() -> &MessageSnapshot` exposes a running snapshot (`text`, `tool_calls`, `stop_reason`, `done`). Custom providers implement `StreamHandler` and attach via `FlexStream::with_handler`.

The byte-level SSE parser is reusable standalone: `SseParser` (`new`, `feed`, `feed_bytes`, `finish`) yields `SseEvent { event_type, data }`, handling the `[DONE]` sentinel, multi-line data, heartbeat comments, and chunked/partial UTF-8 delivery.

> Note: Ollama streaming is currently a placeholder (it reuses an Anthropic-style parser); the `Provider` enum has only `Anthropic` and `OpenAI`.

---

## Handler Registry

`features = ["registry"]`. Dispatch tool calls from LLM responses to typed handlers.

```rust
let mut registry = HandlerRegistry::new();
registry.register("search", |args: SearchArgs| async move { Ok(do_search(&args.query).await) });
registry.register_sync("add", |args: MathArgs| Ok(args.a + args.b));
let results = registry.dispatch_all(&response).await?;   // -> Vec<HandlerResult>
```

Methods: `register` (typed async), `register_sync`, `register_raw` (raw `FlexValue`, async), `dispatch` / `dispatch_sync` (single block), `dispatch_all` / `dispatch_all_sync`, and introspection `has`, `names`, `len`, `is_empty`. `dispatch_sync` returns a helpful error if a name is registered async-only. `HandlerResult` is `{ block_id, name, result: serde_json::Value }`.

---

## SQL and File Data Sources

The `laminate-sql` crate exposes async data sources and file ingestion.

```rust
let rows = SqliteSource::connect("sqlite:data.db").await?
    .query("SELECT * FROM customers").await?;     // -> Vec<FlexValue>
```

The async `DataSource` trait has `query(sql)`, `query_with(sql, params)`, `columns(sql) -> Vec<String>`, and `count(sql) -> u64`. Backends `PostgresSource` / `SqliteSource` / `MysqlSource` each provide `connect(url)`, `from_pool(pool)`, and `pool()`. Errors are `DataSourceError` (`ConnectionFailed`, `QueryFailed`, `SerializationFailed`, `Unsupported`).

Database-independent file ingestion (no feature gate): `read_jsonl(path)` and `read_json_array(path)`, both returning `Result<Vec<FlexValue>, DataSourceError>`.

**Backends (feature-gated):** `postgres`, `sqlite`, `mysql`, and `all-databases`.

---

## Diagnostic System

Every coercion, default, drop, and preservation is recorded; nothing is silently swallowed, and nothing is fatally rejected unless the mode says so.

```rust
pub struct Diagnostic {
    pub path: String,            // "data.user.age"
    pub kind: DiagnosticKind,
    pub risk: RiskLevel,         // Info / Warning / Risky
    pub suggestion: Option<String>,
}
```

`DiagnosticKind` (6 variants): `Coerced { from, to }`, `Defaulted { field, value }`, `Dropped { field }`, `Preserved { field }`, `ErrorDefaulted { field, error }`, `Overridden { from_type, to_type }`.

`RiskLevel` (3): `Info` (allowed), `Warning` (becomes an error in strict contexts), `Risky` (becomes an error). 

### Sinks

`DiagnosticSink` has `receive(&Diagnostic)` and a `receive_all(&[Diagnostic])` batch method (default impl). Implementations:

| Sink | Behavior |
|------|----------|
| `Vec<Diagnostic>` / `CollectSink` | collect |
| `StderrSink` | print to stderr |
| `FilteredSink<S>` | `FilteredSink::new(inner, min_risk)`: forward only at/above a risk level |
| `NullSink` | discard |
| custom `impl DiagnosticSink` | route to logs, metrics, databases, UI |

(`StopReason`: `EndTurn`, `ToolUse`, `MaxTokens`, `StopSequence`, `Unknown(String)`, used by the provider/streaming layers, is defined in this module.)

---

## Use Cases by Domain

- **API integration:** consume evolving REST APIs (lenient), normalize Anthropic/OpenAI/Ollama responses, extract typed fields from deep JSON, tolerate string/number drift.
- **Data engineering / ETL:** audit a dataset against a schema (with a coercible tier), tighten progressively, parse CSV strings into typed values, handle locale numbers.
- **Configuration:** coerce env/TOML/YAML values, round-trip configs preserving unknown keys (absorbing), merge layers with a diagnostic trail.
- **Healthcare:** 44 lab-value conversions across 36 analytes, reference-range classification, clinical calculators (eGFR, BMI, anion gap), HL7 v2 and FHIR parsing.
- **Financial / commerce:** parse currency with symbols/locales/accounting negatives, validate IBAN/credit card/EU VAT.
- **Logistics / supply chain:** pack-size notation, qualified weights, UNECE/X12/DOD codes.
- **Geospatial:** parse coordinates (decimal/DMS/DDM/ISO 6709), disambiguate lat/lng order, detect datum.
- **LLM applications:** `from_llm_response` JSON extraction, SSE streaming with tool-fragment assembly, typed tool-call dispatch, `ToolDefinition` schema generation.
- **Validation / compliance:** strict mode for completeness proof, full audit trails, identifier validation, type detection.
- **Testing:** strict assertions, schema audit as data-quality regression, merge for fixtures.

---

## Feature Flags and Crates

| Feature | Enables |
|---------|---------|
| `core` (default) | `FlexValue`, paths, coercion, modes, diagnostics, the 6 domain packs, type detection, source hints |
| `derive` | `#[derive(Laminate)]` and `#[derive(ToolDefinition)]` |
| `streaming` | SSE parser and stream handlers |
| `providers` | Anthropic/OpenAI/Ollama normalization and `from_llm_response` |
| `registry` | typed handler dispatch for tool calls |
| `schema` | schema inference and data auditing |
| `full` | all of the above |
| `chrono-integration` | `to_naive_date`/`to_naive_datetime` and chrono conversions |
| `uom-integration` | `uom_convert` typed unit conversions |

| Crate | Purpose | Notable features |
|-------|---------|------------------|
| `laminate` | core library | as above |
| `laminate-derive` | proc macros (`Laminate`, `ToolDefinition`) | |
| `laminate-sql` | database + file connectors | `postgres`, `sqlite`, `mysql`, `all-databases` |
| `laminate-cli` | command-line auditing and inspection | `infer`, `audit`, `inspect` |

(`laminate-harness` is an internal test harness; it is `publish = false` and not part of the public surface.)
