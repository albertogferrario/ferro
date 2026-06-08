//! Typed completion entry point for structured LLM output.
//!
//! [`complete`] is the primary surface of the ferro-ai SDK for structured-output use.
//! Callers never import `schemars` or `serde_json` directly — schema generation,
//! normalization, and JSON parsing are fully encapsulated (SC#1, D-01).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ferro_ai::{complete, AnthropicClient};
//! use serde::Deserialize;
//! use schemars::JsonSchema;
//!
//! #[derive(Deserialize, JsonSchema)]
//! struct OrderSummary { name: String, total: f64 }
//!
//! let client = AnthropicClient::from_env().unwrap();
//! let summary: OrderSummary = complete(&client, "Summarize order #42 as JSON").await?;
//! ```
//!
//! ## Internal flow
//!
//! 1. `schemars::schema_for::<T>()` — generate Draft 2020-12 schema from the Rust type.
//! 2. `schema::for_structured_output(raw)` — normalize for Anthropic/OpenAI constraints;
//!    activates the ServiceDef-aware projection-enum closing path when T contains
//!    ferro-projections types in its `$defs` (D-07).
//! 3. Build `CompletionRequest` with `schema: Some(normalized)`.
//! 4. `client.complete(request)` — delegate to the configured LLM provider.
//! 5. `serde_json::from_str::<T>(&text)` — deserialize the JSON response into T.
//!
//! ## Plan 04 dependency note
//!
//! The `CompletionRequest` struct literal in this file lists exactly the five fields
//! that exist after Plan 02 (Phase 165): `system`, `messages`, `max_tokens`,
//! `model_override`, `schema`. Plan 04 adds `tools: Option<Vec<ToolRequest>>` and
//! `tool_choice: Option<ToolChoice>` — when those fields land, Plan 04 is responsible
//! for updating this struct literal to add `tools: None, tool_choice: None` (or
//! restructuring via `Default` if that derive is added).

use crate::client::{CompletionRequest, LlmClient, Message, Role};
use crate::error::Error;
use crate::schema;

/// Options controlling a typed completion request.
///
/// `max_tokens` caps the response; callers map `FERRO_AI_MAX_TOKENS_PER_COMMAND` onto it.
/// `system` supplies an optional system prompt for context-heavy completions.
/// `model_override` selects a non-default model for this request only.
pub struct CompleteOptions {
    /// Maximum number of tokens in the completion response.
    pub max_tokens: u32,
    /// Optional system prompt prepended before the user message.
    pub system: Option<String>,
    /// Override the provider's default model for this request.
    pub model_override: Option<String>,
}

impl Default for CompleteOptions {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            system: None,
            model_override: None,
        }
    }
}

/// Typed completion with explicit options. Same ServiceDef-aware schema-normalizer path as
/// [`complete`], parameterized by [`CompleteOptions`].
///
/// Callers never touch `schemars` or `serde_json` directly (SC#1).
///
/// # Errors
///
/// - `Error::Provider` — the LLM provider returned a non-success HTTP response.
/// - `Error::Deserialization` — the provider response was not valid JSON for `T`.
/// - `Error::Unsupported` — the client does not support non-streaming completions.
/// - `Error::SchemaError` — the type's schema could not be serialized.
pub async fn complete_with<T>(
    client: &dyn LlmClient,
    prompt: &str,
    opts: CompleteOptions,
) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let raw_schema = serde_json::to_value(schemars::schema_for!(T))
        .map_err(|e| Error::SchemaError(format!("schema_for serialization: {e}")))?;
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: opts.system,
        messages: vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            tool_call_id: None,
        }],
        max_tokens: opts.max_tokens,
        model_override: opts.model_override,
        schema: Some(normalized),
        tools: None,
        tool_choice: None,
    };

    let text = client.complete(request).await?;
    serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
}

