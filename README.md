# Laminate

**The missing data layer for AI applications in Rust — and everything else that touches messy JSON.**

[![Crates.io](https://img.shields.io/crates/v/laminate.svg)](https://crates.io/crates/laminate)
[![Docs.rs](https://docs.rs/laminate/badge.svg)](https://docs.rs/laminate)
[![License](https://img.shields.io/crates/l/laminate.svg)](LICENSE)

## Why This Exists

Rust has excellent AI/ML inference libraries (candle, burn, ort) but **no good way to handle the messy JSON that LLM APIs actually return.** Anthropic stringifies tool call arguments. OpenAI streams fragments across dozens of SSE events. Ollama uses a different response shape entirely. Schema changes arrive without warning. And serde — Rust's serialization workhorse — fails on the first unexpected field.

We built laminate to solve this: a unified layer for consuming, normalizing, and dispatching LLM responses across providers. But solving that problem required solving the *general* problem of messy external data in Rust — and that general solution turned out to be just as valuable for REST APIs, config files, ETL pipelines, healthcare data, and logistics.

The serde maintainer himself [explicitly called for this library](https://github.com/serde-rs/serde/issues/464) in 2017: *"I would love to see this explored in a different library specifically geared toward fault-tolerant partially successful deserialization."* Ten years later, laminate is that library.

Laminate bonds layers of structure onto raw data — progressively, configurably, without breaking. Like physical lamination, each layer adds strength and rigidity. You can stop at any ply.

## The Problem

LLM APIs return data that breaks serde. But so does everything else from the outside world:

```rust
// serde breaks on the first surprise
#[derive(Deserialize)]
struct Config {
    port: u16,    // ← "8080" from env var? BOOM.
    debug: bool,  // ← "true" from YAML? BOOM.
}

// serde_json::Value gives you no safety at all
let val: serde_json::Value = serde_json::from_str(&data)?;
let port = val.get("port")?.as_u64()? as u16;  // no coercion, no path safety
```

**There is nothing in between.** Until now.

## The Solution

```rust
use laminate::FlexValue;

// Parse once, extract with automatic type coercion
let config = FlexValue::from_json(r#"{"port": "8080", "debug": "true", "workers": 4}"#)?;

let port: u16 = config.extract("port")?;       // "8080" → 8080  ✓
let debug: bool = config.extract("debug")?;     // "true" → true  ✓
let workers: i32 = config.extract("workers")?;  //  4 → 4         ✓
```

Three lines. No per-field annotations. No custom deserializers. No `#[serde(deserialize_with)]` on every field. It just works.

## Built for AI: LLM Response Handling

Laminate includes built-in adapters for Anthropic, OpenAI, and Ollama that normalize responses into a single type. For full-featured agent frameworks with dozens of providers, agent loops, and RAG, see [Rig](https://rig.rs/), [llm](https://github.com/graniet/llm), or [llm-connector](https://crates.io/crates/llm-connector). Laminate's AI adapters are a lightweight convenience layer — the real value is the data shaping engine beneath them.

```rust
use laminate::provider::anthropic::AnthropicAdapter;
use laminate::provider::ProviderAdapter;

let adapter = AnthropicAdapter;
let response = adapter.parse_response(&raw_api_body)?;

// Same API regardless of which LLM provider you're using
let text = response.text();                    // all text content
let tool_calls = response.tool_uses();         // all tool/function calls
let tokens = response.usage.output_tokens;     // token usage
```

Stream responses with automatic tool call fragment assembly:

```rust
use laminate::streaming::{FlexStream, StreamConfig, Provider, StreamEvent};

let mut stream = FlexStream::new(StreamConfig {
    provider: Provider::Anthropic,
    ..Default::default()
});

for chunk in incoming_sse_bytes {
    for event in stream.feed(&chunk) {
        match event {
            StreamEvent::TextDelta(text) => print!("{}", text),
            StreamEvent::BlockComplete { name, content, .. } => {
                // Tool arguments assembled from fragments automatically
                let city: String = content.extract("city")?;
            }
            _ => {}
        }
    }
}
```

Dispatch tool calls to typed handlers with automatic argument deserialization:

```rust
let mut registry = HandlerRegistry::new();

registry.register("get_weather", |args: WeatherArgs| async move {
    let weather = fetch_weather(&args.city).await?;
    Ok(WeatherResult { temp: weather.temp })
});

// One call dispatches all tool uses from any provider's response
let results = registry.dispatch_all(&response).await?;
```

## And Everything Else: Universal Data Shaping

The same engine that handles LLM responses handles every other source of messy data in Rust.

## Navigate Deep Structures

```rust
let api_response = FlexValue::from_json(r#"{
    "choices": [{
        "message": {
            "content": "Hello!",
            "tool_calls": [{"function": {"name": "search", "arguments": "{\"q\": \"rust\"}"}}]
        }
    }]
}"#)?;

// Dot-path + bracket-index navigation with coercion at every step
let content: String = api_response.extract("choices[0].message.content")?;
let tool_name: String = api_response.extract("choices[0].message.tool_calls[0].function.name")?;
```

## Derive Macro: Typed Structs That Tolerate Messy Data

```rust
use laminate::Laminate;

#[derive(Debug, Laminate)]
struct UserProfile {
    name: String,
    #[laminate(coerce)]
    age: i64,                           // accepts "25", 25, 25.0
    #[laminate(coerce, default)]
    verified: bool,                      // accepts "true", 1, true — defaults to false if missing
    #[laminate(rename = "e-mail")]
    email: String,                       // reads from "e-mail" key
    #[laminate(overflow)]
    extra: HashMap<String, serde_json::Value>,  // captures ALL unknown fields
}

let (user, diagnostics) = UserProfile::from_json(r#"{
    "name": "Alice",
    "age": "25",
    "verified": "yes",
    "e-mail": "alice@example.com",
    "theme": "dark",
    "lang": "en"
}"#)?;

assert_eq!(user.age, 25);                          // coerced from "25"
assert_eq!(user.verified, true);                    // coerced from "yes"
assert_eq!(user.extra["theme"], "dark");            // unknown field preserved!
assert_eq!(user.extra["lang"], "en");               // unknown field preserved!

// Every coercion is recorded — nothing is silent
for d in &diagnostics {
    println!("{}", d);  // "coerced string → i64 at 'age'"
}
```

## Three Modes: Progressive Strictness

| Mode | Unknown Fields | Coercion | Missing Fields | Use For |
|------|---------------|----------|---------------|---------|
| **Lenient** | Dropped | BestEffort (try everything) | Defaulted | API consumption, scraping, logs |
| **Absorbing** | Preserved in overflow | SafeWidening (safe conversions) | Error | Round-trip proxying, config editing |
| **Strict** | Error | Exact (types must match) | Error | Output construction, validation |

```rust
use laminate::coerce::CoercionLevel;

// Same data, different strictness
let val = FlexValue::from(json!({"count": "42"}));

// BestEffort: "42" → 42 ✓
let count: i64 = val.with_coercion(CoercionLevel::BestEffort).extract("count")?;

// Exact: "42" is a string, not an i64 → Error
let result: Result<i64, _> = val.with_coercion(CoercionLevel::Exact).extract("count");
assert!(result.is_err());
```

## Schema Inference & Data Auditing

Infer a schema from data, then audit new data against it:

```rust
use laminate::schema::InferredSchema;

// Learn the schema from 1000 records
let schema = InferredSchema::from_values(&training_data);
println!("{}", schema.summary());
// Fields: name (String, required), age (Integer, 98% present), score (Float, nullable)

// Audit new data against the learned schema
let report = schema.audit(&new_batch);
println!("{}", report.summary());
// 3 violations: row 42 has age="old" (type mismatch), row 99 missing required 'name', ...
```

*See [Built for AI](#built-for-ai-unified-llm-response-handling) above for provider normalization, streaming, and tool call dispatch.*

## Locale-Aware Number Parsing

Laminate understands international number formats out of the box:

```rust
let val = FlexValue::from(json!("1.234,56"))  // European: 1,234.56
    .with_coercion(CoercionLevel::BestEffort);
let amount: f64 = val.extract_root()?;  // 1234.56

// Also handles: "1'234.56" (Swiss), "1 234 567" (French/SI),
// "1_000" (Rust/Python), "0xFF" (hex), "$12.99" (currency),
// "2.5 kg" (units), "Mar 31, 2026" (dates)
```

## SQL Data Sources

Query databases and get `FlexValue` rows with automatic type mapping:

```rust
use laminate_sql::sqlite::SqliteSource;
use laminate_sql::DataSource;

let db = SqliteSource::connect("sqlite:mydata.db").await?;
let rows = db.query("SELECT * FROM products WHERE price > 10").await?;

for row in &rows {
    let name: String = row.extract("name")?;
    let price: f64 = row.extract("price")?;
}
```

## Type Detection: "What IS This String?"

```rust
use laminate::detect::{guess_type, GuessedType};

let guesses = guess_type("$12.99");
assert!(guesses[0].kind == GuessedType::Currency);  // 0.90 confidence

let guesses = guess_type("550e8400-e29b-41d4-a716-446655440000");
assert!(guesses[0].kind == GuessedType::Uuid);       // 0.98 confidence

let guesses = guess_type("42");
// → [(Integer, 0.95), (Float, 0.70), (Boolean, 0.30)]
```

## Source-Aware Coercion

Tell laminate where data came from — it adjusts coercion automatically:

```rust
use laminate::{FlexValue, value::SourceHint};

// CSV data: everything is strings — enable full coercion + pack detection
let val = FlexValue::from_json(data)?
    .with_source_hint(SourceHint::Csv);

let price: f64 = val.extract("price")?;  // "$12.99" → 12.99 (pack coercion)
let port: u16 = val.extract("port")?;    // "8080" → 8080 (string coercion)
```

## Domain Packs

Six built-in domain packs, always compiled (no feature flags needed):

| Pack | What It Does |
|------|-------------|
| **time** | Detects 14+ date/time formats, converts to ISO 8601, batch column detection with US/EU disambiguation, HL7 v2 packed dates, GEDCOM 7.0 qualifiers |
| **currency** | Parses `$12.99`, `€1.234,56`, `(¥500)`, `1'234 CHF` — 30 currency codes, accounting negatives, locale-aware decimals |
| **units** | Parses `2.5 kg`, `120 lbs 4 oz`, `37.2°C` — weight, length, temperature (°C↔°F↔K conversion), volume, time, data, nautical miles, UNECE/X12/DOD standard codes, pack-size notation, SI-prefixed units, weight qualifiers (gross/net/tare) |
| **identifiers** | Validates IBAN (MOD-97), credit cards (Luhn + BIN brand), ISBN-10/13, US SSN/EIN, US NPI, UK NHS Number, EU VAT, UUID, email, phone |
| **geo** | Parses decimal degrees, DMS, ISO 6709 coordinates, detects lat/lng vs lng/lat order, identifies geodetic datums (WGS84, JGD2011, CGCS2000) |
| **medical** | Converts 18 lab values between US (mg/dL) and SI (mmol/L) units with analyte-specific factors, normalizes pharmaceutical notation (mcg/µg/ug), parses HL7 v2 dates |

## Features

```toml
[dependencies]
laminate = "0.1"                          # Core: FlexValue, coercion, modes
laminate = { version = "0.1", features = ["derive"] }     # + #[derive(Laminate)]
laminate = { version = "0.1", features = ["full"] }       # Everything

# Optional: database sources
laminate-sql = { version = "0.1", features = ["sqlite"] }
```

| Feature | What It Adds |
|---------|-------------|
| `core` (default) | FlexValue, path navigation, coercion engine, modes, diagnostics, 6 domain packs, type detection, source hints |
| `derive` | `#[derive(Laminate)]` + `#[derive(ToolDefinition)]` procedural macros |
| `streaming` | SSE parser, stream event handling, MessageSnapshot |
| `providers` | Anthropic, OpenAI, Ollama response normalization |
| `registry` | Typed handler dispatch for tool calls |
| `schema` | Schema inference and data auditing |
| `full` | All of the above |
| `chrono-integration` | Convert detected dates to `chrono::NaiveDate` / `NaiveDateTime` |
| `uom-integration` | Convert parsed units to `uom` type-safe SI quantities |

## Why Not Just Use serde?

Laminate is a **complement to serde**, not a replacement. serde handles serialization brilliantly. Laminate handles the messy reality that comes *before* your carefully typed structs:

| Scenario | serde | laminate |
|----------|-------|---------|
| API returns `"42"` for an integer field | ❌ Error | ✅ Coerces to 42 with diagnostic |
| Unknown fields in response | ❌ Ignored or error | ✅ Preserved in overflow for round-trip |
| Missing optional field | ⚠️ Requires `#[serde(default)]` per field | ✅ Mode-level policy |
| Schema changed upstream | ❌ Hard failure | ✅ Absorb unknown, default missing, report all |
| Multiple error locations | ❌ Stops at first | ✅ Collects all diagnostics |
| Mixed types in array | ❌ Error | ✅ Element-level coercion |

The [serde maintainer explicitly stated](https://github.com/serde-rs/serde/issues/464) that fault-tolerant, partially-successful deserialization should be *"a different library."* This is that library.

## Design Philosophy

Every transformation is **auditable** — laminate never silently changes your data. The diagnostic trail tells you exactly what was coerced, what was defaulted, what was dropped, and what was preserved.

```
coerced string → i64 at 'age' [Info]
defaulted field 'verified' (null → default) [Warning]
preserved unknown field 'theme' in overflow [Info]
overridden object → number at 'config' [Warning: nested data lost]
```

**Progressive strictness** means you start lenient and tighten over time. Ship fast with `BestEffort`, then review diagnostics, then progressively restrict. The same pipeline works for prototyping and production.

## License

[License details]

## Contributing

[Contributing guidelines]
