use serde_json::Value;

use crate::coerce::{coerce_for, Coercible, CoercionLevel};
use crate::diagnostic::Diagnostic;
use crate::error::{FlexError, Result};
use crate::path::{parse_path, Segment};

/// A flexible wrapper around `serde_json::Value` providing ergonomic path-based
/// access with type coercion at extraction points.
///
/// This bridges the gap between fully dynamic (`Value`) and fully typed
/// (`#[derive(Deserialize)]`) that no existing Rust crate provides.
///
/// # Examples
///
/// ```
/// use laminate::FlexValue;
///
/// let val = FlexValue::from_json(r#"{"user": {"name": "Alice", "age": "30"}}"#).unwrap();
///
/// // Navigate and extract with coercion
/// let name: String = val.extract("user.name").unwrap();
/// let age: u32 = val.extract("user.age").unwrap();  // "30" coerced to 30
/// assert_eq!(name, "Alice");
/// assert_eq!(age, 30);
/// ```
#[derive(Debug, Clone)]
pub struct FlexValue {
    inner: Value,
    coercion: CoercionLevel,
    /// Whether coercion level was explicitly set by the user (vs defaulted or hint-set).
    coercion_explicit: bool,
    /// Whether pack coercion was explicitly set by the user.
    pack_coercion_explicit: bool,
    /// Optional data source for domain-specific coercions (exchange rates, conversion factors).
    data_source: Option<std::sync::Arc<dyn crate::coerce::CoercionDataSource>>,
    /// Which domain packs participate in coercion during extract().
    pack_coercion: PackCoercion,
}

/// Hint about where data originated — adjusts coercion defaults.
///
/// Different data sources have different type fidelity:
/// - CSV: everything is a string — needs aggressive coercion
/// - JSON API: types are usually correct — minimal coercion needed
/// - Environment variables: always strings — needs coercion
/// - Database: typed but may have SQLite dynamic typing
///
/// ```
/// use laminate::{FlexValue, value::SourceHint};
///
/// // CSV data — everything is strings, so enable full coercion
/// let val = FlexValue::from_json(r#"{"port": "8080"}"#).unwrap()
///     .with_source_hint(SourceHint::Csv);
/// let port: u16 = val.extract("port").unwrap();  // "8080" → 8080
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceHint {
    /// JSON API response — types are usually correct, minimal coercion.
    Json,
    /// CSV file — everything is a string, aggressive coercion needed.
    Csv,
    /// Environment variable — always a string, coerce to target type.
    Env,
    /// HTML form data — all values are strings.
    FormData,
    /// Database result — typed but may have dynamic typing (SQLite).
    Database,
    /// Unknown source — use current coercion level as-is.
    Unknown,
}

/// Controls which domain packs participate in the extract() coercion pipeline.
///
/// When enabled, `extract::<f64>("price")` on `"$12.99"` will strip the
/// currency symbol and return `12.99`. Without pack coercion, it would
/// require calling `parse_currency()` separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackCoercion {
    /// No pack coercion — packs must be called explicitly (default for Exact/SafeWidening).
    None,
    /// Currency pack: strip symbols, detect locale formats.
    Currency,
    /// Units pack: strip unit suffixes, return numeric value.
    Units,
    /// All packs: currency + units + time detection.
    All,
}

impl PartialEq for FlexValue {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.coercion == other.coercion
    }
}

impl serde::Serialize for FlexValue {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl FlexValue {
    /// Construct from any `serde_json::Value`.
    pub fn new(value: Value) -> Self {
        Self {
            inner: value,
            coercion: CoercionLevel::BestEffort,
            coercion_explicit: false,
            data_source: None,
            pack_coercion: PackCoercion::None,
            pack_coercion_explicit: false,
        }
    }

    /// Parse from a JSON string.
    ///
    /// Strips a leading UTF-8 BOM if present (common in files from Windows Notepad, Excel, etc.).
    pub fn from_json(json: &str) -> Result<Self> {
        let json = json.strip_prefix('\u{FEFF}').unwrap_or(json);
        let value: Value = serde_json::from_str(json).map_err(|e| FlexError::DeserializeError {
            path: "(root)".into(),
            source: e,
        })?;
        Ok(Self::new(value))
    }

    /// Set the coercion level for this value and all values navigated from it.
    pub fn with_coercion(mut self, level: CoercionLevel) -> Self {
        self.coercion = level;
        self.coercion_explicit = true;
        self
    }

    /// Attach a data source for domain-specific coercions.
    ///
    /// The data source provides exchange rates, unit conversion factors,
    /// and domain-specific lookups that the coercion engine can use.
    ///
    /// ```ignore
    /// let rates = MyExchangeRates::new();
    /// let val = FlexValue::from_json(data)?
    ///     .with_data_source(rates);
    /// ```
    pub fn with_data_source<D: crate::coerce::CoercionDataSource + 'static>(
        mut self,
        source: D,
    ) -> Self {
        self.data_source = Some(std::sync::Arc::new(source));
        self
    }

