//! Analyze archetype. Basic tier degrades to stats + tabular list; charts
//! are a Ferro-tier capability (see catalog).

use crate::builder::{browse, emit_title, summarize, Emit};
use crate::context::A2uiContext;
use ferro_projections::{Error, ServiceDef};
use ferro_theme::IntentSlotTemplate;

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
                if let Some(id) = summarize::emit_stat_cards(e, service) {
                    children.push(id);
                }
            }
            "fields" => children.push(browse::emit_item_list(e, service)),
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
    fn analyze_degrades_to_stats_plus_table() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Analyze),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let root = cs.components.iter().find(|c| c.id == "root").unwrap();
        assert_eq!(
            root.props["children"],
            serde_json::json!(["title", "stats", "items"])
        );
    }
}
