//! Text embedding entry point for the ferro-ai SDK.
//!
//! [`embed`] is the primary surface for generating text embeddings.
//! It is a thin delegate over [`LlmClient::embed`], symmetric with
//! [`crate::complete`].
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ferro_ai::{embed, OllamaClient};
//!
//! let client = OllamaClient::new(None, None);
//! let vector: Vec<f32> = embed(&client, "Hello, world!").await?;
//! ```
//!
//! ## Provider support
//!
//! - `OllamaClient` — posts to `/api/embed` using the model from
//!   `FERRO_AI_EMBED_MODEL` (default `nomic-embed-text`).
//! - `OpenAiClient` — posts to `/v1/embeddings` using the model from
//!   `FERRO_AI_EMBED_MODEL` (default `text-embedding-3-small`).
//! - `AnthropicClient` — returns `Err(Error::Unsupported)`; Anthropic has no
//!   embeddings endpoint.

use crate::client::LlmClient;
use crate::error::Error;

/// Generate a text embedding vector using the configured LLM provider.
///
/// Thin pass-through to [`LlmClient::embed`]: no batching, normalization, or retry.
/// Symmetric with [`crate::complete`].
///
/// Returns `Err(Error::Unsupported)` for providers without an embeddings
/// endpoint (e.g. `AnthropicClient`).
pub async fn embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error> {
    client.embed(text).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    use crate::client::{CompletionRequest, TokenStream};

    struct OkClient;

    #[async_trait]
    impl LlmClient for OkClient {
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
            Ok(vec![0.1, 0.2, 0.3])
        }
    }

    struct UnsupportedClient;

    #[async_trait]
    impl LlmClient for UnsupportedClient {
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
    }

    #[tokio::test]
    async fn embed_delegates_to_client() {
        let client = OkClient;
        let result = embed(&client, "hello").await.unwrap();
        assert_eq!(result, vec![0.1f32, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn embed_propagates_unsupported() {
        let client = UnsupportedClient;
        let result = embed(&client, "hello").await;
        assert!(
            matches!(result, Err(Error::Unsupported)),
            "expected Err(Error::Unsupported), got: {result:?}"
        );
    }
}
