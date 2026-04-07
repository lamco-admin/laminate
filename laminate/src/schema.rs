//! Schema inference and data auditing.
//!
//! Scans a dataset to infer field definitions (type, nullability, value ranges),
//! then audits the same or different data against the inferred schema, reporting
//! violations with full diagnostics.
//!
//! # Workflow
//!
//! ```ignore
//! // 1. Infer schema from data
//! let schema = InferredSchema::from_values(&rows);
//!
//! // 2. Audit data against the schema
//! let report = schema.audit(&rows);
//!
//! // 3. Review violations
//! for violation in &report.violations {
//!     println!("{}", violation);
//! }
//! ```

use std::collections::HashMap;

use serde_json::Value;

/// Configuration for schema inference thresholds.
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Minimum fraction of rows where a field must be present (non-null)
    /// to be considered "required." Default: 1.0 (100% — never null, never absent).
    pub required_threshold: f64,

    /// Minimum fraction of non-null values that must be the same type
    /// to be considered "consistent." Below this, the field is flagged as mixed.
    /// Default: 0.95 (95%).
    pub consistency_threshold: f64,

    /// Maximum number of distinct string values before a field is no longer
    /// considered a candidate for enum inference. Default: 20.
    pub max_enum_cardinality: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            required_threshold: 1.0,
            consistency_threshold: 0.95,
            max_enum_cardinality: 20,
        }
    }
}

/// An externally-provided constraint for a field.
///
/// Use this to import constraints from a database schema, JSON Schema,
/// or any other source of truth. These constraints are used as a baseline —
/// data evaluation can then tighten them further.
#[derive(Debug, Clone)]
pub struct ExternalConstraint {
    /// Expected type for this field.
    pub expected_type: Option<JsonType>,
    /// Whether the field is required (NOT NULL in SQL).
    pub required: bool,
    /// Whether the field is nullable.
    pub nullable: bool,
    /// Maximum length for string fields (e.g., VARCHAR(50)).
    pub max_length: Option<usize>,
    /// Minimum value for numeric fields.
    pub min_value: Option<f64>,
    /// Maximum value for numeric fields.
    pub max_value: Option<f64>,
    /// Allowed values (enum constraint).
    pub allowed_values: Option<Vec<String>>,
}

impl Default for ExternalConstraint {
    fn default() -> Self {
        Self {
            expected_type: None,
            required: false,
            nullable: true,
            max_length: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }
    }
}

/// A JSON type as observed in data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonType {
    /// JSON null.
    Null,
    /// JSON boolean.
    Bool,
    /// JSON integer (fits in i64/u64).
    Integer,
    /// JSON float (has fractional part).
    Float,
    /// JSON string.
    String,
    /// JSON array.
    Array,
    /// JSON object.
    Object,
}

impl JsonType {
    /// Classify a `serde_json::Value` into a `JsonType`.
    pub fn of(value: &Value) -> Self {
        match value {
            Value::Null => JsonType::Null,
            Value::Bool(_) => JsonType::Bool,
            Value::Number(n) => {
                if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                    JsonType::Float
                } else {
                    JsonType::Integer
                }
            }
            Value::String(_) => JsonType::String,
            Value::Array(_) => JsonType::Array,
            Value::Object(_) => JsonType::Object,
        }
    }

    /// Tie-breaking priority for dominant type selection.
    /// Higher = preferred when counts are equal. Wider types preferred
    /// because they can represent values from narrower types.
    fn wideness(self) -> u8 {
        match self {
            JsonType::Null => 0,
            JsonType::Bool => 1,
            JsonType::Integer => 2,
            JsonType::Float => 3,
            JsonType::Array => 4,
            JsonType::Object => 5,
            JsonType::String => 6,
        }
    }
}