    /// Get the attached data source, if any.
    pub fn data_source(&self) -> Option<&dyn crate::coerce::CoercionDataSource> {
        self.data_source.as_deref()
    }

    /// Set source hint — adjusts coercion level based on data origin.
    ///
    /// - `Csv`, `Env`, `FormData` → promotes to BestEffort (everything is strings)
    /// - `Json`, `Database` → keeps current coercion level
    /// - `Unknown` → no change
    ///
    /// Also enables pack coercion for string-heavy sources (CSV/Env/FormData).
    pub fn with_source_hint(mut self, hint: SourceHint) -> Self {
        match hint {
            SourceHint::Csv | SourceHint::Env | SourceHint::FormData => {
                // Only set coercion if user hasn't explicitly set it.
                // Explicit with_coercion() takes precedence over source hints.
                if !self.coercion_explicit {
                    self.coercion = CoercionLevel::BestEffort;
                }
                if !self.pack_coercion_explicit && self.pack_coercion == PackCoercion::None {
                    self.pack_coercion = PackCoercion::All;
                }
            }
            SourceHint::Json | SourceHint::Database | SourceHint::Unknown => {
                // Keep current settings
            }
        }
        self
    }

    /// Enable domain pack coercion during extract().
    ///
    /// When enabled, `extract::<f64>("price")` on `"$12.99"` will strip
    /// the currency symbol and return `12.99`.
    ///
    /// ```
    /// use laminate::{FlexValue, value::PackCoercion};
    ///
    /// let val = FlexValue::from_json(r#"{"price": "$12.99"}"#).unwrap()
    ///     .with_pack_coercion(PackCoercion::All);
    /// let price: f64 = val.extract("price").unwrap();  // 12.99
    /// ```
    pub fn with_pack_coercion(mut self, packs: PackCoercion) -> Self {
        self.pack_coercion = packs;
        self.pack_coercion_explicit = true;
        self
    }

    /// Set coercion level from a Mode's default.
    ///
    /// Lenient → BestEffort, Absorbing → SafeWidening, Strict → Exact.
    pub fn with_mode<M: crate::mode::Mode>(self) -> Self {
        self.with_coercion(M::default_coercion())
    }

    /// Set coercion level from a DynamicMode.
    pub fn with_dynamic_mode(self, mode: crate::mode::DynamicMode) -> Self {
        self.with_coercion(mode.default_coercion())
    }

