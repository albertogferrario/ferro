//! Summarize archetype: stat cards for aggregate numeric fields.

use crate::builder::{emit_title, Emit};
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::render::field_display_name;
use ferro_projections::{DataType, Error, FieldMeaning, ServiceDef};
use ferro_theme::IntentSlotTemplate;

/// Emits the `stats` Row of stat Cards; `None` when the service has no
/// Money/Percentage/Quantity fields.
pub(crate) fn emit_stat_cards(e: &mut Emit, service: &ServiceDef) -> Option<String> {
    let stat_fields: Vec<_> = service
        .fields
        .iter()
        .filter(|f| {
            f.readable
                && matches!(
                    f.meaning,
                    FieldMeaning::Money | FieldMeaning::Percentage | FieldMeaning::Quantity
                )
        })
        .collect();
    if stat_fields.is_empty() {
        return None;
    }
    let mut cards = Vec::new();
    for f in stat_fields {
        let card_id = format!("{}_stat", f.name);
        let col_id = format!("{}_stat_col", f.name);
        let value_id = format!("{}_stat_value", f.name);
        let label_id = format!("{}_stat_label", f.name);
        let display_path = format!("/stats/{}/display", f.name);
        // Host pre-formats display strings server-side (spec: Value formatting);
        // the raw value is contracted alongside for future client-side formatting.
        e.contract
            .bind(display_path.clone(), Some(DataType::String), Some(&f.name));
        e.contract.bind(
            format!("/stats/{}/value", f.name),
            Some(f.data_type),
            Some(&f.name),
        );
        e.push(
            Component::new(value_id.clone(), "Text")
                .bound_prop("text", display_path)
                .prop("variant", "heading"),
        );
        e.push(Component::new(label_id.clone(), "Text").prop("text", field_display_name(&f.name)));
        e.push(Component::new(col_id.clone(), "Column").children_ids([value_id, label_id]));
        e.push(Component::new(card_id.clone(), "Card").child(col_id));
        cards.push(card_id);
    }
    e.push(Component::new("stats", "Row").children_ids(cards));
    Some("stats".to_string())
}

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    _ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "stats" => {
                if let Some(id) = emit_stat_cards(e, service) {
                    children.push(id);
                }
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
    fn summarize_emits_stat_cards_bound_to_display_strings() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Summarize),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let by_id = |id: &str| cs.components.iter().find(|c| c.id == id).unwrap();
        // total is the only Money/Percentage/Quantity field in the fixture
        assert_eq!(by_id("stats").component, "Row");
        assert_eq!(by_id("total_stat").component, "Card");
        assert_eq!(
            by_id("total_stat_value").props["text"],
            serde_json::json!({"path": "/stats/total/display"})
        );
        assert_eq!(
            by_id("total_stat_value").props["variant"],
            serde_json::json!("heading")
        );
        assert_eq!(
            by_id("total_stat_label").props["text"],
            serde_json::json!("Total")
        );
        let paths = out.data_contract.paths();
        assert!(paths.contains(&"/stats/total/display"));
        assert!(paths.contains(&"/stats/total/value"));
    }
}
