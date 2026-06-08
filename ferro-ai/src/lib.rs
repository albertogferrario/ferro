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

pub mod classifier;
pub mod client;
pub mod complete;
pub mod config;
pub mod confirmation;
pub mod error;
pub mod schema;
pub mod tools;

pub use classifier::anthropic::AnthropicProvider;
pub use classifier::provider::ClassificationProvider;
pub use classifier::{ClassificationResult, Classifier, ClassifierConfig};
pub use client::{
    AnthropicClient, CompletionRequest, CompletionResponse, LlmClient, OllamaClient, OpenAiClient,
    ToolChoice, ToolRequest, ToolUseBlock, TokenStream,
};
pub use complete::complete;
pub use config::AiConfig;
pub use confirmation::events::ConfirmationExpired;
pub use confirmation::store::InMemoryConfirmationStore;
pub use confirmation::{ConfirmationStore, PendingActionInfo};
pub use error::Error;
pub use schema::for_structured_output;
pub use tools::{ToolDef, ToolError, ToolRegistry};
