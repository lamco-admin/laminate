# Laminate — Progressive Data Shaping for Rust

> "Be liberal in what you accept, conservative in what you send." — Postel's Law

**Laminate** bonds layers of structure onto raw data — progressively, configurably, without breaking. Like physical lamination, each layer adds strength and rigidity. You can stop at any ply.

### Implementation Status Key

| Badge | Meaning |
|---|---|
| **[IMPLEMENTED]** | Fully built, tested, and available |
| **[PARTIAL]** | Core functionality built, some features pending |
| **[PLANNED]** | Designed but not yet implemented |

## Problem Statement

Real-world data is messy. APIs change schemas without notice. Config files evolve across versions. CSV columns are always strings. Streaming responses arrive in fragments. Webhooks from different providers use different shapes for the same concept.

Rust's type system enforces correctness at compile time — but when consuming external data, this strength becomes friction. Today, Rust developers face a binary choice:

- **Fully typed** (`#[derive(Deserialize)]`): Fails on first unexpected field, missing value, or type mismatch.
- **Fully dynamic** (`serde_json::Value`): Zero compile-time guarantees. Every access is `get()?.as_str()?`.

There is nothing in between.

This library provides a **progressive pipeline** for shaping unstructured data into typed Rust values, letting you stop at whatever level of strictness your use case requires:

```
Raw bytes → Parsed value → FlexValue (navigable, coercible)
    → Shaped struct (partially typed) → Strict struct (fully typed, complete)
```

Every layer is an **on-ramp, not a gate**. Quick prototype? Stay at `FlexValue` with path access. Production system? Go all the way to strict mode with compile-time completeness proofs. The key insight is that **consumption and production require different strictness** — Postel's Law applied to data handling.

---

## Architectural Decisions **[IMPLEMENTED]**

These foundational decisions were made during initial design review and govern all subsequent implementation choices.

### AD-1: Coercion System — Hybrid with Graduated Diagnostics

The coercion system is **fully open**, not a closed set of rules. Three layers:

1. **Built-in table** — standard coercions (string↔number, null→default, stringified JSON, etc.)
2. **User traits** — add new coercions the table doesn't cover (e.g., `"$12.99" → f64`)
3. **User overrides** — replace a built-in coercion with custom behavior

**Resolution order:** user override > user addition > built-in table > fail.

Every coercion emits a **graduated diagnostic** with three components:

| Component | Purpose |
|-----------|---------|
| **What happened** | The coercion that fired |
| **Risk level** | Info / Warning / Risky |
| **Suggestion** | How to tighten if desired |

Mode controls how risk levels are treated:
- **Lenient**: all proceed, diagnostics available on request
- **Absorbing**: warnings recorded, risky coercions flagged
- **Strict**: warnings become errors, risky coercions fail

This mirrors Rust's `#[allow]` / `#[warn]` / `#[deny]` pattern — developers progressively tighten behavior by reviewing diagnostics and acting on suggestions.

### AD-2: Error System — Structured Diagnostics with Pluggable Adapters

`LaminateResult<T, M>` with embedded diagnostics is the **core internal representation**. Every shaping operation produces this, regardless of how the user wants to consume errors.

Output adapters transform the canonical diagnostic into ecosystem-specific formats:

| Adapter | Feature flag | Dependency |
|---------|-------------|------------|
| Default | (always) | None — laminate's own `Diagnostic` type |
| `Into<Result>` | (always) | None — standard `Result<T, Vec<Error>>` |
| `miette` | `miette` | `miette` — pretty terminal diagnostics |
| `eyre` | `eyre` | `eyre` — context chains |
| Custom | (always) | None — user implements `DiagnosticSink` trait |

```rust
/// Route diagnostics anywhere — logs, metrics, databases, UI.
trait DiagnosticSink {
    fn receive(&mut self, diagnostic: &Diagnostic);
}
```

The structured diagnostic is the single source of truth. Adapters are views of it.

### AD-3: Proc Macro Generation — Hybrid (Compile-Time + Runtime)

One `#[derive(Laminate)]` generates:
- **Compile-time mode enforcement** via `impl Laminate<Lenient>`, `impl Laminate<Absorbing>`, `impl Laminate<Strict>` — zero-cost, catches mode misuse at compile time
- **Runtime dispatch wrapper** — for when the mode is determined dynamically (e.g., config file says "run lenient in dev, strict in prod")

Users get compile-time safety when they know the mode upfront, runtime flexibility when they don't.

### AD-4: Row Polymorphism — Yes, via Generated Accessor Traits

`#[derive(Laminate)]` generates accessor traits for each field. A struct with `name: String` automatically implements `HasName`. Functions can be generic over "anything with at least these fields":

```rust
fn greet(user: &impl HasName) {
    println!("Hello, {}", user.name());
}
// Works with FullUser, MiniUser, or any Laminate struct that has a name field.
```

This approximates row polymorphism ("any struct with at least these fields") without runtime overhead.

### AD-5: Runtime vs Type-Level Modes — Hybrid

Resolved by AD-3. Compile-time mode enforcement is the default path. Runtime dispatch is available when needed. Both are generated from a single `#[derive(Laminate)]`.

### AD-6: Serde Interop Boundary — Laminate Wraps Serde

Laminate **complements** serde, never competes with it. Clear division of responsibility:

| Layer | Owner | Responsibility |
|-------|-------|---------------|
| Wire format → `Value` | **serde** | Parse JSON/TOML/YAML bytes into `serde_json::Value` |
| `Value` → flexible typed | **laminate** | Coercion, mode-dependent shaping, diagnostics, overflow |
| Flexible → strict typed | **serde** | Final `Deserialize` on cleaned-up value in strict mode |

