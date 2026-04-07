//! Operational modes for progressive data shaping.
//!
//! Three modes control how strictness is applied during shaping:
//!
//! - [`Lenient`] — accept everything, coerce freely, drop unknowns
//! - [`Absorbing`] — accept everything, preserve unknowns in overflow
//! - [`Strict`] — reject unknowns, require exact types, prove completeness
//!
//! The mode is a type parameter on [`LaminateResult`], making strictness
//! explicit and compile-time enforced.

use std::collections::HashMap;

use crate::coerce::CoercionLevel;
use crate::diagnostic::Diagnostic;

// ── Sealed trait pattern ──────────────────────────────────────

mod sealed {
    pub trait Sealed {}
}

/// The overflow type — unknown fields preserved for round-tripping.
pub type Overflow = HashMap<String, serde_json::Value>;

/// A mode that controls shaping behavior across five axes:
/// unknown fields, type coercion, missing fields, error strategy, and transform timing.
///
/// This trait is sealed — only `Lenient`, `Absorbing`, and `Strict` implement it.
pub trait Mode: sealed::Sealed + std::fmt::Debug + Clone + Copy {
    /// What happens to the "leftover" after shaping.
    type Residual: std::fmt::Debug + Clone;

    /// Default coercion level for this mode.
    fn default_coercion() -> CoercionLevel;

    /// Whether unknown fields should cause an error.
    fn reject_unknown_fields() -> bool;

    /// Whether missing fields should cause an error (vs defaulting).
    fn require_all_fields() -> bool;

    /// Whether to fail on the first error (true) or collect all (false).
    fn fail_fast() -> bool;
}

// ── Lenient mode ──────────────────────────────────────────────

/// Lenient mode: accept everything, coerce freely, drop unknowns.
///
/// Use for consuming external APIs, scraping, logs — anything where
/// you want maximum tolerance for messy data.
///
/// - Unknown fields: **dropped** (not preserved)
/// - Coercion: **BestEffort** (try everything)
/// - Missing fields: **defaulted** (use Default::default())
/// - Errors: **collected** (not fail-fast)
/// - Residual: `()` — nothing preserved, zero-cost
#[derive(Debug, Clone, Copy)]
pub struct Lenient;

impl sealed::Sealed for Lenient {}

impl Mode for Lenient {
    type Residual = ();

    fn default_coercion() -> CoercionLevel {
        CoercionLevel::BestEffort
    }
    fn reject_unknown_fields() -> bool {
        false
    }
    fn require_all_fields() -> bool {
        false
    }
    fn fail_fast() -> bool {
        false
    }
}

// ── Absorbing mode ────────────────────────────────────────────

/// Absorbing mode: accept everything, preserve unknowns in overflow.
///
/// Use for round-tripping, protocol proxying, config file editing —
/// anything where unknown data must survive the parse/emit cycle.
///
/// - Unknown fields: **preserved** in `Overflow` (HashMap)
/// - Coercion: **SafeWidening** (safe numeric conversions only)
/// - Missing fields: **error** (required fields must be present)
/// - Errors: **collected** (not fail-fast)
/// - Residual: `Overflow` — unknown fields captured in a HashMap
#[derive(Debug, Clone, Copy)]
pub struct Absorbing;

impl sealed::Sealed for Absorbing {}

impl Mode for Absorbing {
    type Residual = Overflow;

    fn default_coercion() -> CoercionLevel {
        CoercionLevel::SafeWidening
    }
    fn reject_unknown_fields() -> bool {
        false
    }
    fn require_all_fields() -> bool {
        true
    }
    fn fail_fast() -> bool {
        false
    }
}

// ── Strict mode ───────────────────────────────────────────────

/// Strict mode: reject unknowns, require exact types, prove completeness.
///
/// Use for constructing output, validation, test assertions —
/// anything where correctness is paramount.
///
/// - Unknown fields: **error**
/// - Coercion: **Exact** (types must match exactly)
/// - Missing fields: **error**
/// - Errors: **fail-fast** (stop at first error)
/// - Residual: `std::convert::Infallible` — compile-time proof that
///   nothing was left over (you can never construct an Infallible value)
#[derive(Debug, Clone, Copy)]
pub struct Strict;

impl sealed::Sealed for Strict {}

impl Mode for Strict {
    type Residual = std::convert::Infallible;

    fn default_coercion() -> CoercionLevel {
        CoercionLevel::Exact
    }
    fn reject_unknown_fields() -> bool {
        true
    }
    fn require_all_fields() -> bool {
        true
    }
    fn fail_fast() -> bool {
        true
    }
}

// ── LaminateResult ────────────────────────────────────────────

