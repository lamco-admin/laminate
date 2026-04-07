//! # Laminate — Data, shaped layer by layer
//!
//! Progressive data shaping for Rust. Bonds layers of structure onto raw data —
//! progressively, configurably, without breaking.
//!
//! Laminate sits between fully dynamic (`serde_json::Value`) and fully typed
//! (`#[derive(Deserialize)]`), providing a progressive pipeline for shaping
//! unstructured data into typed Rust values.
//!
//! ## Quick Start
//!
//! ```
//! use laminate::FlexValue;
//!
//! let val = FlexValue::from_json(r#"{"port": "8080", "debug": "true"}"#).unwrap();
//!
//! // Type coercion happens automatically
//! let port: u16 = val.extract("port").unwrap();      // "8080" → 8080
//! let debug: bool = val.extract("debug").unwrap();    // "true" → true
//! ```
//!
//! ## Path Navigation
//!
//! ```
//! use laminate::FlexValue;
//!
//! let val = FlexValue::from_json(r#"{"users": [{"name": "Alice"}]}"#).unwrap();
//! let name: String = val.extract("users[0].name").unwrap();
//! ```
//!
//! ## Features
//!
//! - `core` (default) — FlexValue, path access, coercion, modes, diagnostics
//! - `derive` — `#[derive(Laminate)]` macro
//! - `streaming` — SSE parser, Anthropic/OpenAI stream handlers
//! - `providers` — Provider normalization adapters
//! - `registry` — Handler dispatch for tool calls
//! - `schema` — Schema inference and data auditing
//! - `full` — All features

// ── Core modules (always available) ──
pub mod coerce;
/// Type detection — `guess_type()` identifies what kind of data a string contains.
pub mod detect;
/// Graduated diagnostics — every coercion, default, and drop is recorded.
pub mod diagnostic;
pub mod error;
pub mod mode;
pub mod path;
/// FlexValue — the core navigable JSON wrapper with path access, coercion, and diagnostics.
pub mod value;

// ── Domain coercion packs (always available) ──
pub mod packs;

// ── Feature-gated modules ──
#[cfg(feature = "streaming")]
pub mod streaming;

#[cfg(feature = "providers")]
pub mod provider;

#[cfg(feature = "registry")]
pub mod registry;

#[cfg(feature = "schema")]
pub mod schema;

// ── Core re-exports (always available) ──
pub use coerce::{Coercible, CoercionLevel};
pub use diagnostic::{
    CollectSink, Diagnostic, DiagnosticKind, DiagnosticSink, FilteredSink, NullSink, RiskLevel,
    StderrSink, StopReason,
};
pub use error::{FlexError, Result};
pub use mode::{Absorbing, DynamicMode, LaminateResult, Lenient, Mode, Overflow, Strict};
pub use value::FlexValue;

// ── Feature-gated re-exports ──
#[cfg(feature = "providers")]
pub use provider::{ContentBlock, NormalizedResponse, ProviderAdapter, Usage};

#[cfg(feature = "registry")]
pub use registry::HandlerRegistry;

#[cfg(feature = "schema")]
pub use schema::{AuditReport, InferredSchema};

#[cfg(feature = "streaming")]
pub use streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

// Re-export derive macro when the "derive" feature is enabled
#[cfg(feature = "derive")]
pub use laminate_derive::Laminate;

#[cfg(feature = "derive")]
pub use laminate_derive::ToolDefinition;
