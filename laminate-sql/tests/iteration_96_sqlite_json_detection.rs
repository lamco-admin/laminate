/// Iteration 96: SQLite JSON string auto-detection — false positive resilience
///
/// row_to_value() auto-detects JSON in string columns by checking brackets
/// then parsing with serde_json. Verified that non-JSON strings like
/// "{smile}" correctly stay as strings, while valid JSON is parsed.
use laminate_sql::sqlite::SqliteSource;
use laminate_sql::DataSource;

#[tokio::test]
async fn sqlite_json_detection_rejects_non_json() {
    let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE profiles (id INTEGER, bio TEXT)")
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO profiles VALUES (1, '{smile}')")
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO profiles VALUES (2, '{not valid json}')")
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query(r#"INSERT INTO profiles VALUES (3, '{"key": "value"}')"#)
        .execute(source.pool())
        .await
        .unwrap();

    let rows = source
        .query("SELECT * FROM profiles ORDER BY id")
        .await
        .unwrap();

    // "{smile}" — looks like JSON but isn't → stays as string
    assert!(rows[0].at("bio").unwrap().is_string());

    // "{not valid json}" — also not valid → stays as string
    assert!(rows[1].at("bio").unwrap().is_string());

    // '{"key": "value"}' — valid JSON → parsed as object
    assert!(rows[2].at("bio").unwrap().is_object());
    let key: String = rows[2].extract("bio.key").unwrap();
    assert_eq!(key, "value");
}
