//! Rendering abstraction layer for service projections.
//!
//! Defines the `Renderer` trait and supporting types (`RenderContext`, `RenderMode`)
//! that translate `ServiceDef` + `IntentScore` into renderable JSON output.

pub mod field_map;
pub mod relationship_map;

use crate::error::Error;
use crate::field::FieldMeaning;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use serde::{Deserialize, Serialize};

/// Controls whether fields render as read-only display or editable inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// Read-only view (detail pages, lists, summaries).
    Display,
    /// Editable form view (create, edit).
    Input,
}

/// Context passed to a renderer for a single render call.
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
    /// Display or Input mode.
    pub mode: RenderMode,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
        }
    }
}

/// Trait for rendering a service definition into a JSON view specification.
///
/// Implementations translate `ServiceDef` + scored intents into renderer-specific
/// JSON output (e.g., JSON-UI component trees). The output is `serde_json::Value`
/// to avoid coupling to any specific UI framework types.
pub trait Renderer: Send + Sync {
    /// Renders a service definition into a JSON view specification.
    ///
    /// # Arguments
    /// * `service` - The service definition to render
    /// * `intents` - Scored intents from structural analysis (sorted by confidence)
    /// * `ctx` - Rendering context (which intent, mode, state)
    ///
    /// # Errors
    /// Returns `Error::Render` if the rendering process fails.
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &RenderContext,
    ) -> Result<serde_json::Value, Error>;
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
pub(crate) fn is_system_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Identifier | FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_context_default() {
        let ctx = RenderContext::default();
        assert_eq!(ctx.intent_index, 0);
        assert!(ctx.current_state.is_none());
        assert_eq!(ctx.mode, RenderMode::Display);
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

    #[test]
    fn render_mode_serde_round_trip() {
        for mode in [RenderMode::Display, RenderMode::Input] {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: RenderMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    #[test]
    fn render_mode_display_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&RenderMode::Display).unwrap(),
            r#""display""#
        );
        assert_eq!(
            serde_json::to_string(&RenderMode::Input).unwrap(),
            r#""input""#
        );
    }
}
