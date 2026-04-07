/// A record of what happened during a shaping or coercion operation.
///
/// Every coercion, default, drop, and preservation is recorded — not silently
/// swallowed and not fatally rejected. The user controls the response.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// The path where this diagnostic occurred (e.g., "data.user.age").
    pub path: String,
    /// What happened.
    pub kind: DiagnosticKind,
    /// How risky is this transformation.
    pub risk: RiskLevel,
    /// Actionable suggestion for tightening behavior.
    pub suggestion: Option<String>,
}

/// What kind of transformation occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// A value was coerced from one type to another.
    Coerced {
        /// The original type (e.g., "String").
        from: String,
        /// The target type (e.g., "i64").
        to: String,
    },
    /// A field was missing and filled with a default value.
    Defaulted {
        /// The field that was defaulted.
        field: String,
        /// String representation of the default value used.
        value: String,
    },
    /// An unknown field was dropped (lenient mode).
    Dropped {
        /// The field that was dropped.
        field: String,
    },
    /// An unknown field was preserved in overflow (absorbing mode).
    Preserved {
        /// The field that was preserved.
        field: String,
    },
    /// A field failed to deserialize but was defaulted (lenient mode).
    ErrorDefaulted {
        /// The field that failed.
        field: String,
        /// The error message.
        error: String,
    },
    /// A value was replaced by a merge operation (not coercion).
    Overridden {
        /// The type of the original value.
        from_type: String,
        /// The type of the replacement value.
        to_type: String,
    },
}

/// Risk level for a diagnostic — mirrors Rust's allow/warn/deny pattern.
///
/// Mode controls how each level is treated:
/// - **Lenient**: all proceed, diagnostics available on request
/// - **Absorbing**: warnings recorded, risky coercions flagged
/// - **Strict**: warnings become errors, risky coercions fail
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Standard, expected coercion. No action needed.
    Info,
    /// Potentially surprising coercion. Review recommended.
    Warning,
    /// Coercion that may lose data or change semantics. Tightening suggested.
    Risky,
}

impl std::fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticKind::Coerced { from, to } => write!(f, "coerced {from} → {to}"),
            DiagnosticKind::Defaulted { field, value } => {
                write!(f, "defaulted field '{field}' ({value})")
            }
            DiagnosticKind::Dropped { field } => write!(f, "dropped unknown field '{field}'"),
            DiagnosticKind::Preserved { field } => {
                write!(f, "preserved unknown field '{field}' in overflow")
            }
            DiagnosticKind::ErrorDefaulted { field, error } => {
                write!(f, "field '{field}' failed ({error}), used default")
            }
            DiagnosticKind::Overridden { from_type, to_type } => {
                write!(f, "overridden {from_type} → {to_type}")
            }
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] at '{}': {}", self.risk, self.path, self.kind)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " — {suggestion}")?;
        }
        Ok(())
    }
}

/// Why a response or stream ended.
///
/// Shared between the streaming and provider modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    /// The model finished its turn naturally.
    EndTurn,
    /// The model invoked a tool call.
    ToolUse,
    /// The response was truncated at the token limit.
    MaxTokens,
    /// A stop sequence was encountered.
    StopSequence,
    /// An unrecognized stop reason from the provider.
    Unknown(String),
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopReason::EndTurn => write!(f, "end_turn"),
            StopReason::ToolUse => write!(f, "tool_use"),
            StopReason::MaxTokens => write!(f, "max_tokens"),
            StopReason::StopSequence => write!(f, "stop_sequence"),
            StopReason::Unknown(s) => write!(f, "{s}"),
        }
    }
}

// ── DiagnosticSink trait ──────────────────────────────────────

/// Route diagnostics anywhere — logs, metrics, databases, UI.
///
/// Implement this trait to capture diagnostics from shaping operations.
/// Default implementations are provided for common use cases.
///
/// # Example
///
/// ```
/// use laminate::{Diagnostic, DiagnosticSink, RiskLevel};
///
/// struct WarningCounter { count: usize }
///
/// impl DiagnosticSink for WarningCounter {
///     fn receive(&mut self, diagnostic: &Diagnostic) {
///         if diagnostic.risk >= RiskLevel::Warning {
///             self.count += 1;
///         }
///     }
/// }
/// ```
pub trait DiagnosticSink {
    /// Receive a single diagnostic. Called for each coercion, default, or drop.
    fn receive(&mut self, diagnostic: &Diagnostic);

    /// Receive a batch of diagnostics.
    fn receive_all(&mut self, diagnostics: &[Diagnostic]) {
        for d in diagnostics {
            self.receive(d);
        }
    }
}

/// A diagnostic sink that collects into a Vec.
#[derive(Debug, Default)]
pub struct CollectSink {
    /// The collected diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticSink for CollectSink {
    fn receive(&mut self, diagnostic: &Diagnostic) {
        self.diagnostics.push(diagnostic.clone());
    }
}

/// A diagnostic sink that prints to stderr.
#[derive(Debug, Default)]
pub struct StderrSink;

impl DiagnosticSink for StderrSink {
    fn receive(&mut self, diagnostic: &Diagnostic) {
        eprintln!("{diagnostic}");
    }
}

/// A diagnostic sink that filters by minimum risk level.
pub struct FilteredSink<S: DiagnosticSink> {
    inner: S,
    min_risk: RiskLevel,
}

impl<S: DiagnosticSink> FilteredSink<S> {
    /// Create a new filtered sink that only forwards diagnostics at or above `min_risk`.
    pub fn new(inner: S, min_risk: RiskLevel) -> Self {
        Self { inner, min_risk }
    }
}

impl<S: DiagnosticSink> DiagnosticSink for FilteredSink<S> {
    fn receive(&mut self, diagnostic: &Diagnostic) {
        if diagnostic.risk >= self.min_risk {
            self.inner.receive(diagnostic);
        }
    }
}

/// A no-op sink that discards all diagnostics.
#[derive(Debug, Default)]
pub struct NullSink;

impl DiagnosticSink for NullSink {
    fn receive(&mut self, _diagnostic: &Diagnostic) {}
}

// Vec<Diagnostic> is itself a sink (convenience)
impl DiagnosticSink for Vec<Diagnostic> {
    fn receive(&mut self, diagnostic: &Diagnostic) {
        self.push(diagnostic.clone());
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Info => write!(f, "info"),
            RiskLevel::Warning => write!(f, "warning"),
            RiskLevel::Risky => write!(f, "risky"),
        }
    }
}