/// Typed completion: generate a structured `T` from a prompt.
///
/// Delegates to [`complete_with`] with [`CompleteOptions::default`].
/// Internally calls `schemars::schema_for::<T>()`, normalizes the schema via
/// `schema::for_structured_output`, builds a `CompletionRequest` with the normalized
/// schema, calls `client.complete`, and deserializes the JSON response into `T`.
///
/// Callers never touch `schemars` or `serde_json` directly (SC#1).
///
/// # Errors
///
/// - `Error::Provider` — the LLM provider returned a non-success HTTP response.
/// - `Error::Deserialization` — the provider response was not valid JSON for `T`.
/// - `Error::Unsupported` — the client does not support non-streaming completions.
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    complete_with(client, prompt, CompleteOptions::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use schemars::JsonSchema;
    use serde::Deserialize;
    use std::sync::Mutex;

    use crate::client::{CompletionRequest, TokenStream};

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct MyOutput {
        value: String,
    }

    /// Minimal struct for complete_with / delegation tests.
    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct SimpleStruct {
        value: i64,
    }

    /// Mock LLM client that always returns the same fixed JSON string.
    struct ConstClient(String);

    #[async_trait]
    impl LlmClient for ConstClient {
        fn default_model(&self) -> &str {
            "test"
        }

        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> {
            Ok(self.0.clone())
        }

        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> {
            Err(Error::Unsupported)
        }

        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> {
            Err(Error::Unsupported)
        }
    }

    /// Mock LLM client that captures the last CompletionRequest for assertion.
    struct CapturingClient {
        response: String,
        captured: Mutex<Option<CompletionRequest>>,
    }

    impl CapturingClient {
        fn new(response: &str) -> Self {
            Self {
                response: response.to_string(),
                captured: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl LlmClient for CapturingClient {
        fn default_model(&self) -> &str {
            "test"
        }

        async fn complete(&self, req: CompletionRequest) -> Result<String, Error> {
            *self.captured.lock().unwrap() = Some(req);
            Ok(self.response.clone())
        }

        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> {
            Err(Error::Unsupported)
        }

        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> {
            Err(Error::Unsupported)
        }
    }

    /// SC#1: `complete::<T>()` round-trips a typed value via a mock client.
    ///
    /// The caller never imports schemars or serde_json — only `complete`, the client
    /// trait, and the output type are needed. The mock returns a fixed JSON string
    /// and the function deserializes it into the typed struct.
    #[tokio::test]
    async fn complete_returns_typed_result() {
        let client = ConstClient(r#"{"value":"hello"}"#.to_string());
        let result = complete::<MyOutput>(&client, "test prompt").await.unwrap();
        assert_eq!(result.value, "hello");
    }

    /// Deserialization errors are reported as `Error::Deserialization`.
    #[tokio::test]
    async fn complete_propagates_deserialization_error() {
        let client = ConstClient(r#"{"wrong_field":"hello"}"#.to_string());
        let result = complete::<MyOutput>(&client, "test prompt").await;
        // MyOutput has a required `value` field; missing it causes a deserialization error.
        // The error type should not be Unsupported or Provider.
        match result {
            Err(Error::Deserialization(_)) => {}
            other => panic!("expected Deserialization error, got: {other:?}"),
        }
    }

    /// `CompleteOptions::default()` produces the canonical zero-config values.
    #[test]
    fn complete_options_default() {
        let opts = CompleteOptions::default();
        assert_eq!(opts.max_tokens, 4096);
        assert!(opts.system.is_none());
        assert!(opts.model_override.is_none());
    }

    /// `complete_with::<T>()` forwards options to the CompletionRequest fields.
    #[tokio::test]
    async fn complete_with_uses_provided_max_tokens() {
        let client = CapturingClient::new(r#"{"value":1}"#);
        let opts = CompleteOptions {
            max_tokens: 9999,
            system: Some("sys".to_string()),
            model_override: Some("m".to_string()),
        };
        let _: SimpleStruct = complete_with(&client, "p", opts).await.unwrap();
        let req = client.captured.lock().unwrap().take().unwrap();
        assert_eq!(req.max_tokens, 9999);
        assert_eq!(req.system, Some("sys".to_string()));
        assert_eq!(req.model_override, Some("m".to_string()));
        assert!(req.schema.is_some());
    }

    /// `complete::<T>()` is a thin delegate: it passes `CompleteOptions::default()` values.
    #[tokio::test]
    async fn complete_delegates_to_complete_with() {
        let client = CapturingClient::new(r#"{"value":1}"#);
        let _: SimpleStruct = complete(&client, "p").await.unwrap();
        let req = client.captured.lock().unwrap().take().unwrap();
        assert_eq!(req.max_tokens, 4096);
        assert!(req.system.is_none());
        assert!(req.model_override.is_none());
        assert!(req.schema.is_some());
    }
}
