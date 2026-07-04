//! Emission core: dispatches intents to archetype builders and assembles
//! the `createSurface` skeleton.

mod browse;
mod collect;
mod focus;
mod process;
mod summarize;
mod track;

use crate::component::Component;
use crate::context::A2uiContext;
use crate::message::{A2uiMessage, CreateSurface};
use crate::surface::{DataContract, SurfaceRendering};
use crate::template::{resolve_template, Mode};
use ferro_projections::render::{field_display_name, is_system_field};
use ferro_projections::{Error, FieldDef, Intent, IntentScore, ServiceDef};
use serde_json::Value;

/// Accumulates components and contract bindings during emission.
#[derive(Default)]
pub(crate) struct Emit {
    pub components: Vec<Component>,
    pub contract: DataContract,
}

impl Emit {
    pub(crate) fn push(&mut self, c: Component) {
        self.components.push(c);
    }
}

/// Readable, non-system fields in declaration order.
pub(crate) fn display_fields(service: &ServiceDef) -> Vec<&FieldDef> {
    service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .collect()
}

/// Writable, non-system fields in declaration order.
pub(crate) fn writable_fields(service: &ServiceDef) -> Vec<&FieldDef> {
    service
        .fields
        .iter()
        .filter(|f| f.writable && !is_system_field(&f.meaning))
        .collect()
}

/// Emits the heading `Text` for the service; returns its component ID.
pub(crate) fn emit_title(e: &mut Emit, service: &ServiceDef) -> String {
    let label = service
        .display_name
        .clone()
        .unwrap_or_else(|| field_display_name(&service.name));
    e.push(
        Component::new("title", "Text")
            .prop("text", label)
            .prop("variant", "heading"),
    );
    "title".to_string()
}

fn surface_properties(ctx: &A2uiContext) -> Option<Value> {
    let mut map = serde_json::Map::new();
    if let Some(name) = &ctx.config.app_name {
        map.insert("agentDisplayName".into(), Value::String(name.clone()));
    }
    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

/// Builds a full surface rendering for the selected intent.
pub(crate) fn build(
    service: &ServiceDef,
    intents: &[IntentScore],
    ctx: &A2uiContext,
) -> Result<SurfaceRendering, Error> {
    let score = intents.get(ctx.base.intent_index).ok_or(Error::NoIntents)?;
    let intent = &score.intent;
    let mode = if matches!(intent, Intent::Collect) {
        Mode::Input
    } else {
        Mode::Display
    };
    let template = resolve_template(intent, mode, ctx.templates.as_ref());
    let mut emit = Emit::default();

    // Archetype dispatch — the analyze arm lands in Task 14.
    let child_ids = match intent.label() {
        "collect" => collect::emit(&mut emit, service, ctx, &template)?,
        "focus" => focus::emit(&mut emit, service, ctx, &template)?,
        "process" => process::emit(&mut emit, service, ctx, &template)?,
        "summarize" => summarize::emit(&mut emit, service, ctx, &template)?,
        "track" => track::emit(&mut emit, service, ctx, &template)?,
        // Browse and custom intents share the browse shape (mirrors ferro-text).
        _ => browse::emit(&mut emit, service, ctx, &template)?,
    };

    let root = Component::new("root", "Column").children_ids(child_ids);
    let mut components = vec![root];
    components.append(&mut emit.components);

    let send_data_model = if ctx.send_data_model || matches!(intent, Intent::Collect) {
        Some(true)
    } else {
        None
    };

    let create = CreateSurface {
        surface_id: ctx
            .surface_id
            .clone()
            .unwrap_or_else(|| format!("ferro-{}-{}", service.name, intent.label())),
        catalog_id: ctx.tier.catalog_id().to_string(),
        surface_properties: surface_properties(ctx),
        send_data_model,
        components,
        data_model: None,
    };
    Ok(SurfaceRendering {
        messages: vec![A2uiMessage::CreateSurface(create)],
        catalog_id: ctx.tier.catalog_id().to_string(),
        data_contract: emit.contract,
    })
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::{Error, Intent};

    #[test]
    fn empty_intents_is_no_intents_error() {
        let err = A2uiRenderer
            .render(&order_service(), &[], &A2uiContext::default())
            .unwrap_err();
        assert!(matches!(err, Error::NoIntents));
    }

    #[test]
    fn render_produces_create_surface_with_root_first() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Focus),
                &A2uiContext::default(),
            )
            .unwrap();
        assert_eq!(out.catalog_id, crate::catalog::BASIC_CATALOG_ID);
        assert_eq!(out.messages.len(), 1);
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!("expected createSurface")
        };
        assert_eq!(cs.surface_id, "ferro-order-focus");
        assert_eq!(cs.catalog_id, crate::catalog::BASIC_CATALOG_ID);
        assert_eq!(cs.components[0].id, "root");
        assert_eq!(cs.components[0].component, "Column");
        // title emitted by the placeholder arm
        assert!(cs.components.iter().any(|c| c.id == "title"));
    }

    #[test]
    fn surface_properties_carry_app_name() {
        let mut ctx = A2uiContext::default();
        ctx.config.app_name = Some("My Shop".into());
        let out = A2uiRenderer
            .render(&order_service(), &scored(Intent::Focus), &ctx)
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        assert_eq!(
            cs.surface_properties,
            Some(serde_json::json!({"agentDisplayName": "My Shop"}))
        );
    }

    #[test]
    fn title_uses_display_name() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Browse),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let title = cs.components.iter().find(|c| c.id == "title").unwrap();
        assert_eq!(title.props["text"], serde_json::json!("Order"));
        assert_eq!(title.props["variant"], serde_json::json!("heading"));
    }
}
