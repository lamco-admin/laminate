//! Handler registry for dispatching tool calls to typed handler functions.
//!
//! Provides dynamic dispatch of structured blocks (tool calls, function calls)
//! to typed async handler functions with automatic argument deserialization
//! via FlexValue.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::error::{FlexError, Result};
use crate::provider::{ContentBlock, NormalizedResponse};
use crate::value::FlexValue;

/// Result of dispatching a single tool call.
#[derive(Debug, Clone)]
pub struct HandlerResult {
    /// The block/tool call ID.
    pub block_id: String,
    /// The handler name that was dispatched to.
    pub name: String,
    /// The serialized result.
    pub result: Value,
}

/// A boxed async handler function.
type BoxHandler =
    Box<dyn Fn(FlexValue) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> + Send + Sync>;

/// A boxed synchronous handler function.
type BoxSyncHandler = Box<dyn Fn(FlexValue) -> Result<Value> + Send + Sync>;

/// Registry of named tool handlers with automatic argument deserialization.
///
/// # Example
///
/// ```ignore
/// let mut registry = HandlerRegistry::new();
///
/// registry.register("get_weather", |args: WeatherArgs| async move {
///     let weather = fetch_weather(&args.city).await?;
///     Ok(WeatherResult { temp: weather.temp })
/// });
///
/// let results = registry.dispatch_all(&response).await?;
/// ```
pub struct HandlerRegistry {
    handlers: HashMap<String, BoxHandler>,
    sync_handlers: HashMap<String, BoxSyncHandler>,
}