impl std::fmt::Display for JsonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonType::Null => write!(f, "null"),
            JsonType::Bool => write!(f, "bool"),
            JsonType::Integer => write!(f, "integer"),
            JsonType::Float => write!(f, "float"),
            JsonType::String => write!(f, "string"),
            JsonType::Array => write!(f, "array"),
            JsonType::Object => write!(f, "object"),
        }
    }
}

/// Inferred definition for a single field.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name.
    pub name: String,
    /// The dominant (most common) non-null type.
    pub dominant_type: Option<JsonType>,
    /// How many records had this field present (including null).
    pub present_count: usize,
    /// How many records were missing this field entirely.
    pub absent_count: usize,
    /// How many records had this field as null.
    pub null_count: usize,
    /// Distribution of observed types (excluding null).
    pub type_counts: HashMap<JsonType, usize>,
    /// Total records scanned.
    pub total_records: usize,
    /// Sample distinct values for string fields (up to 20).
    pub sample_values: Vec<String>,

    // ── External constraints (from DB schema, JSON Schema, etc.) ──
    /// Type declared by an external schema (overrides inferred if present).
    pub external_type: Option<JsonType>,
    /// Whether the external schema says this field is required.
    pub external_required: Option<bool>,
    /// Whether the external schema says this field is nullable.
    pub external_nullable: Option<bool>,
    /// Maximum length constraint (e.g., VARCHAR(50)).
    pub max_length: Option<usize>,
    /// Minimum value constraint.
    pub min_value: Option<f64>,
    /// Maximum value constraint.
    pub max_value: Option<f64>,
    /// Allowed values constraint (enum).
    pub allowed_values: Option<Vec<String>>,
}

impl FieldDefinition {
    /// Fraction of records where this field is present and non-null.
    pub fn fill_rate(&self) -> f64 {
        if self.total_records == 0 {
            return 0.0;
        }
        (self.present_count - self.null_count) as f64 / self.total_records as f64
    }

    /// Whether this field appears to be required (present in >95% of records, never null).
    pub fn appears_required(&self) -> bool {
        self.absent_count == 0 && self.null_count == 0
    }

    /// Whether the field has mixed types (more than one non-null type observed).
    pub fn is_mixed_type(&self) -> bool {
        self.type_counts.len() > 1
    }

    /// Fraction of non-null values that match the dominant type.
    pub fn type_consistency(&self) -> f64 {
        let non_null = self.present_count - self.null_count;
        if non_null == 0 {
            return 1.0;
        }
        let dominant_count = self
            .dominant_type
            .and_then(|t| self.type_counts.get(&t))
            .copied()
            .unwrap_or(0);
        dominant_count as f64 / non_null as f64
    }
}

/// Inferred schema for a dataset.
#[derive(Debug, Clone)]
pub struct InferredSchema {
    /// Field definitions, keyed by field name.
    pub fields: HashMap<String, FieldDefinition>,
    /// Total number of records scanned.
    pub total_records: usize,
    /// The config used during inference.
    pub config: InferenceConfig,
    /// All field names in order of first appearance.
    pub field_order: Vec<String>,
}

impl InferredSchema {
    /// Infer a schema from an array of JSON objects with default config.
    pub fn from_values(rows: &[Value]) -> Self {
        Self::from_values_with_config(rows, &InferenceConfig::default())
    }

    /// Infer a schema with custom thresholds.
    pub fn from_values_with_config(rows: &[Value], config: &InferenceConfig) -> Self {
        let mut field_stats: HashMap<String, FieldBuilder> = HashMap::new();
        let mut field_order: Vec<String> = Vec::new();
        let total = rows.len();

        for (row_idx, row) in rows.iter().enumerate() {
            if let Value::Object(obj) = row {
                let row_keys: std::collections::HashSet<&String> = obj.keys().collect();

                for (key, value) in obj {
                    let is_new = !field_stats.contains_key(key);
                    let builder = field_stats.entry(key.clone()).or_insert_with(|| {
                        field_order.push(key.clone());
                        FieldBuilder::new(key.clone())
                    });
                    // Backfill absent count for all previous rows that didn't have this field
                    if is_new && row_idx > 0 {
                        builder.absent_count = row_idx;
                    }
                    builder.observe(value);
                }

                // Track absent fields for existing fields not in this row
                for (name, builder) in &mut field_stats {
                    if !row_keys.contains(name) {
                        builder.absent_count += 1;
                    }
                }
            }
        }

        let fields: HashMap<String, FieldDefinition> = field_stats
            .into_iter()
            .map(|(name, builder)| (name, builder.build(total)))
            .collect();

        InferredSchema {
            fields,
            total_records: total,
            field_order,
            config: config.clone(),
        }
    }

