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

/// Typed completion: generate a structured `T` from a prompt.
///
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
    let raw_schema = serde_json::to_value(schemars::schema_for!(T))
        .map_err(|e| Error::SchemaError(format!("schema_for serialization: {e}")))?;
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: prompt.to_string(),
        }],
        max_tokens: 4096,
        model_override: None,
        schema: Some(normalized),
    };

    let text = client.complete(request).await?;
    serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use schemars::JsonSchema;
    use serde::Deserialize;

    use crate::client::{CompletionRequest, TokenStream};

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct MyOutput {
        value: String,
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
}
