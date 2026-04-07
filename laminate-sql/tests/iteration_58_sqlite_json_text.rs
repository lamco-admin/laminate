//! Iteration 58: SQLite TEXT columns containing JSON — auto-parsed.

use laminate_sql::sqlite::SqliteSource;
use laminate_sql::DataSource;

#[tokio::test]
async fn sqlite_text_json_auto_parsed() {
    let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE configs (id INTEGER, settings TEXT)")
        .execute(source.pool())
        .await
        .unwrap();

    sqlx::query(r#"INSERT INTO configs VALUES (1, '{"theme":"dark","font_size":14}')"#)
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query(r#"INSERT INTO configs VALUES (2, '["rust","data"]')"#)
        .execute(source.pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO configs VALUES (3, 'plain text')")
        .execute(source.pool())
        .await
        .unwrap();

    let rows = source
        .query("SELECT * FROM configs ORDER BY id")
        .await
        .unwrap();

    // JSON object: deep path navigation works
    let theme: String = rows[0].extract("settings.theme").unwrap();
    assert_eq!(theme, "dark");
    let font: i64 = rows[0].extract("settings.font_size").unwrap();
    assert_eq!(font, 14);

    // JSON array: index access works
    let tag: String = rows[1].extract("settings[0]").unwrap();
    assert_eq!(tag, "rust");

    // Plain text: stays as string
    let note: String = rows[2].extract("settings").unwrap();
    assert_eq!(note, "plain text");
}
