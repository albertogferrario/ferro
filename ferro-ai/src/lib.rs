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

pub mod classifier;
pub mod error;

pub use classifier::provider::ClassificationProvider;
pub use classifier::{ClassificationResult, Classifier, ClassifierConfig};
pub use error::Error;
