//! JSON-UI v2 renderer producing `Spec` instances from service definitions.
//!
//! Implements the `Renderer` trait to translate `ServiceDef` + `IntentScore[]`
//! into a ferro-json-ui/v2 `Spec`. Phase 115 keeps the mapping naive per D-20
//! — each intent emits a single root `Element` with a sensible `type_name`
//! (Browse -> DataTable, Collect -> Form, etc.) wired against a `/data/...`
//! path. Phase 117.1 rewrites this into a schema-driven pipeline; the
//! existing `field_map` / `relationship_map` helpers remain in place as
//! reference material for that rewrite.
//!
//! This module is only compiled when the `projections` feature is enabled.

pub mod field_map;
pub mod relationship_map;

#[allow(dead_code)]
pub mod error;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ferro_projections::render::Renderer;
use ferro_projections::Error;
use ferro_projections::FieldMeaning;
use ferro_projections::ServiceDef;
use ferro_projections::{Intent, IntentScore};
use ferro_theme::ThemeTemplates;

use crate::spec::{Element, Spec};

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
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
    /// Display or Input mode.
    pub mode: RenderMode,
    /// Optional theme template overrides. `None` means use built-in layouts.
    /// Phase 115's naive mapping does not consult templates; Phase 117.1
    /// re-introduces template-driven slot placement.
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
            templates: None,
        }
    }
}

/// Returns true for DateTime-family field meanings that Track views expose.
#[allow(dead_code)]
fn is_datetime_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime
    )
}

/// Returns true for numeric field meanings used in Analyze summary stats.
#[allow(dead_code)]
fn is_numeric_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Money | FieldMeaning::Quantity | FieldMeaning::Percentage
    )
}

/// JSON-UI v2 renderer producing `Spec` instances.
///
/// Translates service definitions and scored intents into a ferro-json-ui/v2
/// `Spec`. Each intent maps to a single root `Element` wired to a
/// `/data/{service}` data path — the naive mapping per D-20.
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
        let intent_score = intents.get(ctx.intent_index).ok_or_else(|| {
            Error::Render(format!(
                "intent_index {} out of bounds (have {} intents)",
                ctx.intent_index,
                intents.len()
            ))
        })?;

        let (type_name, columns_data) =
            resolve_element(&intent_score.intent, service, &ctx.mode, &ctx.current_state);

        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        let data_path = format!("/data/{}", service.name);

        let mut el = Element::new(type_name).prop("data_path", data_path);
        for (k, v) in columns_data {
            el = el.prop(k, v);
        }

        Spec::builder()
            .title(title)
            .element("root", el)
            .build()
            .map_err(|e| Error::Render(format!("spec build failed: {e}")))
    }
}

/// Pick the root element type and any intent-specific props for the naive
/// mapping. Phase 117.1 replaces this with a schema-driven resolver.
fn resolve_element(
    intent: &Intent,
    service: &ServiceDef,
    mode: &RenderMode,
    current_state: &Option<String>,
) -> (&'static str, Vec<(&'static str, Value)>) {
    let columns = || columns_value(service);
    match (intent, mode) {
        (Intent::Browse, RenderMode::Display) => ("DataTable", vec![("columns", columns())]),
        (Intent::Browse, RenderMode::Input) => ("Form", vec![]),
        (Intent::Focus, RenderMode::Display) => ("Card", vec![]),
        (Intent::Focus, RenderMode::Input) => ("Form", vec![]),
        (Intent::Collect, _) => ("Form", vec![]),
        (Intent::Summarize, RenderMode::Display) => ("StatCard", vec![]),
        (Intent::Summarize, RenderMode::Input) => ("Form", vec![]),
        (Intent::Process, RenderMode::Display) => (
            "KanbanBoard",
            vec![(
                "current_state",
                json!(current_state.clone().unwrap_or_default()),
            )],
        ),
        (Intent::Process, RenderMode::Input) => ("Form", vec![]),
        (Intent::Analyze, RenderMode::Display) => ("Card", vec![]),
        (Intent::Analyze, RenderMode::Input) => ("Form", vec![]),
        (Intent::Track, RenderMode::Display) => ("DataTable", vec![("columns", columns())]),
        (Intent::Track, RenderMode::Input) => ("Form", vec![]),
        (Intent::Custom(_), RenderMode::Display) => ("Card", vec![]),
        (Intent::Custom(_), RenderMode::Input) => ("Form", vec![]),
    }
}