/// The result of a shaping operation.
///
/// Bundles the shaped value, mode-specific residual, and diagnostics.
/// The residual type varies by mode:
/// - `Lenient` → `()` (zero-cost, nothing preserved)
/// - `Absorbing` → `Overflow` (HashMap of unknown fields)
/// - `Strict` → `Infallible` (uninhabitable — proves completeness)
#[derive(Debug, Clone)]
pub struct LaminateResult<T, M: Mode> {
    /// The shaped value.
    pub value: T,
    /// Mode-specific residual (unknown fields, completeness proof, etc.).
    pub residual: M::Residual,
    /// Diagnostics produced during shaping (coercions, defaults, drops).
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> LaminateResult<T, Lenient> {
    /// Create a lenient result with no residual.
    pub fn lenient(value: T, diagnostics: Vec<Diagnostic>) -> Self {
        LaminateResult {
            value,
            residual: (),
            diagnostics,
        }
    }
}

impl<T> LaminateResult<T, Absorbing> {
    /// Create an absorbing result with overflow fields.
    pub fn absorbing(value: T, overflow: Overflow, diagnostics: Vec<Diagnostic>) -> Self {
        LaminateResult {
            value,
            residual: overflow,
            diagnostics,
        }
    }
}

// Note: LaminateResult<T, Strict> cannot be constructed with a residual
// because Infallible is uninhabitable. Strict results are created by
// verifying that no unknown fields remain, then using unsafe construction
// or a separate constructor that proves the invariant.

// ── Runtime mode selection ────────────────────────────────────

/// Runtime mode selection for when the mode is determined dynamically
/// (e.g., from a config file: "run lenient in dev, strict in prod").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicMode {
    /// Lenient: drop unknowns, coerce aggressively, collect diagnostics.
    Lenient,
    /// Absorbing: preserve unknowns in overflow, moderate coercion.
    Absorbing,
    /// Strict: reject unknowns and coercions, fail fast.
    Strict,
}

impl DynamicMode {
    /// Get the default coercion level for this mode.
    pub fn default_coercion(&self) -> CoercionLevel {
        match self {
            DynamicMode::Lenient => Lenient::default_coercion(),
            DynamicMode::Absorbing => Absorbing::default_coercion(),
            DynamicMode::Strict => Strict::default_coercion(),
        }
    }

    /// Whether unknown fields should cause an error.
    pub fn reject_unknown_fields(&self) -> bool {
        match self {
            DynamicMode::Lenient => Lenient::reject_unknown_fields(),
            DynamicMode::Absorbing => Absorbing::reject_unknown_fields(),
            DynamicMode::Strict => Strict::reject_unknown_fields(),
        }
    }

    /// Whether missing fields should cause an error.
    pub fn require_all_fields(&self) -> bool {
        match self {
            DynamicMode::Lenient => Lenient::require_all_fields(),
            DynamicMode::Absorbing => Absorbing::require_all_fields(),
            DynamicMode::Strict => Strict::require_all_fields(),
        }
    }

    /// Whether to fail on the first error.
    pub fn fail_fast(&self) -> bool {
        match self {
            DynamicMode::Lenient => Lenient::fail_fast(),
            DynamicMode::Absorbing => Absorbing::fail_fast(),
            DynamicMode::Strict => Strict::fail_fast(),
        }
    }
}

impl std::fmt::Display for DynamicMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicMode::Lenient => write!(f, "lenient"),
            DynamicMode::Absorbing => write!(f, "absorbing"),
            DynamicMode::Strict => write!(f, "strict"),
        }
    }
}

impl std::str::FromStr for DynamicMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lenient" => Ok(DynamicMode::Lenient),
            "absorbing" => Ok(DynamicMode::Absorbing),
            "strict" => Ok(DynamicMode::Strict),
            other => Err(format!(
                "unknown mode: {other} (expected lenient, absorbing, or strict)"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenient_defaults() {
        assert_eq!(Lenient::default_coercion(), CoercionLevel::BestEffort);
        assert!(!Lenient::reject_unknown_fields());
        assert!(!Lenient::require_all_fields());
        assert!(!Lenient::fail_fast());
    }

    #[test]
    fn absorbing_defaults() {
        assert_eq!(Absorbing::default_coercion(), CoercionLevel::SafeWidening);
        assert!(!Absorbing::reject_unknown_fields());
        assert!(Absorbing::require_all_fields());
        assert!(!Absorbing::fail_fast());
    }

    #[test]
    fn strict_defaults() {
        assert_eq!(Strict::default_coercion(), CoercionLevel::Exact);
        assert!(Strict::reject_unknown_fields());
        assert!(Strict::require_all_fields());
        assert!(Strict::fail_fast());
    }

    #[test]
    fn lenient_result() {
        let result = LaminateResult::<String, Lenient>::lenient("hello".into(), vec![]);
        assert_eq!(result.value, "hello");
        assert_eq!(result.residual, ());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn absorbing_result() {
        let mut overflow = HashMap::new();
        overflow.insert("extra".into(), serde_json::json!("value"));

        let result =
            LaminateResult::<String, Absorbing>::absorbing("hello".into(), overflow, vec![]);
        assert_eq!(result.value, "hello");
        assert_eq!(result.residual.len(), 1);
        assert!(result.residual.contains_key("extra"));
    }

    #[test]
    fn dynamic_mode_from_str() {
        assert_eq!(
            "lenient".parse::<DynamicMode>().unwrap(),
            DynamicMode::Lenient
        );
        assert_eq!(
            "Absorbing".parse::<DynamicMode>().unwrap(),
            DynamicMode::Absorbing
        );
        assert_eq!(
            "STRICT".parse::<DynamicMode>().unwrap(),
            DynamicMode::Strict
        );
        assert!("unknown".parse::<DynamicMode>().is_err());
    }

    #[test]
    fn dynamic_mode_mirrors_static() {
        let dm = DynamicMode::Lenient;
        assert_eq!(dm.default_coercion(), Lenient::default_coercion());
        assert_eq!(dm.reject_unknown_fields(), Lenient::reject_unknown_fields());
        assert_eq!(dm.require_all_fields(), Lenient::require_all_fields());
        assert_eq!(dm.fail_fast(), Lenient::fail_fast());
    }

    #[test]
    fn dynamic_mode_display() {
        assert_eq!(DynamicMode::Lenient.to_string(), "lenient");
        assert_eq!(DynamicMode::Absorbing.to_string(), "absorbing");
        assert_eq!(DynamicMode::Strict.to_string(), "strict");
    }
}