Laminate accepts `serde_json::Value` as input, uses serde types throughout, and delegates final strict deserialization to serde's `Deserialize` impls. This is the space serde's maintainer explicitly carved out (issue #464, 2016).

---

## Product Pillars **[IMPLEMENTED]**

Four guiding requirements for every feature and API decision:

1. **Efficient** — fast, low overhead, minimal configuration for common cases. Don't make the user set up 50 options to parse a JSON response.
2. **Reliable** — deterministic. Same input + same configuration = same output, every time. No hidden state, no ambient configuration.
3. **Auditable** — every transformation is recorded in the diagnostic trail. "Why did this field change?" has a clear answer for every single record. Critical for regulated domains (healthcare, finance, government).
4. **Just works** — zero-config for common cases (provider templates, preset modes). Configure only when you need to. Progressive complexity: simple things are simple, complex things are possible.

---

## Domain Coercion Architecture **[IMPLEMENTED]**

### Beyond Type Coercion: Semantic Coercion

The coercion system (AD-1) handles not just type conversion (string → number) but **semantic conversion** — transformations that carry domain meaning. The architecture supports this through **coercion packs**: feature-gated collections of domain-specific rules.

### Built-in Coercion Packs

| Pack | Feature flag | Coercions |
|------|-------------|-----------|
| `core` | (always) | string↔number, string↔bool, stringified JSON, null→default, lossless numeric narrowing |
| `time` | `time` | 12↔24 hour, timezone normalization, multi-format date detection, ISO 8601 parsing |
| `currency` | `currency` | Symbol stripping (`"$12.99"` → f64), locale-aware decimals (`,` vs `.`), currency code extraction |
| `units` | `units` | Imperial↔metric, temperature scales, weight, distance |
| `locale` | `locale` | Number formatting (`1,000.00` vs `1.000,00`), date order (MM/DD vs DD/MM) |
| `supply-chain` | `supply-chain` | Pack-size normalization (1x100-count = 100 units), UOM conversion |

### External Data Source Trait

Some coercions need live data — exchange rates, conversion factors, locale rules. Laminate provides the **mechanism** (parsing + plumbing), not the **data**:

```rust
/// Bring external knowledge into the coercion pipeline.
/// Laminate owns the parsing and plumbing; the user owns the data.
trait CoercionDataSource: Send + Sync {
    /// Currency exchange rate lookup.
    fn exchange_rate(&self, from: &str, to: &str) -> Option<f64>;

    /// Unit conversion factor lookup.
    fn conversion_factor(&self, from: &str, to: &str) -> Option<f64>;

    /// Custom domain-specific lookup.
    fn lookup(&self, domain: &str, key: &str) -> Option<FlexValue>;
}
```

This keeps the library lightweight — no stale data baked in, no runtime API calls inside what's supposed to be a parsing library.

### Graduated Diagnostics for Domain Coercion

Every domain coercion emits risk-aware diagnostics:

> "Coerced '1x100-count' → 100 units — **Warning**: pack-size decomposition assumes unit equivalence. **Suggestion**: verify UOM matches target field."

> "Coerced '$12.99' → 12.99 f64 — **Info**: currency symbol stripped. **Suggestion**: use Currency type to preserve symbol."

> "Coerced '01/02/2026' → 2026-01-02 — **Risky**: ambiguous date format (could be Feb 1 or Jan 2). **Suggestion**: specify format with `coerce::date::format("MM/DD/YYYY")`."

### Difficult Domains Where Laminate Can Provide Value

Real-world domains where data conversion is notoriously painful and current tooling is expensive, proprietary, or rigid:

| Domain | Why it's hard | Opportunity |
|--------|-------------|-------------|
| **Healthcare (HL7/FHIR)** | Dozens of message versions, every hospital customizes fields | Enterprise Java tooling from 2005. Open-source progressive shaping doesn't exist. |
| **Financial (FIX, SWIFT)** | Every counterparty interprets formats differently, numeric precision is critical | `"100.10"` vs `100.1` vs `10010` (cents) — wrong coercion loses real money |
| **Geospatial** | lat/lng vs lng/lat ordering, dozens of coordinate reference systems | Every mapping project hits coordinate ordering bugs |
| **EDI (X12, EDIFACT)** | 1970s supply chain standard, positional fields, per-partner extensions | Moves trillions annually, tooling is ancient |
| **Genealogical (GEDCOM)** | Dates like `ABT 1850`, `BET 1840 AND 1860`, free-text in date fields | Every genealogy program handles these differently, silently corrupts data |
| **Government/regulatory** | Every country has different tax, ID, and address formats | Address parsing alone is a multi-billion dollar industry |
| **Scientific (NetCDF, HDF5)** | Mixed units, missing value sentinels (-9999 = null), metadata in data | Researchers spend more time cleaning data than analyzing it |

Laminate doesn't need to ship domain packs for all of these at launch. The open coercion system (AD-1) and `CoercionDataSource` trait let domain experts build adapters. Ship the core, ship 1-2 example packs, let the community build the rest.

---

## Data Regularization Workflow **[PARTIAL]**

### The Discover → Tighten → Run Cycle

Laminate's unique value for data cleanup and conversion is the **progressive tightening** workflow:

```rust
// Step 1: DISCOVER — what's actually in this data?
let report = laminate::audit(dirty_data, MySchema::lenient());
// "487 records parsed. 12 string-to-int coercions. 3 unknown fields.
//  2 null-to-default. 1 risky: field 'date' has 4 different formats."

// Step 2: TIGHTEN — address the risky items
let config = MySchema::lenient()
    .field("date", coerce::date::multi_format(["YYYY-MM-DD", "MM/DD/YYYY", "D MMM YYYY"]));

// Step 3: RUN — with full audit trail
let results = laminate::convert(dirty_data, clean_schema, config);
for record in &results {
    // record.value       — the clean data
    // record.diagnostics — exactly what was transformed and why
}
```

### How This Differs From Existing ETL Tools

| Existing tools | Laminate |
|---|---|
| Write transformation rules upfront, hope they're right | Start lenient, let laminate show you what's messy |
| Silent failures — bad data passes through or gets dropped | Every coercion, drop, and default is recorded |
| One-shot — run it and pray | Progressive — tighten incrementally based on diagnostics |
| Rules are code, buried in scripts | Rules are configuration (mode + axis overrides) |
| No audit trail | Full diagnostic log: what changed, why, risk level, suggestion |

### Bidirectional Auto-Shaping (Provider Templates)

For common data formats, laminate ships with built-in knowledge of the data shape — zero-config parsing and translation:

```rust
// Zero config — laminate knows these formats
let response = laminate::anthropic::parse(raw_json)?;
let response = laminate::openai::parse(raw_json)?;

// Both give you the same NormalizedResponse type

// Bidirectional translation
let input = laminate::openai::parse(raw)?;
let output = laminate::anthropic::emit(&input)?;
```

Provider templates are the "just works" entry point. Users can start with zero configuration and tighten later if needed.

---

## Schema-Driven Data Audit **[IMPLEMENTED]**

### The Problem

You have canonical field definitions — the source of truth for what data *should* look like. You want to know where reality deviates. Most audit tools give you pass/fail. Laminate gives you a full diagnostic report with actionable intelligence.

### External Schema Definitions

Schemas can be defined outside of Rust code, in JSON/TOML/YAML. This means data audits don't require writing Rust structs — you define your field definitions in a config file:

```json
{
  "fields": {
    "customer_id": { "type": "integer", "required": true },
    "email": { "type": "string", "required": true, "format": "email" },
    "zip_code": { "type": "string", "pattern": "^\\d{5}(-\\d{4})?$" },
    "created_at": { "type": "datetime", "formats": ["ISO8601", "MM/DD/YYYY"] },
    "status": { "type": "enum", "values": ["active", "inactive", "pending"] }
  }
}
```

### Audit API

```rust
// Load field definitions from file
let schema = laminate::Schema::from_file("field_definitions.json")?;

// Point at any data source (database, CSV, API, JSON Lines)
let report = laminate::audit(data_source, schema, Mode::Lenient)?;
```

### The Audit Report

The report is the product — not just pass/fail, but a per-field diagnostic with counts, categories, and suggestions:

| Field | Records | Clean | Coercible | Invalid | Suggestion |
|---|---|---|---|---|---|
| customer_id | 42,318 | 41,427 (97.9%) | 891 (string "123") | 0 | Tighten source to integer |
| email | 42,318 | 42,306 | 0 | 12 (null) | Add required constraint or default |
| zip_code | 42,318 | 39,100 | 2,918 (int 12345) | 300 (free text) | 300 need manual review |
| created_at | 42,318 | 40,000 | 2,200 (3 formats) | 118 (unparseable) | Standardize to ISO8601 |

This table is what you hand to a DBA or data steward: "here's exactly what's wrong and here's what we can fix automatically." The audit report IS the deliverable.

### Universal Data Source Trait

```rust
/// Any data source that can produce rows as FlexValue.
trait DataSource {
    fn rows(&self) -> impl Iterator<Item = FlexValue>;
}
```

Implementations for common sources (feature-gated):

| Source | Feature flag | Dependency |
|--------|-------------|------------|
| PostgreSQL | `postgres` | `sqlx` |
| MySQL | `mysql` | `sqlx` |
| SQLite | `sqlite` | `sqlx` |
| CSV | `csv` | `csv` |
| JSON Lines | (core) | None |
| API paginator | (core) | None |

### Companion Crates

| Crate | Purpose |
|-------|---------|
| `laminate` | Core library — FlexValue, modes, coercion, diagnostics, derive macro |
| `laminate-sql` | Database connectors — DataSource impls for PostgreSQL, MySQL, SQLite |
| `laminate-cli` | Command-line tool: `laminate audit --db postgres://... --schema schema.json` |

The CLI tool opens laminate to non-Rust users. DBAs, data stewards, and analysts can audit data without writing code.

---

## Core Design: Five Orthogonal Axes **[IMPLEMENTED]**

Cross-language research (Zod, Pydantic, Ecto, AJV, Marshmallow) reveals that all data shaping can be decomposed into five independent axes. No existing library in any language exposes all five as composable configuration:

| Axis | Options | Default (Lenient) | Default (Strict) |
|------|---------|-------------------|-------------------|
| **Unknown Fields** | Error / Drop / Keep | Keep | Error |
| **Type Coercion** | Exact / Safe-widening / String-coercion / Best-effort | Best-effort | Exact |
| **Missing Fields** | Error / Default / Null / Skip | Default | Error |
| **Error Strategy** | Fail-fast / Collect-all / Best-effort | Best-effort | Fail-fast |
| **Transform Timing** | Pre-parse / During / Post-parse / Deferred | During | During |

**Preset modes** configure all five axes at once. Per-axis overrides allow fine-tuning:

```rust
// Preset: one call sets all axes
let shaped = FlexValue::from_str(json)?.shape::<MyStruct>(Mode::Lenient)?;

// Override: lenient but fail on unknown fields
let shaped = FlexValue::from_str(json)?
    .shape::<MyStruct>(Mode::Lenient.with_unknowns(UnknownFields::Error))?;
```

---

## Operational Modes **[IMPLEMENTED]**

### The Consumption / Production Duality

The library recognizes that reading data and writing data have fundamentally different requirements:

- **Consumption** (reading external data) is **contravariant**: accept broader, more general input. "I'll take anything with at least these fields."
- **Production** (writing/constructing output) is **covariant**: produce narrower, more specific output. "I will emit exactly this shape."

This is not a design choice — it's a mathematical consequence of type variance. It's the same structure as serde's `Deserialize` vs `Serialize`, and Diesel's `Queryable` vs `Insertable`.

### Mode Taxonomy

Three binary dimensions yield eight operational modes:

| Schema Open? | Data Open? | Total? | Mode | Intent |
|:---:|:---:|:---:|---|---|
| Yes | Yes | No | **Extract** | Pull specific fields, ignore the rest |
| Yes | Yes | Yes | **Absorb** | Take everything, preserve unknowns for round-tripping |
| No | Yes | Yes | **Validate** | Check data against a shape, report all violations |
| No | No | Yes | **Construct** | Build a value — all required fields must be present |
| Yes | No | Yes | **Synthesize** | Build with defaults filling in for unknowns |
| Mixed | Mixed | Yes | **Migrate** | Transform between versions, handle shape differences |
| Yes | Yes | No | **Merge** | Overlay two partial values into one complete value |
| No | No | No | **Diff** | Compare two values of the same shape |

For most users, three preset modes cover 90% of cases:

| Preset | Unknown Fields | Coercion | Missing Fields | Errors | Use Case |
|--------|:---:|:---:|:---:|:---:|---|
| **`Lenient`** | Keep | Best-effort | Default | Collect | Consuming external APIs, scraping, logs |
| **`Absorbing`** | Keep (typed overflow) | Safe-widening | Error | Collect | Round-tripping, protocol proxying, config editing |
| **`Strict`** | Error | Exact | Error | Fail-fast | Constructing output, validation, test assertions |

### Residuals as First-Class Types

What's "left over" after shaping is a typed value whose type varies by mode:

| Mode | Residual Type | Meaning |
|------|:---:|---|
| Lenient | `()` | Discarded by design — nothing preserved |
| Absorbing | `Overflow` | Unknown fields preserved in a map |
| Strict | `Infallible` | Compile-time proof: nothing was left over |
| Validate | `Vec<Violation>` | The errors ARE the output |

```rust
// Lenient: residual is () — zero-cost, nothing to inspect
let result: LaminateResult<Config, Lenient> = val.shape()?;
assert_eq!(result.residual, ());

// Absorbing: residual carries unknown fields
let result: LaminateResult<Config, Absorbing> = val.shape()?;
for (key, val) in &result.residual {
    println!("Unknown field: {key} = {val}");
}

// Strict: residual is Infallible — can't construct one
let result: LaminateResult<Config, Strict> = val.shape()?;
// result.residual is never accessible — proves completeness at type level
```

---

## Crate Structure **[IMPLEMENTED]**

Single crate with feature flags:

```toml
[package]
name = "laminate"
description = "Data, shaped layer by layer"

[features]
default = ["core"]
core = []               # FlexValue, path access, coercion, preset modes
derive = ["core"]       # #[derive(Laminate)] macro for lenient/strict/absorbing structs
streaming = ["core"]    # SSE/streaming parser, delta accumulator
providers = ["core"]    # Normalization adapters (Anthropic, OpenAI, Ollama, custom)
registry = ["core"]     # Tool/handler dispatch (for agentic patterns)
full = ["core", "derive", "streaming", "providers", "registry"]
```

### Dependencies

- `serde` + `serde_json` (required — this library complements serde)
- `thiserror` (required, error types)
- `syn` + `quote` + `proc-macro2` (for derive macros, in `laminate-derive` proc-macro subcrate)
- `tokio` (optional, streaming feature)
- `futures` (optional, streaming feature)

---

## Layer 0: FlexValue — Flexible Spine **[IMPLEMENTED]**

### Purpose

A wrapper around `serde_json::Value` providing ergonomic path-based access with type coercion at extraction points. This bridges the gap between dynamic access and typed extraction that no existing Rust crate provides.

### API Design

```rust
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Wraps a serde_json::Value with ergonomic path-based access and coercion.
#[derive(Debug, Clone)]
pub struct FlexValue {
    inner: Value,
}

/// Errors during navigation and extraction.
#[derive(Debug, thiserror::Error)]
pub enum FlexError {
    #[error("path not found: {path}")]
    PathNotFound { path: String },

    #[error("type mismatch at '{path}': expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("index {index} out of bounds (len {len}) at '{path}'")]
    IndexOutOfBounds {
        path: String,
        index: usize,
        len: usize,
    },

    #[error("deserialization failed at '{path}': {source}")]
    DeserializeError {
        path: String,
        source: serde_json::Error,
    },

    #[error("invalid path syntax: {detail}")]
    InvalidPath { detail: String },

    #[error("shaping produced {count} diagnostic(s)")]
    ShapingDiagnostics {
        count: usize,
        diagnostics: Vec<Diagnostic>,
    },
}

pub type Result<T> = std::result::Result<T, FlexError>;

impl FlexValue {
    /// Construct from any serde_json::Value.
    pub fn new(value: Value) -> Self;

    /// Parse from a JSON string.
    pub fn from_str(json: &str) -> Result<Self>;

    /// Navigate to a nested value using dot/bracket path syntax.
    ///
    /// Path syntax:
    ///   - `"foo"` — object key
    ///   - `"foo.bar"` — nested object key
    ///   - `"foo[0]"` — array index
    ///   - `"foo[0].bar.baz[2]"` — mixed
    ///   - `"meta[\"content-type\"]"` — quoted keys for special chars
    ///
    /// Returns FlexError::PathNotFound if any segment is missing.
    pub fn at(&self, path: &str) -> Result<FlexValue>;

    /// Navigate to a path and deserialize with coercion into a concrete type.
    ///
    /// Applies coercion rules (see Coercion Table below).
    pub fn extract<T: DeserializeOwned>(&self, path: &str) -> Result<T>;

    /// Like extract, but returns None for missing paths.
    /// Still returns Err for paths that exist but fail to deserialize.
    pub fn maybe<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>>;

    /// Extract from the root value itself (no path navigation).
    pub fn extract_root<T: DeserializeOwned>(&self) -> Result<T>;

    /// Iterate over an array at the given path, yielding FlexValue items.
    /// Returns an empty iterator if the path is missing or not an array.
    pub fn each(&self, path: &str) -> impl Iterator<Item = FlexValue>;

    /// Returns true if a value exists at the given path (even if null).
    pub fn has(&self, path: &str) -> bool;

    /// Shape this value into a typed struct using a mode preset.
    pub fn shape<T: Laminate, M: Mode>(&self) -> Result<LaminateResult<T, M>>;

    /// Shape with per-axis overrides.
    pub fn shape_with<T: Laminate, M: Mode>(
        &self,
        config: LaminateConfig,
    ) -> Result<LaminateResult<T, M>>;

    /// Escape hatch: reference to the underlying serde_json::Value.
    pub fn raw(&self) -> &Value;

    /// Consume and return the underlying Value.
    pub fn into_raw(self) -> Value;

    // Type checks on current root
    pub fn is_null(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_object(&self) -> bool;

    /// Get all keys if this is an object.
    pub fn keys(&self) -> Option<impl Iterator<Item = &str>>;

    /// Get array length if this is an array.
    pub fn len(&self) -> Option<usize>;
}

impl From<Value> for FlexValue { ... }
impl From<&str> for FlexValue { ... }  // parses JSON string
impl std::fmt::Display for FlexValue { ... }  // pretty-print JSON
```

### Coercion Table

LLM APIs, config files, CSV data, and env vars are all "sloppy" about types. The `extract` method applies these coercions **when the target type doesn't match the JSON type**:

| JSON type | Target type | Coercion | Example |
|-----------|-------------|----------|---------|
| String `"3"` | any numeric | parse the string | `"42"` → `42i64` |
| String `"true"`/`"false"` | bool | parse | `"true"` → `true` |
| String (valid JSON) | struct/map/array | parse as JSON, then deserialize | `"{\"a\":1}"` → `MyStruct { a: 1 }` |
| Number `3` | String | format as string | `42` → `"42"` |
| Null | any Default type | `Default::default()` | `null` → `0i64` |
| Array with one element | scalar T | extract the single element | `[42]` → `42` |
| Number `3.0` | integer | truncate if lossless | `3.0` → `3i64` |

The "string containing JSON" coercion is critical — OpenAI's function calling returns `arguments` as a stringified JSON string. Config systems pass complex values through env vars as strings. This must be handled transparently.

Coercion is **configurable**. The default table covers common cases. Users can extend or restrict:

```rust
let config = LaminateConfig::new()
    .coercion(Coercion::Exact)           // no coercion
    .coercion(Coercion::SafeWidening)    // int→float, not float→int
    .coercion(Coercion::StringCoercion)  // parse strings to target types
    .coercion(Coercion::BestEffort);     // try everything (default)
```

### Path Parser Implementation Notes

Implement as a small state machine splitting the path into `enum Segment { Key(String), Index(usize) }` and walking the `Value` tree:
- Dot-separated keys: `"choices.message.content"`
- Bracket indices: `"choices[0]"`
- Mixed: `"choices[0].message.tool_calls[0].function.name"`
- Quoted keys: `"meta[\"content-type\"]"`

### Usage Examples

```rust
let resp = FlexValue::from_str(raw_json)?;

// === API Client: Consuming a third-party REST API ===
let user_id: u64 = resp.extract("data.user.id")?;          // works if "123" or 123
let email: String = resp.extract("data.user.email")?;
let tags: Vec<String> = resp.maybe("data.user.tags")?.unwrap_or_default();

// === Config: Layered configuration (file + env) ===
let port: u16 = resp.extract("server.port")?;               // works if "8080" or 8080
let debug: bool = resp.extract("server.debug")?;             // works if "true" or true
let workers: usize = resp.maybe("server.workers")?.unwrap_or(4);

// === LLM: Tool calls with stringified arguments ===
for tc in resp.each("choices[0].message.tool_calls") {
    let name: String = tc.extract("function.name")?;
    let city: String = tc.extract("function.arguments.city")?;  // parses stringified JSON
}

// === ETL: Best-effort row extraction ===
let amount: f64 = resp.extract("amount")?;                   // handles "$12.99" → 12.99? (custom coercion)
```

---

## Layer 1: Laminate Derive Macro **[IMPLEMENTED]**

### Purpose

Structs that **absorb the unknown** rather than rejecting it. A single `#[derive(Laminate)]` generates implementations for all modes — lenient, absorbing, and strict.

### Derive Macro: `Laminate`

Lives in the `laminate-derive` proc-macro crate. Generates custom deserialization logic that operates on an intermediate `HashMap<String, Value>`.

```rust
use laminate::Laminate;

#[derive(Debug, Laminate)]
pub struct ApiResponse {
    /// Normal required field.
    pub id: String,

    /// Rename support (like serde).
    #[laminate(rename = "type")]
    pub response_type: String,

    /// Nested shaped struct.
    pub data: UserData,

    /// Captures ALL unrecognized fields from this level.
    /// Must be HashMap<String, serde_json::Value>.
    #[laminate(overflow)]
    pub _extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Laminate)]
pub struct UserData {
    pub name: String,

    /// Coerce strings to numbers automatically.
    #[laminate(coerce)]
    pub age: u32,

    /// Default if missing (doesn't require Option).
    #[laminate(default)]
    pub verified: bool,

    /// Auto-detect and parse stringified JSON.
    #[laminate(parse_json_string)]
    pub metadata: FlexValue,

    #[laminate(overflow)]
    pub _extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Laminate)]
pub struct Config {
    /// All env vars are strings — coerce to target types.
    #[laminate(coerce)]
    pub port: u16,

    #[laminate(coerce)]
    pub workers: usize,

    #[laminate(coerce)]
    pub debug: bool,

    /// Version-resilient: new config fields won't break old code.
    #[laminate(default)]
    pub new_feature_flag: bool,

    #[laminate(overflow)]
    pub _extra: HashMap<String, serde_json::Value>,
}
```

### Attribute Behaviors

| Attribute | Behavior |
|-----------|----------|
| `#[laminate(overflow)]` | Captures unrecognized fields into `HashMap<String, Value>`. One per struct. |
| `#[laminate(rename = "x")]` | Deserialize from a different JSON key. |
| `#[laminate(parse_json_string)]` | If the value is a string, try parsing it as JSON first. |
| `#[laminate(coerce)]` | Apply coercion rules from the coercion table. |
| `#[laminate(default)]` | Use `Default::default()` if missing or null. Not just `Option` — any `Default` type. |
| `#[laminate(flatten)]` | Merge fields from a nested object (like serde flatten, without the bugs). |
| `#[laminate(skip)]` | Don't attempt to deserialize this field from input. |

### Key Behaviors

1. **Never fail on unknown fields** — unrecognized keys go to `_extra` (if present) or are silently dropped (in lenient mode) or cause errors (in strict mode). The MODE controls behavior, not the struct definition.
2. **Coercion on marked fields** — string-to-number, number-to-string, stringified JSON, null-to-default.
3. **Composable with serde** — `Laminate` structs can contain regular serde structs and vice versa. The boundary is clean.
4. **Mode-dependent behavior** — the same struct definition behaves differently in lenient vs strict mode. You laminate the same data at different plies.

### Diagnostics

Every shaping operation can produce diagnostics — a record of what happened:

```rust
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub path: String,
    pub kind: DiagnosticKind,
}

#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    /// A field was coerced from one type to another.
    Coerced { from: String, to: String },
    /// A field was missing and defaulted.
    Defaulted { field: String, value: String },
    /// An unknown field was dropped.
    Dropped { field: String },
    /// An unknown field was preserved in overflow.
    Preserved { field: String },
    /// A field failed to deserialize but was defaulted (lenient mode).
    ErrorDefaulted { field: String, error: String },
}
```

```rust
let result = val.shape::<Config, Lenient>()?;
for d in &result.diagnostics {
    match &d.kind {
        DiagnosticKind::Coerced { from, to } =>
            log::debug!("Coerced {}: {} → {}", d.path, from, to),
        DiagnosticKind::Dropped { field } =>
            log::debug!("Dropped unknown field: {}", field),
        _ => {}
    }
}
```

---

## Layer 2: Streaming Parser **[IMPLEMENTED]**

### Purpose

Handle SSE-based streaming responses, accumulating partial deltas into complete typed values. While designed with LLM streaming in mind, the SSE parser is general-purpose.

### API Design

```rust
pub struct StreamConfig {
    pub provider: Provider,
    pub max_buffer_bytes: usize,  // default 1MB
}

pub struct FlexStream {
    config: StreamConfig,
}

/// Events emitted by the stream parser.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Incremental text content.
    TextDelta(String),

    /// A structured block has started (we know the type/name but content is streaming).
    BlockStart {
        index: usize,
        id: String,
        block_type: String,
        name: Option<String>,
    },

    /// A fragment of a block's content.
    BlockDelta {
        index: usize,
        id: String,
        fragment: String,
    },

    /// A block is fully accumulated and parseable as FlexValue.
    BlockComplete {
        index: usize,
        id: String,
        block_type: String,
        name: Option<String>,
        content: FlexValue,
    },

    /// Usage / metadata information.
    Metadata(FlexValue),

    /// Stop/end signal.
    Stop(StopReason),

    /// Unrecognized event — forward-compatible.
    Unknown {
        event_type: String,
        data: FlexValue,
    },
}

#[derive(Debug, Clone)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    StopSequence,
    Unknown(String),
}

impl FlexStream {
    pub fn new(config: StreamConfig) -> Self;

    /// Feed raw SSE bytes into the parser. Returns zero or more events.
    pub fn feed(&mut self, chunk: &[u8]) -> Result<Vec<StreamEvent>>;

    /// Feed a single data line (the JSON after "data: ").
    pub fn feed_data(&mut self, data: &str) -> Result<Vec<StreamEvent>>;

    /// Signal end-of-stream. Flushes remaining content.
    pub fn finish(self) -> Result<Vec<StreamEvent>>;

    /// Get the fully accumulated response so far as a FlexValue.
    pub fn accumulated(&self) -> FlexValue;
}
```

### SSE Parser Notes

The SSE parser handles:
- `data: {...}\n\n` — standard events
- `data: [DONE]\n\n` — OpenAI end-of-stream
- Anthropic event types: `message_start`, `content_block_delta`, etc.
- Multi-line data fields (rare but spec-legal)
- Heartbeat comments (`: heartbeat\n`)

### Provider-Specific Delta Handling

**Anthropic streaming:**
- `message_start` → envelope with model, usage
- `content_block_start` → signals text or tool_use block
- `content_block_delta` → `text_delta` or `input_json_delta`
- `content_block_stop` → block complete
- `message_delta` → stop_reason, final usage
- `message_stop` → end

**OpenAI streaming:**
- Each chunk has `choices[0].delta` with partial content/tool_calls
- Tool call arguments accumulate by `index`
- Final chunk has `finish_reason`

Both map to the same `StreamEvent` types via `Provider` dispatch.

### Usage Example

```rust
use tokio_stream::StreamExt;

let mut stream = FlexStream::new(StreamConfig {
    provider: Provider::Anthropic,
    ..Default::default()
});

while let Some(chunk) = sse_stream.next().await {
    for event in stream.feed(&chunk?)? {
        match event {
            StreamEvent::TextDelta(s) => print!("{s}"),
            StreamEvent::BlockStart { name, .. } => {
                if let Some(name) = name {
                    println!("\n[calling: {name}]");
                }
            }
            StreamEvent::BlockComplete { name, content, .. } => {
                let args: MyToolArgs = content.extract_root()?;
                execute_tool(&name.unwrap(), args).await?;
            }
            StreamEvent::Stop(reason) => println!("\n[done: {reason:?}]"),
            _ => {}
        }
    }
}
```

---

## Layer 3: Provider Normalization **[IMPLEMENTED]**

### Purpose

Map provider-specific response shapes to a common envelope. Downstream code doesn't care which API it's talking to.

### Normalized Types

```rust
#[derive(Debug, Clone)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Ollama,
    Custom(String),
}

/// Normalized response envelope.
#[derive(Debug, Clone)]
pub struct NormalizedResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: Usage,
    /// The original raw response, always accessible.
    pub raw: FlexValue,
}

/// Normalized content block.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: FlexValue,
    },
    /// Forward-compatible: new block types don't break existing code.
    Unknown {
        block_type: String,
        data: FlexValue,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    /// Provider-specific extra fields.
    pub extra: HashMap<String, serde_json::Value>,
}

/// Implement this trait to add new providers.
pub trait ProviderAdapter: Send + Sync {
    fn parse_response(&self, body: &FlexValue) -> Result<NormalizedResponse>;
    fn stream_parser(&self) -> FlexStream;
}
```

### Normalization Examples

**Anthropic response:**
```json
{
  "id": "msg_xxx",
  "content": [
    {"type": "text", "text": "Hello"},
    {"type": "tool_use", "id": "tu_1", "name": "search", "input": {"q": "rust"}}
  ],
  "stop_reason": "tool_use",
  "usage": {"input_tokens": 50, "output_tokens": 30}
}
```

**OpenAI response:**
```json
{
  "id": "chatcmpl-xxx",
  "choices": [{
    "message": {
      "content": "Hello",
      "tool_calls": [
        {"id": "tc_1", "type": "function", "function": {"name": "search", "arguments": "{\"q\":\"rust\"}"}}
      ]
    },
    "finish_reason": "tool_calls"
  }],
  "usage": {"prompt_tokens": 50, "completion_tokens": 30}
}
```

Both normalize to the same `NormalizedResponse`. Note that OpenAI's stringified `arguments` is automatically parsed via FlexValue coercion.

---

## Layer 4: Handler Registry **[IMPLEMENTED]**

### Purpose

Dynamic dispatch of structured blocks (tool calls, function calls, etc.) to typed async handler functions with automatic argument deserialization.

```rust
pub struct HandlerRegistry {
    handlers: HashMap<String, Box<dyn Handler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self;

    /// Register a typed handler. Arguments are auto-deserialized from FlexValue.
    pub fn register<A, R, F, Fut>(&mut self, name: &str, handler: F)
    where
        A: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        F: Fn(A) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R>> + Send;

    /// Register a handler that receives raw FlexValue.
    pub fn register_raw<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(FlexValue) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send;

    /// Dispatch a content block from a NormalizedResponse.
    pub async fn dispatch(&self, block: &ContentBlock) -> Result<Value>;

    /// Dispatch all tool-use blocks, returning results in order.
    pub async fn dispatch_all(&self, response: &NormalizedResponse) -> Result<Vec<HandlerResult>>;

    pub fn has(&self, name: &str) -> bool;
    pub fn names(&self) -> Vec<&str>;
}

#[derive(Debug)]
pub struct HandlerResult {
    pub block_id: String,
    pub name: String,
    pub result: Value,
}
```

### Usage Example

```rust
let mut registry = HandlerRegistry::new();

registry.register("get_weather", |args: WeatherArgs| async move {
    let weather = fetch_weather(&args.city).await?;
    Ok(WeatherResult { temp: weather.temp, conditions: weather.desc })
});

// Dispatch from normalized response
let response: NormalizedResponse = provider.parse_response(&raw)?;
let results = registry.dispatch_all(&response).await?;
```

---

## Implementation Order

Build and test in this order. Each layer is independently useful and publishable.

### Phase 1: Core (`FlexValue`)
1. Path parser (`Vec<Segment>` from path string)
2. `FlexValue` struct with `at()`, `raw()`, `into_raw()`
3. `extract::<T>()` with serde deserialization (no coercion yet)
4. `maybe::<T>()` and `each()`
5. Coercion rules (string-to-number, stringified JSON, null-to-default)
6. `LaminateConfig` and preset modes (Lenient, Absorbing, Strict)
7. `Diagnostic` type and collection
8. Tests against real JSON fixtures: API responses (REST, LLM), config files, CSV-derived JSON

### Phase 2: Laminate Derive Macro
1. Basic `Laminate` proc macro — generates deserialization via intermediate HashMap
2. `#[laminate(overflow)]` support
3. `#[laminate(rename)]` and `#[laminate(default)]`
4. `#[laminate(coerce)]` and `#[laminate(parse_json_string)]`
5. Mode-dependent behavior (lenient vs strict from same struct definition)
6. Diagnostic emission from derive-generated code
7. Tests with real payloads containing unknown fields, type mismatches, missing data

### Phase 3: Streaming
1. SSE line parser (bytes → event frames)
2. Anthropic delta handler (event types → StreamEvent)
3. OpenAI delta handler (choices delta → StreamEvent)
4. Block content accumulator
5. Integration tests with recorded SSE streams

### Phase 4: Provider Normalization
1. `NormalizedResponse` and `ContentBlock` types
2. Anthropic adapter
3. OpenAI adapter
4. Ollama adapter
5. `ProviderAdapter` trait for custom providers

### Phase 5: Handler Registry
1. `HandlerRegistry` with typed registration
2. Dispatch with auto-deserialization via FlexValue
3. Raw dispatch fallback
4. `dispatch_all` for batch handling

### Future: Extended Capabilities

**Implemented:**
- ~~**Schema Inference**~~ — `InferredSchema::from_values()` with configurable thresholds and external constraints ✓
- ~~**Custom Coercion**~~ — `Coercible` trait + `CoercionDataSource` trait ✓
- ~~**Domain Coercion Packs**~~ — time, currency, units packs in `laminate::packs` ✓
- ~~**CoercionDataSource Trait**~~ — `StaticDataSource`, `NoDataSource` ✓
- ~~**Data Audit Mode**~~ — `InferredSchema::audit()` with full violation reporting ✓

**Planned:**
- **Merge/Overlay** — combine partial values for config layering
- **Round-Trip Preservation** — absorb unknowns on read, emit on write
- **Multiple Type Families from One Definition** — Reader/Builder/Partial/Delta types
- **Validation Integration** — compose with `validator` or custom validation
- **Bidirectional Provider Templates** — parse and emit common formats with zero configuration

---

## Test Fixtures

Real (anonymized) data in `testdata/` organized by use case, not just provider:

```
testdata/
├── api-responses/
│   ├── anthropic/
│   │   ├── text_response.json
│   │   ├── tool_use_response.json
│   │   ├── streaming_text.sse
│   │   └── streaming_tool_use.sse
│   ├── openai/
│   │   ├── text_response.json
│   │   ├── tool_calls_response.json    # stringified arguments
│   │   ├── streaming_text.sse
│   │   └── streaming_tool_calls.sse
│   ├── ollama/
│   │   └── text_response.json
│   └── rest/
│       ├── github_api.json             # schema changes between versions
│       ├── stripe_webhook.json         # deeply nested, optional fields
│       └── inconsistent_types.json     # string "42" vs integer 42
├── config/
│   ├── layered_toml.json              # config after merging file + env
│   ├── versioned_old.json             # old config format
│   └── versioned_new.json             # new config format with extra fields
├── etl/
│   ├── csv_as_json.json               # all-string values from CSV
│   ├── jsonlines_inconsistent.jsonl   # schema drifts between rows
│   └── partial_records.json           # missing fields in some records
└── protocols/
    ├── rdp_pdu_with_extensions.json   # unknown extension fields
    └── event_sourced_v1_v2.json       # schema evolution
```

Every feature should have tests parsing these fixtures and asserting correct extraction, coercion, and diagnostics.

---

## Design Principles

1. **Never panic on external data.** All parsing returns `Result`. Unknown shapes are captured, not rejected.
2. **Escape hatches everywhere.** Every typed layer provides access back to the raw `FlexValue` or `serde_json::Value`.
3. **Coerce at boundaries, not throughout.** Coercion happens at extraction time, not during internal navigation.
4. **Forward-compatible by default.** `Unknown` variants and overflow fields mean new data additions never break existing code.
5. **Zero cost if unused.** Feature flags ensure you only compile what you need. Lenient residual is `()` (ZST).
6. **Complement serde, don't compete.** Accept `serde_json::Value` as input. Use serde types. Sit in the space serde's maintainer explicitly carved out.
7. **Diagnostics, not silence.** Coercions, defaults, and drops are recorded — not silently swallowed and not fatally rejected. The user controls the response.
8. **Modes are explicit.** Strictness is never context-dependent or implicit. The mode is a parameter.
9. **Consumption and production are different.** Reading external data (be liberal) and writing output (be conservative) have different correctness requirements. The type system should reflect this.

---

## Adoption & Marketing Strategy **[PLANNED]**

### Organic Adoption Path

1. **crates.io presence** — name `laminate` is available, strong SEO
2. **Killer README example** — 5 lines showing the pain, 5 lines showing the fix. People share these.
3. **r/rust "Show" post** — the Rust subreddit is the single biggest discovery channel for new crates
4. **This Week in Rust** — newsletter that features new crates, accepts submissions
5. **Blog post** — "Why I built laminate" explaining the gap, linking to dtolnay's serde issue #464 quote. The serde maintainer's endorsement of the *concept* is gold.
6. **Real usage in own projects** — lamco-rdp-server consuming messy protocol data through laminate is a credibility signal

### Lead With the Killer Feature

Type coercion is the #1 pain point (appeared in 10 of 16 surveyed use cases). The one-sentence pitch:

> "Turn `"42"` into `i64` without annotating every field."

Every Rust developer who's fought with serde understands this instantly.

### Positioning

- **Complement to serde, not competitor** — accept Value, output serde types
- **Postel's Law for Rust data** — universally understood by systems programmers
- **"Every layer is an on-ramp, not a gate"** — progressive, not all-or-nothing
- **Start simple, tighten later** — FlexValue in 5 minutes, full derive macro when ready

### Launch Sequence

1. Ship `core` feature (FlexValue + coercion + modes + diagnostics) — independently useful, low barrier
2. Ship `derive` feature — the full `#[derive(Laminate)]` experience
3. Ship 1-2 domain packs as examples (LLM providers + date/time)
4. Let the community tell you where the pain is worst — build domain packs based on demand
