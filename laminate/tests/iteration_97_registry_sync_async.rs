#![cfg(feature = "registry")]
use laminate::diagnostic::StopReason;
use laminate::provider::{ContentBlock, NormalizedResponse, Usage};
/// Iteration 97: HandlerRegistry — sync dispatch error for async-only handlers
///
/// dispatch_all_sync() silently returned empty results when only async
/// handlers were registered. Now it returns an error explaining the
/// handler exists but isn't callable synchronously.
use laminate::registry::HandlerRegistry;
use laminate::FlexValue;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Args {
    value: i64,
}

fn make_response() -> NormalizedResponse {
    NormalizedResponse {
        id: "msg_1".into(),
        model: "test".into(),
        content: vec![ContentBlock::ToolUse {
            id: "tu_1".into(),
            name: "compute".into(),
            input: FlexValue::from_json(r#"{"value": 21}"#).unwrap(),
        }],
        stop_reason: StopReason::ToolUse,
        usage: Usage::default(),
        raw: FlexValue::new(serde_json::json!({})),
    }
}

#[test]
fn sync_dispatch_errors_for_async_only_handler() {
    let mut registry = HandlerRegistry::new();
    registry.register("compute", |args: Args| async move {
        Ok(serde_json::json!({"doubled": args.value * 2}))
    });

    let response = make_response();
    let result = registry.dispatch_all_sync(&response);

    assert!(result.is_err(), "should error, not silently return empty");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("async only"),
        "error should mention async: {}",
        err
    );
}

#[test]
fn sync_dispatch_works_with_sync_handler() {
    let mut registry = HandlerRegistry::new();
    registry.register_sync("compute", |args: Args| {
        Ok(serde_json::json!({"doubled": args.value * 2}))
    });

    let response = make_response();
    let results = registry.dispatch_all_sync(&response).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].result["doubled"], 42);
}
