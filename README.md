# Laminate

**The missing data layer for Rust: progressive type coercion, format detection, and fault-tolerant deserialization, built on serde rather than against it.**

[![Crates.io](https://img.shields.io/crates/v/laminate.svg)](https://crates.io/crates/laminate)
[![Docs.rs](https://docs.rs/laminate/badge.svg)](https://docs.rs/laminate)
[![CI](https://github.com/lamco-admin/laminate/actions/workflows/ci.yml/badge.svg)](https://github.com/lamco-admin/laminate/actions/workflows/ci.yml)
[![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#requirements)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/laminate.svg)](#license)

**[Website](https://lamco.ai/open-source/laminate/) · [Documentation](https://docs.rs/laminate) · [crates.io](https://crates.io/crates/laminate) · [Source](https://github.com/lamco-admin/laminate)**

Laminate bonds layers of structure onto raw data: progressively, configurably, without breaking. Like physical lamination, each layer adds strength, and you can stop at any ply. It handles the messy JSON that real systems produce, from LLM API responses and REST payloads to CSV columns, config files, and SQL rows.

## Quick Start

```toml
[dependencies]
laminate = "0.4"                          # core: FlexValue, coercion, modes, diagnostics, packs
# laminate = { version = "0.4", features = ["derive"] }   # add #[derive(Laminate)]
# laminate = { version = "0.4", features = ["full"] }     # everything
```

```rust
use laminate::FlexValue;

fn main() -> Result<(), laminate::FlexError> {
    // Parse once; extract with automatic, auditable type coercion.
    let cfg = FlexValue::from_json(r#"{"port": "8080", "debug": "true", "workers": 4}"#)?;

    let port: u16 = cfg.extract("port")?;       // "8080" -> 8080
    let debug: bool = cfg.extract("debug")?;     // "true" -> true
    let workers: i32 = cfg.extract("workers")?;  //      4 -> 4
    assert_eq!((port, debug, workers), (8080, true, 4));
    Ok(())
}
```

No per-field annotations, no custom deserializers. The same engine drives everything below.

## Why It Exists

Rust gives you two extremes for external data, and nothing in between:

| Approach | Strength | Weakness |
|---|---|---|
| `#[derive(Deserialize)]` | compile-time type safety | fails on the first unexpected field or type mismatch |
| `serde_json::Value` | accepts anything | no guarantees; everything is `.get()?.as_str()?` |
| **`FlexValue`** (laminate) | navigable, coercible, **auditable** | the middle ground that was missing |

When serde's maintainer closed the long-standing request for fault-tolerant deserialization ([serde#464](https://github.com/serde-rs/serde/issues/464)), the note was that it belonged in *"a different library specifically geared toward fault-tolerant partially successful deserialization."* Laminate is built to be that library. It does not replace serde; it sits in front of it, adding the progressive coercion and graceful degradation serde deliberately leaves out.

## Capabilities at a Glance

A complete map of what laminate does. The deep dives and reference sections below expand each one; the [full feature reference](FEATURES.md) and [docs.rs](https://docs.rs/laminate) go further still.

| Area | What you get |
|---|---|
| **Navigate** | `FlexValue` with dot/bracket paths (`users[0].address.city`), transparent crossing of stringified-JSON boundaries, iteration, introspection, deep `merge`/`set` mutation |
| **Coerce** | 4-level engine (Exact, SafeWidening, StringCoercion, BestEffort), 30+ built-in rules, integer-range validation, locale-aware numbers, opt-in pack coercion, pluggable external data sources |
| **Modes** | Lenient / Absorbing / Strict with **type-level residuals** (`()` / `HashMap` / `Infallible`), plus runtime `DynamicMode` |
| **Diagnose** | every coercion, default, drop, and preservation recorded with a risk level; routable to pluggable sinks (collect, stderr, filtered, null, custom) |
| **Detect** | `guess_type()` ranks 20 types from raw strings with confidence scores; column-level batch detection |
| **Derive** | `#[derive(Laminate)]` (8 field attributes, string-valued enums with an `#[laminate(unknown)]` fallback) and `#[derive(ToolDefinition)]` for LLM tool schemas |
| **AI / LLM** | `from_llm_response` (extract JSON from prose/fenced model output), provider normalization (Anthropic / OpenAI / Ollama), SSE streaming, typed tool-call dispatch |
| **Schema** | infer a schema from sample data, attach external constraints, then audit new data into a per-field violation report |
| **Domain packs** | always-compiled parsers for time, currency, units, identifiers, geospatial, and medical data |
| **Sources & tooling** | `laminate-sql` (PostgreSQL / SQLite / MySQL rows as `FlexValue`) and `laminate-cli` (`infer`, `audit`, `inspect`) |

## Shaping and Coercion

`FlexValue` wraps `serde_json::Value` and adds navigation, coercion, and a diagnostic trail. Coercion has four levels, from strictest to most permissive:

| Level | Behavior | Use for |
|---|---|---|
| `Exact` | no coercion; types must match | building output, validation |
| `SafeWidening` | safe numeric widening, range-checked | round-tripping, proxying |
| `StringCoercion` | parse strings to targets (`"42"` -> `42`, `"true"` -> `true`) | config, env vars |
| `BestEffort` | everything: locale numbers, null-sentinels, stringified JSON, single-element-array unwrap | external APIs, scraping, CSV |

```rust
use laminate::{FlexValue, CoercionLevel};

let val = FlexValue::from_json(r#"{"amount": "1.234,56"}"#)?  // European format
    .with_coercion(CoercionLevel::BestEffort);
let amount: f64 = val.extract("amount")?;                     // 1234.56
```

Currency symbols and unit suffixes are handled by **pack coercion**, which is opt-in (it fires at `StringCoercion` and above):

```rust
use laminate::{FlexValue, value::PackCoercion};

let order = FlexValue::from_json(r#"{"price": "$12.99", "weight": "5.2 kg"}"#)?
    .with_pack_coercion(PackCoercion::All);
let price: f64 = order.extract("price")?;    // 12.99
let weight: f64 = order.extract("weight")?;  // 5.2
```

A `SourceHint` (`Csv`, `Env`, `FormData`, `Database`, `Json`) sets sensible coercion defaults for where the data came from, and a `CoercionDataSource` lets you plug in live exchange rates or conversion factors instead of baking stale data into the crate.

## Modes, Residuals, and Diagnostics

A mode answers one question: what happens to data that does not fit? Laminate encodes the answer in the **type**, so the remainder policy is checked at compile time.

| Mode | Unknown fields | Coercion | Missing fields | Residual type |
|---|---|---|---|---|
| **Lenient** | dropped | BestEffort | defaulted | `()` (zero cost) |
| **Absorbing** | preserved | SafeWidening | error | `HashMap<String, Value>` |
| **Strict** | error | Exact | error | `Infallible` (compile-time proof) |

```rust
// Absorbing mode returns a LaminateResult carrying every unknown field
// in its residual, for round-tripping.
let result = UserProfile::shape_absorbing(&json)?;
for (key, value) in &result.residual {
    println!("preserved unknown field: {key} = {value}");
}

// Strict mode returns the value itself; if this line runs, it is a
// compile-time proof that nothing was dropped or coerced.
let user = UserProfile::shape_strict(&json)?;
```

`shape_lenient` and `shape_absorbing` return a `LaminateResult` that bundles the value, the mode-typed residual, and diagnostics. `shape_strict` returns the value directly (its success is the proof; the `Strict` residual type is the uninhabitable `Infallible`).

Nothing is changed silently. Every transformation is recorded as a `Diagnostic` with a path, a kind (`Coerced`, `Defaulted`, `Dropped`, `Preserved`, `Overridden`, `ErrorDefaulted`), and a risk level (`Info`, `Warning`, `Risky`). In `Strict` mode, `Warning` and `Risky` become errors. Diagnostics route to pluggable sinks (collect, stderr, filtered-by-risk, null, or your own).

```
coerced string -> i64 at 'age' [Info]
defaulted field 'verified' (null -> default) [Warning]
preserved unknown field 'theme' [Info]
```

## Derive: Typed Structs That Tolerate Messy Data

```rust
use laminate::Laminate;                                    // features = ["derive"]
use std::collections::HashMap;

#[derive(Debug, Laminate)]
struct UserProfile {
    name: String,
    #[laminate(coerce)]
    age: i64,                                              // accepts "25", 25, 25.0
    #[laminate(coerce, default)]
    verified: bool,                                        // "yes" -> true; missing -> false
    #[laminate(rename = "e-mail")]
    email: String,                                         // reads the "e-mail" key
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,            // captures every unknown field
}

let (user, diagnostics) = UserProfile::from_json(r#"{
    "name": "Alice", "age": "25", "verified": "yes",
    "e-mail": "alice@example.com", "theme": "dark"
}"#)?;
assert_eq!(user.age, 25);
assert_eq!(user.extra["theme"], "dark");                  // unknown field preserved
```

The seven field attributes (`coerce`, `default`, `overflow`, `rename`, `skip`, `flatten`, `parse_json_string`) compose freely. As of 0.4, the derive also shapes **string-valued enums**, with an optional `#[laminate(unknown)]` newtype variant that captures unrecognized values instead of erroring. A companion `#[derive(ToolDefinition)]` generates an LLM tool definition (`{ name, description, input_schema }`) from a struct and its doc comments.

## Built for AI: LLM Response Handling

Laminate began with the LLM data problem and keeps a first-class, lightweight layer for it. For full agent frameworks (agent loops, RAG), reach for [Rig](https://rig.rs/), [llm](https://github.com/graniet/llm), or [llm-connector](https://crates.io/crates/llm-connector); laminate is the data layer beneath them.

Models wrap JSON in prose and code fences. `from_llm_response` pulls the payload out before parsing, where strict `from_json` would fail:

```rust
// features = ["providers"] (or "full")
let reply = "Sure! Here is the result:\n```json\n{\"city\": \"Paris\", \"temp_c\": 18}\n```";
let value = FlexValue::from_llm_response(reply)?;          // finds and parses the embedded JSON
let city: String = value.extract("city")?;                // "Paris"
```

Beyond extraction, the AI layer normalizes Anthropic, OpenAI, and Ollama responses into one `NormalizedResponse` (`text()`, `tool_uses()`, and a `usage` that now carries OpenAI/Anthropic cache tokens), parses SSE streams with automatic tool-call-fragment assembly (`features = ["streaming"]`), and dispatches tool calls to typed handlers via `HandlerRegistry` (`features = ["registry"]`).

## Schema Inference and Data Auditing

```rust
use laminate::schema::InferredSchema;                     // features = ["schema"]

let schema = InferredSchema::from_values(&training_rows);  // learn types, nullability, fill rates
let report = schema.audit(&new_rows);                      // validate new data against it
println!("{}", report.summary());
// e.g. row 42: age="old" (TypeMismatch); row 99: missing required 'name'
```

Inference derives the dominant type, nullability, required-ness, fill rate, and type consistency per field. You can override with `ExternalConstraint`s (expected type, min/max, max length, allowed values), and the audit reports `TypeMismatch`, `UnexpectedNull`, `MissingField`, `OutOfRange`, `MaxLengthExceeded`, `NotInAllowedValues`, and `UnknownField`.

## Type Detection

`guess_type()` answers "what IS this string?" by running every detector and returning ranked candidates with confidence scores:

```rust
use laminate::detect::{guess_type, GuessedType};

assert_eq!(guess_type("$12.99")[0].kind, GuessedType::Currency);
assert_eq!(guess_type("4111111111111111")[0].kind, GuessedType::CreditCard);  // Luhn + Visa BIN
// "42" -> [Integer, Float, Boolean]; "N/A" -> NullSentinel; "hello" -> PlainString
```

It recognizes 20 types (integer, float, boolean, date, currency, unit value, JSON, UUID, URL, IP, IBAN, credit card, ISBN, SSN, EIN, VAT, phone, email, null-sentinel, plain string) and supports column-level batch detection for profiling a whole CSV column.

## Domain Packs

Six packs, always compiled, no feature flags:

| Pack | Coverage |
|---|---|
| **time** | 18 date/time formats (ISO 8601, US, European, abbreviated month, GEDCOM 7.0, HL7 v2, Unix, ISO week); `convert_to_iso8601`; column-level detection; optional chrono |
| **currency** | 29 codes, 19 symbols, US/European/Swiss/Japanese/Indian locales, accounting negatives `(1,234.56)`, crypto codes, optional built-in exchange rates |
| **units** | 10 categories (weight, length, temperature °C/°F/K, volume, data, speed, pressure, energy, frequency, power); conversions; UNECE/X12/DOD codes; pack-size and qualified-weight notation; optional uom |
| **identifiers** | 12 types with checksums: IBAN (mod-97), credit card (Luhn + brand), ISBN-10/13, SSN, EIN, NPI, NHS (mod-11), EU VAT, UUID, email, phone |
| **geospatial** | decimal degrees, DMS, DDM, ISO 6709, lat/lng order disambiguation, compass suffixes, datum detection |
| **medical** | 36 lab analytes (44 US/SI conversions), clinical calculators (eGFR, BMI, anion gap, creatinine clearance), reference-range classification, pharmaceutical-unit normalization, HL7 v2 and FHIR parsing |

## SQL Sources and CLI

`laminate-sql` returns database rows as `FlexValue`, ready for extraction, schema inference, and auditing:

```rust
let rows = SqliteSource::connect("sqlite:data.db").await?
    .query("SELECT * FROM customers").await?;             // Vec<FlexValue>
```

Backends: PostgreSQL, SQLite, MySQL (via sqlx, feature-gated). The `laminate-cli` tool brings the engine to the shell with `infer` (schema from data), `audit` (validate against a schema), and `inspect` (type detection on values).

## Why Not Just serde (or serde_with, or figment)?

Laminate complements serde; it does not compete with it. serde handles serialization of well-formed data; laminate handles the messy reality just before your typed structs.

| Scenario | serde | laminate |
|---|---|---|
| API returns `"42"` for an integer field | error | coerces to `42` with a diagnostic |
| unknown fields in a response | ignored or error | preserved in the residual for round-tripping |
| missing optional field | needs `#[serde(default)]` per field | mode-level policy |
| upstream schema changed | hard failure | absorb unknown, default missing, report all |
| multiple errors at once | stops at the first | collects every diagnostic |
| mixed types in an array | error | element-level coercion |

Versus **`serde_with`**: no per-field `DeserializeAs` annotations; coercion is a mode-level policy with an audit trail. Versus **`figment`**: figment merges config sources into typed structs; laminate shapes arbitrary runtime-messy data (LLM output, scraped JSON, SQL rows) with element-level coercion and overflow capture, not just config.

## Feature Flags and Crates

| Feature | Enables |
|---|---|
| `core` (default) | `FlexValue`, paths, coercion, modes, diagnostics, the 6 domain packs, type detection, source hints |
| `derive` | `#[derive(Laminate)]` and `#[derive(ToolDefinition)]` |
| `streaming` | SSE parser and stream handlers |
| `providers` | Anthropic / OpenAI / Ollama normalization and `from_llm_response` |
| `registry` | typed handler dispatch for tool calls |
| `schema` | schema inference and data auditing |
| `full` | all of the above |
| `chrono-integration`, `uom-integration` | convert detected dates/units to `chrono` / `uom` types |

| Crate | Purpose |
|---|---|
| `laminate` | core library |
| `laminate-derive` | proc macros (`Laminate`, `ToolDefinition`) |
| `laminate-sql` | database connectors (PostgreSQL, SQLite, MySQL) |
| `laminate-cli` | command-line data auditing and inspection |

## Requirements

Rust 1.85 or newer (edition 2021). Three core dependencies (`serde`, `serde_json`, `thiserror`).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you state otherwise, any contribution you intentionally submit for inclusion, as defined in the Apache-2.0 license, is dual licensed as above, without additional terms.

(c) Lamco Development LLC.

## Contributing

Issues and pull requests are welcome at [github.com/lamco-admin/laminate](https://github.com/lamco-admin/laminate). Please run `cargo test`, `cargo clippy`, and `cargo fmt` before submitting.
