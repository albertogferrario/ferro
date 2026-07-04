//! Track archetype: timeline list of events.

use crate::builder::{emit_title, Emit};
use crate::catalog::CatalogTier;
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::{Error, FieldMeaning, ServiceDef};
use ferro_theme::IntentSlotTemplate;

/// First readable field matching the earliest meaning in `meanings` —
/// meaning-priority order, not declaration order.
fn first_field_with<'a>(service: &'a ServiceDef, meanings: &[FieldMeaning]) -> Option<&'a str> {
    meanings.iter().find_map(|m| {
        service
            .fields
            .iter()
            .find(|f| f.readable && f.meaning == *m)
            .map(|f| f.name.as_str())
    })
}

/// Ferro-tier `fields`: a `Timeline` bound to `/events`.
fn emit_timeline_component(e: &mut Emit) -> String {
    e.contract.bind("/events", None, None);
    e.push(Component::new("timeline", "Timeline").bound_prop("events", "/events"));
    "timeline".to_string()
}

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let time_field = first_field_with(
        service,
        &[
            FieldMeaning::CreatedAt,
            FieldMeaning::UpdatedAt,
            FieldMeaning::DateTime,
        ],
    )
    .unwrap_or("occurred_at")
    .to_string();
    let text_field = first_field_with(
        service,
        &[
            FieldMeaning::Status,
            FieldMeaning::EntityName,
            FieldMeaning::FreeText,
        ],
    )
    .unwrap_or("description")
    .to_string();

    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "fields" if ctx.tier == CatalogTier::Ferro => {
                children.push(emit_timeline_component(e));
            }
            "fields" => {
                e.contract.bind("/events", None, None);
                e.contract
                    .bind(format!("/events/*/{time_field}"), None, Some(&time_field));
                e.contract
                    .bind(format!("/events/*/{text_field}"), None, Some(&text_field));
                e.push(Component::new("timeline", "List").template_children("/events", "event"));
                e.push(Component::new("event_icon", "Icon").prop("name", "circle"));
                e.push(Component::new("event_time", "Text").bound_prop("text", time_field.clone()));
                e.push(Component::new("event_text", "Text").bound_prop("text", text_field.clone()));
                e.push(Component::new("event", "Row").children_ids([
                    "event_icon",
                    "event_time",
                    "event_text",
                ]));
                children.push("timeline".to_string());
            }
            _ => {}
        }
    }
    Ok(children)
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::Intent;

    #[test]
    fn track_emits_timeline_with_time_and_text_bindings() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Track),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let by_id = |id: &str| cs.components.iter().find(|c| c.id == id).unwrap();
        assert_eq!(
            by_id("timeline").props["children"],
            serde_json::json!({"path": "/events", "componentId": "event"})
        );
        assert_eq!(
            by_id("event_icon").props["name"],
            serde_json::json!("circle")
        );
        // fixture: created_at is the first temporal field; status the first status-ish field
        assert_eq!(
            by_id("event_time").props["text"],
            serde_json::json!({"path": "created_at"})
        );
        assert_eq!(
            by_id("event_text").props["text"],
            serde_json::json!({"path": "status"})
        );
        let paths = out.data_contract.paths();
        assert!(paths.contains(&"/events"));
        assert!(paths.contains(&"/events/*/created_at"));
        assert!(paths.contains(&"/events/*/status"));
    }

    #[test]
    fn ferro_tier_track_emits_timeline_component() {
        let ctx = A2uiContext {
            tier: crate::CatalogTier::Ferro,
            ..Default::default()
        };
        let out = A2uiRenderer
            .render(&order_service(), &scored(Intent::Track), &ctx)
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let timeline = cs.components.iter().find(|c| c.id == "timeline").unwrap();
        assert_eq!(timeline.component, "Timeline");
        assert_eq!(
            timeline.props["events"],
            serde_json::json!({"path": "/events"})
        );
        assert!(out.data_contract.paths().contains(&"/events"));
    }
}
