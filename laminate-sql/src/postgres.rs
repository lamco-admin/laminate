//! PostgreSQL data source for laminate.
//!
//! Connects to PostgreSQL via sqlx and returns query results as FlexValue rows.
//! Each row is converted to a JSON object via PostgreSQL's `row_to_json()` or
//! column-by-column extraction.

use laminate::FlexValue;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use crate::{DataSource, DataSourceError};

/// PostgreSQL data source.
///
/// # Example
///
/// ```ignore
/// let source = PostgresSource::connect("postgres://user:pass@localhost/mydb").await?;
/// let rows = source.query("SELECT id, name, email FROM customers").await?;
/// ```
pub struct PostgresSource {
    pool: PgPool,
}

impl PostgresSource {
    /// Connect to PostgreSQL with a connection string.
    pub async fn connect(url: &str) -> Result<Self, DataSourceError> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
        Ok(Self { pool })
    }

    /// Create from an existing connection pool.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert a PgRow to a serde_json::Value object.
    fn row_to_value(row: &PgRow) -> Result<Value, DataSourceError> {
        let columns = row.columns();
        let mut obj = serde_json::Map::with_capacity(columns.len());

        for col in columns {
            let name = col.name().to_string();
            let value = Self::extract_column_value(row, col)?;
            obj.insert(name, value);
        }

        Ok(Value::Object(obj))
    }

    /// Extract a column value as serde_json::Value, handling all common PG types.
    fn extract_column_value(
        row: &PgRow,
        col: &sqlx::postgres::PgColumn,
    ) -> Result<Value, DataSourceError> {
        let type_name = col.type_info().name();
        let idx = col.ordinal();

        // Check for NULL first
        if row.try_get_raw(idx).map(|v| v.is_null()).unwrap_or(true) {
            return Ok(Value::Null);
        }

        let value = match type_name {
            "BOOL" => {
                let v: bool = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::Bool(v)
            }
            "INT2" | "SMALLINT" => {
                let v: i16 = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::Number(v.into())
            }
            "INT4" | "INT" | "INTEGER" => {
                let v: i32 = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::Number(v.into())
            }
            "INT8" | "BIGINT" => {
                let v: i64 = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::Number(v.into())
            }
            "FLOAT4" | "REAL" => {
                let v: f32 = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                serde_json::Number::from_f64(v as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let v: f64 = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                serde_json::Number::from_f64(v)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            "NUMERIC" | "DECIMAL" => {
                // Numeric types: extract as string to preserve precision
                let v: String = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::String(v)
            }
            "JSON" | "JSONB" => {
                let v: Value = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                v
            }
            _ => {
                // Default: try as string (covers TEXT, VARCHAR, UUID, TIMESTAMP, DATE, etc.)
                let v: String = row
                    .try_get(idx)
                    .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                Value::String(v)
            }
        };

        Ok(value)
    }
}

#[async_trait::async_trait]
impl DataSource for PostgresSource {
    async fn query(&self, sql: &str) -> Result<Vec<FlexValue>, DataSourceError> {
        let rows: Vec<PgRow> = sqlx::query(sql).fetch_all(&self.pool).await?;

        rows.iter()
            .map(|row| {
                let val = Self::row_to_value(row)?;
                Ok(FlexValue::new(val))
            })
            .collect()
    }

    async fn query_with(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Vec<FlexValue>, DataSourceError> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = match param {
                Value::String(s) => query.bind(s.as_str()),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        query.bind(f)
                    } else {
                        query.bind(n.to_string())
                    }
                }
                Value::Bool(b) => query.bind(*b),
                Value::Null => query.bind(Option::<String>::None),
                _ => query.bind(param.to_string()),
            };
        }

        let rows: Vec<PgRow> = query.fetch_all(&self.pool).await?;

        rows.iter()
            .map(|row| {
                let val = Self::row_to_value(row)?;
                Ok(FlexValue::new(val))
            })
            .collect()
    }

    async fn columns(&self, sql: &str) -> Result<Vec<String>, DataSourceError> {
        // Use LIMIT 0 to get column metadata without fetching rows
        let limited = format!("SELECT * FROM ({sql}) AS _cols LIMIT 0");
        let row = sqlx::query(&limited).fetch_optional(&self.pool).await?;

        // If no rows, try to get columns from the query descriptor
        match row {
            Some(r) => Ok(r.columns().iter().map(|c| c.name().to_string()).collect()),
            None => {
                // Execute and immediately check columns
                let rows: Vec<PgRow> = sqlx::query(&limited).fetch_all(&self.pool).await?;
                if rows.is_empty() {
                    // Can't determine columns without at least metadata
                    Ok(vec![])
                } else {
                    Ok(rows[0]
                        .columns()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect())
                }
            }
        }
    }

    async fn count(&self, sql: &str) -> Result<u64, DataSourceError> {
        let count_sql = format!("SELECT COUNT(*) AS cnt FROM ({sql}) AS _count");
        let row: PgRow = sqlx::query(&count_sql).fetch_one(&self.pool).await?;
        let cnt: i64 = row
            .try_get("cnt")
            .map_err(|e| DataSourceError::QueryFailed(e.to_string()))?;
        Ok(cnt as u64)
    }
}
