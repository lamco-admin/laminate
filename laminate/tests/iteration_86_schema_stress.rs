#![cfg(feature = "schema")]
/// Iteration 86: Schema inference stress test — 100 fields × 1000 records
///
/// Tests performance and correctness of schema inference at scale.
/// The O(rows × fields²) absent-tracking inner loop may cause slowdown.
use laminate::schema::InferredSchema;

fn generate_records(num_records: usize, num_fields: usize) -> Vec<serde_json::Value> {
    let mut records = Vec::with_capacity(num_records);
    for i in 0..num_records {
        let mut obj = serde_json::Map::new();
        for f in 0..num_fields {
            let key = format!("field_{:03}", f);
            // Every 10th record, skip field_050 to test absent detection
            if f == 50 && i % 10 == 0 {
                continue;
            }
            // Every 20th record, field_099 is null to test null tracking
            if f == 99 && i % 20 == 0 {
                obj.insert(key, serde_json::Value::Null);
                continue;
            }
            // Even fields are numbers, odd are strings
            let val = if f % 2 == 0 {
                serde_json::json!(i as f64 * 1.1 + f as f64)
            } else {
                serde_json::json!(format!("val_{}_{}", f, i))
            };
            obj.insert(key, val);
        }
        records.push(serde_json::Value::Object(obj));
    }
    records
}

#[test]
fn schema_inference_100_fields_1000_records() {
    let records = generate_records(1000, 100);

    let start = std::time::Instant::now();
    let schema = InferredSchema::from_values(&records);
    let elapsed = start.elapsed();

    println!(
        "Schema inference: {} fields, {} records, took {:?}",
        schema.fields.len(),
        schema.total_records,
        elapsed
    );

    // Correctness checks
    assert_eq!(schema.total_records, 1000);
    assert_eq!(
        schema.fields.len(),
        100,
        "all 100 fields should be inferred"
    );

    // field_000 (even) should be Number, present in all 1000 rows
    let f000 = &schema.fields["field_000"];
    println!(
        "field_000: dominant={:?}, present={}, absent={}",
        f000.dominant_type, f000.present_count, f000.absent_count
    );
    assert_eq!(f000.present_count, 1000);
    assert_eq!(f000.absent_count, 0);

    // field_001 (odd) should be String
    let f001 = &schema.fields["field_001"];
    println!(
        "field_001: dominant={:?}, present={}",
        f001.dominant_type, f001.present_count
    );

    // field_050 should have 100 absences (skipped every 10th record out of 1000)
    let f050 = &schema.fields["field_050"];
    println!(
        "field_050: present={}, absent={}",
        f050.present_count, f050.absent_count
    );
    assert_eq!(
        f050.absent_count, 100,
        "field_050 should be absent in 100/1000 records"
    );

    // field_099 should have 50 nulls (null every 20th record out of 1000)
    let f099 = &schema.fields["field_099"];
    println!(
        "field_099: present={}, absent={}, null_count={}",
        f099.present_count, f099.absent_count, f099.null_count
    );
    assert_eq!(f099.null_count, 50, "field_099 should have 50 nulls");

    // Performance: should complete well under 2 seconds
    assert!(
        elapsed.as_secs() < 2,
        "schema inference took {:?} — too slow for 100×1000",
        elapsed
    );
}