    /// Navigate to a nested value using dot/bracket path syntax.
    ///
    /// Path syntax:
    ///   - `"foo"` — object key
    ///   - `"foo.bar"` — nested object key
    ///   - `"foo[0]"` — array index
    ///   - `"foo[0].bar.baz[2]"` — mixed
    ///   - `"meta[\"content-type\"]"` — quoted keys for special chars
    ///
    /// Returns `FlexError::PathNotFound` if any segment is missing.
    pub fn at(&self, path: &str) -> Result<FlexValue> {
        let segments = parse_path(path)?;
        let mut current = &self.inner;
        // Holds a parsed Value when we cross a stringified-JSON boundary,
        // so `current` can borrow from it for the remaining segments.
        let mut parsed_holder: Option<Value>;

        for (i, segment) in segments.iter().enumerate() {
            match segment {
                Segment::Key(key) => {
                    if !current.is_object() && !current.is_array() {
                        // Transparent stringified-JSON boundary: if the value is a
                        // string containing JSON, parse it and continue navigating.
                        if let Some(s) = current.as_str() {
                            let trimmed = s.trim();
                            if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                                || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                            {
                                if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                                    parsed_holder = Some(parsed);
                                    // SAFETY: just assigned Some above
                                    current = parsed_holder.as_ref().unwrap();
                                    // Now retry this segment on the parsed value
                                    current = current.get(key.as_str()).ok_or_else(|| {
                                        FlexError::PathNotFound {
                                            path: path_up_to(&segments, i, path),
                                        }
                                    })?;
                                    continue;
                                }
                            }
                        }
                        return Err(FlexError::TypeMismatch {
                            path: path_up_to(&segments, i, path),
                            expected: "object".into(),
                            actual: value_type_name(current).into(),
                        });
                    }
                    current = current
                        .get(key.as_str())
                        .ok_or_else(|| FlexError::PathNotFound {
                            path: path_up_to(&segments, i, path),
                        })?;
                }
                Segment::Index(idx) => {
                    if !current.is_array() {
                        // Same stringified-JSON boundary crossing for arrays
                        if let Some(s) = current.as_str() {
                            let trimmed = s.trim();
                            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                                if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                                    parsed_holder = Some(parsed);
                                    // SAFETY: just assigned Some above
                                    current = parsed_holder.as_ref().unwrap();
                                    let arr = current.as_array().ok_or_else(|| {
                                        FlexError::TypeMismatch {
                                            path: path_up_to(&segments, i, path),
                                            expected: "array".into(),
                                            actual: value_type_name(current).into(),
                                        }
                                    })?;
                                    current = arr.get(*idx).ok_or_else(|| {
                                        FlexError::IndexOutOfBounds {
                                            path: path_up_to(&segments, i, path),
                                            index: *idx,
                                            len: arr.len(),
                                        }
                                    })?;
                                    continue;
                                }
                            }
                        }
                    }
                    let arr = current.as_array().ok_or_else(|| FlexError::TypeMismatch {
                        path: path_up_to(&segments, i, path),
                        expected: "array".into(),
                        actual: value_type_name(current).into(),
                    })?;
                    current = arr.get(*idx).ok_or_else(|| FlexError::IndexOutOfBounds {
                        path: path_up_to(&segments, i, path),
                        index: *idx,
                        len: arr.len(),
                    })?;
                }
            }
        }

        Ok(FlexValue {
            inner: current.clone(),
            coercion: self.coercion,
            data_source: self.data_source.clone(),
            coercion_explicit: self.coercion_explicit,
            pack_coercion: self.pack_coercion,
            pack_coercion_explicit: self.pack_coercion_explicit,
        })
    }

    /// Navigate to a path and deserialize with coercion into a concrete type.
    ///
    /// Uses the `Coercible` trait to determine what coercions are valid for
    /// the target type. Works correctly with primitives, `Option<T>`, `Vec<T>`,
    /// `String`, and custom types (which skip coercion and fall through to serde).
    pub fn extract<T: Coercible>(&self, path: &str) -> Result<T> {
        let target = self.at(path)?;
        // Re-map "(root)" in errors to the actual navigation path so callers get
        // useful path context (e.g., "value 1.3T overflows i32 range" at "[0].market_cap"
        // instead of at "(root)").
        target.extract_root().map_err(|e| match e {
            FlexError::CoercionFailed {
                path: ref p,
                ref detail,
            } if p == "(root)" => FlexError::CoercionFailed {
                path: path.into(),
                detail: detail.clone(),
            },
            FlexError::DeserializeError {
                path: ref p,
                source,
            } if p == "(root)" => FlexError::DeserializeError {
                path: path.into(),
                source,
            },
            _ => e,
        })
    }

    /// Like `extract`, but returns `None` for missing paths AND null values.
    /// Still returns `Err` for paths that exist but fail to deserialize.
    pub fn maybe<T: Coercible>(&self, path: &str) -> Result<Option<T>> {
        match self.at(path) {
            Ok(val) if val.is_null() => Ok(None),
            Ok(_) => self.extract(path).map(Some),
            Err(FlexError::PathNotFound { .. }) => Ok(None),
            // Out-of-bounds array index is the array equivalent of a missing key —
            // the value at that position is absent, not a caller logic error.
            Err(FlexError::IndexOutOfBounds { .. }) => Ok(None),
            // Null in the middle of a path (e.g., "roi.times" where roi=null) means
            // the value is semantically absent — treat as None, not a hard error.
            Err(FlexError::TypeMismatch { ref actual, .. }) if actual == "null" => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Extract from the root value itself (no path navigation).
    ///
    /// Uses the `Coercible` trait for stable type-aware coercion.
    pub fn extract_root<T: Coercible>(&self) -> Result<T> {
        let coercion_result = coerce_for::<T>(&self.inner, self.coercion, "(root)");

        match serde_json::from_value(coercion_result.value.clone()) {
            Ok(v) => Ok(v),
            Err(serde_err) => {
                // Try pack coercion as fallback if enabled AND coercion level permits it.
                // Exact and SafeWidening should NOT use pack coercion.
                if self.pack_coercion != PackCoercion::None
                    && self.coercion >= CoercionLevel::StringCoercion
                {
                    if let Some(pack_result) = self.try_pack_coercion::<T>("(root)") {
                        return pack_result;
                    }
                }
                // Standard error path
                if !coercion_result.coerced {
                    if let Some(ref d) = coercion_result.diagnostic {
                        if let Some(ref suggestion) = d.suggestion {
                            return Err(FlexError::CoercionFailed {
                                path: d.path.clone(),
                                detail: suggestion.clone(),
                            });
                        }
                    }
                }
                Err(FlexError::DeserializeError {
                    path: "(root)".into(),
                    source: serde_err,
                })
            }
        }
    }

    /// Try domain pack coercion as a fallback.
    /// Returns Some(Ok(value)) if a pack handled it, Some(Err) if pack tried and failed,
    /// None if no pack applies.
    fn try_pack_coercion<T: Coercible>(&self, _path: &str) -> Option<Result<T>> {
        let hint = T::coercion_hint()?;
        let s = self.inner.as_str()?;

        // Try currency pack for numeric targets
        if matches!(hint, "f64" | "f32" | "i64" | "i32" | "u64" | "u32")
            && matches!(
                self.pack_coercion,
                PackCoercion::Currency | PackCoercion::All
            )
        {
            if let Some((amount, _code)) = crate::packs::currency::parse_currency(s) {
                if let Some(num) = serde_json::Number::from_f64(amount) {
                    let val = Value::Number(num);
                    if let Ok(result) = serde_json::from_value::<T>(val) {
                        return Some(Ok(result));
                    }
                }
            }
        }

        // Try units pack for numeric targets
        if matches!(hint, "f64" | "f32" | "i64" | "i32" | "u64" | "u32")
            && matches!(self.pack_coercion, PackCoercion::Units | PackCoercion::All)
        {
            if let Some(uv) = crate::packs::units::parse_unit_value(s) {
                if let Some(num) = serde_json::Number::from_f64(uv.amount) {
                    let val = Value::Number(num);
                    if let Ok(result) = serde_json::from_value::<T>(val) {
                        return Some(Ok(result));
                    }
                }
            }
        }

        None
    }

    /// Extract from the root, also returning any diagnostics produced by coercion.
    pub fn extract_root_with_diagnostics<T: Coercible>(&self) -> Result<(T, Vec<Diagnostic>)> {
        let coercion_result = coerce_for::<T>(&self.inner, self.coercion, "(root)");
        let mut diagnostics = Vec::new();
        if let Some(d) = coercion_result.diagnostic {
            diagnostics.push(d);
        }

        let value = serde_json::from_value(coercion_result.value).map_err(|e| {
            if !coercion_result.coerced {
                if let Some(d) = diagnostics.last() {
                    if let Some(ref suggestion) = d.suggestion {
                        return FlexError::CoercionFailed {
                            path: d.path.clone(),
                            detail: suggestion.clone(),
                        };
                    }
                }
            }
            FlexError::DeserializeError {
                path: "(root)".into(),
                source: e,
            }
        })?;

        Ok((value, diagnostics))
    }

    /// Extract with path, also returning diagnostics.
    pub fn extract_with_diagnostics<T: Coercible>(
        &self,
        path: &str,
    ) -> Result<(T, Vec<Diagnostic>)> {
        let target = self.at(path)?;
        let coercion_result = coerce_for::<T>(&target.inner, self.coercion, path);
        let mut diagnostics = Vec::new();
        if let Some(d) = coercion_result.diagnostic {
            diagnostics.push(d);
        }

        let value = serde_json::from_value(coercion_result.value).map_err(|e| {
            if !coercion_result.coerced {
                if let Some(d) = diagnostics.last() {
                    if let Some(ref suggestion) = d.suggestion {
                        return FlexError::CoercionFailed {
                            path: d.path.clone(),
                            detail: suggestion.clone(),
                        };
                    }
                }
            }
            FlexError::DeserializeError {
                path: path.to_string(),
                source: e,
            }
        })?;

        Ok((value, diagnostics))
    }

    /// Shape this value into a typed result with mode-controlled behavior.
    ///
    /// The mode determines coercion level, error handling, and what happens
    /// with unknown data:
    /// - **Lenient**: BestEffort coercion, errors defaulted, unknowns dropped
    /// - **Absorbing**: SafeWidening coercion, errors collected, unknowns preserved
    /// - **Strict**: Exact coercion, fail on first error, unknowns rejected
    ///
    /// For Lenient mode, extraction failures return `Default::default()` with a diagnostic.
    /// For Strict mode, any coercion or missing data returns `Err`.
    ///
    /// ```
    /// use laminate::{FlexValue, Strict, Lenient};
    ///
    /// let val = FlexValue::from_json(r#"{"port": "8080"}"#).unwrap();
    ///
    /// // Lenient: coerces "8080" → 8080
    /// let port: u16 = val.clone().with_mode::<Lenient>().extract("port").unwrap();
    ///
    /// // Strict: rejects "8080" (string, not number)
    /// let result: Result<u16, _> = val.with_mode::<Strict>().extract("port");
    /// assert!(result.is_err());
    /// ```
    pub fn shape<T, M>(&self, path: &str) -> Result<(T, Vec<Diagnostic>)>
    where
        T: Coercible,
        M: crate::mode::Mode,
    {
        let mode_val = self.clone().with_mode::<M>();
        mode_val.extract_with_diagnostics(path)
    }

    /// Iterate over an array at the given path, yielding `FlexValue` items.
    /// Returns an empty Vec if the path is missing or not an array.
    ///
    /// For lazy evaluation over large arrays, use [`each_iter()`](Self::each_iter).
    pub fn each(&self, path: &str) -> Vec<FlexValue> {
        self.each_iter(path).collect()
    }

    /// Lazily iterate over an array at the given path.
    /// Returns an empty iterator if the path is missing or not an array.
    ///
    /// Unlike [`each()`](Self::each), this doesn't clone the entire array upfront.
    /// Transparently parses stringified JSON arrays (e.g., `"[1,2,3]"` → iterate 3 elements).
    pub fn each_iter(&self, path: &str) -> FlexIter {
        match self.at(path) {
            Ok(val) => match val.inner {
                Value::Array(arr) => FlexIter {
                    items: arr,
                    index: 0,
                    coercion: self.coercion,
                    coercion_explicit: self.coercion_explicit,
                    pack_coercion: self.pack_coercion,
                    pack_coercion_explicit: self.pack_coercion_explicit,
                    data_source: self.data_source.clone(),
                },
                // Transparent stringified-JSON boundary: if the value is a string
                // containing a JSON array, parse and iterate it — same as at() does
                // for path navigation.
                Value::String(ref s) => {
                    let trimmed = s.trim();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(s) {
                            return FlexIter {
                                items: arr,
                                index: 0,
                                coercion: self.coercion,
                                coercion_explicit: self.coercion_explicit,
                                pack_coercion: self.pack_coercion,
                                pack_coercion_explicit: self.pack_coercion_explicit,
                                data_source: self.data_source.clone(),
                            };
                        }
                    }
                    FlexIter::empty(self.coercion)
                }
                _ => FlexIter::empty(self.coercion),
            },
            Err(_) => FlexIter::empty(self.coercion),
        }
    }

    /// Returns `true` if a value exists at the given path (even if null).
    pub fn has(&self, path: &str) -> bool {
        self.at(path).is_ok()
    }

    /// Reference to the underlying `serde_json::Value`.
    pub fn raw(&self) -> &Value {
        &self.inner
    }

    /// Consume and return the underlying `Value`.
    pub fn into_raw(self) -> Value {
        self.inner
    }

    /// Returns `true` if the root value is null.
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Returns `true` if the root value is a string.
    pub fn is_string(&self) -> bool {
        self.inner.is_string()
    }

    /// Returns `true` if the root value is an array.
    pub fn is_array(&self) -> bool {
        self.inner.is_array()
    }

    /// Returns `true` if the root value is an object.
    pub fn is_object(&self) -> bool {
        self.inner.is_object()
    }

    /// Get all keys if this is an object.
    pub fn keys(&self) -> Option<Vec<&str>> {
        self.inner
            .as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
    }

    /// Get array length if this is an array.
    pub fn len(&self) -> Option<usize> {
        match &self.inner {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    /// Returns `true` if this is an empty array or empty object.
    pub fn is_empty(&self) -> Option<bool> {
        match &self.inner {
            Value::Array(arr) => Some(arr.is_empty()),
            Value::Object(obj) => Some(obj.is_empty()),
            _ => None,
        }
    }

    // ── Merge / Overlay ───────────────────────────────────────

    /// Deep merge another FlexValue into this one.
    ///
    /// For objects: keys from `other` override keys in `self`. Nested objects
    /// are merged recursively. For all other types, `other` replaces `self`.
    ///
    /// This is the standard config layering operation:
    /// `base.merge(overrides)` where overrides take precedence.
    ///
    /// # Example
    ///
    /// ```
    /// use laminate::FlexValue;
    ///
    /// let base = FlexValue::from_json(r#"{"a": 1, "b": {"x": 10, "y": 20}}"#).unwrap();
    /// let overlay = FlexValue::from_json(r#"{"b": {"y": 99}, "c": 3}"#).unwrap();
    /// let merged = base.merge(&overlay);
    ///
    /// // a preserved, b.x preserved, b.y overridden, c added
    /// assert_eq!(merged.extract::<i64>("a").unwrap(), 1);
    /// assert_eq!(merged.extract::<i64>("b.x").unwrap(), 10);
    /// assert_eq!(merged.extract::<i64>("b.y").unwrap(), 99);
    /// assert_eq!(merged.extract::<i64>("c").unwrap(), 3);
    /// ```
    pub fn merge(&self, other: &FlexValue) -> FlexValue {
        FlexValue {
            inner: deep_merge_values(&self.inner, &other.inner),
            coercion: self.coercion,
            data_source: self.data_source.clone(),
            coercion_explicit: self.coercion_explicit,
            pack_coercion: self.pack_coercion,
            pack_coercion_explicit: self.pack_coercion_explicit,
        }
    }

    /// Shallow merge: keys from `other` replace entire values in `self`.
    /// Nested objects are NOT recursively merged — they're replaced wholesale.
    pub fn merge_shallow(&self, other: &FlexValue) -> FlexValue {
        FlexValue {
            inner: shallow_merge_values(&self.inner, &other.inner),
            coercion: self.coercion,
            data_source: self.data_source.clone(),
            coercion_explicit: self.coercion_explicit,
            pack_coercion: self.pack_coercion,
            pack_coercion_explicit: self.pack_coercion_explicit,
        }
    }

    /// Merge with diagnostics: returns the merged value plus a list of
    /// what was overridden, added, or preserved.
    pub fn merge_with_diagnostics(
        &self,
        other: &FlexValue,
    ) -> (FlexValue, Vec<crate::diagnostic::Diagnostic>) {
        let mut diagnostics = Vec::new();
        let merged = deep_merge_with_diagnostics(&self.inner, &other.inner, "", &mut diagnostics);
        (
            FlexValue {
                inner: merged,
                coercion: self.coercion,
                data_source: self.data_source.clone(),
                coercion_explicit: self.coercion_explicit,
                pack_coercion: self.pack_coercion,
                pack_coercion_explicit: self.pack_coercion_explicit,
            },
            diagnostics,
        )
    }

    /// Set a value at a dot-path, creating intermediate objects as needed.
    ///
    /// # Example
    ///
    /// ```
    /// use laminate::FlexValue;
    ///
    /// let mut val = FlexValue::from_json(r#"{"a": 1}"#).unwrap();
    /// val.set("b.c", serde_json::json!(42));
    /// assert_eq!(val.extract::<i64>("b.c").unwrap(), 42);
    /// ```
    pub fn set(&mut self, path: &str, value: serde_json::Value) -> Result<()> {
        let segments = crate::path::parse_path(path)?;
        set_at_path(&mut self.inner, &segments, value);
        Ok(())
    }
}

/// Deep merge two JSON values. Objects are recursively merged.
/// For non-object types, `b` replaces `a`.
fn deep_merge_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            let mut merged = a_obj.clone();
            for (key, b_val) in b_obj {
                let merged_val = if let Some(a_val) = a_obj.get(key) {
                    deep_merge_values(a_val, b_val)
                } else {
                    b_val.clone()
                };
                merged.insert(key.clone(), merged_val);
            }
            Value::Object(merged)
        }
        _ => b.clone(),
    }
}

