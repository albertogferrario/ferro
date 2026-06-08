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

pub use anthropic::AnthropicClient;
pub use ollama::OllamaClient;
pub use openai::OpenAiClient;

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
    /// A tool result message.
    ///
    /// Anthropic: sent as `role: "user"` with `type: "tool_result"` content.
    /// OpenAI: sent as `role: "tool"` with `tool_call_id`.
    Tool,
}

/// A single message in a completion conversation.
#[derive(Debug, Clone)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The text content of the message.
    pub content: String,
}

/// A tool definition included in a completion request.
///
/// `parameters_schema` must already be normalized via
/// `schema::for_structured_output` before being placed here (D-14).
#[derive(Debug, Clone)]
pub struct ToolRequest {
    /// The tool name. Must match the name in [`crate::tools::ToolDef`].
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema for the tool's input parameters (normalized).
    pub parameters_schema: serde_json::Value,
}

/// Controls how the LLM selects a tool when tools are available.
#[derive(Debug, Clone)]
pub enum ToolChoice {
    /// The LLM decides whether to call a tool (default).
    Auto,
    /// The LLM must not call any tool.
    None,
}

/// A single tool-use block returned by the LLM.
#[derive(Debug, Clone)]
pub struct ToolUseBlock {
    /// Provider-assigned call identifier (used when sending back tool results).
    pub id: String,
    /// The tool name the LLM chose to call.
    pub name: String,
    /// The arguments the LLM generated for the tool call.
    pub input: serde_json::Value,
}

/// The result of a `complete_with_tools` call.
///
/// Either the LLM produced a final text answer or it wants to call one or more tools.
#[derive(Debug)]
pub enum CompletionResponse {
    /// The LLM produced a final text response (stop_reason "end_turn" / finish_reason "stop").
    Text(String),
    /// The LLM wants to invoke tools (stop_reason "tool_use" / finish_reason "tool_calls").
    ToolUse(Vec<ToolUseBlock>),
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
    /// Optional tool definitions for the tool-calling dispatch loop.
    ///
    /// Each entry's `parameters_schema` must be pre-normalized via
    /// `schema::for_structured_output`. Set by `ToolRegistry::dispatch` (D-14).
    pub tools: Option<Vec<ToolRequest>>,
    /// Controls how the LLM selects a tool when `tools` is `Some`.
    pub tool_choice: Option<ToolChoice>,
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

    /// Run a completion that may invoke tools.
    ///
    /// Returns [`CompletionResponse::Text`] when the LLM produces a final answer,
    /// or [`CompletionResponse::ToolUse`] when the LLM requests tool execution.
    ///
    /// The default implementation returns `Err(Error::Unsupported)` — providers
    /// that do not support tool calling (e.g. [`ollama::OllamaClient`]) inherit this
    /// and existing callers of `complete()` are unaffected (D-14).
    async fn complete_with_tools(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Error> {
        let _ = request;
        Err(Error::Unsupported)
    }
}
