//! Iteration 59: SQLite 10K rows — scale test for query + FlexValue conversion.

use laminate_sql::sqlite::SqliteSource;
use laminate_sql::DataSource;

#[tokio::test]
async fn sqlite_10k_rows_query_and_extract() {
    let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE metrics (id INTEGER PRIMARY KEY, sensor TEXT, value REAL, active INTEGER)",
    )
    .execute(source.pool())
    .await
    .unwrap();

    for i in 0..10_000i64 {
        sqlx::query("INSERT INTO metrics VALUES (?, ?, ?, ?)")
            .bind(i)
            .bind(format!("sensor_{}", i % 100))
            .bind(i as f64 * 0.1)
            .bind(if i % 10 == 0 { 0i64 } else { 1i64 })
            .execute(source.pool())
            .await
            .unwrap();
    }

    let rows = source.query("SELECT * FROM metrics").await.unwrap();
    assert_eq!(rows.len(), 10_000);

    // Spot-check
    let last_id: i64 = rows[9999].extract("id").unwrap();
    assert_eq!(last_id, 9999);

    let sensor: String = rows[5000].extract("sensor").unwrap();
    assert_eq!(sensor, "sensor_0");

    // Coercion: integer 0/1 → bool
    let active: bool = rows[0].extract("active").unwrap();
    assert!(!active);
}
