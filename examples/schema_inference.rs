//! Example: Schema inference and data auditing
//!
//! Demonstrates: inferring a schema from sample data, then auditing
//! new data against the inferred schema to find violations.

#[cfg(feature = "schema")]
fn main() {
    use laminate::schema::InferredSchema;
    use serde_json::json;

    println!("=== Schema Inference ===\n");

    // Training data — learn the schema from examples
    let training = vec![
        json!({"name": "Alice", "age": 28, "email": "alice@example.com", "active": true}),
        json!({"name": "Bob", "age": 35, "email": "bob@example.com", "active": false}),
        json!({"name": "Charlie", "age": 42, "email": "charlie@example.com", "active": true}),
        json!({"name": "Diana", "age": 31, "email": "diana@example.com", "active": true}),
    ];

    let schema = InferredSchema::from_values(&training);
    println!("{}", schema.summary());

    // New data — audit against the learned schema
    println!("\n=== Data Audit ===\n");

    let new_data = vec![
        json!({"name": "Eve", "age": 25, "email": "eve@example.com", "active": true}),
        json!({"name": "Frank", "age": "old", "email": "frank@example.com", "active": true}), // type mismatch
        json!({"name": "Grace", "age": 29, "active": false}),                                   // missing email
        json!({"name": "Hank", "age": 45, "email": "hank@example.com", "active": "yes", "extra": 1}), // extra field + wrong type
    ];

    let report = schema.audit(&new_data);
    println!("{}", report.summary());
}

#[cfg(not(feature = "schema"))]
fn main() {
    println!("This example requires the 'schema' feature: cargo run --example schema_inference --features schema");
}
