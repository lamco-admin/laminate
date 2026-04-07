//! Medical data pack — lab value conversion, clinical calculations, and clinical notation.
//!
//! Provides analyte-aware unit conversion between US conventional units
//! and international SI units, reference range classification, clinical
//! calculations (BMI, BSA, eGFR, etc.), pharmaceutical notation normalization,
//! HL7 v2 parsing, and FHIR Observation extraction.
//!
//! ```
//! use laminate::packs::medical::{convert_lab_value, classify_lab_value, LabClassification};
//!
//! // Glucose: 126 mg/dL (US) → 7.0 mmol/L (EU)
//! let result = convert_lab_value(126.0, "glucose", "mg/dL", "mmol/L");
//! assert!((result.unwrap() - 7.0).abs() < 0.1);
//!
//! // Clinical classification
//! let class = classify_lab_value(126.0, "glucose", "mg/dL");
//! assert_eq!(class, Some(LabClassification::High));
//! ```

use serde_json::Value;

// ── Configuration ────────────────────────────────────────────

/// Configuration for medical conversion behavior.
#[derive(Debug, Clone)]
pub struct MedicalConfig {
    /// Whether analyte matching is case-insensitive (default: true).
    pub case_insensitive: bool,
    /// Custom analyte aliases (maps alias → canonical name).
    pub aliases: std::collections::HashMap<String, String>,
}

impl Default for MedicalConfig {
    fn default() -> Self {
        Self {
            case_insensitive: true,
            aliases: std::collections::HashMap::new(),
        }
    }
}

// ── Lab Value Conversions ────────────────────────────────────

/// A lab value conversion entry.
struct LabConversion {
    analyte: &'static str,
    aliases: &'static [&'static str],
    us_unit: &'static str,
    si_unit: &'static str,
    factor: f64,
}

