//! Integration tests for SQLite data source.
//!
//! Uses in-memory SQLite databases — no external dependencies.

#[cfg(feature = "sqlite")]
mod sqlite {
    use laminate::schema::InferredSchema;
    use laminate_sql::{read_json_array, read_jsonl, DataSource, SqliteSource};

    #[tokio::test]
    async fn connect_and_query() {
        let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

        // Create table and insert data
        sqlx::query(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER, active BOOLEAN)",
        )
        .execute(source.pool())
        .await
        .unwrap();

        sqlx::query("INSERT INTO users VALUES (1, 'Alice', 30, 1), (2, 'Bob', 25, 0), (3, 'Charlie', 35, 1)")
            .execute(source.pool())
            .await
            .unwrap();

        let rows = source.query("SELECT * FROM users").await.unwrap();
        assert_eq!(rows.len(), 3);

        // First row
        let name: String = rows[0].extract("name").unwrap();
        assert_eq!(name, "Alice");

        let age: i64 = rows[0].extract("age").unwrap();
        assert_eq!(age, 30);
    }

    #[tokio::test]
    async fn query_with_params() {
        let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

        sqlx::query("CREATE TABLE products (id INTEGER, name TEXT, price REAL)")
            .execute(source.pool())
            .await
            .unwrap();

        sqlx::query("INSERT INTO products VALUES (1, 'Widget', 12.99), (2, 'Gadget', 24.50), (3, 'Doohickey', 3.49)")
            .execute(source.pool())
            .await
            .unwrap();

        let rows = source
            .query_with(
                "SELECT * FROM products WHERE price > ?",
                &[serde_json::json!(10.0)],
            )
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn count_rows() {
        let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

        sqlx::query("CREATE TABLE items (id INTEGER)")
            .execute(source.pool())
            .await
            .unwrap();

        sqlx::query("INSERT INTO items VALUES (1), (2), (3), (4), (5)")
            .execute(source.pool())
            .await
            .unwrap();

        let count = source.count("SELECT * FROM items").await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn schema_inference_from_query() {
        let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE customers (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                score REAL,
                active BOOLEAN
            )",
        )
        .execute(source.pool())
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO customers VALUES
                (1, 'Alice', 'alice@test.com', 95.5, 1),
                (2, 'Bob', NULL, 87.0, 0),
                (3, 'Charlie', 'charlie@test.com', 92.3, 1)",
        )
        .execute(source.pool())
        .await
        .unwrap();

        let rows = source.query("SELECT * FROM customers").await.unwrap();
        let raw_rows: Vec<serde_json::Value> = rows.iter().map(|r| r.raw().clone()).collect();

        let schema = InferredSchema::from_values(&raw_rows);

        assert_eq!(schema.total_records, 3);
        assert!(schema.fields.contains_key("id"));
        assert!(schema.fields.contains_key("name"));
        assert!(schema.fields.contains_key("email"));

        // email should have nulls
        let email = &schema.fields["email"];
        assert!(email.null_count > 0);

        eprintln!("\n{}", schema.summary());
    }

    #[tokio::test]
    async fn audit_query_results() {
        let source = SqliteSource::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE mixed_data (
                id INTEGER,
                value TEXT,
                flag INTEGER
            )",
        )
        .execute(source.pool())
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO mixed_data VALUES
                (1, 'hello', 1),
                (2, '42', 0),
                (3, NULL, 1),
                (4, 'world', NULL)",
        )
        .execute(source.pool())
        .await
        .unwrap();

        let rows = source.query("SELECT * FROM mixed_data").await.unwrap();
        let raw_rows: Vec<serde_json::Value> = rows.iter().map(|r| r.raw().clone()).collect();

        let schema = InferredSchema::from_values(&raw_rows);
        let report = schema.audit(&raw_rows);

        eprintln!("\n{}", schema.summary());
        eprintln!("{}", report.summary());

        // Should detect nulls
        assert!(schema.fields["value"].null_count > 0);
        assert!(schema.fields["flag"].null_count > 0);
    }

    // ── JSON Lines and JSON Array tests (no database needed) ──

    #[test]
    fn read_jsonl_test() {
        let content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}
"#;
        let rows = read_jsonl(content).unwrap();
        assert_eq!(rows.len(), 3);

        let name: String = rows[1].extract("name").unwrap();
        assert_eq!(name, "Bob");
    }

    #[test]
    fn read_json_array_test() {
        let json = r#"[{"id":1},{"id":2},{"id":3}]"#;
        let rows = read_json_array(json).unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn read_jsonl_empty_lines_skipped() {
        let content = "  \n{\"a\":1}\n\n{\"a\":2}\n  \n";
        let rows = read_jsonl(content).unwrap();
        assert_eq!(rows.len(), 2);
    }
}
