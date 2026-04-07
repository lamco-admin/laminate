//! SQLite data source for laminate.

use laminate::FlexValue;
use serde_json::Value;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Row, ValueRef};

use crate::{DataSource, DataSourceError};

/// SQLite data source.
///
/// # Example
///
/// ```ignore
/// let source = SqliteSource::connect("sqlite:mydb.db").await?;
/// let rows = source.query("SELECT * FROM products").await?;
/// ```
pub struct SqliteSource {
    pool: SqlitePool,
}

impl SqliteSource {
    /// Connect to SQLite with a connection string.
    ///
    /// Use `"sqlite::memory:"` for an in-memory database.
    pub async fn connect(url: &str) -> Result<Self, DataSourceError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// Create from an existing connection pool.
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a reference to the underlying connection pool.
    ///
    /// Useful for setup operations (creating tables, seeding data) in tests.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn row_to_value(row: &SqliteRow) -> Result<Value, DataSourceError> {
        let columns = row.columns();
        let mut obj = serde_json::Map::with_capacity(columns.len());

        for col in columns {
            let name = col.name().to_string();
            let idx = col.ordinal();

            // SQLite is dynamically typed — try types in order
            let value = if row.try_get_raw(idx).map(|v| v.is_null()).unwrap_or(true) {
                Value::Null
            } else {
                match row.try_get::<i64, _>(idx) {
                    Ok(v) => Value::Number(v.into()),
                    _ => {
                        match row.try_get::<f64, _>(idx) {
                            Ok(v) => serde_json::Number::from_f64(v)
                                .map(Value::Number)
                                .unwrap_or(Value::Null),
                            _ => {
                                match row.try_get::<bool, _>(idx) {
                                    Ok(v) => Value::Bool(v),
                                    _ => {
                                        match row.try_get::<String, _>(idx) {
                                            Ok(v) => {
                                                // Try to detect JSON strings
                                                let trimmed = v.trim();
                                                if (trimmed.starts_with('{')
                                                    && trimmed.ends_with('}'))
                                                    || (trimmed.starts_with('[')
                                                        && trimmed.ends_with(']'))
                                                {
                                                    serde_json::from_str::<Value>(&v)
                                                        .unwrap_or(Value::String(v))
                                                } else {
                                                    Value::String(v)
                                                }
                                            }
                                            _ => Value::Null,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };

            obj.insert(name, value);
        }

        Ok(Value::Object(obj))
    }
}

#[async_trait::async_trait]
impl DataSource for SqliteSource {
    async fn query(&self, sql: &str) -> Result<Vec<FlexValue>, DataSourceError> {
        let rows: Vec<SqliteRow> = sqlx::query(sql).fetch_all(&self.pool).await?;

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

        let rows: Vec<SqliteRow> = query.fetch_all(&self.pool).await?;

        rows.iter()
            .map(|row| {
                let val = Self::row_to_value(row)?;
                Ok(FlexValue::new(val))
            })
            .collect()
    }

    async fn columns(&self, sql: &str) -> Result<Vec<String>, DataSourceError> {
        let limited = format!("SELECT * FROM ({sql}) LIMIT 0");
        let rows: Vec<SqliteRow> = sqlx::query(&limited).fetch_all(&self.pool).await?;
        if rows.is_empty() {
            Ok(vec![])
        } else {
            Ok(rows[0]
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect())
        }
    }

    async fn count(&self, sql: &str) -> Result<u64, DataSourceError> {
        let count_sql = format!("SELECT COUNT(*) AS cnt FROM ({sql})");
        let row: SqliteRow = sqlx::query(&count_sql).fetch_one(&self.pool).await?;
        let cnt: i64 = row
            .try_get("cnt")
            .map_err(|e| DataSourceError::QueryFailed(e.to_string()))?;
        Ok(cnt as u64)
    }
}
