//! CLI for laminate — schema inference, data auditing, and JSON inspection.

use clap::{Parser, Subcommand};
use laminate::schema::{ExternalConstraint, InferenceConfig, InferredSchema, JsonType};
use serde_json::Value;
use std::io::Read;

/// laminate — Data, shaped layer by layer.
///
/// CLI tool for data auditing, schema inference, and format inspection.
#[derive(Parser)]
#[command(name = "laminate", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Infer a schema from a JSON data file and display field definitions.
    Infer {
        /// Path to JSON file (array of objects). Use "-" for stdin.
        #[arg(short, long)]
        source: String,

        /// Required field threshold (0.0 to 1.0). Default: 1.0
        #[arg(long, default_value = "1.0")]
        required_threshold: f64,

        /// Output format: table (default) or json
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Audit a JSON data file against an inferred or provided schema.
    Audit {
        /// Path to JSON file (array of objects). Use "-" for stdin.
        #[arg(short, long)]
        source: String,

        /// Path to schema JSON file (optional — infers from data if not provided).
        #[arg(long)]
        schema: Option<String>,

        /// Required field threshold for inference (0.0 to 1.0).
        #[arg(long, default_value = "1.0")]
        required_threshold: f64,

        /// Show individual violations (not just summary).
        #[arg(short, long)]
        verbose: bool,
    },

    /// Inspect a JSON value using FlexValue path navigation.
    Inspect {
        /// Path to JSON file. Use "-" for stdin.
        #[arg(short, long)]
        source: String,

        /// Dot/bracket path to navigate to (e.g., "data.user.name").
        #[arg(short, long)]
        path: Option<String>,

        /// Extract and coerce to a type (string, number, bool).
        #[arg(short = 't', long)]
        as_type: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Infer {
            source,
            required_threshold,
            format,
        } => cmd_infer(&source, required_threshold, &format),
        Commands::Audit {
            source,
            schema,
            required_threshold,
            verbose,
        } => cmd_audit(&source, schema.as_deref(), required_threshold, verbose),
        Commands::Inspect {
            source,
            path,
            as_type,
        } => cmd_inspect(&source, path.as_deref(), as_type.as_deref()),
    }
}

fn read_source(source: &str) -> String {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .expect("failed to read stdin");
        buf
    } else {
        std::fs::read_to_string(source).unwrap_or_else(|e| {
            eprintln!("Error reading {source}: {e}");
            std::process::exit(1);
        })
    }
}

fn parse_rows(json: &str) -> Vec<Value> {
    let value: Value = serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Error parsing JSON: {e}");
        std::process::exit(1);
    });

    match value {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![value], // Single object → array of one
        _ => {
            eprintln!("Expected a JSON array or object");
            std::process::exit(1);
        }
    }
}

