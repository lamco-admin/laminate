# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

### Added

#### Core Engine
- `FlexValue` — navigable JSON wrapper with dot/bracket path access, coercion, merge, set
- 4-level coercion pipeline: Exact → SafeWidening → StringCoercion → BestEffort
- 30+ coercion rules including locale-aware number parsing (US, European, Swiss, French, Indian)
- Transparent stringified-JSON navigation (OpenAI tool args navigable in one `extract()` call)
- Mode system with type-level proofs: Lenient, Absorbing, Strict
- `DynamicMode` for runtime mode selection
- Graduated diagnostics: Coerced, Defaulted, Dropped, Preserved, Overridden with risk levels
- `DiagnosticSink` trait with Collect, Stderr, Filtered, Null implementations
- `CoercionDataSource` trait for external data (exchange rates, unit conversion factors)
- `StaticDataSource` for HashMap-backed lookups
- Built-in reference exchange rates for ~30 currency pairs

#### Type Detection
- `guess_type()` API — ranked type candidates with confidence scores
- Detects: Integer, Float, Boolean, Date, Currency, UnitValue, JSON, UUID, Email, URL, IP, IBAN, CreditCard, ISBN, SSN, EIN, VAT, Phone, NullSentinel

#### Domain Packs
- **Time**: 14+ date/time format detection (ISO 8601, US/EU, Unix, GEDCOM, HL7), `convert_to_iso8601()`, optional chrono integration (`to_naive_date()`, `to_naive_datetime()`)
- **Currency**: 30 codes, 13 symbols, locale-aware parsing, symbol stripping
- **Units**: Weight, length, temperature (°C↔°F↔K with formulas), volume, time, data, nautical; UNECE/X12/DOD code recognition; `convert()` function; optional uom integration; pack-size notation parser
- **Identifiers**: IBAN (mod-97), credit card (Luhn + BIN detection), ISBN-10/13, US SSN/EIN, UK NHS, US NPI, EU VAT, UUID, email, phone
- **Geospatial**: Decimal degrees, DMS, ISO 6709, lat/lng ordering, datum detection (WGS84, JGD2011, CGCS2000, PZ-90.11)
- **Medical**: 18 US↔SI lab value conversions (glucose, cholesterol, hemoglobin, creatinine, etc.), pharmaceutical notation normalization, HL7 date parsing

#### Schema Inference
- `InferredSchema::from_values()` — infer field types, cardinality, required/nullable from data samples
- `InferredSchema::audit()` — validate new data against inferred schema
- Configurable thresholds: required_threshold, consistency_threshold, max_enum_cardinality
- External constraints support (expected_type, max_length, min/max_value, allowed_values)

#### Derive Macros
- `#[derive(Laminate)]` with 7 field attributes: coerce, default, overflow, rename, skip, flatten, parse_json_string
- `#[derive(ToolDefinition)]` for JSON schema generation from Rust structs

#### AI Convenience Layers
- Provider adapters: Anthropic, OpenAI, Ollama → unified `NormalizedResponse`
- Streaming SSE parser with tool call fragment assembly and `MessageSnapshot`
- `HandlerRegistry` with typed async/sync tool dispatch

#### SQL Sources
- `laminate-sql` crate with SQLite (tested), PostgreSQL, MySQL via sqlx

#### CLI
- `laminate-cli` crate with `infer`, `audit`, `inspect` subcommands

#### Project Infrastructure
- MIT OR Apache-2.0 dual license
- GitHub Actions CI (test matrix, clippy, fmt, doc build, benchmarks, cargo-deny)
- Criterion benchmarks for core hot paths with serde_json comparison
- 4 example programs (API consumption, CSV ETL, medical data, schema inference)
- 164 integration test files from 201 adversarial iterations