impl HandlerRegistry {
    /// Create an empty handler registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            sync_handlers: HashMap::new(),
        }
    }

    /// Register a typed handler. Arguments are auto-deserialized from FlexValue.
    ///
    /// The handler function receives deserialized arguments of type `A` and
    /// returns a serializable result of type `R`.
    pub fn register<A, R, F, Fut>(&mut self, name: &str, handler: F)
    where
        A: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        F: Fn(A) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let wrapper =
            move |input: FlexValue| -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
                let args_result: std::result::Result<A, _> =
                    serde_json::from_value(input.into_raw());
                let handler = Arc::clone(&handler);

                Box::pin(async move {
                    let args = args_result.map_err(|e| FlexError::DeserializeError {
                        path: "(handler args)".into(),
                        source: e,
                    })?;

                    let result = handler(args).await?;

                    serde_json::to_value(result).map_err(|e| FlexError::DeserializeError {
                        path: "(handler result)".into(),
                        source: e,
                    })
                })
            };

        self.handlers.insert(name.to_string(), Box::new(wrapper));
    }

    /// Register a handler that receives raw FlexValue (no deserialization).
    pub fn register_raw<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(FlexValue) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send + 'static,
    {
        let handler =
            move |input: FlexValue| -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
                Box::pin((handler)(input))
            };
        self.handlers.insert(name.to_string(), Box::new(handler));
    }

    /// Register a synchronous typed handler (no async runtime needed).
    pub fn register_sync<A, R, F>(&mut self, name: &str, handler: F)
    where
        A: serde::de::DeserializeOwned + 'static,
        R: serde::Serialize + 'static,
        F: Fn(A) -> Result<R> + Send + Sync + 'static,
    {
        let wrapper = move |input: FlexValue| -> Result<Value> {
            let args: A = serde_json::from_value(input.into_raw()).map_err(|e| {
                FlexError::DeserializeError {
                    path: "(handler args)".into(),
                    source: e,
                }
            })?;
            let result = handler(args)?;
            serde_json::to_value(result).map_err(|e| FlexError::DeserializeError {
                path: "(handler result)".into(),
                source: e,
            })
        };
        self.sync_handlers
            .insert(name.to_string(), Box::new(wrapper));
    }

    /// Dispatch a single content block synchronously.
    ///
    /// Checks sync handlers first. If no sync handler is registered but an
    /// async handler exists, returns an error explaining the mismatch.
    /// Returns None only if the block is not a tool use or no handler
    /// (sync or async) is registered for it.
    pub fn dispatch_sync(&self, block: &ContentBlock) -> Option<Result<HandlerResult>> {
        let (id, name, input) = block.as_tool_use()?;
        if let Some(handler) = self.sync_handlers.get(name) {
            let result = handler(input.clone());
            Some(result.map(|value| HandlerResult {
                block_id: id.to_string(),
                name: name.to_string(),
                result: value,
            }))
        } else if self.handlers.contains_key(name) {
            Some(Err(FlexError::HandlerError {
                name: name.to_string(),
                detail: format!(
                    "handler '{}' is registered as async only; use dispatch() instead of dispatch_sync()",
                    name
                ),
            }))
        } else {
            None
        }
    }

    /// Dispatch all tool-use blocks synchronously.
    pub fn dispatch_all_sync(&self, response: &NormalizedResponse) -> Result<Vec<HandlerResult>> {
        let mut results = Vec::new();
        for block in &response.content {
            if let Some(result) = self.dispatch_sync(block) {
                results.push(result?);
            }
        }
        Ok(results)
    }

    /// Dispatch a single content block (async).
    ///
    /// Returns `None` if the block is not a tool use, or if no handler is registered.
    pub async fn dispatch(&self, block: &ContentBlock) -> Option<Result<HandlerResult>> {
        let (id, name, input) = block.as_tool_use()?;

        let handler = self.handlers.get(name)?;

        let result = handler(input.clone()).await;

        Some(result.map(|value| HandlerResult {
            block_id: id.to_string(),
            name: name.to_string(),
            result: value,
        }))
    }

    /// Dispatch all tool-use blocks from a response, returning results in order.
    pub async fn dispatch_all(&self, response: &NormalizedResponse) -> Result<Vec<HandlerResult>> {
        let mut results = Vec::new();

        for block in &response.content {
            if let Some(result) = self.dispatch(block).await {
                results.push(result?);
            }
        }

        Ok(results)
    }

    /// Check if a handler (async or sync) is registered for the given name.
    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name) || self.sync_handlers.contains_key(name)
    }

    /// Get all registered handler names (async + sync, deduplicated).
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.handlers.keys().map(|k| k.as_str()).collect();
        for k in self.sync_handlers.keys() {
            if !self.handlers.contains_key(k.as_str()) {
                names.push(k.as_str());
            }
        }
        names
    }

    /// Number of registered handlers (async + sync).
    pub fn len(&self) -> usize {
        let async_count = self.handlers.len();
        let sync_only = self
            .sync_handlers
            .keys()
            .filter(|k| !self.handlers.contains_key(k.as_str()))
            .count();
        async_count + sync_only
    }

    /// Returns `true` if no handlers are registered.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty() && self.sync_handlers.is_empty()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize)]
    struct SearchArgs {
        query: String,
    }

    #[derive(Debug, Serialize)]
    struct SearchResult {
        results: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct MathArgs {
        a: f64,
        b: f64,
    }

    #[derive(Debug, Serialize)]
    struct MathResult {
        sum: f64,
    }

    #[tokio::test]
    async fn register_and_dispatch_typed() {
        let mut registry = HandlerRegistry::new();

        registry.register("search", |args: SearchArgs| async move {
            Ok(SearchResult {
                results: vec![format!("Result for: {}", args.query)],
            })
        });

        assert!(registry.has("search"));
        assert_eq!(registry.len(), 1);

        let block = ContentBlock::ToolUse {
            id: "tu_1".into(),
            name: "search".into(),
            input: FlexValue::from_json(r#"{"query": "rust"}"#).unwrap(),
        };

        let result = registry.dispatch(&block).await.unwrap().unwrap();
        assert_eq!(result.block_id, "tu_1");
        assert_eq!(result.name, "search");
        assert_eq!(result.result["results"][0], "Result for: rust");
    }

    #[tokio::test]
    async fn register_raw_handler() {
        let mut registry = HandlerRegistry::new();

        registry.register_raw(
            "echo",
            |input: FlexValue| async move { Ok(input.into_raw()) },
        );

        let block = ContentBlock::ToolUse {
            id: "tu_2".into(),
            name: "echo".into(),
            input: FlexValue::from_json(r#"{"message": "hello"}"#).unwrap(),
        };

        let result = registry.dispatch(&block).await.unwrap().unwrap();
        assert_eq!(result.result["message"], "hello");
    }

    #[tokio::test]
    async fn dispatch_text_block_returns_none() {
        let registry = HandlerRegistry::new();
        let block = ContentBlock::Text {
            text: "Hello".into(),
        };
        assert!(registry.dispatch(&block).await.is_none());
    }

    #[tokio::test]
    async fn dispatch_unregistered_handler_returns_none() {
        let registry = HandlerRegistry::new();
        let block = ContentBlock::ToolUse {
            id: "tu_3".into(),
            name: "unknown_tool".into(),
            input: FlexValue::new(serde_json::json!({})),
        };
        assert!(registry.dispatch(&block).await.is_none());
    }

    #[tokio::test]
    async fn dispatch_all_from_response() {
        let mut registry = HandlerRegistry::new();

        registry.register("search", |args: SearchArgs| async move {
            Ok(SearchResult {
                results: vec![format!("Found: {}", args.query)],
            })
        });

        registry.register("add", |args: MathArgs| async move {
            Ok(MathResult {
                sum: args.a + args.b,
            })
        });

        let response = NormalizedResponse {
            id: "msg_test".into(),
            model: "test".into(),
            content: vec![
                ContentBlock::Text {
                    text: "Let me help.".into(),
                },
                ContentBlock::ToolUse {
                    id: "tu_1".into(),
                    name: "search".into(),
                    input: FlexValue::from_json(r#"{"query": "laminate"}"#).unwrap(),
                },
                ContentBlock::ToolUse {
                    id: "tu_2".into(),
                    name: "add".into(),
                    input: FlexValue::from_json(r#"{"a": 2.5, "b": 3.7}"#).unwrap(),
                },
            ],
            stop_reason: crate::diagnostic::StopReason::ToolUse,
            usage: crate::provider::Usage::default(),
            raw: FlexValue::new(serde_json::json!({})),
        };

        let results = registry.dispatch_all(&response).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "search");
        assert_eq!(results[0].result["results"][0], "Found: laminate");
        assert_eq!(results[1].name, "add");
        assert_eq!(results[1].result["sum"], 6.2);
    }

    #[test]
    fn names_and_len() {
        let mut registry = HandlerRegistry::new();
        assert!(registry.is_empty());

        registry.register_raw("a", |_| async { Ok(serde_json::json!(null)) });
        registry.register_raw("b", |_| async { Ok(serde_json::json!(null)) });

        assert_eq!(registry.len(), 2);
        let mut names = registry.names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
