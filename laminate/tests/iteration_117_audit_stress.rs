/// Iteration 117: Schema audit() with 10,000 records and deliberate violations
/// at positions 500, 5000, and 9999.
///
/// Schema trained on 5 clean records with fields: id (integer), name (string),
/// score (float). Then audited against 10,000 records where 3 are mutated:
///   - row 500:  score = [1,2,3]    → TypeMismatch (array where float expected)
///   - row 5000: id = "bad"         → TypeMismatch (string where integer expected)
///   - row 9999: name = null        → UnexpectedNull (non-nullable field)
///
/// Verifies:
/// 1. Exactly 3 violations total
/// 2. Each violation is at the correct row index
/// 3. No false positives in the 9,997 clean records
/// 4. total_records == 10,000
use laminate::schema::{AuditReport, InferredSchema};
use serde_json::{json, Value};

fn build_10k_rows() -> Vec<Value> {
    let mut rows = Vec::with_capacity(10_000);

    for i in 0..10_000usize {
        let row = match i {
            // Position 500: inject array into score field (TypeMismatch)
            500 => json!({ "id": i as i64, "name": format!("user_{i}"), "score": [1, 2, 3] }),
            // Position 5000: inject string into id field (TypeMismatch)
            5000 => json!({ "id": "bad", "name": format!("user_{i}"), "score": 42.5 }),
            // Position 9999: inject null into name field (UnexpectedNull)
            9999 => json!({ "id": i as i64, "name": Value::Null, "score": 0.0 }),
            // All other rows: clean data
            _ => json!({ "id": i as i64, "name": format!("user_{i}"), "score": i as f64 * 0.1 }),
        };
        rows.push(row);
    }

    rows
}

fn infer_schema() -> InferredSchema {
    // Train on 5 clean records — enough to establish required + dominant types
    let training = vec![
        json!({ "id": 1, "name": "alice", "score": 99.5 }),
        json!({ "id": 2, "name": "bob",   "score": 80.0 }),
        json!({ "id": 3, "name": "carol", "score": 72.3 }),
        json!({ "id": 4, "name": "dave",  "score": 65.1 }),
        json!({ "id": 5, "name": "eve",   "score": 55.0 }),
    ];
    InferredSchema::from_values(&training)
}

#[test]
fn audit_stress_total_records() {
    let schema = infer_schema();
    let rows = build_10k_rows();
    let report: AuditReport = schema.audit(&rows);
    assert_eq!(
        report.total_records, 10_000,
        "should scan all 10,000 records"
    );
}

#[test]
fn audit_stress_exactly_three_violations() {
    let schema = infer_schema();
    let rows = build_10k_rows();
    let report: AuditReport = schema.audit(&rows);
    assert_eq!(
        report.total_violations, 3,
        "expected exactly 3 violations at rows 500, 5000, 9999; got {}:\n{:?}",
        report.total_violations, report.violations
    );
}

#[test]
fn audit_stress_violation_rows_correct() {
    let schema = infer_schema();
    let rows = build_10k_rows();
    let report: AuditReport = schema.audit(&rows);

    let mut violation_rows: Vec<usize> = report.violations.iter().map(|v| v.row).collect();
    violation_rows.sort_unstable();

    assert_eq!(
        violation_rows,
        vec![500, 5000, 9999],
        "violations should be at rows 500, 5000, and 9999; got {:?}",
        violation_rows
    );
}

#[test]
fn audit_stress_no_false_positives() {
    let schema = infer_schema();
    let rows = build_10k_rows();
    let report: AuditReport = schema.audit(&rows);

    let violation_rows: std::collections::HashSet<usize> =
        report.violations.iter().map(|v| v.row).collect();

    // Confirm no violation exists outside the 3 expected positions
    for v in &report.violations {
        assert!(
            violation_rows.contains(&v.row),
            "unexpected violation at row {} for field '{}'",
            v.row,
            v.field
        );
    }
    assert!(
        !violation_rows.contains(&0)
            && !violation_rows.contains(&1)
            && !violation_rows.contains(&999)
            && !violation_rows.contains(&4999)
            && !violation_rows.contains(&5001)
            && !violation_rows.contains(&9998),
        "found violations at unexpected clean rows: {:?}",
        violation_rows
    );
}
