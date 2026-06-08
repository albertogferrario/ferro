//! Tool calling: `ToolDef`, `ToolError`, `ToolRegistry`, and the bounded dispatch loop.
//!
//! ## Safety contract (D-12, SC#5)
//!
//! [`ToolRegistry::new`] is the ONLY full constructor. `max_iterations` is required —
//! there is no `Default` impl and no zero-arg constructor. The dispatch loop returns
//! [`Error::ToolIterationLimit`] at the hard cap with no override path.
//!
//! ## Error surfacing (D-13, SC#6)
//!
//! Tool handler failures are surfaced to the LLM as [`ToolError`] messages, never as
//! raw Rust panics, stack traces, or DB-constraint strings. The Rust caller receives
//! [`Error::ToolIterationLimit`] when the loop exceeds its cap.
//!
//! ## Handler lifetime (D-11)
//!
//! Handler closures must satisfy `'static` — all captured state must be owned or
//! `Arc`-wrapped. Capturing `&references` will not compile.

use crate::client::{
    CompletionRequest, CompletionResponse, LlmClient, Message, Role, ToolChoice, ToolRequest,
    ToolUseBlock,
};
use crate::error::Error;
use futures::future::BoxFuture;
use std::collections::HashMap;
use tracing::{error, warn};

/// Model-legible tool error.
///
/// Surfaced to the LLM as a `tool_result` message carrying only `message`.
/// Never exposed to Rust callers as a panic or raw DB string (SC#6, T-166-02).
///
/// Handler implementations are responsible for mapping domain errors to a
/// human-readable `message` before returning `Err(ToolError { ... })`.
#[derive(Debug, Clone)]
pub struct ToolError {
    /// The model-legible error message. Must not contain raw Rust panics,
    /// stack traces, or DB-constraint strings.
    pub message: String,
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// A registered tool with its async handler.
///
/// `parameters_schema` must already be normalized via `schema::for_structured_output`
/// before registration. The handler must own all captured state (no `&references` —
/// wrap shared state in `Arc<T>` to satisfy the `'static` bound).
pub struct ToolDef {
    /// The tool name. Must match what the LLM will call.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    ///
    /// Must be pre-normalized via `schema::for_structured_output`. The LLM-generated
    /// input is passed as-is to the handler — handler implementations are responsible
    /// for validating their own inputs before privileged actions (T-166-03).
    pub parameters_schema: serde_json::Value,
    /// The async handler closure.
    ///
    /// Receives the LLM-generated `serde_json::Value` and returns either a JSON result
    /// or a [`ToolError`] with a model-legible message.
    pub handler: Box<
        dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>>
            + Send
            + Sync,
    >,
}

/// Helper to wrap an `async fn` or closure into the boxed handler type required by [`ToolDef`].
///
/// # Example
///
/// ```rust,ignore
/// use ferro_ai::tools::{make_handler, ToolDef, ToolError};
///
/// let def = ToolDef {
///     name: "greet".into(),
///     description: "Greet a user by name".into(),
///     parameters_schema: serde_json::json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
///     handler: make_handler(|input| async move {
///         let name = input["name"].as_str().unwrap_or("world");
///         Ok(serde_json::json!({"greeting": format!("Hello, {name}!")}))
///     }),
/// };
/// ```
pub fn make_handler<F, Fut>(
    f: F,
) -> Box<
    dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>>
        + Send
        + Sync,
>
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, ToolError>> + Send + 'static,
{
    Box::new(move |input| Box::pin(f(input)))
}

/// Registry of named tools for the LLM dispatch loop.
///
/// ## Construction
///
/// `max_iterations` is required at construction — there is no zero-arg constructor
/// and no way to create an unbounded loop (SC#5, D-12). Suggested default: 10.
///
/// ```rust,ignore
/// let registry = ToolRegistry::new(10);
/// // or equivalently:
/// let registry = ToolRegistry::with_default_iterations();
/// ```
///
/// ## Dispatch
///
/// [`ToolRegistry::dispatch`] loops until the LLM returns a text response or the
/// iteration cap is reached. At iteration 5 a warning is logged; at the cap an error
/// is logged and [`Error::ToolIterationLimit`] is returned.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
    max_iterations: u32,
}

impl ToolRegistry {
    /// Create a new registry with an explicit iteration cap.
    ///
    /// There is no `Default` impl and no zero-arg `new()`. Every `ToolRegistry`
    /// must carry an explicit `max_iterations` to prevent unbounded loops (SC#5).
    pub fn new(max_iterations: u32) -> Self {
        Self {
            tools: HashMap::new(),
            max_iterations,
        }
    }

    /// Convenience constructor with `max_iterations = 10`.
    pub fn with_default_iterations() -> Self {
        Self::new(10)
    }