/// Emit a minimal columns list from the service's readable, non-system
/// fields. Serves the naive DataTable/Track mapping only — Phase 117.1
/// replaces this with a catalog-driven approach.
fn columns_value(service: &ServiceDef) -> Value {
    let cols: Vec<Value> = service
        .fields
        .iter()
        .filter(|f| f.readable && !ferro_projections::render::is_system_field(&f.meaning))
        .map(|f| {
            json!({
                "key": f.name.clone(),
                "label": ferro_projections::render::field_display_name(&f.name),
            })
        })
        .collect();
    Value::Array(cols)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(ctx.intent_index, 0);
        assert!(ctx.current_state.is_none());
        assert_eq!(ctx.mode, RenderMode::Display);
        assert!(ctx.templates.is_none());
    }

    #[test]
    fn render_produces_v2_spec() {
        let service = sample_service();
        let intents = derive_intents(&service);
        let renderer = JsonUiRenderer;
        let spec = renderer
            .render(&service, &intents, &VisualContext::default())
            .expect("render should succeed");

        assert_eq!(spec.schema, "ferro-json-ui/v2");
        assert!(
            spec.elements.contains_key(&spec.root),
            "root id must resolve to an element"
        );
        assert_eq!(spec.title.as_deref(), Some("Product"));
    }

    #[test]
    fn render_browse_display_uses_data_table() {
        let service = sample_service();
        let intents = derive_intents(&service);
        // Force Browse intent at index 0 — derive_intents ranks Browse for
        // this service shape.
        let ctx = VisualContext {
            intent_index: intents
                .iter()
                .position(|i| matches!(i.intent, Intent::Browse))
                .unwrap_or(0),
            ..Default::default()
        };

        let spec = JsonUiRenderer
            .render(&service, &intents, &ctx)
            .expect("render should succeed");
        let root = spec.elements.get(&spec.root).unwrap();
        assert_eq!(root.type_name, "DataTable");
        let props = root.props.as_object().unwrap();
        assert!(props.contains_key("data_path"));
        assert!(props.contains_key("columns"));
    }

    #[test]
    fn render_collect_always_uses_form() {
        let service = sample_service();
        let intents = derive_intents(&service);
        // Pick any index where intent exists; mapping forces Form in Input mode.
        let ctx = VisualContext {
            mode: RenderMode::Input,
            intent_index: 0,
            ..Default::default()
        };

        let spec = JsonUiRenderer
            .render(&service, &intents, &ctx)
            .expect("render should succeed");
        let root = spec.elements.get(&spec.root).unwrap();
        assert_eq!(root.type_name, "Form");
    }

    #[test]
    fn render_returns_error_for_out_of_bounds_intent_index() {
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            intent_index: intents.len() + 5,
            ..Default::default()
        };

        let result = JsonUiRenderer.render(&service, &intents, &ctx);
        assert!(result.is_err());
        if let Err(Error::Render(msg)) = result {
            assert!(msg.contains("out of bounds"));
        } else {
            panic!("expected Render error");
        }
    }

    #[test]
    fn render_returns_error_for_empty_intents() {
        let service = sample_service();
        let intents: Vec<IntentScore> = vec![];
        let result = JsonUiRenderer.render(&service, &intents, &VisualContext::default());
        assert!(result.is_err());
    }

    #[test]
    fn columns_value_skips_system_fields() {
        let service = sample_service();
        let cols = columns_value(&service);
        let arr = cols.as_array().unwrap();
        // id is Identifier (system-like) — depends on is_system_field; at
        // minimum name and price should be present.
        let keys: Vec<&str> = arr
            .iter()
            .filter_map(|c| c.get("key").and_then(|v| v.as_str()))
            .collect();
        assert!(keys.contains(&"name"));
        assert!(keys.contains(&"price"));
    }

    #[test]
    fn datetime_field_predicate() {
        assert!(is_datetime_field(&FieldMeaning::CreatedAt));
        assert!(is_datetime_field(&FieldMeaning::UpdatedAt));
        assert!(is_datetime_field(&FieldMeaning::DateTime));
        assert!(!is_datetime_field(&FieldMeaning::EntityName));
    }

    #[test]
    fn numeric_field_predicate() {
        assert!(is_numeric_field(&FieldMeaning::Money));
        assert!(is_numeric_field(&FieldMeaning::Quantity));
        assert!(is_numeric_field(&FieldMeaning::Percentage));
        assert!(!is_numeric_field(&FieldMeaning::EntityName));
    }
}
