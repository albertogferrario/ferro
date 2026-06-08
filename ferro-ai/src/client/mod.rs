//! Provider-agnostic LLM client trait and request/response types.
//!
//! The [`LlmClient`] trait is the central abstraction — implemented by
//! [`anthropic::AnthropicClient`], [`openai::OpenAiClient`], and [`ollama::OllamaClient`].
//! All three are instantiable as `Box<dyn LlmClient>`.
//!
//! Use [`crate::config::AiConfig::from_env`] to construct the configured client from
//! environment variables at startup.

pub mod anthropic;
pub mod ollama;
pub mod openai;

use crate::error::Error;
use async_trait::async_trait;
use futures::stream::BoxStream;

/// Opaque stream of text tokens from a streaming LLM completion.
///
/// Each item is either a text token chunk (`Ok(String)`) or a provider error
/// (`Err(Error)`). Callers consume via [`futures::StreamExt::next`].
///
/// `reqwest-eventsource` is NOT re-exported — this type alias hides the
/// underlying stream implementation (D-09).
pub type TokenStream = BoxStream<'static, Result<String, Error>>;

/// Role of a message participant in a completion request.
#[derive(Debug, Clone)]
pub enum Role {
    /// A message from the end user or calling code.
    User,
    /// A message from the assistant (used for multi-turn conversations).
    Assistant,
}

/// A single message in a completion conversation.
#[derive(Debug, Clone)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The text content of the message.
    pub content: String,
}

/// Request for a text completion from an LLM provider.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Optional system prompt. Sent before the conversation messages.
    pub system: Option<String>,
    /// Conversation messages in chronological order.
    pub messages: Vec<Message>,
    /// Maximum number of tokens in the response.
    pub max_tokens: u32,
    /// Optional per-request model override.
    ///
    /// `None` resolves to the client's [`LlmClient::default_model`] at call time.
    pub model_override: Option<String>,
    /// Optional JSON schema for structured output.
    ///
    /// Passed through to the provider as-is. Phase 166 adds a typed
    /// `complete::<T>()` wrapper with schemars normalization on top of this field.
    /// With streaming + schema, tokens arrive as raw JSON fragments — callers
    /// must accumulate before parsing (Pitfall 1).
    pub schema: Option<serde_json::Value>,
}

/// Provider-agnostic LLM client.
///
/// Implement this trait to add a new provider. All methods use `&self` so the
/// client can be shared via `Arc<dyn LlmClient>` or `Box<dyn LlmClient>`.
///
/// Providers that lack a capability (e.g. Anthropic has no embeddings endpoint)
/// return `Err(Error::Unsupported)` — they never panic.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// The provider's default model identifier.
    ///
    /// Used when [`CompletionRequest::model_override`] is `None`. Overridable
    /// at startup via `FERRO_AI_MODEL`.
    fn default_model(&self) -> &str;

    /// Run a non-streaming completion, returning the full response text.
    async fn complete(&self, request: CompletionRequest) -> Result<String, Error>;

    /// Run a streaming completion, returning a token stream.
    ///
    /// Each yielded item is a text token chunk. When `request.schema` is set,
    /// tokens are raw JSON fragments; accumulate them before parsing.
    async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, Error>;

    /// Generate a text embedding vector.
    ///
    /// Returns `Err(Error::Unsupported)` for providers without an embeddings
    /// endpoint (e.g. [`anthropic::AnthropicClient`]).
    async fn embed(&self, text: &str) -> Result<Vec<f32>, Error>;
}
