//! JSON-UI v2 projection pipeline — schema-driven spec generation.
//!
//! `JsonUiRenderer` implements the modality-agnostic `Renderer` trait from
//! `ferro-projections`, delegating to [`Spec::from_service_def`] (see
//! [`builder`]). The pipeline consumes the static vocabulary in
//! [`component_map`] (meaning → component + typed Props) and [`intent_layout`]
//! (intent → slot template), validates every output against
//! [`crate::catalog::global_catalog`], and returns [`ProjectionError`] on
//! schema drift or spec-build failure.
//!
//! This module is only compiled when the `projections` feature is enabled.

pub mod builder;
pub mod component_map;
pub mod error;
pub mod intent_layout;

pub use error::ProjectionError;

use serde::{Deserialize, Serialize};

use ferro_projections::render::{BaseContext, Renderer};
use ferro_projections::Error;
use ferro_projections::IntentScore;
use ferro_projections::ServiceDef;
use ferro_theme::ThemeTemplates;

use crate::spec::Spec;

/// Controls whether fields render as read-only display or editable inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// Read-only view (detail pages, lists, summaries).
    Display,
    /// Editable form view (create, edit).
    Input,
}

/// Visual rendering context for `JsonUiRenderer`.
///
/// Extends the modality-agnostic fields from `BaseContext` with
/// visual-specific concerns: render mode and theme template overrides.
#[derive(Debug, Clone)]
pub struct VisualContext {
    /// Modality-agnostic context (intent index, state, guards, verbosity).
    pub base: BaseContext,
    /// Display or Input mode.
    pub mode: RenderMode,
    /// Optional theme template overrides. `None` means use built-in
    /// `default_template` per intent; when present, `pick_intent_template`
    /// selects the override for the current intent (or falls back to the
    /// built-in default if that intent's override is `None`).
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            base: BaseContext::default(),
            mode: RenderMode::Display,
            templates: None,
        }
    }
}

/// JSON-UI v2 renderer producing `Spec` instances.
///
/// Translates service definitions and scored intents into a ferro-json-ui/v2
/// `Spec` via [`Spec::from_service_def`]. Every output is validated against
/// the global catalog; mismatches surface as `Error::Render` at this boundary.
///
/// # Example
///
/// ```
/// use ferro_projections::{ServiceDef, DataType, FieldMeaning, derive_intents};
/// use ferro_json_ui::{JsonUiRenderer, VisualContext};
/// use ferro_projections::render::Renderer;
///
/// let product = ServiceDef::new("product")
///     .display_name("Product")
///     .field("id", DataType::Integer, FieldMeaning::Identifier)
///     .field("name", DataType::String, FieldMeaning::EntityName)
///     .field("price", DataType::Float, FieldMeaning::Money);
///
/// let intents = derive_intents(&product);
/// let renderer = JsonUiRenderer;
/// let result = renderer.render(&product, &intents, &VisualContext::default());
/// assert!(result.is_ok());
///
/// let spec = result.unwrap();
/// assert_eq!(spec.schema, "ferro-json-ui/v2");
/// assert!(spec.elements.contains_key(&spec.root));
/// ```
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = Spec;
    type Context = VisualContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Spec, Error> {
        Spec::from_service_def(service, intents, ctx).map_err(|e| Error::Render(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    //! Tests here exercise `JsonUiRenderer::render` and the trivially-derived
    //! `RenderMode` / `VisualContext` surface. Happy-path tests that would
    //! require `global_catalog()` inside `Spec::from_service_def` are covered
    //! by `projection::builder::tests` using the injected-catalog pattern
    //! (`Spec::from_service_def_with_catalog`) to stay immune to OnceLock
    //! pollution from sibling plugin tests (see `catalog.rs:1117` and Plan 01
    //! SUMMARY deviation #1). The tests in this module cover:
    //!   * `RenderMode` serde semantics
    //!   * `VisualContext::default` sensible values
    //!   * `JsonUiRenderer::render` error-path delegation — both short-circuit
    //!     paths (`EmptyIntents`, `IntentIndexOutOfBounds`) are checked BEFORE
    //!     `global_catalog()` is consulted (see `from_service_def` body), so
    //!     these assertions run cleanly without touching the global catalog.
    use super::*;
    use ferro_projections::render::BaseContext;
    use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};

    fn sample_service() -> ServiceDef {
        ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
    }

    #[test]
    fn render_mode_serde_round_trip() {
        let mode = RenderMode::Display;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"display\"");
        let parsed: RenderMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mode);
    }

    #[test]
    fn render_mode_display_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&RenderMode::Display).unwrap(),
            "\"display\""
        );
        assert_eq!(
            serde_json::to_string(&RenderMode::Input).unwrap(),
            "\"input\""
        );
    }

    #[test]
    fn visual_context_default_has_sensible_values() {
        let ctx = VisualContext::default();
        assert_eq!(ctx.base.intent_index, 0);
        assert!(ctx.base.current_state.is_none());
        assert_eq!(ctx.mode, RenderMode::Display);
        assert!(ctx.templates.is_none());
    }

    #[test]
    fn render_out_of_bounds_intent_returns_render_error() {
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            base: BaseContext {
                intent_index: intents.len() + 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = JsonUiRenderer.render(&service, &intents, &ctx);
        match result {
            Err(Error::Render(msg)) => assert!(msg.contains("out of bounds")),
            _ => panic!("expected Error::Render, got {result:?}"),
        }
    }

    #[test]
    fn render_empty_intents_returns_render_error() {
        let service = sample_service();
        let result = JsonUiRenderer.render(&service, &[], &VisualContext::default());
        assert!(matches!(result, Err(Error::Render(_))));
    }
}