/// Shallow merge: b's top-level keys override a's. No recursion.
fn shallow_merge_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            let mut merged = a_obj.clone();
            for (key, val) in b_obj {
                merged.insert(key.clone(), val.clone());
            }
            Value::Object(merged)
        }
        _ => b.clone(),
    }
}

/// Deep merge with diagnostic tracking.
fn deep_merge_with_diagnostics(
    a: &Value,
    b: &Value,
    path_prefix: &str,
    diagnostics: &mut Vec<crate::diagnostic::Diagnostic>,
) -> Value {
    use crate::diagnostic::{Diagnostic, DiagnosticKind, RiskLevel};

    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            let mut merged = a_obj.clone();
            for (key, b_val) in b_obj {
                let full_path = if path_prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{path_prefix}.{key}")
                };

                if let Some(a_val) = a_obj.get(key) {
                    // Key exists in both — recurse (the recursive call emits
                    // diagnostics for leaf-level changes)
                    let merged_val =
                        deep_merge_with_diagnostics(a_val, b_val, &full_path, diagnostics);
                    merged.insert(key.clone(), merged_val);
                } else {
                    // New key from overlay
                    diagnostics.push(Diagnostic {
                        path: full_path,
                        kind: DiagnosticKind::Preserved { field: key.clone() },
                        risk: RiskLevel::Info,
                        suggestion: Some("new field added by merge".into()),
                    });
                    merged.insert(key.clone(), b_val.clone());
                }
            }
            Value::Object(merged)
        }
        _ => {
            if a != b {
                let diag_path = if path_prefix.is_empty() {
                    "(root)".to_string()
                } else {
                    path_prefix.to_string()
                };
                // Structural replacement (object/array → scalar) is higher risk
                let structural = matches!(a, Value::Object(_) | Value::Array(_))
                    && !matches!(b, Value::Object(_) | Value::Array(_));
                let risk = if structural {
                    RiskLevel::Warning
                } else {
                    RiskLevel::Info
                };
                let suggestion = if structural {
                    "structured value (object/array) replaced by scalar — nested data lost"
                } else {
                    "value replaced by merge"
                };
                diagnostics.push(Diagnostic {
                    path: diag_path,
                    kind: DiagnosticKind::Overridden {
                        from_type: value_type_name(a).to_string(),
                        to_type: value_type_name(b).to_string(),
                    },
                    risk,
                    suggestion: Some(suggestion.into()),
                });
            }
            b.clone()
        }
    }
}

