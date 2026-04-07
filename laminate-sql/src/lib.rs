//! # laminate-sql — Database connectors for laminate
//!
//! Provides the [`DataSource`] trait and implementations for PostgreSQL,
//! SQLite, and MySQL. Queries return rows as [`FlexValue`] for shaping,
//! coercion, and schema inference.
//!
//! ## Features
//!
//! - `postgres` — PostgreSQL via sqlx
//! - `sqlite` — SQLite via sqlx
//! - `mysql` — MySQL via sqlx
//! - `all-databases` — All of the above
//!
//! ## Quick Start
//!
//! ```ignore
//! use laminate_sql::{DataSource, PostgresSource};
//!
//! let source = PostgresSource::connect("postgres://user:pass@localhost/mydb").await?;
//! let rows = source.query("SELECT * FROM customers LIMIT 100").await?;
//!
//! // Use with laminate schema inference
//! let raw_rows: Vec<serde_json::Value> = rows.iter().map(|r| r.into_raw()).collect();
//! let schema = laminate::InferredSchema::from_values(&raw_rows);
//! let report = schema.audit(&raw_rows);
//! println!("{}", report.summary());
//! ```

use laminate::FlexValue;
use serde_json::Value;

/// A data source that produces rows as FlexValue.
///
/// Implementations exist for PostgreSQL, SQLite, and MySQL via the
/// corresponding feature flags. Custom data sources can implement this
/// trait directly.
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Execute a query and return all rows as FlexValue.
    async fn query(&self, sql: &str) -> Result<Vec<FlexValue>, DataSourceError>;

    /// Execute a query with bind parameters and return all rows.
    async fn query_with(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Vec<FlexValue>, DataSourceError>;

    /// Get the column names from a query without fetching data.
    async fn columns(&self, sql: &str) -> Result<Vec<String>, DataSourceError>;

    /// Count rows matching a query.
    async fn count(&self, sql: &str) -> Result<u64, DataSourceError>;
}

/// Errors from data source operations.
#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    /// Failed to connect to the database.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// A SQL query failed to execute.
    #[error("query failed: {0}")]
    QueryFailed(String),

    /// Row data could not be converted to JSON.
    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    /// The requested operation is not supported by this backend.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

/// Convert sqlx errors to DataSourceError.
impl From<sqlx::Error> for DataSourceError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Configuration(_) | sqlx::Error::Io(_) => {
                DataSourceError::ConnectionFailed(e.to_string())
            }
            _ => DataSourceError::QueryFailed(e.to_string()),
        }
    }
}

// ── Database-specific modules ─────────────────────────────────

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "mysql")]
pub mod mysql;

// ── Re-exports ────────────────────────────────────────────────

#[cfg(feature = "postgres")]
pub use postgres::PostgresSource;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteSource;

#[cfg(feature = "mysql")]
pub use mysql::MysqlSource;

// ── CSV and JSON Lines sources (always available) ─────────────

/// Read JSON Lines (newline-delimited JSON) as FlexValue rows.
pub fn read_jsonl(content: &str) -> Result<Vec<FlexValue>, DataSourceError> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            FlexValue::from_json(line)
                .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))
        })
        .collect()
}

/// Read a JSON array as FlexValue rows.
pub fn read_json_array(json: &str) -> Result<Vec<FlexValue>, DataSourceError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;

    match value {
        Value::Array(arr) => Ok(arr.into_iter().map(FlexValue::new).collect()),
        _ => Err(DataSourceError::SerializationFailed(
            "expected a JSON array".into(),
        )),
    }
}