    /// Register a tool definition.
    ///
    /// If a tool with the same name is already registered, it is replaced.
    pub fn register(&mut self, tool: ToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Build a `CompletionRequest` for one dispatch iteration.
    fn build_request(&self, messages: Vec<Message>) -> CompletionRequest {
        let tool_requests: Vec<ToolRequest> = self
            .tools
            .values()
            .map(|t| ToolRequest {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters_schema: t.parameters_schema.clone(),
            })
            .collect();

        CompletionRequest {
            system: None,
            messages,
            max_tokens: 4096,
            model_override: None,
            schema: None,
            tools: if tool_requests.is_empty() {
                None
            } else {
                Some(tool_requests)
            },
            tool_choice: Some(ToolChoice::Auto),
        }
    }

    /// Convert a tool handler result into a `Message` to send back to the LLM.
    ///
    /// On `Ok(value)` → JSON-serialized result.
    /// On `Err(ToolError { message })` → the model-legible message (SC#6).
    fn result_to_message(block_id: &str, result: Result<serde_json::Value, ToolError>) -> Message {
        let content = match result {
            Ok(value) => value.to_string(),
            Err(te) => te.message,
        };
        // Role::Tool — providers translate to their wire format in build_body
        // (Anthropic: role "user" + type "tool_result"; OpenAI: role "tool")
        // Content carries the block_id reference so the provider can link it.
        Message {
            role: Role::Tool,
            content: format!("[tool_use_id:{block_id}] {content}"),
        }
    }

    /// Dispatch a tool-calling conversation loop.
    ///
    /// Calls `client.complete_with_tools` repeatedly until the LLM returns a text
    /// response or `max_iterations` is reached. Each `ToolUse` response dispatches
    /// registered handlers and appends results before the next iteration.
    ///
    /// ## Iteration limits (SC#5, T-166-01)
    ///
    /// - At iteration 5: `tracing::warn!` (advisory — loop still continues).
    /// - At `max_iterations`: `tracing::error!` + `Err(Error::ToolIterationLimit)`.
    ///   This is a hard cap with no override path.
    ///
    /// ## Error surfacing (SC#6, T-166-02)
    ///
    /// Handler `Err(ToolError { message })` is sent to the LLM as a tool_result
    /// message carrying only `message`. Unknown tool names are also surfaced to the
    /// LLM as model-recoverable error strings (not `Error::ToolNotFound`) so the
    /// model can adapt its tool selection.
    pub async fn dispatch(
        &self,
        mut messages: Vec<Message>,
        client: &dyn LlmClient,
    ) -> Result<Vec<Message>, Error> {
        for iteration in 0..=self.max_iterations {
            if iteration == self.max_iterations {
                error!(
                    max_iterations = self.max_iterations,
                    "tool dispatch hit iteration limit"
                );
                return Err(Error::ToolIterationLimit(self.max_iterations));
            }
            if iteration == 5 {
                warn!(
                    iteration,
                    max = self.max_iterations,
                    "tool dispatch at iteration 5"
                );
            }

            let request = self.build_request(messages.clone());
            let response = client.complete_with_tools(request).await?;

            match response {
                CompletionResponse::Text(text) => {
                    messages.push(Message {
                        role: Role::Assistant,
                        content: text,
                    });
                    return Ok(messages);
                }
                CompletionResponse::ToolUse(blocks) => {
                    for block in &blocks {
                        let result = self.call_tool(block).await;
                        messages.push(Self::result_to_message(&block.id, result));
                    }
                }
            }
        }
        unreachable!()
    }

    /// Call the handler for one tool-use block.
    ///
    /// Unknown tool names are surfaced to the LLM as a model-recoverable error string
    /// rather than aborting the dispatch loop — the model can select a different tool.
    async fn call_tool(&self, block: &ToolUseBlock) -> Result<serde_json::Value, ToolError> {
        match self.tools.get(&block.name) {
            None => Err(ToolError {
                message: format!("tool '{}' is not registered", block.name),
            }),
            Some(tool) => (tool.handler)(block.input.clone()).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{CompletionRequest, TokenStream};
    use async_trait::async_trait;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    // ─── SC#4: ToolDef construction ──────────────────────────────────────────

    /// SC#4: ToolDef carries name, description, parameters_schema, and async handler.
    #[tokio::test]
    async fn tool_def_construction() {
        let schema = serde_json::json!({"type": "object", "properties": {"x": {"type": "string"}}});
        let def = ToolDef {
            name: "my_tool".into(),
            description: "does a thing".into(),
            parameters_schema: schema.clone(),
            handler: make_handler(
                |_input| async move { Ok(serde_json::json!({"result": "done"})) },
            ),
        };
        assert_eq!(def.name, "my_tool");
        assert_eq!(def.description, "does a thing");
        assert_eq!(def.parameters_schema, schema);
        // Handler must be callable and return Ok.
        let result = (def.handler)(serde_json::json!({})).await;
        assert!(result.is_ok());
    }

    // ─── SC#6: ToolError is model-legible ───────────────────────────────────

    /// SC#6: ToolError Display returns exactly the message, nothing else.
    #[test]
    fn tool_error_is_model_legible() {
        let err = ToolError {
            message: "domain message".into(),
        };
        assert_eq!(format!("{err}"), "domain message");
        // Debug output contains the struct name and field, but Display is the
        // model-facing representation — assert Display == message only.
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("domain message"));
    }

    // ─── No unbounded path ───────────────────────────────────────────────────

    /// Documents that ToolRegistry::new(n) works and with_default_iterations works.
    /// The absence of Default and a zero-arg new() is enforced by the compiler —
    /// this test documents the expected construction API.
    #[test]
    fn tool_registry_requires_max_iterations() {
        let r1 = ToolRegistry::new(3);
        assert_eq!(r1.max_iterations, 3);
        let r2 = ToolRegistry::with_default_iterations();
        assert_eq!(r2.max_iterations, 10);
    }

    // ─── Dispatch loop tests (used in Task 3, defined here for Task 2 GREEN) ─

    /// Mock LlmClient that returns ToolUse for `stop_after` calls then returns Text.
    struct LoopingClient {
        calls: Arc<AtomicU32>,
        stop_after: u32,
        tool_name: String,
    }

    #[async_trait]
    impl LlmClient for LoopingClient {
        fn default_model(&self) -> &str {
            "test"
        }

        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> {
            Err(Error::Unsupported)
        }

        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> {
            Err(Error::Unsupported)
        }

        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> {
            Err(Error::Unsupported)
        }

        async fn complete_with_tools(
            &self,
            _: CompletionRequest,
        ) -> Result<CompletionResponse, Error> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.stop_after {
                Ok(CompletionResponse::Text("done".into()))
            } else {
                Ok(CompletionResponse::ToolUse(vec![ToolUseBlock {
                    id: format!("call_{n}"),
                    name: self.tool_name.clone(),
                    input: serde_json::json!({}),
                }]))
            }
        }
    }

    /// SC#5: dispatch returns Err(ToolIterationLimit) at the hard cap.
    #[tokio::test]
    async fn tool_registry_enforces_max_iterations() {
        let registry = ToolRegistry::new(3);
        let calls = Arc::new(AtomicU32::new(0));
        let client = LoopingClient {
            calls,
            stop_after: 99, // never stops on its own
            tool_name: "no_op".into(),
        };
        let result = registry.dispatch(vec![], &client).await;
        assert!(
            matches!(result, Err(Error::ToolIterationLimit(3))),
            "expected ToolIterationLimit(3), got {result:?}"
        );
    }

    /// dispatch returns Ok when the client returns Text on the first call.
    #[tokio::test]
    async fn dispatch_returns_on_text() {
        let registry = ToolRegistry::new(5);
        let calls = Arc::new(AtomicU32::new(0));
        let client = LoopingClient {
            calls,
            stop_after: 0, // returns Text immediately
            tool_name: "no_op".into(),
        };
        let result = registry.dispatch(vec![], &client).await;
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert!(
            messages
                .iter()
                .any(|m| matches!(m.role, Role::Assistant) && m.content == "done"),
            "expected assistant message with 'done'"
        );
    }

    /// SC#6: a handler returning ToolError surfaces only its message to the LLM.
    ///
    /// The dispatch loop must complete (not abort) when a registered handler fails,
    /// and the tool_result message must carry the model-legible ToolError message,
    /// not a raw panic or Rust debug string.
    #[tokio::test]
    async fn dispatch_surfaces_tool_error() {
        let mut registry = ToolRegistry::new(5);

        // Register a tool that always fails with a model-legible message.
        registry.register(ToolDef {
            name: "failing_tool".into(),
            description: "always fails".into(),
            parameters_schema: serde_json::json!({}),
            handler: make_handler(|_| async move {
                Err(ToolError {
                    message: "order not found".into(),
                })
            }),
        });

        // Client: first call returns ToolUse for failing_tool, second returns Text.
        let calls = Arc::new(AtomicU32::new(0));
        let client = LoopingClient {
            calls,
            stop_after: 1, // after 1 ToolUse call → Text
            tool_name: "failing_tool".into(),
        };

        let result = registry.dispatch(vec![], &client).await;
        assert!(
            result.is_ok(),
            "dispatch must complete even after tool error"
        );

        let messages = result.unwrap();
        // There must be a Role::Tool message carrying the model-legible error.
        let tool_result = messages.iter().find(|m| matches!(m.role, Role::Tool));
        assert!(
            tool_result.is_some(),
            "expected a Role::Tool result message"
        );
        let content = &tool_result.unwrap().content;
        assert!(
            content.contains("order not found"),
            "ToolError message must appear in tool result, got: {content}"
        );
        // Must NOT contain raw Rust panic text or debug noise.
        assert!(
            !content.contains("panicked at"),
            "tool result must not contain panic text"
        );
    }
}
