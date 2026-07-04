//! Analyze archetype. Basic tier degrades to stats + tabular list; charts
//! are a Ferro-tier capability (see catalog).

use crate::builder::{browse, emit_title, summarize, Emit};
use crate::catalog::CatalogTier;
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::{Error, ServiceDef};
use ferro_theme::IntentSlotTemplate;

/// Ferro-tier `fields`: a `LineChart` bound to `/series`.
fn emit_line_chart(e: &mut Emit) -> String {
    e.contract.bind("/series", None, None);
    e.push(Component::new("chart", "LineChart").bound_prop("series", "/series"));
    "chart".to_string()
}

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "stats" => {
                if let Some(id) = summarize::emit_stats_slot(e, service, ctx) {
                    children.push(id);
                }
            }
            "fields" => children.push(if ctx.tier == CatalogTier::Ferro {
                emit_line_chart(e)
            } else {
                browse::emit_item_list(e, service)
            }),
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

    #[test]
    fn ferro_tier_analyze_emits_line_chart() {
        let ctx = A2uiContext {
            tier: crate::CatalogTier::Ferro,
            ..Default::default()
        };
        let out = A2uiRenderer
            .render(&order_service(), &scored(Intent::Analyze), &ctx)
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let root = cs.components.iter().find(|c| c.id == "root").unwrap();
        assert_eq!(
            root.props["children"],
            serde_json::json!(["title", "stats", "chart"])
        );
        let chart = cs.components.iter().find(|c| c.id == "chart").unwrap();
        assert_eq!(chart.component, "LineChart");
        assert_eq!(
            chart.props["series"],
            serde_json::json!({"path": "/series"})
        );
        assert!(out.data_contract.paths().contains(&"/series"));
    }
}
