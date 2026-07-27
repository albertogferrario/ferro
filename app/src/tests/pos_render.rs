//! POS register render contracts: the /pos page is projection-derived and the
//! rendered page carries the fill_viewport class chain.

#[cfg(test)]
mod tests {
    use crate::controllers::pos::{pos_products, pos_service_def};
    use ferro::serde_json::json;
    use ferro::{
        derive_intents, register_template, JsonUi, JsonUiRenderer, Renderer, VisualContext,
    };
    use ferro_json_ui::lint;

    #[test]
    fn pos_render_is_projection_derived_fill_viewport() {
        let service = pos_service_def();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            templates: Some(register_template()),
            ..Default::default()
        };
        // In debug this panics if the spec is catalog-invalid — SC-1 gate, app-side.
        let spec = JsonUiRenderer
            .render(&service, &intents, &ctx)
            .expect("register spec must project");

        assert!(spec.fill_viewport, "register spec must set fill_viewport");
        assert_eq!(spec.layout.as_deref(), Some("dashboard"));

        // Belt-and-suspenders: derived spec is lint-clean for the register rules (D-05).
        let register_rules = [
            "register-fill-viewport",
            "register-grid-fill",
            "register-selection-present",
            "fill-viewport-layout-unknown",
        ];
        let hits: Vec<_> = lint(&spec)
            .into_iter()
            .filter(|f| register_rules.contains(&f.rule))
            .collect();
        assert!(
            hits.is_empty(),
            "register spec must be lint-clean: {hits:#?}"
        );

        let data = json!({ "data": { "pos": pos_products() } });
        let resp = JsonUi::render(&spec, &data).expect("render ok");
        assert_eq!(resp.status_code(), 200);
        let html = resp.body();
        // fill_viewport → `ferro-fill` body class chain on the rendered page.
        assert!(
            html.contains("ferro-fill"),
            "page must carry the ferro-fill class chain"
        );
        // Register composition markers (256 render contracts):
        assert!(
            html.contains("data-selection-panel"),
            "SelectionPanel must render"
        );
        assert!(
            html.contains("data-filter-search"),
            "TileGrid search input must render"
        );
        assert!(
            html.contains("Confirm order"),
            "confirm button label must render"
        );
        // Guard against empty-grid regressions: a real product row must render
        // from the $each expansion over /data/pos.
        assert!(
            html.contains("Espresso"),
            "product tiles must render from $each expansion"
        );
        // 257-04: the sale_form carries the fill height-chain marker so the
        // SelectionPanel footer pins in-viewport (256 D-15 contract). Geometry
        // itself cannot be asserted in Rust; the class-chain presence is the
        // proxy — live geometry re-verify happens in UAT.
        assert!(
            html.contains("[&>*]:flex-1 [&>*]:min-h-0"),
            "sale_form must carry the fill height-chain marker [&>*]:flex-1 [&>*]:min-h-0"
        );
    }
}