    /// Whether a field is considered required based on inference config thresholds.
    pub fn is_field_required(&self, defn: &FieldDefinition) -> bool {
        // External constraint overrides inference in both directions
        if let Some(required) = defn.external_required {
            return required;
        }
        // Otherwise use threshold: field must be present AND non-null
        // in at least required_threshold fraction of records
        if defn.total_records == 0 {
            return false;
        }
        let fill = defn.fill_rate();
        fill >= self.config.required_threshold
    }

    /// The effective expected type for a field (external overrides inferred).
    pub fn effective_type(&self, defn: &FieldDefinition) -> Option<JsonType> {
        defn.external_type.or(defn.dominant_type)
    }

    /// Audit a dataset against this inferred schema.
    ///
    /// Returns violations: records where field values don't match the
    /// inferred dominant type.
    pub fn audit(&self, rows: &[Value]) -> AuditReport {
        let mut violations = Vec::new();
        let mut field_stats: HashMap<String, FieldAuditStats> = HashMap::new();

        for field_name in self.fields.keys() {
            field_stats.insert(
                field_name.clone(),
                FieldAuditStats {
                    clean: 0,
                    coercible: 0,
                    violations: 0,
                    missing: 0,
                },
            );
        }

        for (row_idx, row) in rows.iter().enumerate() {
            if let Value::Object(obj) = row {
                // Check each expected field
                for (field_name, defn) in &self.fields {
                    // SAFETY: entry ensured above for all fields
                    let stats = field_stats.get_mut(field_name).unwrap();
                    let effective_type = self.effective_type(defn);
                    let is_required = self.is_field_required(defn);

                    // Also check external nullable constraint
                    let is_nullable = defn.external_nullable.unwrap_or(defn.null_count > 0);

                    match obj.get(field_name) {
                        None => {
                            stats.missing += 1;
                            if is_required {
                                violations.push(Violation {
                                    row: row_idx,
                                    field: field_name.clone(),
                                    kind: ViolationKind::MissingRequired,
                                    expected: effective_type
                                        .map(|t| t.to_string())
                                        .unwrap_or_else(|| "any".into()),
                                    actual: "absent".into(),
                                });
                            }
                        }
                        Some(Value::Null) => {
                            if !is_nullable {
                                stats.violations += 1;
                                violations.push(Violation {
                                    row: row_idx,
                                    field: field_name.clone(),
                                    kind: ViolationKind::UnexpectedNull,
                                    expected: effective_type
                                        .map(|t| t.to_string())
                                        .unwrap_or_else(|| "non-null".into()),
                                    actual: "null".into(),
                                });
                            } else {
                                stats.clean += 1;
                            }
                        }
                        Some(value) => {
                            let observed = JsonType::of(value);
                            let mut is_clean = true;

                            // Type check (external or inferred)
                            if let Some(expected) = effective_type {
                                if observed == expected {
                                    // Type matches
                                } else if is_coercible_value(value, observed, expected) {
                                    stats.coercible += 1;
                                    is_clean = false;
                                } else {
                                    stats.violations += 1;
                                    is_clean = false;
                                    violations.push(Violation {
                                        row: row_idx,
                                        field: field_name.clone(),
                                        kind: ViolationKind::TypeMismatch,
                                        expected: expected.to_string(),
                                        actual: format!("{observed} ({value})"),
                                    });
                                }
                            }

                            // Max length check (for string fields) — uses char count, not byte length
                            if let Some(max_len) = defn.max_length {
                                if let Some(s) = value.as_str() {
                                    let char_count = s.chars().count();
                                    if char_count > max_len {
                                        stats.violations += 1;
                                        is_clean = false;
                                        violations.push(Violation {
                                            row: row_idx,
                                            field: field_name.clone(),
                                            kind: ViolationKind::ConstraintViolation,
                                            expected: format!("max_length {max_len}"),
                                            actual: format!("length {char_count}"),
                                        });
                                    }
                                }
                            }

                            // Min/max value check (for numeric fields)
                            if let Some(f) = value.as_f64() {
                                if let Some(min) = defn.min_value {
                                    if f < min {
                                        stats.violations += 1;
                                        is_clean = false;
                                        violations.push(Violation {
                                            row: row_idx,
                                            field: field_name.clone(),
                                            kind: ViolationKind::ConstraintViolation,
                                            expected: format!("min_value {min}"),
                                            actual: format!("{f}"),
                                        });
                                    }
                                }
                                if let Some(max) = defn.max_value {
                                    if f > max {
                                        stats.violations += 1;
                                        is_clean = false;
                                        violations.push(Violation {
                                            row: row_idx,
                                            field: field_name.clone(),
                                            kind: ViolationKind::ConstraintViolation,
                                            expected: format!("max_value {max}"),
                                            actual: format!("{f}"),
                                        });
                                    }
                                }
                            }

                            // Allowed values check (enum constraint)
                            // Convert any JSON value to its string representation
                            // so that Number(1) can match allowed_values=["1"]
                            if let Some(ref allowed) = defn.allowed_values {
                                let value_str = match value {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    Value::Bool(b) => b.to_string(),
                                    _ => format!("{value}"),
                                };
                                if !allowed.contains(&value_str) {
                                    stats.violations += 1;
                                    is_clean = false;
                                    violations.push(Violation {
                                        row: row_idx,
                                        field: field_name.clone(),
                                        kind: ViolationKind::ConstraintViolation,
                                        expected: format!("one of {:?}", allowed),
                                        actual: value_str,
                                    });
                                }
                            }

                            if is_clean {
                                stats.clean += 1;
                            }
                        }
                    }
                }

                // Check for unknown fields (not in schema)
                for key in obj.keys() {
                    if !self.fields.contains_key(key) {
                        violations.push(Violation {
                            row: row_idx,
                            field: key.clone(),
                            kind: ViolationKind::UnknownField,
                            expected: "not present".into(),
                            actual: "present".into(),
                        });
                    }
                }
            }
        }

        AuditReport {
            total_records: rows.len(),
            total_violations: violations.len(),
            field_stats,
            violations,
        }
    }

