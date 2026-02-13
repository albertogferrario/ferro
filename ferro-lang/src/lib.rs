//! # Ferro Lang
//!
//! Localization for the Ferro web framework.
//!
//! Provides JSON-based translation loading, `:param` interpolation,
//! pipe-separated pluralization, and locale fallback chains.
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_lang::Translator;
//!
//! // Load translations from lang/ directory with "en" as fallback
//! let translator = Translator::load("lang", "en").unwrap();
//!
//! // Simple translation with parameter interpolation
//! let msg = translator.get("en", "welcome", &[("name", "Alice")]);
//! assert_eq!(msg, "Welcome, Alice!");
//!
//! // Pluralized translation
//! let msg = translator.choice("en", "items.count", 5, &[]);
//! assert_eq!(msg, "5 items");
//! ```

mod error;
mod interpolation;
mod loader;
mod pluralization;
mod translator;

pub use error::LangError;
pub use translator::Translator;
