//! # Ferro Theme
//!
//! Semantic theme tokens and intent template schema for Ferro.
//!
//! Provides the [`Theme`] struct which holds an embedded or filesystem-loaded
//! CSS file (using Tailwind v4 `@theme` syntax) and optional [`ThemeTemplates`]
//! that configure how each of the 7 intents renders its slot layout.
//!
//! ## Example
//!
//! ```rust
//! use ferro_theme::Theme;
//!
//! // Always available — no filesystem dependency
//! let theme = Theme::default_theme();
//! assert!(theme.css.contains("--color-primary"));
//! ```

mod error;
mod loader;
mod template;
pub mod token;

pub use error::ThemeError;
pub use loader::Theme;
pub use template::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};