/// Set a value at a path, creating intermediate objects as needed.
fn set_at_path(root: &mut Value, segments: &[crate::path::Segment], value: Value) {
    use crate::path::Segment;

    if segments.is_empty() {
        *root = value;
        return;
    }

    match &segments[0] {
        Segment::Key(key) => {
            if !root.is_object() {
                if root.is_null() {
                    // Null → create object (intermediate creation)
                    *root = Value::Object(serde_json::Map::new());
                } else {
                    // Type conflict: existing value is array/scalar/bool/string
                    // but path expects object. Don't destroy existing data.
                    return;
                }
            }
            // SAFETY: matched Object above (or just assigned Object from Null)
            let obj = root.as_object_mut().unwrap();
            if segments.len() == 1 {
                obj.insert(key.clone(), value);
            } else {
                // Look ahead: create Array if next segment is Index, Object otherwise
                let default = match &segments[1] {
                    Segment::Index(_) => Value::Array(Vec::new()),
                    Segment::Key(_) => Value::Object(serde_json::Map::new()),
                };
                let child = obj.entry(key.clone()).or_insert(default);
                set_at_path(child, &segments[1..], value);
            }
        }
        Segment::Index(idx) => {
            if !root.is_array() {
                if root.is_null() {
                    // Null → create array (intermediate creation)
                    *root = Value::Array(Vec::new());
                } else {
                    // Type conflict: existing value is object/scalar
                    // but path expects array. Don't destroy existing data.
                    return;
                }
            }
            // SAFETY: matched Array above (or just assigned Array from Null)
            let arr = root.as_array_mut().unwrap();
            while arr.len() <= *idx {
                arr.push(Value::Null);
            }
            if segments.len() == 1 {
                arr[*idx] = value;
            } else {
                set_at_path(&mut arr[*idx], &segments[1..], value);
            }
        }
    }
}

