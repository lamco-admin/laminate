//! Iteration 57: SQLite BOOLEAN columns — stored as 0/1 integers.
//!
//! SQLite has no native BOOLEAN type — stores as INTEGER 0/1.
//! Laminate's coercion bridges this: extract::<bool> on 0/1 works.

use laminate_sql::sqlite::SqliteSource;
use laminate_sql::DataSource;

#[tokio::test]
async fn sqlite_boolean_as_integer_coerces_correctly() {
    let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE flags (id INTEGER, active BOOLEAN NOT NULL)")
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO flags VALUES (1, 1)")
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO flags VALUES (2, 0)")
        .execute(source.pool())
        .await
        .unwrap();

    let rows = source
        .query("SELECT * FROM flags ORDER BY id")
        .await
        .unwrap();

    // Active comes out as integer, but extract::<bool> coerces correctly
    let active_true: bool = rows[0].extract("active").unwrap();
    assert!(active_true);

    let active_false: bool = rows[1].extract("active").unwrap();
    assert!(!active_false);

    // Also works as i64
    let active_int: i64 = rows[0].extract("active").unwrap();
    assert_eq!(active_int, 1);
}
