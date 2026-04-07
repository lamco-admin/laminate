//! Error types for laminate operations.
//!
//! All errors include path context so you know WHERE the problem occurred.
//! `FlexError` variants cover navigation, type mismatch, coercion failure,
//! deserialization, and shaping diagnostics.

use crate::diagnostic::Diagnostic;

/// Errors during navigation, extraction, and shaping.
#[derive(Debug, thiserror::Error)]
pub enum FlexError {
    /// A path segment did not resolve to any value.
    #[error("path not found: {path}")]
    PathNotFound {
        /// The path that was not found.
        path: String,
    },

    /// The value at a path was a different type than expected.
    #[error("type mismatch at '{path}': expected {expected}, got {actual}")]
    TypeMismatch {
        /// Where the mismatch occurred.
        path: String,
        /// The type that was requested.
        expected: String,
        /// The type that was found.
        actual: String,
    },

    /// An array index exceeded the array length.
    #[error("index {index} out of bounds (len {len}) at '{path}'")]
    IndexOutOfBounds {
        /// Where the out-of-bounds access occurred.
        path: String,
        /// The requested index.
        index: usize,
        /// The actual array length.
        len: usize,
    },

    /// Serde deserialization failed after coercion.
    #[error("deserialization failed at '{path}': {source}")]
    DeserializeError {
        /// Where deserialization failed.
        path: String,
        /// The underlying serde_json error.
        source: serde_json::Error,
    },

    /// The path string itself was malformed.
    #[error("invalid path syntax: {detail}")]
    InvalidPath {
        /// Description of the syntax error.
        detail: String,
    },

    /// A coercion was attempted but could not succeed.
    #[error("coercion failed at '{path}': {detail}")]
    CoercionFailed {
        /// Where coercion failed.
        path: String,
        /// Why coercion failed.
        detail: String,
    },

    /// Shaping produced diagnostics that the mode treats as errors.
    #[error("shaping produced {count} diagnostic(s)")]
    ShapingDiagnostics {
        /// Number of diagnostics produced.
        count: usize,
        /// The individual diagnostics.
        diagnostics: Vec<Diagnostic>,
    },

    /// A tool call handler returned an error.
    #[error("handler '{name}': {detail}")]
    HandlerError {
        /// Name of the handler that failed.
        name: String,
        /// Error detail from the handler.
        detail: String,
    },
}

/// Result type alias for laminate operations.
pub type Result<T> = std::result::Result<T, FlexError>;
