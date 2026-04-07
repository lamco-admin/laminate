//! Iteration 114: CoinGecko real data — null mid-path navigation + extract() path context
//!
//! Two bugs discovered:
//! 1. maybe() returned Err(TypeMismatch) when navigating through null mid-path.
//!    Fixed: TypeMismatch{actual:"null"} is now collapsed to Ok(None) in maybe().
//! 2. extract() reported path "(root)" in coercion errors instead of the actual path.
//!    Fixed: extract() now re-maps "(root)" errors to the original navigation path.

#[cfg(test)]
mod tests {
    use laminate::FlexValue;

    fn coingecko_data() -> FlexValue {
        // Real structure from CoinGecko /coins/markets: roi is null for some coins
        // and an object {times, currency, percentage} for others.
        let data = serde_json::json!([
            {"id":"bitcoin","current_price":65926,"market_cap":1320113536849_i64,"roi":null},
            {"id":"ethereum","current_price":2024.59,"market_cap":244319818673_i64,
             "roi":{"times":40.062692,"currency":"btc","percentage":4006.27}},
            {"id":"tether","current_price":0.999733,"market_cap":184114998901_i64,"roi":null}
        ]);
        FlexValue::new(data)
    }

    /// maybe() with null mid-path should return Ok(None), not Err(TypeMismatch).
    /// This is the common case when iterating heterogeneous API arrays.
    #[test]
    fn maybe_null_midpath_returns_none() {
        let fv = coingecko_data();
        // roi is null for bitcoin — navigating into roi.times should be None
        assert_eq!(fv.maybe::<f64>("[0].roi.times").unwrap(), None);
        assert_eq!(fv.maybe::<f64>("[2].roi.times").unwrap(), None);
        // roi is an object for ethereum — should extract correctly
        assert_eq!(fv.maybe::<f64>("[1].roi.times").unwrap(), Some(40.062692));
    }

    /// extract() through null mid-path still returns TypeMismatch (informative error preserved).
    #[test]
    fn extract_null_midpath_returns_type_mismatch() {
        let fv = coingecko_data();
        let err = fv.extract::<f64>("[0].roi.times").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("TypeMismatch"),
            "expected TypeMismatch, got: {msg}"
        );
        assert!(msg.contains("null"), "expected 'null' in error, got: {msg}");
    }

    /// extract() coercion errors now report the original path, not "(root)".
    #[test]
    fn extract_coercion_error_includes_path() {
        let fv = coingecko_data();
        // market_cap = 1320113536849 — overflows i32::MAX (2147483647)
        let err = fv.extract::<i32>("[0].market_cap").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("[0].market_cap"),
            "path should be '[0].market_cap', got: {msg}"
        );
        assert!(
            !msg.contains("(root)"),
            "path should not be '(root)', got: {msg}"
        );
        assert!(msg.contains("overflows i32"), "got: {msg}");
    }

    /// Schema inference: roi field is nullable Object (null in 2/3, object in 1/3).
    #[test]
    #[cfg(feature = "schema")]
    fn schema_heterogeneous_nullable_object_field() {
        use laminate::schema::InferredSchema;
        let rows = serde_json::json!([
            {"id":"bitcoin","roi":null},
            {"id":"ethereum","roi":{"times":40.06,"currency":"btc"}},
            {"id":"tether","roi":null}
        ]);
        let schema = InferredSchema::from_values(rows.as_array().unwrap());
        let roi = schema.fields.get("roi").expect("roi field missing");
        assert_eq!(roi.null_count, 2);
        assert!(
            !roi.is_mixed_type(),
            "roi is not mixed type — null is separate from type_counts"
        );
        assert!(roi.null_count > 0, "roi should be nullable");
    }
}