/// Iterator over array elements as `FlexValue`.
///
/// Created by [`FlexValue::each_iter()`].
pub struct FlexIter {
    items: Vec<Value>,
    index: usize,
    coercion: CoercionLevel,
    coercion_explicit: bool,
    pack_coercion: PackCoercion,
    pack_coercion_explicit: bool,
    data_source: Option<std::sync::Arc<dyn crate::coerce::CoercionDataSource>>,
}

impl FlexIter {
    fn empty(coercion: CoercionLevel) -> Self {
        Self {
            items: vec![],
            index: 0,
            coercion,
            coercion_explicit: false,
            pack_coercion: PackCoercion::None,
            pack_coercion_explicit: false,
            data_source: None,
        }
    }
}

impl Iterator for FlexIter {
    type Item = FlexValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let val = self.items[self.index].clone();
            self.index += 1;
            Some(FlexValue {
                inner: val,
                coercion: self.coercion,
                coercion_explicit: self.coercion_explicit,
                data_source: self.data_source.clone(),
                pack_coercion: self.pack_coercion,
                pack_coercion_explicit: self.pack_coercion_explicit,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for FlexIter {}

impl From<Value> for FlexValue {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for FlexValue {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

impl std::fmt::Display for FlexValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(&self.inner) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{:?}", self.inner),
        }
    }
}

/// Build a human-readable path string up to and including segment index `i`.
fn path_up_to(segments: &[Segment], i: usize, _original: &str) -> String {
    let mut path = String::new();
    for (j, seg) in segments.iter().enumerate() {
        if j > i {
            break;
        }
        match seg {
            Segment::Key(k) => {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(k);
            }
            Segment::Index(idx) => {
                path.push_str(&format!("[{idx}]"));
            }
        }
    }
    path
}

/// Get a human-readable type name for a JSON value.
fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_and_raw() {
        let v = json!({"a": 1});
        let fv = FlexValue::new(v.clone());
        assert_eq!(fv.raw(), &v);
    }

    #[test]
    fn from_json_string() {
        let fv = FlexValue::from_json(r#"{"a": 1}"#).unwrap();
        assert!(fv.is_object());
    }

    #[test]
    fn navigate_simple_key() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        let name = fv.at("name").unwrap();
        assert_eq!(name.raw(), &json!("Alice"));
    }

    #[test]
    fn navigate_nested() {
        let fv = FlexValue::from_json(r#"{"user": {"name": "Alice", "age": 30}}"#).unwrap();
        let name = fv.at("user.name").unwrap();
        assert_eq!(name.raw(), &json!("Alice"));
    }

    #[test]
    fn navigate_array_index() {
        let fv = FlexValue::from_json(r#"{"items": [10, 20, 30]}"#).unwrap();
        let second = fv.at("items[1]").unwrap();
        assert_eq!(second.raw(), &json!(20));
    }

    #[test]
    fn navigate_complex_path() {
        let fv = FlexValue::from_json(
            r#"{"choices": [{"message": {"tool_calls": [{"function": {"name": "search"}}]}}]}"#,
        )
        .unwrap();
        let name = fv
            .at("choices[0].message.tool_calls[0].function.name")
            .unwrap();
        assert_eq!(name.raw(), &json!("search"));
    }

    #[test]
    fn extract_string() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        let name: String = fv.extract("name").unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn extract_with_string_to_int_coercion() {
        let fv = FlexValue::from_json(r#"{"port": "8080"}"#).unwrap();
        let port: u16 = fv.extract("port").unwrap();
        assert_eq!(port, 8080);
    }

    #[test]
    fn extract_with_string_to_bool_coercion() {
        let fv = FlexValue::from_json(r#"{"debug": "true"}"#).unwrap();
        let debug: bool = fv.extract("debug").unwrap();
        assert!(debug);
    }

    #[test]
    fn maybe_present() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        let name: Option<String> = fv.maybe("name").unwrap();
        assert_eq!(name, Some("Alice".to_string()));
    }

    #[test]
    fn maybe_missing() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        let email: Option<String> = fv.maybe("email").unwrap();
        assert_eq!(email, None);
    }

    #[test]
    fn each_iterates_array() {
        let fv = FlexValue::from_json(r#"{"tags": ["a", "b", "c"]}"#).unwrap();
        let tags = fv.each("tags");
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].raw(), &json!("a"));
    }

    #[test]
    fn each_missing_returns_empty() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        let items = fv.each("items");
        assert!(items.is_empty());
    }

    #[test]
    fn has_returns_true_for_existing() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        assert!(fv.has("name"));
    }