/// Built-in lab value conversion table.
/// Sources: GlobalRPH, Labcorp, Mayo Clinic, IFCC
const LAB_CONVERSIONS: &[LabConversion] = &[
    // ── Basic Metabolic Panel ────────────────────────────
    LabConversion {
        analyte: "glucose",
        aliases: &[
            "blood sugar",
            "glu",
            "blood glucose",
            "fasting glucose",
            "fbg",
        ],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.0555,
    },
    LabConversion {
        analyte: "sodium",
        aliases: &["na", "na+"],
        us_unit: "mEq/L",
        si_unit: "mmol/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "potassium",
        aliases: &["k", "k+"],
        us_unit: "mEq/L",
        si_unit: "mmol/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "chloride",
        aliases: &["cl", "cl-"],
        us_unit: "mEq/L",
        si_unit: "mmol/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "bicarbonate",
        aliases: &["hco3", "co2", "total co2"],
        us_unit: "mEq/L",
        si_unit: "mmol/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "bun",
        aliases: &["blood urea nitrogen", "urea nitrogen", "urea"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.357,
    },
    LabConversion {
        analyte: "creatinine",
        aliases: &["creat", "cr", "scr"],
        us_unit: "mg/dL",
        si_unit: "µmol/L",
        factor: 88.4,
    },
    LabConversion {
        analyte: "calcium",
        aliases: &["ca", "ca++", "total calcium"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.25,
    },
    // ── Lipid Panel ──────────────────────────────────────
    LabConversion {
        analyte: "cholesterol",
        aliases: &["total cholesterol", "chol", "tc"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.02586, // 1 mmol/L = 38.67 mg/dL
    },
    LabConversion {
        analyte: "hdl",
        aliases: &["hdl cholesterol", "hdl-c"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.02586, // 1 mmol/L = 38.67 mg/dL
    },
    LabConversion {
        analyte: "ldl",
        aliases: &["ldl cholesterol", "ldl-c"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.02586, // 1 mmol/L = 38.67 mg/dL
    },
    LabConversion {
        analyte: "triglycerides",
        aliases: &["trig", "tg"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.01129, // 1 mmol/L = 88.57 mg/dL
    },
    // ── Complete Blood Count ─────────────────────────────
    LabConversion {
        analyte: "hemoglobin",
        aliases: &["hgb", "hb"],
        us_unit: "g/dL",
        si_unit: "g/L",
        factor: 10.0,
    },
    LabConversion {
        analyte: "hematocrit",
        aliases: &["hct"],
        us_unit: "%",
        si_unit: "L/L",
        factor: 0.01,
    },
    LabConversion {
        analyte: "wbc",
        aliases: &["white blood cells", "leukocytes"],
        us_unit: "K/µL",
        si_unit: "×10⁹/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "platelets",
        aliases: &["plt"],
        us_unit: "K/µL",
        si_unit: "×10⁹/L",
        factor: 1.0,
    },
    // ── Liver Function ───────────────────────────────────
    LabConversion {
        analyte: "alt",
        aliases: &["sgpt", "alanine aminotransferase"],
        us_unit: "U/L",
        si_unit: "µkat/L",
        factor: 0.0167,
    },
    LabConversion {
        analyte: "ast",
        aliases: &["sgot", "aspartate aminotransferase"],
        us_unit: "U/L",
        si_unit: "µkat/L",
        factor: 0.016667, // 1 µkat/L = 60 U/L
    },
    LabConversion {
        analyte: "alp",
        aliases: &["alkaline phosphatase", "alk phos"],
        us_unit: "U/L",
        si_unit: "µkat/L",
        factor: 0.0167,
    },
    LabConversion {
        analyte: "ggt",
        aliases: &["gamma-gt", "gamma glutamyl transferase"],
        us_unit: "U/L",
        si_unit: "µkat/L",
        factor: 0.0167,
    },
    LabConversion {
        analyte: "ldh",
        aliases: &["lactate dehydrogenase"],
        us_unit: "U/L",
        si_unit: "µkat/L",
        factor: 0.0167,
    },
    LabConversion {
        analyte: "bilirubin",
        aliases: &["bili", "total bilirubin", "tbili"],
        us_unit: "mg/dL",
        si_unit: "µmol/L",
        factor: 17.1,
    },
    LabConversion {
        analyte: "albumin",
        aliases: &["alb"],
        us_unit: "g/dL",
        si_unit: "g/L",
        factor: 10.0,
    },
    LabConversion {
        analyte: "total protein",
        aliases: &["tp", "protein"],
        us_unit: "g/dL",
        si_unit: "g/L",
        factor: 10.0,
    },
    // ── Thyroid ──────────────────────────────────────────
    LabConversion {
        analyte: "tsh",
        aliases: &["thyroid stimulating hormone", "thyrotropin"],
        us_unit: "µIU/mL",
        si_unit: "mIU/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "free t4",
        aliases: &["ft4", "free thyroxine"],
        us_unit: "ng/dL",
        si_unit: "pmol/L",
        factor: 12.87,
    },
    LabConversion {
        analyte: "free t3",
        aliases: &["ft3", "free triiodothyronine"],
        us_unit: "pg/mL",
        si_unit: "pmol/L",
        factor: 1.536,
    },
    // ── Diabetes ─────────────────────────────────────────
    LabConversion {
        analyte: "hba1c",
        aliases: &["a1c", "hemoglobin a1c", "glycated hemoglobin"],
        us_unit: "%",
        si_unit: "mmol/mol",
        factor: 10.93, // IFCC = (NGSP% - 2.15) * 10.929 — approximated as linear
    },
    // ── Inflammation ─────────────────────────────────────
    LabConversion {
        analyte: "crp",
        aliases: &["c-reactive protein", "hs-crp"],
        us_unit: "mg/L",
        si_unit: "nmol/L",
        factor: 9.524,
    },
    // ── Iron Panel ───────────────────────────────────────
    LabConversion {
        analyte: "iron",
        aliases: &["serum iron", "fe"],
        us_unit: "µg/dL",
        si_unit: "µmol/L",
        factor: 0.179,
    },
    LabConversion {
        analyte: "ferritin",
        aliases: &[],
        us_unit: "ng/mL",
        si_unit: "µg/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "tibc",
        aliases: &["total iron binding capacity"],
        us_unit: "µg/dL",
        si_unit: "µmol/L",
        factor: 0.179,
    },
    // ── Vitamins ─────────────────────────────────────────
    LabConversion {
        analyte: "vitamin d",
        aliases: &["25-oh vitamin d", "25-hydroxyvitamin d", "calcidiol"],
        us_unit: "ng/mL",
        si_unit: "nmol/L",
        factor: 2.496,
    },
    LabConversion {
        analyte: "vitamin b12",
        aliases: &["b12", "cobalamin"],
        us_unit: "pg/mL",
        si_unit: "pmol/L",
        factor: 0.738,
    },
    LabConversion {
        analyte: "folate",
        aliases: &["folic acid", "vitamin b9"],
        us_unit: "ng/mL",
        si_unit: "nmol/L",
        factor: 2.266,
    },
    // ── Minerals ─────────────────────────────────────────
    LabConversion {
        analyte: "uric acid",
        aliases: &["urate", "ua"],
        us_unit: "mg/dL",
        si_unit: "µmol/L",
        factor: 59.48,
    },
    LabConversion {
        analyte: "phosphorus",
        aliases: &["phosphate", "phos"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.323,
    },
    LabConversion {
        analyte: "magnesium",
        aliases: &["mg++"],
        us_unit: "mg/dL",
        si_unit: "mmol/L",
        factor: 0.411,
    },
    // ── Cardiac ──────────────────────────────────────────
    LabConversion {
        analyte: "troponin i",
        aliases: &["tni", "hs-tni", "troponin"],
        us_unit: "ng/mL",
        si_unit: "µg/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "bnp",
        aliases: &["b-type natriuretic peptide"],
        us_unit: "pg/mL",
        si_unit: "ng/L",
        factor: 1.0,
    },
    LabConversion {
        analyte: "nt-probnp",
        aliases: &["n-terminal pro-bnp"],
        us_unit: "pg/mL",
        si_unit: "ng/L",
        factor: 1.0,
    },
    // ── Tumor Markers ────────────────────────────────────
    LabConversion {
        analyte: "psa",
        aliases: &["prostate specific antigen"],
        us_unit: "ng/mL",
        si_unit: "µg/L",
        factor: 1.0,
    },
    // ── Endocrine ────────────────────────────────────────
    LabConversion {
        analyte: "cortisol",
        aliases: &["serum cortisol"],
        us_unit: "µg/dL",
        si_unit: "nmol/L",
        factor: 27.59,
    },
    LabConversion {
        analyte: "testosterone",
        aliases: &["total testosterone"],
        us_unit: "ng/dL",
        si_unit: "nmol/L",
        factor: 0.0347,
    },
];

// ── Reference Ranges ─────────────────────────────────────────

/// Clinical classification of a lab value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabClassification {
    /// Below normal range.
    Low,
    /// Within normal range.
    Normal,
    /// Above normal range.
    High,
    /// Dangerously low — requires immediate attention.
    CriticalLow,
    /// Dangerously high — requires immediate attention.
    CriticalHigh,
}

/// Reference range for a lab value.
#[derive(Debug, Clone)]
pub struct ReferenceRange {
    pub analyte: &'static str,
    pub unit: &'static str,
    pub low: f64,
    pub high: f64,
    pub critical_low: Option<f64>,
    pub critical_high: Option<f64>,
    pub context: &'static str,
}

/// Adult reference ranges (sex-neutral where possible).
/// Sources: Mayo Clinic, Labcorp, ARUP Laboratories
const REFERENCE_RANGES: &[ReferenceRange] = &[
    ReferenceRange {
        analyte: "glucose",
        unit: "mg/dL",
        low: 70.0,
        high: 100.0,
        critical_low: Some(40.0),
        critical_high: Some(400.0),
        context: "adult_fasting",
    },
    ReferenceRange {
        analyte: "glucose",
        unit: "mmol/L",
        low: 3.9,
        high: 5.6,
        critical_low: Some(2.2),
        critical_high: Some(22.2),
        context: "adult_fasting",
    },
    ReferenceRange {
        analyte: "sodium",
        unit: "mEq/L",
        low: 136.0,
        high: 145.0,
        critical_low: Some(120.0),
        critical_high: Some(160.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "potassium",
        unit: "mEq/L",
        low: 3.5,
        high: 5.0,
        critical_low: Some(2.5),
        critical_high: Some(6.5),
        context: "adult",
    },
    ReferenceRange {
        analyte: "chloride",
        unit: "mEq/L",
        low: 98.0,
        high: 106.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "bicarbonate",
        unit: "mEq/L",
        low: 22.0,
        high: 29.0,
        critical_low: Some(10.0),
        critical_high: Some(40.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "bun",
        unit: "mg/dL",
        low: 7.0,
        high: 20.0,
        critical_low: None,
        critical_high: Some(100.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "creatinine",
        unit: "mg/dL",
        low: 0.7,
        high: 1.3,
        critical_low: None,
        critical_high: Some(10.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "calcium",
        unit: "mg/dL",
        low: 8.5,
        high: 10.5,
        critical_low: Some(6.0),
        critical_high: Some(13.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "cholesterol",
        unit: "mg/dL",
        low: 0.0,
        high: 200.0,
        critical_low: None,
        critical_high: None,
        context: "adult_desirable",
    },
    ReferenceRange {
        analyte: "hdl",
        unit: "mg/dL",
        low: 40.0,
        high: 60.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "ldl",
        unit: "mg/dL",
        low: 0.0,
        high: 100.0,
        critical_low: None,
        critical_high: None,
        context: "adult_optimal",
    },
    ReferenceRange {
        analyte: "triglycerides",
        unit: "mg/dL",
        low: 0.0,
        high: 150.0,
        critical_low: None,
        critical_high: Some(500.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "hemoglobin",
        unit: "g/dL",
        low: 12.0,
        high: 17.5,
        critical_low: Some(7.0),
        critical_high: Some(20.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "hematocrit",
        unit: "%",
        low: 36.0,
        high: 51.0,
        critical_low: Some(20.0),
        critical_high: Some(60.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "wbc",
        unit: "K/µL",
        low: 4.5,
        high: 11.0,
        critical_low: Some(2.0),
        critical_high: Some(30.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "platelets",
        unit: "K/µL",
        low: 150.0,
        high: 400.0,
        critical_low: Some(50.0),
        critical_high: Some(1000.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "alt",
        unit: "U/L",
        low: 7.0,
        high: 56.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "ast",
        unit: "U/L",
        low: 10.0,
        high: 40.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "alp",
        unit: "U/L",
        low: 44.0,
        high: 147.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "bilirubin",
        unit: "mg/dL",
        low: 0.1,
        high: 1.2,
        critical_low: None,
        critical_high: Some(15.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "albumin",
        unit: "g/dL",
        low: 3.5,
        high: 5.5,
        critical_low: Some(1.5),
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "tsh",
        unit: "µIU/mL",
        low: 0.4,
        high: 4.0,
        critical_low: Some(0.01),
        critical_high: Some(100.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "free t4",
        unit: "ng/dL",
        low: 0.8,
        high: 1.8,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "hba1c",
        unit: "%",
        low: 4.0,
        high: 5.6,
        critical_low: None,
        critical_high: Some(14.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "crp",
        unit: "mg/L",
        low: 0.0,
        high: 3.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "iron",
        unit: "µg/dL",
        low: 60.0,
        high: 170.0,
        critical_low: None,
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "ferritin",
        unit: "ng/mL",
        low: 12.0,
        high: 300.0,
        critical_low: None,
        critical_high: Some(1000.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "vitamin d",
        unit: "ng/mL",
        low: 30.0,
        high: 100.0,
        critical_low: Some(10.0),
        critical_high: Some(150.0),
        context: "adult",
    },
    ReferenceRange {
        analyte: "vitamin b12",
        unit: "pg/mL",
        low: 200.0,
        high: 900.0,
        critical_low: Some(100.0),
        critical_high: None,
        context: "adult",
    },
    ReferenceRange {
        analyte: "psa",
        unit: "ng/mL",
        low: 0.0,
        high: 4.0,
        critical_low: None,
        critical_high: Some(100.0),
        context: "adult_male",
    },
    ReferenceRange {
        analyte: "cortisol",
        unit: "µg/dL",
        low: 6.0,
        high: 18.0,
        critical_low: None,
        critical_high: Some(60.0),
        context: "adult_morning",
    },
    ReferenceRange {
        analyte: "uric acid",
        unit: "mg/dL",
        low: 3.0,
        high: 7.0,
        critical_low: None,
        critical_high: Some(12.0),
        context: "adult",
    },
];

/// Look up a reference range for an analyte.
pub fn reference_range(analyte: &str, unit: &str) -> Option<&'static ReferenceRange> {
    let analyte_lower = analyte.to_lowercase();
    let unit_lower = unit.to_lowercase();

    // Try direct match first
    REFERENCE_RANGES
        .iter()
        .find(|r| r.analyte == analyte_lower && r.unit.to_lowercase() == unit_lower)
        .or_else(|| {
            // Try alias match
            let canonical = find_canonical_analyte(&analyte_lower)?;
            REFERENCE_RANGES
                .iter()
                .find(|r| r.analyte == canonical && r.unit.to_lowercase() == unit_lower)
        })
}

/// Classify a lab value against reference ranges.
pub fn classify_lab_value(value: f64, analyte: &str, unit: &str) -> Option<LabClassification> {
    let range = reference_range(analyte, unit)?;

    if let Some(cl) = range.critical_low {
        if value < cl {
            return Some(LabClassification::CriticalLow);
        }
    }
    if let Some(ch) = range.critical_high {
        if value > ch {
            return Some(LabClassification::CriticalHigh);
        }
    }
    if value < range.low {
        Some(LabClassification::Low)
    } else if value > range.high {
        Some(LabClassification::High)
    } else {
        Some(LabClassification::Normal)
    }
}

fn find_canonical_analyte(name: &str) -> Option<&'static str> {
    LAB_CONVERSIONS.iter().find_map(|e| {
        if e.analyte == name || e.aliases.contains(&name) {
            Some(e.analyte)
        } else {
            None
        }
    })
}

// ── Conversion Functions ─────────────────────────────────────

/// Convert a lab value between US conventional and SI units.
pub fn convert_lab_value(value: f64, analyte: &str, from_unit: &str, to_unit: &str) -> Option<f64> {
    convert_lab_value_with_config(
        value,
        analyte,
        from_unit,
        to_unit,
        &MedicalConfig::default(),
    )
}

/// Convert with custom configuration (case sensitivity, aliases).
pub fn convert_lab_value_with_config(
    value: f64,
    analyte: &str,
    from_unit: &str,
    to_unit: &str,
    config: &MedicalConfig,
) -> Option<f64> {
    let analyte_lower = if config.case_insensitive {
        analyte.to_lowercase()
    } else {
        analyte.to_string()
    };
    let from_lower = from_unit.to_lowercase();
    let to_lower = to_unit.to_lowercase();

    let canonical = config
        .aliases
        .get(&analyte_lower)
        .cloned()
        .unwrap_or(analyte_lower.clone());

    let entry = LAB_CONVERSIONS.iter().find(|e| {
        if config.case_insensitive {
            e.analyte == canonical || e.aliases.iter().any(|a| *a == canonical)
        } else {
            e.analyte == analyte || e.aliases.contains(&analyte)
        }
    })?;

    if from_lower == to_lower {
        return Some(value);
    }

    let us_lower = entry.us_unit.to_lowercase();
    let si_lower = entry.si_unit.to_lowercase();

    // HbA1c uses an affine formula: IFCC mmol/mol = (NGSP% - 2.15) × 10.929
    // This is NOT a simple linear factor — it has an offset.
    if canonical == "hba1c" {
        if from_lower == us_lower && to_lower == si_lower {
            return Some((value - 2.15) * 10.929);
        } else if from_lower == si_lower && to_lower == us_lower {
            return Some(value / 10.929 + 2.15);
        }
    }

    if from_lower == us_lower && to_lower == si_lower {
        Some(value * entry.factor)
    } else if from_lower == si_lower && to_lower == us_lower {
        Some(value / entry.factor)
    } else {
        None
    }
}

/// List all known analyte names (canonical + aliases).
pub fn known_analytes() -> Vec<&'static str> {
    let mut names = Vec::new();
    for entry in LAB_CONVERSIONS {
        names.push(entry.analyte);
        names.extend(entry.aliases.iter());
    }
    names
}

// ── Clinical Calculations ────────────────────────────────────

/// Calculate Body Mass Index.
///
/// BMI = weight (kg) / height (m)²
pub fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    if height_m <= 0.0 {
        return 0.0;
    }
    weight_kg / (height_m * height_m)
}

/// Classify BMI according to WHO categories.
pub fn classify_bmi(bmi: f64) -> &'static str {
    match bmi {
        b if b < 16.0 => "severe thinness",
        b if b < 17.0 => "moderate thinness",
        b if b < 18.5 => "mild thinness",
        b if b < 25.0 => "normal",
        b if b < 30.0 => "overweight",
        b if b < 35.0 => "obese class I",
        b if b < 40.0 => "obese class II",
        _ => "obese class III",
    }
}

/// Calculate Body Surface Area using the Du Bois formula.
///
/// BSA (m²) = 0.007184 × weight(kg)^0.425 × height(cm)^0.725
pub fn calculate_bsa(weight_kg: f64, height_cm: f64) -> f64 {
    0.007184 * weight_kg.powf(0.425) * height_cm.powf(0.725)
}

/// Calculate eGFR using CKD-EPI 2021 (race-free) equation.
///
/// eGFR = 142 × min(SCr/κ, 1)^α × max(SCr/κ, 1)^(-1.200) × 0.9938^age × (1.012 if female)
///
/// Where κ = 0.7 (female), 0.9 (male); α = -0.241 (female), -0.302 (male)
pub fn calculate_egfr_ckd_epi(creatinine_mg_dl: f64, age: u32, is_female: bool) -> f64 {
    let (kappa, alpha, sex_factor) = if is_female {
        (0.7, -0.241, 1.012)
    } else {
        (0.9, -0.302, 1.0)
    };

    let scr_ratio = creatinine_mg_dl / kappa;
    let min_term = scr_ratio.min(1.0).powf(alpha);
    let max_term = scr_ratio.max(1.0).powf(-1.200);

    142.0 * min_term * max_term * 0.9938_f64.powf(age as f64) * sex_factor
}

/// Calculate corrected calcium for albumin level.
///
/// Corrected Ca = Total Ca + 0.8 × (4.0 - Albumin)
pub fn calculate_corrected_calcium(total_ca_mg_dl: f64, albumin_g_dl: f64) -> f64 {
    total_ca_mg_dl + 0.8 * (4.0 - albumin_g_dl)
}

/// Calculate anion gap.
///
/// AG = Na - (Cl + HCO3)
pub fn calculate_anion_gap(na: f64, cl: f64, hco3: f64) -> f64 {
    na - (cl + hco3)
}

/// Calculate creatinine clearance using Cockcroft-Gault formula.
///
/// CrCl = ((140 - age) × weight(kg) × (0.85 if female)) / (72 × SCr)
pub fn calculate_creatinine_clearance(
    creatinine_mg_dl: f64,
    age: u32,
    weight_kg: f64,
    is_female: bool,
) -> f64 {
    if creatinine_mg_dl <= 0.0 {
        return 0.0;
    }
    let sex_factor = if is_female { 0.85 } else { 1.0 };
    ((140.0 - age as f64) * weight_kg * sex_factor) / (72.0 * creatinine_mg_dl)
}

// ── Pharmaceutical Normalization ─────────────────────────────

/// Normalize pharmaceutical unit notation.
///
/// Maps common variants to a canonical form:
/// - `"mcg"`, `"ug"`, `"microgram"` → `"µg"`
/// - `"cc"` → `"mL"`
/// - `"IU"`, `"iu"` → `"IU"`
pub fn normalize_pharma_unit(unit: &str) -> String {
    let unit = unit.replace('\u{03BC}', "\u{00B5}");
    match unit.to_lowercase().as_str() {
        "mcg" | "ug" | "microgram" | "micrograms" | "\u{00B5}g" => "\u{00B5}g".to_string(),
        "cc" => "mL".to_string(),
        "iu" | "i.u." => "IU".to_string(),
        "mg" => "mg".to_string(),
        "ml" => "mL".to_string(),
        "g" | "gram" | "grams" => "g".to_string(),
        "l" | "liter" | "litre" => "L".to_string(),
        "meq" => "mEq".to_string(),
        "units" | "unit" => "units".to_string(),
        _ => unit.to_string(),
    }
}

/// Normalize pharmaceutical abbreviations (routes, frequencies, dosage forms).
///
/// Returns the expanded form, or `None` if the abbreviation is not recognized.
pub fn normalize_pharma_abbreviation(abbrev: &str) -> Option<&'static str> {
    match abbrev.to_uppercase().as_str() {
        // Routes of administration
        "PO" => Some("oral"),
        "IV" => Some("intravenous"),
        "IM" => Some("intramuscular"),
        "SQ" | "SC" | "SUBQ" | "SUB-Q" => Some("subcutaneous"),
        "PR" => Some("rectal"),
        "SL" => Some("sublingual"),
        "TOP" | "TOPICAL" => Some("topical"),
        "INH" => Some("inhaled"),
        "OD" => Some("right eye"),
        "OS" => Some("left eye"),
        "OU" => Some("both eyes"),
        "GT" | "NGT" => Some("nasogastric tube"),
        "NAS" => Some("nasal"),
        "VAG" => Some("vaginal"),
        "TD" | "TRANSDERMAL" => Some("transdermal"),

        // Frequency
        "QD" | "DAILY" => Some("once daily"),
        "BID" => Some("twice daily"),
        "TID" => Some("three times daily"),
        "QID" => Some("four times daily"),
        "Q4H" => Some("every 4 hours"),
        "Q6H" => Some("every 6 hours"),
        "Q8H" => Some("every 8 hours"),
        "Q12H" => Some("every 12 hours"),
        "QHS" | "HS" => Some("at bedtime"),
        "QAM" => Some("every morning"),
        "QPM" => Some("every evening"),
        "PRN" => Some("as needed"),
        "STAT" => Some("immediately"),
        "QOD" => Some("every other day"),
        "QW" | "WEEKLY" => Some("once weekly"),
        "BIW" => Some("twice weekly"),
        "AC" => Some("before meals"),
        "PC" => Some("after meals"),

        // Dosage forms
        "TAB" | "TABS" => Some("tablet"),
        "CAP" | "CAPS" => Some("capsule"),
        "SUSP" => Some("suspension"),
        "SOLN" | "SOL" => Some("solution"),
        "INJ" => Some("injection"),
        "SUPP" => Some("suppository"),
        "CR" | "ER" | "XR" | "XL" | "SR" | "LA" => Some("extended-release"),
        "DR" | "EC" => Some("delayed-release"),
        "ODT" => Some("orally disintegrating tablet"),
        "MDI" => Some("metered-dose inhaler"),
        "DPI" => Some("dry powder inhaler"),
        "GTT" | "GTTS" => Some("drops"),
        "AMP" => Some("ampule"),
        "ELIX" => Some("elixir"),
        "OINT" => Some("ointment"),
        "LOT" => Some("lotion"),

        _ => None,
    }
}

// ── HL7 v2 Parsing ───────────────────────────────────────────

/// Parse an HL7 v2 packed date/time string.
///
/// Supports formats: `YYYY`, `YYYYMM`, `YYYYMMDD`, `YYYYMMDDHHMMSS`,
/// `YYYYMMDDHHMMSS.SSSS`, `YYYYMMDDHHMMSS.SSSS±ZZZZ`
pub fn parse_hl7_datetime(s: &str) -> Option<String> {
    let s = s.trim();

    // Year only: "2026"
    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = s.parse().ok()?;
        if (1900..=9999).contains(&year) {
            return Some(s.to_string());
        }
        return None;
    }

    // Year-month: "202604"
    if s.len() == 6 && s.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = s[..4].parse().ok()?;
        let month: u32 = s[4..6].parse().ok()?;
        if (1900..=9999).contains(&year) && (1..=12).contains(&month) {
            return Some(format!("{}-{}", &s[..4], &s[4..6]));
        }
        return None;
    }

    if s.len() < 8 || !s[..8].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let year = &s[..4];
    let month = &s[4..6];
    let day = &s[6..8];

    let m: u32 = month.parse().ok()?;
    let d: u32 = day.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }

    if s.len() >= 14 && s[8..14].chars().all(|c| c.is_ascii_digit()) {
        let hour = &s[8..10];
        let min = &s[10..12];
        let sec = &s[12..14];

        // Handle fractional seconds and timezone
        let rest = if s.len() > 14 { &s[14..] } else { "" };

        // Extract fractional seconds
        let (frac, tz_rest) = if let Some(after_dot) = rest.strip_prefix('.') {
            let tz_start = after_dot.find(['+', '-']).unwrap_or(after_dot.len());
            (&rest[..tz_start + 1], &after_dot[tz_start..])
        } else {
            ("", rest)
        };

        // Extract timezone
        let tz = if !tz_rest.is_empty() && tz_rest.len() >= 5 {
            format!("{}:{}", &tz_rest[..3], &tz_rest[3..5])
        } else {
            String::new()
        };

        Some(format!(
            "{}-{}-{}T{}:{}:{}{}{}",
            year, month, day, hour, min, sec, frac, tz
        ))
    } else {
        Some(format!("{}-{}-{}", year, month, day))
    }
}

/// Parse an HL7 v2 segment into fields and components.
///
/// HL7 uses `|` as field separator and `^` as component separator.
/// Returns a vector of fields, each field being a vector of components.
///
/// ```ignore
/// let fields = parse_hl7_segment("OBX|1|NM|2345-7^Glucose^LN||126|mg/dL|70-100|H");
/// assert_eq!(fields[0], vec!["OBX"]);
/// assert_eq!(fields[3], vec!["2345-7", "Glucose", "LN"]);
/// ```
pub fn parse_hl7_segment(segment: &str) -> Vec<Vec<String>> {
    segment
        .split('|')
        .map(|field| field.split('^').map(|c| c.to_string()).collect())
        .collect()
}

// ── FHIR Observation Extraction ──────────────────────────────

/// Extracted data from a FHIR Observation resource.
#[derive(Debug, Clone)]
pub struct FhirObservation {
    /// LOINC or other code (e.g., "2345-7").
    pub code: String,
    /// Human-readable display name (e.g., "Glucose").
    pub display: String,
    /// Numeric value.
    pub value: Option<f64>,
    /// Value unit.
    pub unit: Option<String>,
    /// String value (for non-numeric observations).
    pub value_string: Option<String>,
    /// Reference range low.
    pub reference_low: Option<f64>,
    /// Reference range high.
    pub reference_high: Option<f64>,
    /// Observation status (final, preliminary, etc.).
    pub status: String,
    /// When the observation was effective.
    pub effective_datetime: Option<String>,
}

/// Extract a FHIR Observation from a JSON Value.
///
/// Handles the standard FHIR Observation resource structure with
/// `valueQuantity`, `valueString`, `code.coding`, and `referenceRange`.
pub fn extract_fhir_observation(value: &Value) -> Option<FhirObservation> {
    let obj = value.as_object()?;

    // Status
    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Code (from code.coding[0])
    let code_obj = obj.get("code")?;
    let coding = code_obj
        .get("coding")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first());

    let code = coding
        .and_then(|c| c.get("code"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let display = coding
        .and_then(|c| c.get("display"))
        .and_then(|v| v.as_str())
        .or_else(|| code_obj.get("text").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    // Value — try valueQuantity first, then valueString
    let (obs_value, unit) = if let Some(vq) = obj.get("valueQuantity") {
        let v = vq.get("value").and_then(|v| v.as_f64());
        let u = vq
            .get("unit")
            .or_else(|| vq.get("code"))
            .and_then(|v| v.as_str())
            .map(String::from);
        (v, u)
    } else {
        (None, None)
    };

    let value_string = obj
        .get("valueString")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Reference range
    let (ref_low, ref_high) = if let Some(ranges) = obj.get("referenceRange") {
        let range = ranges.as_array().and_then(|arr| arr.first());
        let low = range
            .and_then(|r| r.get("low"))
            .and_then(|l| l.get("value"))
            .and_then(|v| v.as_f64());
        let high = range
            .and_then(|r| r.get("high"))
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_f64());
        (low, high)
    } else {
        (None, None)
    };

    // Effective datetime
    let effective = obj
        .get("effectiveDateTime")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(FhirObservation {
        code,
        display,
        value: obs_value,
        unit,
        value_string,
        reference_low: ref_low,
        reference_high: ref_high,
        status,
        effective_datetime: effective,
    })
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glucose_conversion() {
        let result = convert_lab_value(126.0, "glucose", "mg/dL", "mmol/L").unwrap();
        assert!((result - 7.0).abs() < 0.1);
    }

    #[test]
    fn glucose_reverse() {
        let result = convert_lab_value(7.0, "glucose", "mmol/L", "mg/dL").unwrap();
        assert!((result - 126.0).abs() < 1.0);
    }

    #[test]
    fn creatinine_conversion() {
        let result = convert_lab_value(1.0, "creatinine", "mg/dL", "µmol/L").unwrap();
        assert!((result - 88.4).abs() < 0.1);
    }

    #[test]
    fn alias_matching() {
        assert!(convert_lab_value(126.0, "blood sugar", "mg/dL", "mmol/L").is_some());
        assert!(convert_lab_value(126.0, "GLU", "mg/dL", "mmol/L").is_some());
    }

    #[test]
    fn case_insensitive() {
        assert!(convert_lab_value(126.0, "Glucose", "mg/dL", "mmol/L").is_some());
    }

    #[test]
    fn unknown_analyte() {
        assert!(convert_lab_value(100.0, "unknown_test", "mg/dL", "mmol/L").is_none());
    }

    // ── New analyte tests ────────────────────────────────
    #[test]
    fn free_t4_conversion() {
        let result = convert_lab_value(1.0, "free t4", "ng/dL", "pmol/L").unwrap();
        assert!((result - 12.87).abs() < 0.1);
    }

    #[test]
    fn iron_conversion() {
        let result = convert_lab_value(100.0, "iron", "µg/dL", "µmol/L").unwrap();
        assert!((result - 17.9).abs() < 0.1);
    }

    #[test]
    fn vitamin_d_conversion() {
        let result = convert_lab_value(30.0, "vitamin d", "ng/mL", "nmol/L").unwrap();
        assert!((result - 74.88).abs() < 0.5);
    }

    #[test]
    fn cortisol_conversion() {
        let result = convert_lab_value(10.0, "cortisol", "µg/dL", "nmol/L").unwrap();
        assert!((result - 275.9).abs() < 1.0);
    }

    // ── Reference range tests ────────────────────────────
    #[test]
    fn classify_glucose_normal() {
        assert_eq!(
            classify_lab_value(90.0, "glucose", "mg/dL"),
            Some(LabClassification::Normal)
        );
    }

    #[test]
    fn classify_glucose_high() {
        assert_eq!(
            classify_lab_value(126.0, "glucose", "mg/dL"),
            Some(LabClassification::High)
        );
    }

    #[test]
    fn classify_glucose_critical_high() {
        assert_eq!(
            classify_lab_value(500.0, "glucose", "mg/dL"),
            Some(LabClassification::CriticalHigh)
        );
    }

    #[test]
    fn classify_potassium_critical_low() {
        assert_eq!(
            classify_lab_value(2.0, "potassium", "mEq/L"),
            Some(LabClassification::CriticalLow)
        );
    }

    // ── Clinical calculations ────────────────────────────
    #[test]
    fn bmi_calculation() {
        let bmi = calculate_bmi(70.0, 1.75);
        assert!((bmi - 22.86).abs() < 0.1);
        assert_eq!(classify_bmi(bmi), "normal");
    }

    #[test]
    fn bsa_calculation() {
        let bsa = calculate_bsa(70.0, 175.0);
        assert!((bsa - 1.85).abs() < 0.1);
    }

    #[test]
    fn egfr_calculation() {
        let egfr = calculate_egfr_ckd_epi(1.0, 50, false);
        assert!(egfr > 80.0 && egfr < 110.0); // Normal range for healthy 50yo male
    }

    #[test]
    fn corrected_calcium() {
        let ca = calculate_corrected_calcium(8.0, 2.0);
        assert!((ca - 9.6).abs() < 0.01); // 8.0 + 0.8*(4.0-2.0)
    }

    #[test]
    fn anion_gap() {
        let ag = calculate_anion_gap(140.0, 100.0, 24.0);
        assert!((ag - 16.0).abs() < 0.01);
    }

    #[test]
    fn creatinine_clearance() {
        let crcl = calculate_creatinine_clearance(1.0, 50, 70.0, false);
        assert!(crcl > 80.0 && crcl < 100.0);
    }

    // ── Pharma normalization ─────────────────────────────
    #[test]
    fn pharma_normalization() {
        assert_eq!(normalize_pharma_unit("mcg"), "µg");
        assert_eq!(normalize_pharma_unit("ug"), "µg");
        assert_eq!(normalize_pharma_unit("cc"), "mL");
        assert_eq!(normalize_pharma_unit("IU"), "IU");
    }

    #[test]
    fn pharma_abbreviations() {
        assert_eq!(normalize_pharma_abbreviation("PO"), Some("oral"));
        assert_eq!(normalize_pharma_abbreviation("BID"), Some("twice daily"));
        assert_eq!(normalize_pharma_abbreviation("PRN"), Some("as needed"));
        assert_eq!(normalize_pharma_abbreviation("TAB"), Some("tablet"));
        assert_eq!(normalize_pharma_abbreviation("XYZ"), None);
    }

    // ── HL7 parsing ──────────────────────────────────────
    #[test]
    fn hl7_date() {
        assert_eq!(
            parse_hl7_datetime("20260402"),
            Some("2026-04-02".to_string())
        );
    }

    #[test]
    fn hl7_datetime() {
        assert_eq!(
            parse_hl7_datetime("20260402143022"),
            Some("2026-04-02T14:30:22".to_string())
        );
    }

    #[test]
    fn hl7_datetime_with_fraction() {
        let result = parse_hl7_datetime("20260402143022.1234").unwrap();
        assert_eq!(result, "2026-04-02T14:30:22.1234");
    }

    #[test]
    fn hl7_datetime_with_tz() {
        let result = parse_hl7_datetime("20260402143022.1234-0500").unwrap();
        assert_eq!(result, "2026-04-02T14:30:22.1234-05:00");
    }

    #[test]
    fn hl7_year_only() {
        assert_eq!(parse_hl7_datetime("2026"), Some("2026".to_string()));
    }

    #[test]
    fn hl7_year_month() {
        assert_eq!(parse_hl7_datetime("202604"), Some("2026-04".to_string()));
    }

    #[test]
    fn hl7_segment_parsing() {
        let fields = parse_hl7_segment("OBX|1|NM|2345-7^Glucose^LN||126|mg/dL|70-100|H");
        assert_eq!(fields[0], vec!["OBX"]);
        assert_eq!(fields[3], vec!["2345-7", "Glucose", "LN"]);
        assert_eq!(fields[5], vec!["126"]);
    }

    // ── FHIR extraction ──────────────────────────────────
    #[test]
    fn fhir_observation_extraction() {
        let obs = serde_json::json!({
            "resourceType": "Observation",
            "status": "final",
            "code": {
                "coding": [{"system": "http://loinc.org", "code": "2345-7", "display": "Glucose"}]
            },
            "valueQuantity": {"value": 126, "unit": "mg/dL"},
            "referenceRange": [{"low": {"value": 70}, "high": {"value": 100}}],
            "effectiveDateTime": "2026-04-06T14:30:00Z"
        });

        let result = extract_fhir_observation(&obs).unwrap();
        assert_eq!(result.code, "2345-7");
        assert_eq!(result.display, "Glucose");
        assert_eq!(result.value, Some(126.0));
        assert_eq!(result.unit.as_deref(), Some("mg/dL"));
        assert_eq!(result.reference_low, Some(70.0));
        assert_eq!(result.reference_high, Some(100.0));
        assert_eq!(result.status, "final");
    }
}
