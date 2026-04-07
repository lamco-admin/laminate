# Laminate — Comprehensive Feature & Use Case Reference

> Progressive data shaping for Rust. Bonds layers of structure onto raw data —
> progressively, configurably, without breaking.

---

## Table of Contents

1. [Core Concept](#core-concept)
2. [FlexValue — The Central API](#flexvalue--the-central-api)
3. [Coercion Engine](#coercion-engine)
4. [Operational Modes](#operational-modes)
5. [Derive Macro](#derive-macro)
6. [Domain Packs](#domain-packs)
7. [Schema Inference & Data Auditing](#schema-inference--data-auditing)
8. [Type Detection](#type-detection)
9. [Streaming & Providers](#streaming--providers)
10. [Handler Registry](#handler-registry)
11. [SQL Data Sources](#sql-data-sources)
12. [Diagnostic System](#diagnostic-system)
13. [Use Cases by Domain](#use-cases-by-domain)

---

## Core Concept

Laminate fills the gap between Rust's two extremes for handling external data:

| Approach | Strength | Weakness |
|----------|----------|----------|
| `#[derive(Deserialize)]` | Compile-time type safety | Fails on first unexpected field or type mismatch |
| `serde_json::Value` | Accepts anything | Zero compile-time guarantees, everything is `.get()?.as_str()?` |
| **`FlexValue`** (laminate) | Navigable, coercible, auditable | The middle ground |

The progressive pipeline:

```
Raw bytes → Parsed JSON → FlexValue (navigable, coercible)
    → Shaped struct (partially typed) → Strict struct (fully typed)
```

Every layer is an **on-ramp, not a gate**. Stop wherever your use case requires.

---

## FlexValue — The Central API

`FlexValue` wraps `serde_json::Value` with path-based navigation, type coercion, and diagnostics.

### Construction

| Method | Description |
|--------|-------------|
| `FlexValue::from_json(json_str)` | Parse a JSON string |
| `FlexValue::new(serde_json::Value)` | Wrap an existing Value |

### Configuration (Builder Pattern)

| Method | Effect |
|--------|--------|
| `.with_coercion(CoercionLevel)` | Set coercion aggressiveness (Exact → BestEffort) |
| `.with_source_hint(SourceHint)` | Declare data origin for smarter defaults |
| `.with_pack_coercion(PackCoercion)` | Enable domain pack coercion (Currency, Units, All) |
| `.with_mode::<M>()` | Set mode (Lenient, Absorbing, Strict) — configures coercion level |
| `.with_dynamic_mode(DynamicMode)` | Set mode at runtime (from config/env) |
| `.with_data_source(impl CoercionDataSource)` | Attach external data (exchange rates, etc.) |

### Navigation

| Method | Returns | Behavior |
|--------|---------|----------|
| `.at("path.to.field")` | `Result<FlexValue>` | Navigate to a nested value |
| `.extract::<T>("path")` | `Result<T>` | Navigate + coerce + deserialize |
| `.maybe::<T>("path")` | `Result<Option<T>>` | Like extract, but None for missing/null/OOB |
| `.extract_root::<T>()` | `Result<T>` | Coerce + deserialize the root value |
| `.has("path")` | `bool` | Check if a path exists |

Path syntax supports dot notation and array indices: `"users[0].address.city"`.
Transparently crosses stringified-JSON boundaries (e.g., `"data"` containing `'{"inner": 42}'`).

### Iteration

| Method | Returns | Behavior |
|--------|---------|----------|
| `.each("path")` | `Vec<FlexValue>` | Collect all elements of an array at path |
| `.each_iter("path")` | `FlexIter` | Lazy iterator over array elements |

Both methods transparently parse stringified JSON arrays (e.g., `"[1,2,3]"` as a string).

### Introspection

| Method | Returns | Description |
|--------|---------|-------------|
| `.is_null()` | `bool` | Root is null |
| `.is_string()` | `bool` | Root is a string |
| `.is_array()` | `bool` | Root is an array |
| `.is_object()` | `bool` | Root is an object |
| `.keys()` | `Option<Vec<&str>>` | Object keys (None if not an object) |
| `.len()` | `Option<usize>` | Array length or object key count |
| `.is_empty()` | `Option<bool>` | Whether array/object is empty |
| `.raw()` | `&Value` | Reference to underlying serde_json::Value |
| `.into_raw()` | `Value` | Consume and return underlying Value |

### Mutation

| Method | Description |
|--------|-------------|
| `.merge(&other)` | Deep merge — objects recursively merged, scalars replaced |
| `.merge_shallow(&other)` | Shallow merge — top-level keys replaced wholesale |
| `.merge_with_diagnostics(&other)` | Deep merge with diagnostic trail (what was overridden, added) |
| `.set("path", value)` | Set a value at a path, creating intermediates as needed |

### Diagnostics

| Method | Returns | Description |
|--------|---------|-------------|
| `.extract_with_diagnostics::<T>("path")` | `Result<(T, Vec<Diagnostic>)>` | Extract with coercion audit trail |
| `.extract_root_with_diagnostics::<T>()` | `Result<(T, Vec<Diagnostic>)>` | Same, for root value |

### Source Hints

| Hint | Default Coercion | Use When |
|------|-----------------|----------|
| `SourceHint::Json` | SafeWidening | JSON API responses (types usually correct) |
| `SourceHint::Csv` | BestEffort | CSV files (everything is a string) |
| `SourceHint::Env` | BestEffort | Environment variables (always strings) |
| `SourceHint::FormData` | BestEffort | HTML form submissions (all strings) |
| `SourceHint::Database` | SafeWidening | Database results (typed but may have SQLite dynamics) |
| `SourceHint::Unknown` | (unchanged) | Use current coercion level as-is |

---

## Coercion Engine

Four levels of coercion aggressiveness, from strictest to most permissive:

### Coercion Levels

| Level | What It Does | Use When |
|-------|-------------|----------|
| **Exact** | No coercion. Types must match exactly. Rejects int→float. | Constructing output, validation |
| **SafeWidening** | Safe numeric widening (int→float). Integer range checks. | Round-tripping, protocol proxying |
| **StringCoercion** | Parse strings to target types (string→number, string→bool). | Config files, env vars |
| **BestEffort** | Try everything: string coercion + null→default + stringified JSON + locale numbers + array unwrap. | External APIs, scraping, CSV |

### Built-in Coercion Table

| From → To | Level Required | Example |
|-----------|---------------|---------|
| String → Integer | StringCoercion | `"42"` → `42` |
| String → Float | StringCoercion | `"3.14"` → `3.14` |
| String → Bool | StringCoercion | `"true"`, `"yes"`, `"1"` → `true` |
| Integer → Float | SafeWidening | `42` → `42.0` |
| Float → Integer (lossless) | SafeWidening | `3.0` → `3` (rejects `3.5`) |
| Bool → Integer | SafeWidening | `true` → `1`, `false` → `0` |
| Bool → String | StringCoercion | `true` → `"true"` |
| Integer → Bool | SafeWidening | `0` → `false`, `1` → `true` (rejects others) |
| Null → Default | BestEffort | `null` → `0`, `""`, `false` (per target type) |
| Null sentinel → Null | BestEffort | `"N/A"`, `"null"`, `"none"`, `"-"`, `""` → null |
| Stringified JSON → Parsed | BestEffort | `'{"a":1}'` → parsed JSON object |
| Single-element array → Scalar | BestEffort | `[42]` → `42` |
| Object/Array → String | BestEffort | `{"a":1}` → `'{"a":1}'` (JSON serialization) |
| Comma thousands → Number | BestEffort | `"1,234.56"` → `1234.56` |
| European format → Number | BestEffort | `"1.234,56"` → `1234.56` |
| Swiss/French thousands → Number | BestEffort | `"1'234.56"` → `1234.56` |
| Hex/Octal/Binary strings → Integer | StringCoercion | `"0xFF"` → `255` |
| f64 → f32 (overflow guard) | SafeWidening | `1e308` → error (not silent infinity) |

### Integer Range Validation

Narrowing conversions are checked at SafeWidening+:
- `256` → `u8`: error ("overflows u8 range")
- `-1` → `u32`: error
- `2^53+1` → `f64`: warning (precision loss)

### Pack Coercion

When enabled, domain packs participate in extraction:

| PackCoercion | Effect |
|-------------|--------|
| `None` | Packs must be called explicitly (default for Exact/SafeWidening) |
| `Currency` | `extract::<f64>("price")` on `"$12.99"` → `12.99` |
| `Units` | `extract::<f64>("weight")` on `"5.2 kg"` → `5.2` |
| `All` | All packs participate |

Pack coercion only fires at StringCoercion level and above.

### External Data Sources

```rust
trait CoercionDataSource: Send + Sync {
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64>;
    fn conversion_factor(&self, from: &str, to: &str) -> Option<f64>;
    fn lookup(&self, domain: &str, key: &str) -> Option<FlexValue>;
}
```

Keeps the library lightweight — no stale data baked in.

---

## Operational Modes

### Three Preset Modes

| Mode | Unknown Fields | Coercion | Missing Fields | Errors | Residual Type |
|------|:-:|:-:|:-:|:-:|:-:|
| **Lenient** | Dropped | BestEffort | Defaulted | Collected | `()` (zero-cost) |
| **Absorbing** | Preserved | SafeWidening | Error | Collected | `HashMap<String, Value>` |
| **Strict** | Error | Exact | Error | Fail-fast | `Infallible` (compile-time proof) |

### Residuals — What's Left Over

Every shaping operation produces a `LaminateResult<T, M>` that bundles three things:
1. **The shaped value** (`T`) — your struct
2. **The residual** (`M::Residual`) — what's "left over," typed by mode
3. **Diagnostics** (`Vec<Diagnostic>`) — audit trail of every transformation

The residual type varies by mode, encoding the remainder policy at the type level:

| Mode | Residual Type | Meaning | What It Contains |
|------|:---:|---|---|
| **Lenient** | `()` | Discarded by design | Nothing — zero-cost, unknowns were dropped |
| **Absorbing** | `HashMap<String, Value>` | Preserved for round-tripping | Every unknown field and its value |
| **Strict** | `Infallible` | Compile-time proof of completeness | Uninhabitable — *proves* nothing was left over |

```rust
// Lenient: residual is () — zero-cost, unknowns silently dropped
let result = MyStruct::shape_lenient(&json)?;
assert_eq!(result.residual, ());
// Diagnostics record what was dropped (DiagnosticKind::Dropped)

// Absorbing: residual carries every unknown field
let result = MyStruct::shape_absorbing(&json)?;
for (key, val) in &result.residual {
    println!("Unknown field preserved: {key} = {val}");
}
// Diagnostics record what was preserved (DiagnosticKind::Preserved)

// Strict: residual is Infallible — can never be constructed
let result = MyStruct::shape_strict(&json)?;
// If this line executes, it is a compile-time proof that no unknown
// fields existed. result.residual cannot be accessed or inspected.
```

This is how laminate encodes the "what happens to extra data" question as a type-level guarantee rather than a runtime check.

### Compile-Time vs Runtime Mode Selection

```rust
// Compile-time: mode is a type parameter
let result = MyStruct::shape_strict(&json_value)?;   // Strict at compile time
let result = MyStruct::shape_lenient(&json_value)?;   // Lenient at compile time

// Runtime: mode from config/env
let mode = DynamicMode::from_str("strict")?;
let val = FlexValue::from_json(json)?.with_dynamic_mode(mode);
```

### Use Cases by Mode

**Lenient** — Maximum tolerance for messy data:
- Consuming external APIs that change without notice
- Scraping web data
- Processing log files
- Reading CSV/config files
- Prototyping / exploration

**Absorbing** — Round-trip preservation:
- Protocol proxying (pass through fields you don't understand)
- Config file editing (preserve user's custom keys)
- API gateway middleware (forward unknown fields)
- Data migration (preserve fields during schema transitions)

**Strict** — Correctness is paramount:
- Constructing output to send to an API
- Validating data before database insertion
- Test assertions
- Regulatory compliance (prove no data was silently transformed)

---

## Derive Macro

`#[derive(Laminate)]` generates progressive deserialization with mode-dependent behavior.

### Generated Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `T::from_flex_value(&Value)` | `Result<(T, Vec<Diagnostic>)>` | Deserialize with diagnostics |
| `T::from_json(json_str)` | `Result<(T, Vec<Diagnostic>)>` | Parse + deserialize |
| `T::shape_lenient(&Value)` | `Result<LaminateResult<T, Lenient>>` | Shape in lenient mode — residual is `()`, unknowns dropped |
| `T::shape_absorbing(&Value)` | `Result<LaminateResult<T, Absorbing>>` | Shape in absorbing mode — residual is `Overflow` with unknown fields |
| `T::shape_strict(&Value)` | `Result<LaminateResult<T, Strict>>` | Shape in strict mode — errors on unknowns/coercions, residual is `Infallible` |
| `t.to_value()` | `Value` | Round-trip serialization |

Each `shape_*` method returns a `LaminateResult` that bundles the value, the mode-specific residual (see [Residuals](#residuals--whats-left-over)), and the full diagnostic trail.

### Field Attributes

| Attribute | Effect |
|-----------|--------|
| `#[laminate(coerce)]` | Apply coercion rules to this field (BestEffort level) |
| `#[laminate(default)]` | Use `Default::default()` if missing or null |
| `#[laminate(coerce, default)]` | Try coercion; on failure, fall back to default with `ErrorDefaulted` diagnostic |
| `#[laminate(rename = "x")]` | Read from JSON key `"x"` instead of the field name |
| `#[laminate(skip)]` | Don't read from input; always use `Default::default()` |
| `#[laminate(overflow)]` | Capture unrecognized fields into `HashMap<String, Value>` or `Option<HashMap<...>>` |
| `#[laminate(flatten)]` | Merge fields from a nested object into the parent |
| `#[laminate(parse_json_string)]` | If value is a string, try parsing it as JSON first |

### Attribute Combinations

All attributes compose freely:
- `#[laminate(coerce, rename = "camelCase")]` — read renamed key, coerce to target type
- `#[laminate(parse_json_string, coerce)]` — pipeline: parse string → coerce → deserialize
- `#[laminate(coerce, default)]` — coerce if possible, default if coercion fails

### Supported Field Types

- All primitive types: `i8`–`i64`, `u8`–`u64`, `f32`, `f64`, `bool`, `String`
- `Option<T>` — null/missing → `None`
- `Vec<T>` — with element-level coercion
- `Option<Vec<T>>` — combined null guard + element coercion
- `HashMap<String, Value>` — for overflow fields
- Any type implementing `serde::Deserialize` — falls through to serde

### ToolDefinition Derive

`#[derive(ToolDefinition)]` generates JSON Schema for LLM tool definitions:

```rust
#[derive(ToolDefinition)]
struct SearchArgs {
    /// The search query
    query: String,
    /// Maximum results to return
    max_results: Option<u32>,
}
// Generates: { "type": "object", "properties": { "query": {...}, "max_results": {...} } }
```

---

## Domain Packs

Six built-in packs — always compiled, no feature flags needed.

### Time Pack

| Function | Description |
|----------|-------------|
| `detect_format(s)` | Identify date/time format from a string |
| `convert_to_iso8601(s)` | Convert any recognized date format to ISO 8601 |
| `convert_to_iso8601_with_hint(s, day_first)` | Convert with ambiguity disambiguation |
| `detect_column_format(values)` | Batch detect dominant date format in a column |
| `to_naive_date(s)` | Parse to `chrono::NaiveDate` |
| `to_naive_datetime(s)` | Parse to `chrono::NaiveDateTime` |
| `parse_hl7_datetime(s)` | Parse HL7 v2 datetime format |

**Supported formats (14+):** ISO 8601, US (MM/DD/YYYY), European (DD/MM/YYYY), abbreviated month (01-Jan-2026), slash dates, dash dates, dot dates, 2-digit years, GEDCOM 7.0 approximate dates, HL7 v2 with fractional seconds and timezone, Unix timestamps, ISO week dates, time-without-date.

### Currency Pack

| Function | Description |
|----------|-------------|
| `detect_currency_format(s)` | Identify currency format (symbol, code, locale) |
| `parse_currency(s)` | Extract amount and currency code from string |
| `coerce_currency(value, path)` | Coercion engine integration |

**Coverage:** 30 currency codes, 13 symbols, US/European/Swiss/Japanese/Indian locale support, accounting-negative format `(1,234.56)`, crypto codes (BTC, ETH).

### Units Pack

| Function | Description |
|----------|-------------|
| `parse_unit_value(s)` | Parse "5.2 kg" → `UnitValue { amount: 5.2, unit: "kg" }` |
| `resolve_standard_code(code)` | Look up UNECE/X12/DOD standard codes |
| `conversion_factor(from, to)` | Get conversion factor between units |
| `convert(amount, from, to)` | Convert between units |
| `parse_pack_notation(s)` | Parse "1x100-count" pack-size notation |
| `parse_qualified_weight(s)` | Parse "G.W. 15.5kg" qualified weight |

**Unit categories:** Weight, length, temperature (°C↔°F↔K), volume, time, data, area, speed, pressure, energy, force. SI prefix support (kHz, MHz, GHz). UNECE, X12, and DOD standard code lookup.

### Identifiers Pack

| Function | Description |
|----------|-------------|
| `validate(s, IdentifierType)` | Validate against specific type with normalization |
| `detect(s)` | Auto-detect identifier type(s) with confidence scores |

**Supported types:** IBAN (mod-97), credit card (Luhn + BIN/brand detection), ISBN-10, ISBN-13, US SSN, US EIN, US NPI (Luhn), UK NHS (mod-11), EU VAT, UUID, email, phone.

### Geospatial Pack

| Function | Description |
|----------|-------------|
| `parse_coordinate(s)` | Parse any coordinate format to `Coordinate { lat, lng }` |
| `detect_coordinate_order(pairs)` | Disambiguate lat,lng vs lng,lat |

**Supported formats:** Decimal degrees, DMS (degrees/minutes/seconds) with Unicode primes, ISO 6709, signed coordinates, compass-suffixed ("40.7128°N").

### Medical Pack

| Function | Description |
|----------|-------------|
| `convert_lab_value(value, analyte, from, to)` | Convert between US (conventional) and SI units |
| `convert_lab_value_with_config(...)` | Same with custom analyte config |
| `known_analytes()` | List supported analytes |
| `normalize_pharma_unit(unit)` | Normalize drug units (μg → mcg, Unicode mu handling) |
| `parse_hl7_datetime(s)` | Parse HL7 v2 datetime with fractional seconds |

**Analyte coverage (18):** Glucose, cholesterol (total/HDL/LDL), triglycerides, creatinine, BUN, hemoglobin, hematocrit, sodium, potassium, chloride, calcium, albumin, bilirubin, ALT, AST, TSH.

---

## Schema Inference & Data Auditing

### Schema Inference

```rust
let schema = InferredSchema::from_values(&rows);
```

Per-field inference:
- **Dominant type** — most common JSON type (with wideness-based tiebreaking)
- **Nullability** — whether the field contains nulls
- **Required** — whether the field is always present and non-null
- **Fill rate** — fraction of rows where the field has a non-null value
- **Type consistency** — what percentage of values match the dominant type
- **Mixed type detection** — flags fields with inconsistent types

### External Constraints

Override inferred constraints with external schema definitions:

```rust
schema.with_constraints("field_name", ExternalConstraint {
    expected_type: Some(JsonType::Integer),
    required: true,
    nullable: false,
    max_length: Some(50),
    min_value: Some(0.0),
    allowed_values: Some(vec!["active".into(), "inactive".into()]),
    ..Default::default()
});
```

### Data Audit

```rust
let report = schema.audit(&rows);
println!("{}", report.summary());
```

**Violation types detected:**
- `TypeMismatch` — value is wrong type for the field
- `UnexpectedNull` — null in a non-nullable field
- `MissingField` — required field absent from row
- `OutOfRange` — numeric value outside min/max bounds
- `MaxLengthExceeded` — string exceeds maximum length (Unicode-aware, counts chars not bytes)
- `NotInAllowedValues` — value not in enum constraint
- `UnknownField` — field not in schema

---

## Type Detection — `guess_type()`

The "What IS this?" function. Given any unknown string, `guess_type()` returns ranked type candidates with confidence scores by orchestrating all domain packs and format detectors.

```rust
use laminate::detect::{guess_type, GuessedType};

let guesses = guess_type("$12.99");
assert_eq!(guesses[0].kind, GuessedType::Currency);    // 0.90 confidence
// Also returns Float at lower confidence

let guesses = guess_type("2026-04-02");
assert!(matches!(guesses[0].kind, GuessedType::Date(_))); // ISO 8601

let guesses = guess_type("4111111111111111");
assert_eq!(guesses[0].kind, GuessedType::CreditCard);   // Luhn-valid, Visa BIN
```

### How It Works

`guess_type()` runs the input through **every detector in priority order** and returns ALL matches ranked by confidence. A single string can match multiple types:

| Input | Matches (by confidence) |
|-------|------------------------|
| `"42"` | Integer (0.95), Float (0.70), Boolean (0.30 — "could be truthy") |
| `"01/02/2026"` | Date (0.60 — Ambiguous: could be Jan 2 or Feb 1) |
| `"192.168.1.1"` | IP Address (0.90) |
| `"N/A"` | NullSentinel (0.95) |
| `""` (empty) | NullSentinel (0.95) |
| `"hello world"` | PlainString (1.0 — nothing else matched) |

### Detection Priority & Confidence Scoring

| Detector | Confidence Range | What It Recognizes |
|----------|:---:|---|
| **Null sentinels** | 0.95 | `null`, `none`, `nil`, `N/A`, `na`, `NaN`, `undefined`, `-`, `""`, `unknown` |
| **Boolean** | 0.90 (words), 0.30 (0/1) | `true`/`false`, `yes`/`no`, `y`/`n`, `on`/`off`, `t`/`f`, `1`/`0` |
| **Integer** | 0.95 | Any valid `i64` |
| **Float** | 0.95 (non-integer), 0.70 (also integer) | Any valid `f64` (excludes NaN, Infinity) |
| **Email** | 0.90 | `user@domain.tld` pattern (early return — skips date detection) |
| **Date/Time** | 0.85 (recognized), 0.60 (ambiguous) | 14+ formats via time pack (ISO 8601, US, European, abbreviated month, etc.) |
| **Currency** | 0.90 (symbol), 0.70 (European locale) | 30 codes, 13 symbols, locale-aware formatting |
| **Unit value** | 0.85 | Weight, length, temperature, volume, etc. with category metadata |
| **UUID** | 0.95 | 8-4-4-4-12 hex pattern |
| **URL** | 0.95 | `http://`, `https://`, `ftp://` prefixes |
| **IP address** | 0.90 (v4), 0.80 (v6) | IPv4 dot-quad, IPv6 colon-hex |
| **JSON** | 0.95 | Parseable `{...}` or `[...]` |
| **Identifiers** | varies | IBAN, credit card (Luhn+BIN), ISBN, SSN, EIN, EU VAT, phone |
| **PlainString** | 1.0 | Fallback when nothing else matches |

### Column-Level Detection

For batch analysis of an entire column (e.g., a CSV column), use the time pack's `detect_column_format()`:

```rust
use laminate::packs::time::detect_column_format;

let values = vec!["01/02/2026", "03/04/2026", "12/25/2025"];
let info = detect_column_format(&values);
// info.dominant_format — the most common format in the column
// info.ambiguous_count — how many values couldn't be disambiguated
// info.day_first — statistical inference of DD/MM vs MM/DD ordering
```

### Use Cases

- **Schema inference** — "What types are in this CSV column?"
- **Data profiling** — "What's the breakdown of data types in this field?"
- **Smart coercion** — Route values to the right parser based on detection
- **Data validation** — Flag values that don't match the expected type
- **PII detection** — Find SSNs, credit cards, emails in unknown data
- **Format migration** — Detect current format before converting to canonical form

---

## Streaming & Providers

### Provider Normalization

All providers normalize to a common `NormalizedResponse`:

| Provider | Module | Parsing |
|----------|--------|---------|
| **Anthropic Claude** | `provider::anthropic` | Messages API format |
| **OpenAI ChatGPT** | `provider::openai` | Chat Completions format |
| **Ollama** | `provider::ollama` | OpenAI-compatible format |

```rust
let response = laminate::provider::anthropic::normalize(&raw_json)?;
let response = laminate::provider::openai::normalize(&raw_json)?;
// Both → NormalizedResponse { id, model, content: Vec<ContentBlock>, stop_reason, usage }
```

**ContentBlock variants:** `Text`, `ToolUse` (with FlexValue input), `Unknown` (forward-compatible).

### SSE Streaming

Provider-agnostic Server-Sent Events parser:

```rust
let mut stream = FlexStream::new(StreamConfig { provider: Provider::Anthropic, .. });
stream.feed(chunk);
while let Some(event) = stream.next_event() {
    match event {
        StreamEvent::TextDelta(text) => print!("{}", text),
        StreamEvent::ToolCallDelta { id, name, args_json } => { ... },
        StreamEvent::Done { stop_reason } => break,
        _ => {}
    }
}
```

**Stream events:** TextDelta, BlockStart, ToolCallDelta, InputJsonDelta, BlockStop, Done, Error.

---

## Handler Registry

Dispatch tool calls from LLM responses to typed handler functions:

```rust
let mut registry = HandlerRegistry::new();

// Async handler with auto-deserialized args
registry.register("search", |args: SearchArgs| async move {
    Ok(SearchResult { results: do_search(&args.query).await })
});

// Sync handler
registry.register_sync("add", |args: MathArgs| {
    Ok(MathResult { sum: args.a + args.b })
});

// Dispatch all tool calls from a response
let results = registry.dispatch_all(&response).await?;
```

**Features:** Typed auto-deserialization, raw FlexValue handlers, sync and async handlers, dispatch single or batch, handler lookup (`has`, `names`, `len`).

---

## SQL Data Sources

`laminate-sql` crate with async `DataSource` trait:

```rust
let source = SqliteSource::connect("sqlite:data.db").await?;
let rows = source.query("SELECT * FROM customers").await?;
// rows: Vec<FlexValue> — ready for extraction, schema inference, auditing
```

**Supported backends:** PostgreSQL, SQLite, MySQL (via sqlx, feature-gated).

---

## Diagnostic System

Every coercion, default, drop, and preservation is recorded — never silently swallowed, never fatally rejected unless the mode says so.

### Diagnostic Structure

```rust
struct Diagnostic {
    path: String,           // Where: "data.user.age"
    kind: DiagnosticKind,   // What happened
    risk: RiskLevel,        // How risky: Info / Warning / Risky
    suggestion: Option<String>, // How to tighten
}
```

### Diagnostic Kinds

| Kind | Meaning |
|------|---------|
| `Coerced { from, to }` | Value was type-coerced |
| `Defaulted { field, value }` | Missing field filled with default |
| `Dropped { field }` | Unknown field was dropped (lenient) |
| `Preserved { field }` | Unknown field was preserved (absorbing) |
| `ErrorDefaulted { field, error }` | Coercion failed, fell back to default |
| `Overridden { from_type, to_type }` | Merge replaced a value |

### Risk Levels

| Level | Meaning | Strict Mode Behavior |
|-------|---------|---------------------|
| `Info` | Expected, standard coercion | Allowed |
| `Warning` | Potentially surprising | Becomes error |
| `Risky` | May lose data or change semantics | Becomes error |

### Diagnostic Sinks

Route diagnostics anywhere:

| Sink | Behavior |
|------|----------|
| `Vec<Diagnostic>` | Collect into a vector |
| `CollectSink` | Same, explicit wrapper |
| `StderrSink` | Print to stderr |
| `FilteredSink<S>` | Forward only diagnostics at/above a risk level |
| `NullSink` | Discard all |
| Custom `impl DiagnosticSink` | Route to logs, metrics, databases, UI |

---

## Use Cases by Domain

### API Integration
- Consume REST APIs that evolve without warning (lenient mode)
- Normalize responses from multiple providers (Anthropic/OpenAI/Ollama) to a common type
- Extract typed fields from deeply nested JSON with dot-path notation
- Handle API responses where fields are sometimes strings, sometimes numbers

### Data Engineering / ETL
- Audit a dataset against a schema and get a per-field diagnostic report
- Progressive tightening: start lenient, review diagnostics, tighten incrementally
- Infer schema from data, then audit new data against it
- Parse CSV columns where everything is a string → extract typed values
- Handle locale-specific number formats (US commas, European dots, Swiss apostrophes)

### Configuration Management
- Parse config files with type coercion (env vars, TOML, YAML → typed values)
- Round-trip config files preserving unknown user keys (absorbing mode)
- Merge configuration layers (defaults + file + env + CLI args) with diagnostic trail

### Healthcare
- Convert lab values between US conventional and SI units (18 analytes)
- Parse HL7 v2 datetime formats with fractional seconds and timezones
- Normalize pharmaceutical units (μg, mcg, IU variations)

### Financial / Commerce
- Parse currency amounts with symbols and locale formatting
- Validate identifiers: IBAN, credit card (Luhn + brand), EU VAT
- Handle accounting-negative formats `(1,234.56)`

### Logistics / Supply Chain
- Parse pack-size notation: "1x100-count" → 100 units
- Convert between units with UNECE/X12/DOD standard code support
- Parse qualified weights: "G.W. 15.5kg"

### Geospatial
- Parse coordinates in any format (decimal, DMS, ISO 6709)
- Disambiguate lat/lng vs lng/lat ordering

### LLM Applications
- Parse streaming SSE responses from Anthropic and OpenAI
- Dispatch tool calls to typed handler functions
- Generate JSON Schema from Rust structs for tool definitions
- Normalize responses across providers

### Data Validation & Compliance
- Strict mode provides compile-time proof of completeness
- Every transformation recorded for audit trails
- Validate identifiers (SSN, EIN, NPI, NHS, IBAN, etc.)
- Type detection: "what IS this string?" with ranked candidates

### Testing
- Strict mode for test assertions (exact type matching, no coercion)
- Schema audit as regression testing for data quality
- Merge for building test fixtures from partial data

---

## Feature Flags

| Feature | What It Enables |
|---------|----------------|
| `core` | FlexValue, path access, coercion, modes, diagnostics, packs (default) |
| `derive` | `#[derive(Laminate)]` and `#[derive(ToolDefinition)]` |
| `streaming` | SSE parser, Anthropic/OpenAI stream handlers |
| `providers` | Provider normalization (Anthropic, OpenAI, Ollama) |
| `registry` | Handler dispatch for tool calls |
| `schema` | Schema inference and data auditing |
| `full` | All features |

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `laminate` | Core library |
| `laminate-derive` | Proc macros (Laminate, ToolDefinition) |
| `laminate-sql` | Database connectors (PostgreSQL, SQLite, MySQL) |
| `laminate-cli` | Command-line tool for data auditing |