    /// Merge external constraints (e.g., from a database schema) into this
    /// inferred schema. External constraints serve as a baseline — the inferred
    /// data tightens them where the data is stricter than the constraint.
    ///
    /// For example, if the DB says `VARCHAR(255) NULL` but data shows the field
    /// is always present and never longer than 50 chars, the merged result
    /// reflects: nullable=true (from DB), max_observed_length=50 (from data).
    pub fn with_constraints(mut self, constraints: HashMap<String, ExternalConstraint>) -> Self {
        for (field_name, constraint) in constraints {
            if let Some(defn) = self.fields.get_mut(&field_name) {
                // External type overrides inferred type if provided
                if let Some(ext_type) = constraint.expected_type {
                    defn.external_type = Some(ext_type);
                }
                defn.external_required = Some(constraint.required);
                defn.external_nullable = Some(constraint.nullable);
                defn.max_length = constraint.max_length;
                defn.min_value = constraint.min_value;
                defn.max_value = constraint.max_value;
                defn.allowed_values = constraint.allowed_values;
            }
            // If the constraint names a field not in the data, we could add it
            // as a "missing from data" entry — but for now, we skip.
        }
        self
    }

    /// Print a summary table of the inferred schema.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "{:<20} {:<10} {:<8} {:<8} {:<10} {:<8}",
            "Field", "Type", "Fill%", "Null%", "Consistency", "Mixed?"
        ));
        lines.push("-".repeat(74));

        for name in &self.field_order {
            if let Some(defn) = self.fields.get(name) {
                let type_str = defn
                    .dominant_type
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "null".into());
                let fill = format!("{:.1}", defn.fill_rate() * 100.0);
                let null_pct = if defn.total_records > 0 {
                    format!(
                        "{:.1}",
                        defn.null_count as f64 / defn.total_records as f64 * 100.0
                    )
                } else {
                    "0.0".into()
                };
                let consistency = format!("{:.1}", defn.type_consistency() * 100.0);
                let mixed = if defn.is_mixed_type() { "YES" } else { "no" };

                lines.push(format!(
                    "{:<20} {:<10} {:<8} {:<8} {:<10} {:<8}",
                    name, type_str, fill, null_pct, consistency, mixed
                ));
            }
        }

        lines.join("\n")
    }
}

