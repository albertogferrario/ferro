//! Rendering abstraction layer for service projections.
//!
//! Defines the `Renderer` trait and `BaseContext` (modality-agnostic fields)
//! that translate `ServiceDef` + `IntentScore` into renderable output.
//! Concrete renderer implementations live in their respective output crates
//! (e.g., `JsonUiRenderer` in ferro-json-ui).

// Research sketch — not stable API
pub(crate) mod sketch;
pub mod template;

use crate::error::Error;
use crate::field::FieldMeaning;
use crate::intent::IntentScore;
use crate::service::ServiceDef;
use std::collections::HashMap;

/// Text detail level for non-visual rendering.
///
/// `Full` reproduces the current full-render behavior, so it is the
/// backward-compatible default. `Brief` is consumed by non-visual renderers
/// (e.g., the Phase 216 conversational-text renderer) to omit secondary detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Full detail — preserves current visual render behavior. (default)
    #[default]
    Full,
    /// Reduced detail for non-visual channels.
    Brief,
}

/// Modality-agnostic rendering context shared by all `Renderer` implementations.
///
/// Visual-only fields (render mode, theme templates) live in `VisualContext`
/// inside `ferro-json-ui`.
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
    /// Guard-name → evaluated result. Absent key = render the action
    /// (guard not-yet-evaluated / unconstrained); only an explicit `false`
    /// filters an action out. Keyed by the strings in `ActionDef::preconditions`
    /// / `GuardDef::name`. Default = empty map = render everything. (D-03/D-04)
    pub evaluated_guards: HashMap<String, bool>,
    /// Text detail level. `Full` (default) preserves current full-render
    /// behavior; `Brief` is consumed by non-visual renderers. (D-05)
    pub verbosity: Verbosity,
}

/// Trait for rendering a service definition into output of an associated type.
///
/// Implementations translate `ServiceDef` + scored intents into renderer-specific
/// output. The associated `Output` and `Context` types allow renderers to operate
/// on different targets (JSON-UI visual trees, template contexts, voice payloads,
/// etc.) without coupling to any single output format.
pub trait Renderer: Send + Sync {
    /// The output type produced by this renderer (e.g., `serde_json::Value`).
    type Output;
    /// The context type consumed by this renderer. Must implement `Default`.
    type Context: Default;

    /// Renders a service definition into the renderer's output type.
    ///
    /// # Arguments
    /// * `service` - The service definition to render
    /// * `intents` - Scored intents from structural analysis (sorted by confidence)
    /// * `ctx` - Rendering context (renderer-specific)
    ///
    /// # Errors
    /// Returns `Error::Render` if the rendering process fails.
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &Self::Context,
    ) -> Result<Self::Output, Error>;
}

/// Converts a snake_case field name to a title case display label.
///
/// Splits on underscores, capitalizes each word's first character.
///
/// ```
/// use ferro_projections::render::field_display_name;
///
/// assert_eq!(field_display_name("user_name"), "User Name");
/// assert_eq!(field_display_name("email"), "Email");
/// ```
pub fn field_display_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.collect::<String>()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns true for system/infrastructure field meanings that should not
/// contribute to domain intent signals or appear in user-facing layouts.
pub fn is_system_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Identifier | FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_context_default() {
        let ctx = BaseContext::default();
        assert_eq!(ctx.intent_index, 0);
        assert!(ctx.current_state.is_none());
        assert!(ctx.evaluated_guards.is_empty());
        assert_eq!(ctx.verbosity, Verbosity::Full);
    }

    #[test]
    fn verbosity_default_is_full() {
        assert_eq!(Verbosity::default(), Verbosity::Full);
    }

    #[test]
    fn field_display_name_multi_word() {
        assert_eq!(field_display_name("user_name"), "User Name");
    }

    #[test]
    fn field_display_name_single_word() {
        assert_eq!(field_display_name("email"), "Email");
    }

    #[test]
    fn field_display_name_timestamp() {
        assert_eq!(field_display_name("created_at"), "Created At");
    }

    #[test]
    fn field_display_name_empty() {
        assert_eq!(field_display_name(""), "");
    }

    #[test]
    fn is_system_field_identifies_system_meanings() {
        assert!(is_system_field(&FieldMeaning::Identifier));
        assert!(is_system_field(&FieldMeaning::CreatedAt));
        assert!(is_system_field(&FieldMeaning::UpdatedAt));
    }

    #[test]
    fn is_system_field_rejects_domain_meanings() {
        assert!(!is_system_field(&FieldMeaning::Money));
        assert!(!is_system_field(&FieldMeaning::EntityName));
        assert!(!is_system_field(&FieldMeaning::FreeText));
        assert!(!is_system_field(&FieldMeaning::Status));
        assert!(!is_system_field(&FieldMeaning::Custom("x".into())));
    }
}