    #[test]
    fn has_returns_false_for_missing() {
        let fv = FlexValue::from_json(r#"{"name": "Alice"}"#).unwrap();
        assert!(!fv.has("email"));
    }

    #[test]
    fn has_returns_true_for_null() {
        let fv = FlexValue::from_json(r#"{"value": null}"#).unwrap();
        assert!(fv.has("value"));
    }

    #[test]
    fn type_checks() {
        assert!(FlexValue::new(json!(null)).is_null());
        assert!(FlexValue::new(json!("hi")).is_string());
        assert!(FlexValue::new(json!([1, 2])).is_array());
        assert!(FlexValue::new(json!({"a": 1})).is_object());
    }

    #[test]
    fn keys_on_object() {
        let fv = FlexValue::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let mut keys = fv.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn len_on_array() {
        let fv = FlexValue::from_json(r#"[1, 2, 3]"#).unwrap();
        assert_eq!(fv.len(), Some(3));
    }

    #[test]
    fn exact_mode_rejects_coercion() {
        let fv = FlexValue::from_json(r#"{"port": "8080"}"#)
            .unwrap()
            .with_coercion(CoercionLevel::Exact);
        let result: std::result::Result<u16, _> = fv.extract("port");
        assert!(result.is_err());
    }

    #[test]
    fn extract_with_diagnostics_reports_coercion() {
        let fv = FlexValue::from_json(r#"{"port": "8080"}"#).unwrap();
        let (port, diagnostics): (u16, _) = fv.extract_with_diagnostics("port").unwrap();
        assert_eq!(port, 8080);
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn display_pretty_prints() {
        let fv = FlexValue::new(json!({"a": 1}));
        let s = fv.to_string();
        assert!(s.contains("\"a\""));
        assert!(s.contains("1"));
    }

    #[test]
    fn into_raw_consumes() {
        let original = json!({"a": 1});
        let fv = FlexValue::new(original.clone());
        assert_eq!(fv.into_raw(), original);
    }

    // ── Real-world fixture: OpenAI tool call response ──
    #[test]
    fn openai_tool_call_extraction() {
        let raw = r#"{
            "id": "chatcmpl-abc123",
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "tc_1",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"London\",\"units\":\"celsius\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 50, "completion_tokens": 30}
        }"#;

        let fv = FlexValue::from_json(raw).unwrap();

        let id: String = fv.extract("id").unwrap();
        assert_eq!(id, "chatcmpl-abc123");

        let func_name: String = fv
            .extract("choices[0].message.tool_calls[0].function.name")
            .unwrap();
        assert_eq!(func_name, "get_weather");

        // Stringified JSON arguments — extract as string, then parse and navigate
        let args_str: String = fv
            .extract("choices[0].message.tool_calls[0].function.arguments")
            .unwrap();
        let args_parsed = FlexValue::from_json(&args_str).unwrap();
        let city: String = args_parsed.extract("city").unwrap();
        assert_eq!(city, "London");

        let prompt_tokens: u64 = fv.extract("usage.prompt_tokens").unwrap();
        assert_eq!(prompt_tokens, 50);
    }

    // ── Real-world fixture: config with string values ──
    #[test]
    fn config_string_coercion() {
        let raw = r#"{
            "server": {
                "port": "8080",
                "workers": "4",
                "debug": "true",
                "host": "0.0.0.0"
            }
        }"#;

        let fv = FlexValue::from_json(raw).unwrap();

        let port: u16 = fv.extract("server.port").unwrap();
        let workers: usize = fv.extract("server.workers").unwrap();
        let debug: bool = fv.extract("server.debug").unwrap();
        let host: String = fv.extract("server.host").unwrap();

        assert_eq!(port, 8080);
        assert_eq!(workers, 4);
        assert!(debug);
        assert_eq!(host, "0.0.0.0");
    }
}