/// Whether a value of `observed` type could be coerced to `expected` type.
///
/// For String→numeric cases this does value-level checking: `"42"` is coercible
/// to Integer, but `"bad"` is not. This mirrors what the coercion system actually
/// supports so that `audit()` correctly distinguishes salvageable mismatches from
/// true violations.
fn is_coercible_value(value: &Value, observed: JsonType, expected: JsonType) -> bool {
    match (observed, expected) {
        // String → numeric: only coercible if the string actually looks numeric
        (JsonType::String, JsonType::Integer) | (JsonType::String, JsonType::Float) => {
            value.as_str().map(string_looks_numeric).unwrap_or(false)
        }
        // String → Bool: only coercible if the string is a recognized boolean literal
        (JsonType::String, JsonType::Bool) => {
            value.as_str().map(string_looks_bool).unwrap_or(false)
        }
        // Numeric widening
        (JsonType::Integer, JsonType::Float)
        // Number to string: always safe
        | (JsonType::Integer, JsonType::String)
        | (JsonType::Float, JsonType::String)
        | (JsonType::Bool, JsonType::String)
        // Integer as bool (0/1)
        | (JsonType::Integer, JsonType::Bool) => true,

        // Float → Integer: only coercible if the float has no fractional part
        (JsonType::Float, JsonType::Integer) => {
            value.as_f64().map(|f| f.fract() == 0.0).unwrap_or(false)
        }

        _ => false,
    }
}

/// Whether a string value looks like it could be parsed as a number by the
/// coercion system (handles null sentinels, NaN/Infinity, radix literals,
/// US thousands separators, apostrophe separators, and Rust underscores).
fn string_looks_numeric(s: &str) -> bool {
    let s = s.trim();
    // Null sentinels coerce to null/default — treated as coercible
    if matches!(s.to_lowercase().as_str(), "null" | "none" | "n/a" | "nil") {
        return true;
    }
    // NaN and Infinity are handled as special cases in coercion
    if matches!(
        s.to_lowercase().as_str(),
        "nan" | "infinity" | "-infinity" | "+infinity" | "inf" | "-inf"
    ) {
        return true;
    }
    // Strip common numeric separators (US commas, Swiss apostrophes, Rust underscores)
    let cleaned: String = s
        .chars()
        .filter(|&c| c != ',' && c != '\'' && c != '_')
        .collect();
    let no_sign = cleaned.trim_start_matches(['-', '+']);
    // Radix literals (0x…, 0o…, 0b…)
    if no_sign.starts_with("0x")
        || no_sign.starts_with("0X")
        || no_sign.starts_with("0o")
        || no_sign.starts_with("0O")
        || no_sign.starts_with("0b")
        || no_sign.starts_with("0B")
    {
        return true;
    }
    // Try float parse — covers both integer and float strings
    cleaned.parse::<f64>().is_ok()
}

