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
