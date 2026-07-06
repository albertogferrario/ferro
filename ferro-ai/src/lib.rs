//! # ferro-ai
//!
//! AI structured classification and confirmation primitives for the Ferro framework.
//!
//! ## Classification
//!
//! Provider-abstracted wrapper for LLM structured JSON output with configurable
//! schema, model selection, confidence threshold, and retry behavior.
//!
//! ```rust,ignore
//! use ferro_ai::{Classifier, ClassifierConfig, AnthropicProvider};
//! use serde::Deserialize;
//! use std::sync::Arc;
//!
//! #[derive(Deserialize)]
//! struct CommandIntent {
//!     action: String,
//!     confidence: f64,
//! }
//!
//! let provider = AnthropicProvider::from_env().unwrap();
//! let classifier = Classifier::<CommandIntent>::new(
//!     Arc::new(provider),
//!     ClassifierConfig::default(),
//! );
//! ```
//!
//! ## Confirmation
//!
//! State machine for gating destructive actions behind explicit user confirmation
//! with configurable TTL expiry and event-driven observability.
//!
//! ```rust,ignore
//! use ferro_ai::{InMemoryConfirmationStore, ConfirmationStore};
//! use std::time::Duration;
//!
//! let store = InMemoryConfirmationStore::new();
//! let payload = serde_json::json!({"action": "delete_user", "user_id": 42});
//!
//! store.request_confirmation("confirm-delete-42", payload, Duration::from_secs(60)).await?;
//! let confirmed = store.confirm("confirm-delete-42").await?;
//! ```

#[cfg(any(feature = "llm", feature = "classifier-trait"))]
pub mod classifier;
#[cfg(feature = "llm")]
pub mod client;
#[cfg(feature = "llm")]
pub mod complete;
#[cfg(feature = "llm")]
pub mod config;
pub mod confirmation;
#[cfg(feature = "llm")]
pub mod embed;
pub mod error;
#[cfg(feature = "llm")]
pub mod schema;
#[cfg(feature = "llm")]
pub mod similarity;
#[cfg(feature = "llm")]
pub mod tools;

#[cfg(feature = "pgvector")]
pub mod pgvector;

#[cfg(feature = "llm")]
pub use classifier::anthropic::AnthropicProvider;
// ClassificationProvider, ClassifierConfig, Classifier, and ClassificationResult are
// reqwest-free; available under `classifier-trait` (replay/CI) or `llm` (live path).
#[cfg(any(feature = "llm", feature = "classifier-trait"))]
pub use classifier::provider::ClassificationProvider;
#[cfg(any(feature = "llm", feature = "classifier-trait"))]
pub use classifier::{ClassificationResult, Classifier, ClassifierConfig};
#[cfg(feature = "llm")]
pub use client::{
    AnthropicClient, CompletionRequest, CompletionResponse, LlmClient, OllamaClient, OpenAiClient,
    TokenStream, ToolChoice, ToolRequest, ToolUseBlock,
};
#[cfg(feature = "llm")]
pub use complete::{complete, complete_with, CompleteOptions};
#[cfg(feature = "llm")]
pub use config::AiConfig;
// Confirmation re-exports — always available (reqwest-free).
pub use confirmation::events::ConfirmationExpired;
pub use confirmation::store::InMemoryConfirmationStore;
pub use confirmation::{ConfirmationStore, PendingActionInfo};
#[cfg(feature = "llm")]
pub use embed::embed;
pub use error::Error;
#[cfg(feature = "llm")]
pub use schema::for_structured_output;
#[cfg(feature = "llm")]
pub use similarity::cosine_similarity;
#[cfg(feature = "llm")]
pub use tools::{make_handler, ToolDef, ToolError, ToolRegistry};

#[cfg(feature = "pgvector")]
pub use pgvector::{Neighbor, PgVectorStore};

/// Process-wide mutex that serializes env-var mutation across all test modules.
///
/// Tests that read or write `FERRO_AI_*` environment variables must hold this
/// lock for the duration of the test to prevent cross-module interference.
/// Per-module `static ENV_LOCK` instances are insufficient because each module
/// gets its own mutex instance; only a crate-level mutex serializes across modules.
#[cfg(test)]
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
