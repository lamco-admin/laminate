//! MySQL data source for laminate.

use laminate::FlexValue;
use serde_json::Value;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use crate::{DataSource, DataSourceError};

/// MySQL/MariaDB data source.
///
/// # Example
///
/// ```ignore
/// let source = MysqlSource::connect("mysql://user:pass@localhost/mydb").await?;
/// let rows = source.query("SELECT * FROM orders LIMIT 100").await?;
/// ```
pub struct MysqlSource {
    pool: MySqlPool,
}

impl MysqlSource {
    /// Connect to MySQL/MariaDB with a connection string.
    pub async fn connect(url: &str) -> Result<Self, DataSourceError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// Create from an existing connection pool.
    pub fn from_pool(pool: MySqlPool) -> Self {
        Self { pool }
    }

    fn row_to_value(row: &MySqlRow) -> Result<Value, DataSourceError> {
        let columns = row.columns();
        let mut obj = serde_json::Map::with_capacity(columns.len());

        for col in columns {
            let name = col.name().to_string();
            let idx = col.ordinal();

            let type_name = col.type_info().name();

            let value = if row.try_get_raw(idx).map(|v| v.is_null()).unwrap_or(true) {
                Value::Null
            } else {
                match type_name {
                    "BOOLEAN" | "TINYINT(1)" => {
                        let v: bool = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        Value::Bool(v)
                    }
                    "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => {
                        let v: i64 = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        Value::Number(v.into())
                    }
                    "FLOAT" | "DOUBLE" => {
                        let v: f64 = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        serde_json::Number::from_f64(v)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }
                    "DECIMAL" | "NUMERIC" => {
                        let v: String = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        Value::String(v)
                    }
                    "JSON" => {
                        let v: Value = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        v
                    }
                    _ => {
                        // Default to string for TEXT, VARCHAR, DATE, DATETIME, TIMESTAMP, BLOB, etc.
                        let v: String = row
                            .try_get(idx)
                            .map_err(|e| DataSourceError::SerializationFailed(e.to_string()))?;
                        Value::String(v)
                    }
                }
            };

            obj.insert(name, value);
        }

        Ok(Value::Object(obj))
    }
}

#[async_trait::async_trait]
impl DataSource for MysqlSource {
    async fn query(&self, sql: &str) -> Result<Vec<FlexValue>, DataSourceError> {
        let rows: Vec<MySqlRow> = sqlx::query(sql).fetch_all(&self.pool).await?;

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

        let rows: Vec<MySqlRow> = query.fetch_all(&self.pool).await?;

        rows.iter()
            .map(|row| {
                let val = Self::row_to_value(row)?;
                Ok(FlexValue::new(val))
            })
            .collect()
    }

    async fn columns(&self, sql: &str) -> Result<Vec<String>, DataSourceError> {
        let limited = format!("SELECT * FROM ({sql}) AS _cols LIMIT 0");
        let rows: Vec<MySqlRow> = sqlx::query(&limited).fetch_all(&self.pool).await?;
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
        let count_sql = format!("SELECT COUNT(*) AS cnt FROM ({sql}) AS _count");
        let row: MySqlRow = sqlx::query(&count_sql).fetch_one(&self.pool).await?;
        let cnt: i64 = row
            .try_get("cnt")
            .map_err(|e| DataSourceError::QueryFailed(e.to_string()))?;
        Ok(cnt as u64)
    }
}