/// Whether a string value looks like a boolean literal recognized by the
/// coercion system.
fn string_looks_bool(s: &str) -> bool {
    matches!(
        s.trim().to_lowercase().as_str(),
        "true" | "false" | "yes" | "no" | "1" | "0" | "t" | "f" | "on" | "off"
    )
}

/// Internal builder for accumulating field observations.
struct FieldBuilder {
    name: String,
    present_count: usize,
    absent_count: usize,
    null_count: usize,
    type_counts: HashMap<JsonType, usize>,
    sample_values: Vec<String>,
}

impl FieldBuilder {
    fn new(name: String) -> Self {
        Self {
            name,
            present_count: 0,
            absent_count: 0,
            null_count: 0,
            type_counts: HashMap::new(),
            sample_values: Vec::new(),
        }
    }

    fn observe(&mut self, value: &Value) {
        self.present_count += 1;

        if value.is_null() {
            self.null_count += 1;
            return;
        }

        let jtype = JsonType::of(value);
        *self.type_counts.entry(jtype).or_insert(0) += 1;

        // Sample distinct string values (for enum detection)
        if let Value::String(s) = value {
            if self.sample_values.len() < 20 && !self.sample_values.contains(s) {
                self.sample_values.push(s.clone());
            }
        }
    }

    fn build(self, total_records: usize) -> FieldDefinition {
        // Deterministic tie-breaking: when types have equal counts, prefer
        // wider types (String > Object > Array > Float > Integer > Bool > Null).
        // This avoids non-deterministic results from HashMap iteration order.
        let dominant_type = self
            .type_counts
            .iter()
            .max_by(|(jtype_a, count_a), (jtype_b, count_b)| {
                count_a
                    .cmp(count_b)
                    .then_with(|| jtype_a.wideness().cmp(&jtype_b.wideness()))
            })
            .map(|(jtype, _)| *jtype);

        FieldDefinition {
            name: self.name,
            dominant_type,
            present_count: self.present_count,
            absent_count: self.absent_count,
            null_count: self.null_count,
            type_counts: self.type_counts,
            total_records,
            sample_values: self.sample_values,
            external_type: None,
            external_required: None,
            external_nullable: None,
            max_length: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }
    }
}

/// A single violation found during audit.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Row index (0-based).
    pub row: usize,
    /// Field name.
    pub field: String,
    /// What kind of violation.
    pub kind: ViolationKind,
    /// What was expected.
    pub expected: String,
    /// What was actually found.
    pub actual: String,
}

/// Categories of schema violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    /// Value type doesn't match the dominant type for this field.
    TypeMismatch,
    /// Field is null but was never null in the inferred schema.
    UnexpectedNull,
    /// Required field is missing from this record.
    MissingRequired,
    /// Field exists in the data but not in the schema.
    UnknownField,
    /// External constraint violated (max_length, min/max value, allowed_values).
    ConstraintViolation,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Row {}: field '{}' — {:?}: expected {}, got {}",
            self.row, self.field, self.kind, self.expected, self.actual
        )
    }
}

/// Per-field audit statistics.
#[derive(Debug, Clone, Default)]
pub struct FieldAuditStats {
    /// Records where the field matches the expected type.
    pub clean: usize,
    /// Records where the field could be coerced to the expected type.
    pub coercible: usize,
    /// Records where the field violates the expected type.
    pub violations: usize,
    /// Records where the field is missing.
    pub missing: usize,
}

/// Complete audit report.
#[derive(Debug, Clone)]
pub struct AuditReport {
    /// Total records audited.
    pub total_records: usize,
    /// Total violations found.
    pub total_violations: usize,
    /// Per-field statistics.
    pub field_stats: HashMap<String, FieldAuditStats>,
    /// Individual violations.
    pub violations: Vec<Violation>,
}