fn cmd_infer(source: &str, required_threshold: f64, format: &str) {
    let json = read_source(source);
    let rows = parse_rows(&json);

    let config = InferenceConfig {
        required_threshold,
        ..Default::default()
    };

    let schema = InferredSchema::from_values_with_config(&rows, &config);

    match format {
        "json" => {
            let mut fields = serde_json::Map::new();
            for name in &schema.field_order {
                if let Some(defn) = schema.fields.get(name) {
                    let mut field_obj = serde_json::Map::new();
                    field_obj.insert(
                        "type".into(),
                        Value::String(
                            defn.dominant_type
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "null".into()),
                        ),
                    );
                    field_obj.insert("fill_rate".into(), serde_json::json!(defn.fill_rate()));
                    field_obj.insert("null_count".into(), serde_json::json!(defn.null_count));
                    field_obj.insert(
                        "present_count".into(),
                        serde_json::json!(defn.present_count),
                    );
                    field_obj.insert("absent_count".into(), serde_json::json!(defn.absent_count));
                    field_obj.insert("mixed_type".into(), serde_json::json!(defn.is_mixed_type()));
                    field_obj.insert(
                        "consistency".into(),
                        serde_json::json!(defn.type_consistency()),
                    );
                    if !defn.sample_values.is_empty() {
                        field_obj.insert(
                            "sample_values".into(),
                            serde_json::json!(defn.sample_values),
                        );
                    }
                    fields.insert(name.clone(), Value::Object(field_obj));
                }
            }
            let output = serde_json::json!({
                "total_records": schema.total_records,
                "fields": fields,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!("Schema inferred from {} records:\n", schema.total_records);
            println!("{}", schema.summary());
        }
    }
}

fn cmd_audit(source: &str, schema_path: Option<&str>, required_threshold: f64, verbose: bool) {
    let json = read_source(source);
    let rows = parse_rows(&json);

    let config = InferenceConfig {
        required_threshold,
        ..Default::default()
    };

    let schema = if let Some(path) = schema_path {
        // Load external schema and apply as constraints
        let schema_json = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading schema {path}: {e}");
            std::process::exit(1);
        });
        let schema_val: Value = serde_json::from_str(&schema_json).unwrap_or_else(|e| {
            eprintln!("Error parsing schema JSON: {e}");
            std::process::exit(1);
        });

        // Build constraints from schema file
        let mut constraints = std::collections::HashMap::new();
        if let Some(fields) = schema_val.get("fields").and_then(|f| f.as_object()) {
            for (name, defn) in fields {
                let expected_type = defn
                    .get("type")
                    .and_then(|t| t.as_str())
                    .and_then(parse_json_type);
                let required = defn
                    .get("required")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(false);
                let nullable = defn
                    .get("nullable")
                    .and_then(|n| n.as_bool())
                    .unwrap_or(!required);
                let max_length = defn
                    .get("max_length")
                    .and_then(|m| m.as_u64())
                    .map(|m| m as usize);

                constraints.insert(
                    name.clone(),
                    ExternalConstraint {
                        expected_type,
                        required,
                        nullable,
                        max_length,
                        ..Default::default()
                    },
                );
            }
        }

        InferredSchema::from_values_with_config(&rows, &config).with_constraints(constraints)
    } else {
        InferredSchema::from_values_with_config(&rows, &config)
    };

    let report = schema.audit(&rows);

    println!("{}", report.summary());

    if verbose {
        println!();
        for v in &report.violations {
            println!("  {v}");
        }
    }
}

fn cmd_inspect(source: &str, path: Option<&str>, as_type: Option<&str>) {
    let json = read_source(source);
    let fv = laminate::FlexValue::from_json(&json).unwrap_or_else(|e| {
        eprintln!("Error parsing JSON: {e}");
        std::process::exit(1);
    });

    let target = if let Some(p) = path {
        fv.at(p).unwrap_or_else(|e| {
            eprintln!("Path error: {e}");
            std::process::exit(1);
        })
    } else {
        fv
    };

    match as_type {
        Some("string") => {
            let val: String = target.extract_root().unwrap_or_else(|e| {
                eprintln!("Extract error: {e}");
                std::process::exit(1);
            });
            println!("{val}");
        }
        Some("number" | "int" | "i64") => {
            let val: i64 = target.extract_root().unwrap_or_else(|e| {
                eprintln!("Extract error: {e}");
                std::process::exit(1);
            });
            println!("{val}");
        }
        Some("float" | "f64") => {
            let val: f64 = target.extract_root().unwrap_or_else(|e| {
                eprintln!("Extract error: {e}");
                std::process::exit(1);
            });
            println!("{val}");
        }
        Some("bool") => {
            let val: bool = target.extract_root().unwrap_or_else(|e| {
                eprintln!("Extract error: {e}");
                std::process::exit(1);
            });
            println!("{val}");
        }
        _ => {
            println!("{target}");
        }
    }
}

fn parse_json_type(s: &str) -> Option<JsonType> {
    match s.to_lowercase().as_str() {
        "string" | "text" | "varchar" => Some(JsonType::String),
        "integer" | "int" | "bigint" | "smallint" => Some(JsonType::Integer),
        "float" | "double" | "real" | "numeric" | "decimal" => Some(JsonType::Float),
        "boolean" | "bool" => Some(JsonType::Bool),
        "array" => Some(JsonType::Array),
        "object" | "json" | "jsonb" => Some(JsonType::Object),
        "null" => Some(JsonType::Null),
        _ => None,
    }
}