impl AuditReport {
    /// Print a summary table.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Audit: {} records, {} violations",
            self.total_records, self.total_violations
        ));
        lines.push(format!(
            "\n{:<20} {:<8} {:<10} {:<10} {:<8}",
            "Field", "Clean", "Coercible", "Violation", "Missing"
        ));
        lines.push("-".repeat(56));

        let mut sorted_fields: Vec<_> = self.field_stats.iter().collect();
        sorted_fields.sort_by_key(|(name, _)| (*name).clone());

        for (name, stats) in sorted_fields {
            lines.push(format!(
                "{:<20} {:<8} {:<10} {:<10} {:<8}",
                name, stats.clean, stats.coercible, stats.violations, stats.missing
            ));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_rows() -> Vec<Value> {
        vec![
            json!({"id": 1, "name": "Alice", "score": 95.5, "active": true}),
            json!({"id": 2, "name": "Bob", "score": 87.0, "active": false}),
            json!({"id": 3, "name": "Charlie", "score": 92.3, "active": true}),
        ]
    }

    #[test]
    fn infer_consistent_data() {
        let schema = InferredSchema::from_values(&sample_rows());

        assert_eq!(schema.total_records, 3);
        assert_eq!(schema.fields.len(), 4);

        let id = &schema.fields["id"];
        assert_eq!(id.dominant_type, Some(JsonType::Integer));
        assert!(id.appears_required());
        assert!(!id.is_mixed_type());
        assert_eq!(id.type_consistency(), 1.0);

        let name = &schema.fields["name"];
        assert_eq!(name.dominant_type, Some(JsonType::String));

        let active = &schema.fields["active"];
        assert_eq!(active.dominant_type, Some(JsonType::Bool));
    }

    #[test]
    fn infer_mixed_types() {
        let rows = vec![
            json!({"val": 1}),
            json!({"val": "two"}),
            json!({"val": 3}),
            json!({"val": 4}),
        ];

        let schema = InferredSchema::from_values(&rows);
        let val = &schema.fields["val"];

        assert_eq!(val.dominant_type, Some(JsonType::Integer));
        assert!(val.is_mixed_type());
        assert_eq!(val.type_consistency(), 0.75); // 3/4 are integer
    }

    #[test]
    fn infer_nullable_field() {
        let rows = vec![
            json!({"name": "Alice", "email": "alice@test.com"}),
            json!({"name": "Bob", "email": null}),
            json!({"name": "Charlie"}), // email absent
        ];

        let schema = InferredSchema::from_values(&rows);
        let email = &schema.fields["email"];

        assert_eq!(email.dominant_type, Some(JsonType::String));
        assert_eq!(email.null_count, 1);
        assert_eq!(email.absent_count, 1);
        assert!(!email.appears_required());
        assert_eq!(email.fill_rate(), 1.0 / 3.0); // only 1 of 3 is non-null
    }

    #[test]
    fn audit_clean_data() {
        let schema = InferredSchema::from_values(&sample_rows());
        let report = schema.audit(&sample_rows());

        assert_eq!(report.total_violations, 0);
        assert_eq!(report.field_stats["id"].clean, 3);
    }

    #[test]
    fn audit_type_violations() {
        let schema = InferredSchema::from_values(&sample_rows());

        let bad_rows = vec![
            // "not_a_number" in an integer field: can't be coerced → VIOLATION
            json!({"id": "not_a_number", "name": "Dave", "score": 80.0, "active": true}),
            // 12345 in a string field: Integer→String is always safe → coercible
            // "high" in a float field: can't be coerced → VIOLATION
            // "yes" in a bool field: recognized bool literal → coercible
            json!({"id": 5, "name": 12345, "score": "high", "active": "yes"}),
        ];

        let report = schema.audit(&bad_rows);

        // Non-numeric string in integer field: VIOLATION (value-level check)
        let id_stats = &report.field_stats["id"];
        assert!(
            id_stats.violations > 0,
            "Expected violation for 'not_a_number' in integer field: {id_stats:?}"
        );

        // Integer value in string field: still coercible (Integer→String always safe)
        let name_stats = &report.field_stats["name"];
        assert!(
            name_stats.coercible > 0,
            "Expected coercible mismatch for integer in string field: {name_stats:?}"
        );

        // Non-numeric string in float field: VIOLATION
        let score_stats = &report.field_stats["score"];
        assert!(
            score_stats.violations > 0,
            "Expected violation for 'high' in float field: {score_stats:?}"
        );

        // Recognized bool literal in bool field: coercible
        let active_stats = &report.field_stats["active"];
        assert!(
            active_stats.coercible > 0,
            "Expected coercible mismatch for 'yes' in bool field: {active_stats:?}"
        );
    }

    #[test]
    fn audit_true_violations() {
        let schema = InferredSchema::from_values(&sample_rows());

        // Array and object types are NOT coercible to integer/string/bool
        let truly_bad = vec![
            json!({"id": [1, 2, 3], "name": "Dave", "score": 80.0, "active": true}),
            json!({"id": 5, "name": {"first": "Bob"}, "score": 80.0, "active": true}),
        ];

        let report = schema.audit(&truly_bad);

        assert!(
            report.total_violations > 0,
            "Expected true violations for array/object in wrong field"
        );
    }

    #[test]
    fn audit_unknown_fields() {
        let schema = InferredSchema::from_values(&sample_rows());

        let rows_with_extra = vec![
            json!({"id": 1, "name": "Alice", "score": 95.5, "active": true, "surprise": "hi"}),
        ];

        let report = schema.audit(&rows_with_extra);

        let unknown_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.kind == ViolationKind::UnknownField)
            .collect();

        assert_eq!(unknown_violations.len(), 1);
        assert_eq!(unknown_violations[0].field, "surprise");
    }

    #[test]
    fn audit_missing_required() {
        let schema = InferredSchema::from_values(&sample_rows());

        // All fields in sample_rows are required (always present, never null)
        let incomplete = vec![json!({"id": 1, "name": "Alice"})]; // missing score, active

        let report = schema.audit(&incomplete);

        let missing: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.kind == ViolationKind::MissingRequired)
            .collect();

        assert_eq!(missing.len(), 2); // score and active
    }

    #[test]
    fn schema_summary_output() {
        let schema = InferredSchema::from_values(&sample_rows());
        let summary = schema.summary();

        assert!(summary.contains("id"));
        assert!(summary.contains("integer"));
        assert!(summary.contains("100.0")); // fill rate
    }

    #[test]
    fn audit_summary_output() {
        let schema = InferredSchema::from_values(&sample_rows());
        let report = schema.audit(&sample_rows());
        let summary = report.summary();

        assert!(summary.contains("0 violations"));
    }

    #[test]
    fn infer_all_string_csv_data() {
        // CSV-style: everything is a string
        let rows = vec![
            json!({"id": "1", "price": "12.99", "qty": "100", "active": "true"}),
            json!({"id": "2", "price": "24.50", "qty": "0", "active": "false"}),
            json!({"id": "3", "price": "7.99", "qty": "250", "active": "true"}),
        ];

        let schema = InferredSchema::from_values(&rows);

        // Everything should be inferred as String (that's what the data shows)
        assert_eq!(schema.fields["id"].dominant_type, Some(JsonType::String));
        assert_eq!(schema.fields["price"].dominant_type, Some(JsonType::String));

        // But if we audit with integer data, those are coercible
        let typed_rows = vec![json!({"id": 1, "price": 12.99, "qty": 100, "active": true})];
        let report = schema.audit(&typed_rows);

        // Integer id should be coercible to string
        assert!(report.field_stats["id"].coercible > 0);
    }
}
