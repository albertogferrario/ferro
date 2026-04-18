//! Phase 116 walker: flat-element renderer for v2 Specs.
//!
//! Replaces the Phase 115 placeholder. Walks `spec.elements` by ID starting at
//! `spec.root`, dispatches per-element by `type_name`, and lets each container
//! recurse via `render_element` for child IDs. Per CONTEXT D-09 the renderer is
//! infallible — every failure path emits an HTML comment per D-10 and returns
//! an empty string for the offending element.
//!
//! Per-component bodies live in:
//! - `render/atoms.rs` — leaf renderers (Plan 116-03)
//! - `render/containers.rs` — multi-child layout components (Plan 116-04)
//! - `render/form.rs` — Form/Input/Select/Checkbox/Switch (Plan 116-05)
//! - `render/data.rs` — Table/DataTable (Plan 116-05)

use serde_json::Value;
use std::collections::HashSet;

use crate::plugin::{collect_plugin_assets, with_plugin, Asset};
use crate::spec::{Spec, MAX_NESTING_DEPTH};

pub(crate) mod atoms;
pub(crate) mod containers;
pub(crate) mod data;
pub(crate) mod form;

/// Plugin-asset bundle returned by `render_spec_to_html_with_plugins`.
pub struct RenderResult {
    pub html: String,
    pub css_head: String,
    pub scripts: String,
}

/// Canonical list of built-in component type names. Single source of truth for
/// distinguishing built-ins (handled by the dispatch match below) from plugins
/// (handled by the default arm via `with_plugin`). Per CONTEXT D-19 plugins
/// cannot shadow built-ins — if `type_name` matches an entry here, the dispatch
/// arm wins regardless of plugin registry contents.
///
/// Order matches the dispatch match below for reviewability. Adding a new
/// built-in requires updating BOTH this list AND the dispatch arm.
pub(crate) const BUILTIN_TYPES: &[&str] = &[
    // Leaves (atoms.rs)
    "Text",
    "Button",
    "Badge",
    "Alert",
    "Separator",
    "Progress",
    "Avatar",
    "Image",
    "Skeleton",
    "Breadcrumb",
    "Pagination",
    "DescriptionList",
    "EmptyState",
    "StatCard",
    "Checklist",
    "Toast",
    "NotificationDropdown",
    "Sidebar",
    "Header",
    "DropdownMenu",
    "CalendarCell",
    "ActionCard",
    "ProductTile",
    // Containers (containers.rs)
    "Card",
    "Modal",
    "Tabs",
    "KanbanBoard",
    "PageHeader",
    "Grid",
    "Collapsible",
    "FormSection",
    "ButtonGroup",
    // Form controls (form.rs)
    "Form",
    "Input",
    "Select",
    "Checkbox",
    "Switch",
    // Data displays (data.rs)
    "Table",
    "DataTable",
];

/// Top-level entry point. Walks the spec from `spec.root`, returns the rendered
/// HTML wrapped in v1's flex-wrap container. Per CONTEXT D-09 always returns a
/// String — never panics, never returns `Result`.
pub fn render_spec_to_html(spec: &Spec, data: &Value) -> String {
    let body = render_element(&spec.root, spec, data, 1);
    let body_or_root_hidden = if body.is_empty() && spec_root_was_hidden(spec, data) {
        String::from("<!-- ferro-json-ui: root hidden -->")
    } else {
        body
    };
    format!(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">{body_or_root_hidden}</div>"
    )
}

/// Plugin-aware variant. Walks `spec.elements` to collect plugin type names,
/// then asks the registry for their CSS/JS asset URLs.
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    let html = render_spec_to_html(spec, data);
    let plugin_types = collect_plugin_types(spec);
    if plugin_types.is_empty() {
        return RenderResult {
            html,
            css_head: String::new(),
            scripts: String::new(),
        };
    }
    let type_names: Vec<String> = plugin_types.into_iter().collect();
    let assets = collect_plugin_assets(&type_names);
    RenderResult {
        html,
        css_head: render_css_tags(&assets.css),
        scripts: render_js_tags(&assets.js, &assets.init_scripts),
    }
}

/// The one recursive function. All dispatch, visibility, depth-guard, and
/// diagnostic logic lives here. Per CONTEXT D-04 the per-element pipeline is:
/// (1) depth guard, (2) ID lookup, (3) visibility check, (4) dispatch.
pub(crate) fn render_element(id: &str, spec: &Spec, data: &Value, depth: usize) -> String {
    // (1) Depth tripwire (D-11). Phase 115 caps parse-time depth at MAX_NESTING_DEPTH = 3;
    // this fires only for hand-mutated Specs that bypassed `from_json`.
    if depth > MAX_NESTING_DEPTH + 1 {
        return format!(
            "<!-- ferro-json-ui: cycle guard tripped at depth {depth} — spec should have been rejected at parse time -->"
        );
    }

    // (2) ID lookup (D-10 missing-child diagnostic).
    let Some(el) = spec.elements.get(id) else {
        return format!(
            "<!-- ferro-json-ui: element references missing id '{}' -->",
            html_escape(id)
        );
    };

    // (3) Visibility check (D-13/D-14). Invisible → no output, no children walked.
    if let Some(vis) = &el.visible {
        if !vis.evaluate(data) {
            return String::new();
        }
    }

    // (4) Dispatch by type_name. Default arm consults plugin registry.
    match el.type_name.as_str() {
        // Atoms
        "Text" => atoms::render_text(el, spec, data, depth),
        "Button" => atoms::render_button(el, spec, data, depth),
        "Badge" => atoms::render_badge(el, spec, data, depth),
        "Alert" => atoms::render_alert(el, spec, data, depth),
        "Separator" => atoms::render_separator(el, spec, data, depth),
        "Progress" => atoms::render_progress(el, spec, data, depth),
        "Avatar" => atoms::render_avatar(el, spec, data, depth),
        "Image" => atoms::render_image(el, spec, data, depth),
        "Skeleton" => atoms::render_skeleton(el, spec, data, depth),
        "Breadcrumb" => atoms::render_breadcrumb(el, spec, data, depth),
        "Pagination" => atoms::render_pagination(el, spec, data, depth),
        "DescriptionList" => atoms::render_description_list(el, spec, data, depth),
        "EmptyState" => atoms::render_empty_state(el, spec, data, depth),
        "StatCard" => atoms::render_stat_card(el, spec, data, depth),
        "Checklist" => atoms::render_checklist(el, spec, data, depth),
        "Toast" => atoms::render_toast(el, spec, data, depth),
        "NotificationDropdown" => atoms::render_notification_dropdown(el, spec, data, depth),
        "Sidebar" => atoms::render_sidebar(el, spec, data, depth),
        "Header" => atoms::render_header(el, spec, data, depth),
        "DropdownMenu" => atoms::render_dropdown_menu(el, spec, data, depth),
        "CalendarCell" => atoms::render_calendar_cell(el, spec, data, depth),
        "ActionCard" => atoms::render_action_card(el, spec, data, depth),
        "ProductTile" => atoms::render_product_tile(el, spec, data, depth),
        // Containers
        "Card" => containers::render_card(el, spec, data, depth),
        "Modal" => containers::render_modal(el, spec, data, depth),
        "Tabs" => containers::render_tabs(el, spec, data, depth),
        "KanbanBoard" => containers::render_kanban_board(el, spec, data, depth),
        "PageHeader" => containers::render_page_header(el, spec, data, depth),
        "Grid" => containers::render_grid(el, spec, data, depth),
        "Collapsible" => containers::render_collapsible(el, spec, data, depth),
        "FormSection" => containers::render_form_section(el, spec, data, depth),
        "ButtonGroup" => containers::render_button_group(el, spec, data, depth),
        // Form controls
        "Form" => form::render_form(el, spec, data, depth),
        "Input" => form::render_input(el, spec, data, depth),
        "Select" => form::render_select(el, spec, data, depth),
        "Checkbox" => form::render_checkbox(el, spec, data, depth),
        "Switch" => form::render_switch(el, spec, data, depth),
        // Data displays
        "Table" => data::render_table(el, spec, data, depth),
        "DataTable" => data::render_data_table(el, spec, data, depth),
        // Plugin or unknown (D-03, D-17)
        other => render_plugin_or_unknown(other, el, data),
    }
}

fn render_plugin_or_unknown(type_name: &str, el: &crate::spec::Element, data: &Value) -> String {
    match with_plugin(type_name, |p| p.render(&el.props, data)) {
        Some(html) => html,
        None => format!(
            "<!-- ferro-json-ui: unknown component type '{}' -->",
            html_escape(type_name)
        ),
    }
}

/// Helper: detect whether `spec.root` exists and has a visibility rule that
/// evaluates false. Used to choose between empty body and the root-hidden
/// diagnostic comment.
fn spec_root_was_hidden(spec: &Spec, data: &Value) -> bool {
    spec.elements
        .get(&spec.root)
        .and_then(|el| el.visible.as_ref())
        .map(|vis| !vis.evaluate(data))
        .unwrap_or(false)
}

/// Flat pass over `spec.elements` collecting plugin type names. Replaces v1's
/// recursive `collect_plugin_types_node` per CONTEXT D-18.
pub(crate) fn collect_plugin_types(spec: &Spec) -> HashSet<String> {
    let mut types = HashSet::new();
    for el in spec.elements.values() {
        if !BUILTIN_TYPES.contains(&el.type_name.as_str()) {
            types.insert(el.type_name.clone());
        }
    }
    types
}

/// HTML-escapes interpolated identifiers in diagnostic comments and any prop
/// content per V5 (input validation) of the Phase 116 security domain.
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Emits `<link rel="stylesheet" href="..." [integrity] [crossorigin]>` per CSS
/// Asset. Ported from v1 render.rs lines 200–221. `Asset.crossorigin` is an
/// `Option<String>` (e.g., `Some("anonymous")`) — emitted verbatim when present.
pub(crate) fn render_css_tags(assets: &[Asset]) -> String {
    let mut out = String::new();
    for asset in assets {
        out.push_str("<link rel=\"stylesheet\" href=\"");
        out.push_str(&html_escape(&asset.url));
        out.push('"');
        if let Some(integrity) = &asset.integrity {
            out.push_str(" integrity=\"");
            out.push_str(&html_escape(integrity));
            out.push('"');
        }
        if let Some(co) = &asset.crossorigin {
            out.push_str(" crossorigin=\"");
            out.push_str(&html_escape(co));
            out.push('"');
        }
        out.push_str(">\n");
    }
    out
}

/// Emits `<script src="..." [integrity] [crossorigin]></script>` per JS Asset,
/// then `<script>{init}</script>` per init script. Ported from v1 render.rs
/// lines 222–249.
pub(crate) fn render_js_tags(assets: &[Asset], init_scripts: &[String]) -> String {
    let mut out = String::new();
    for asset in assets {
        out.push_str("<script src=\"");
        out.push_str(&html_escape(&asset.url));
        out.push('"');
        if let Some(integrity) = &asset.integrity {
            out.push_str(" integrity=\"");
            out.push_str(&html_escape(integrity));
            out.push('"');
        }
        if let Some(co) = &asset.crossorigin {
            out.push_str(" crossorigin=\"");
            out.push_str(&html_escape(co));
            out.push('"');
        }
        out.push_str("></script>\n");
    }
    for init in init_scripts {
        out.push_str("<script>");
        out.push_str(init);
        out.push_str("</script>\n");
    }
    out
}

#[cfg(test)]
mod tests {
    // Walker-level tests live in this module. Per-component HTML emission tests
    // live in atoms/containers/form/data submodules (Plans 03/04/05).
    use super::*;
    use crate::plugin::{register_plugin, Asset, JsonUiPlugin};
    use crate::spec::{Element, Spec};
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
    use serde_json::json;

    /// Construct an `Element` directly from its public fields. Used when tests
    /// need to bypass Phase 115's parse-time structural validator (e.g. for
    /// dangling-child or cycle scenarios).
    fn mk_element(type_name: &str) -> Element {
        Element {
            type_name: type_name.to_string(),
            props: Value::Null,
            children: Vec::new(),
            action: None,
            visible: None,
        }
    }

    /// Build a `Spec` whose elements map is overwritten post-build to bypass
    /// Phase 115's structural validator. This is ONLY for testing the walker's
    /// defense-in-depth guards on hand-mutated specs.
    fn build_spec_unchecked(root: &str, elements: Vec<(&str, Element)>) -> Spec {
        // Build a minimal valid spec through the normal builder so we get a
        // correctly-initialized Spec shell; then overwrite root + elements.
        let mut spec = Spec::builder()
            .element("__tmp__", Element::new("Text"))
            .build()
            .expect("builder accepts trivial well-formed spec");
        spec.root = root.to_string();
        spec.elements.clear();
        for (id, el) in elements {
            spec.elements.insert(id.to_string(), el);
        }
        spec
    }

    #[test]
    fn walker_unknown_type_emits_diagnostic() {
        let spec = build_spec_unchecked("root", vec![("root", mk_element("ImaginaryWidget"))]);
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(
            html.contains("<!-- ferro-json-ui: unknown component type 'ImaginaryWidget' -->"),
            "got: {html}"
        );
    }

    #[test]
    fn walker_missing_child_emits_diagnostic() {
        // The simplest way to force the missing-child diagnostic without needing
        // a real container renderer (still stubbed in Plan 02) is to point the
        // spec's root at an ID that isn't in the elements map.
        let mut spec = Spec::builder()
            .element("real", Element::new("Text"))
            .build()
            .expect("ok");
        spec.root = "ghost".to_string();
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(
            html.contains("<!-- ferro-json-ui: element references missing id 'ghost' -->"),
            "got: {html}"
        );
    }

    #[test]
    fn walker_root_hidden_emits_root_hidden_comment() {
        let mut spec = Spec::builder()
            .element("root", Element::new("Text"))
            .build()
            .expect("ok");
        let el = spec.elements.get_mut("root").unwrap();
        el.visible = Some(Visibility::Condition(VisibilityCondition {
            path: "/show".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!(true)),
        }));
        let html = render_spec_to_html(&spec, &json!({"show": false}));
        assert!(
            html.contains("<!-- ferro-json-ui: root hidden -->"),
            "got: {html}"
        );
    }

    #[test]
    fn walker_cycle_tripwire_fires_at_depth_4() {
        // Phase 115 would reject a self-cycle at parse time, so call the walker
        // directly with a depth exceeding MAX_NESTING_DEPTH + 1 to exercise the
        // defense-in-depth tripwire.
        let spec = build_spec_unchecked("A", vec![("A", mk_element("Text"))]);
        let html = render_element("A", &spec, &json!({}), MAX_NESTING_DEPTH + 2);
        assert!(html.contains("cycle guard tripped at depth"), "got: {html}");
    }

    #[test]
    fn walker_plugin_dispatch_invokes_with_plugin() {
        struct TestPlugin;
        impl JsonUiPlugin for TestPlugin {
            fn component_type(&self) -> &str {
                "FerroPhase116PluginDispatchTest"
            }
            fn props_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            fn render(&self, _props: &Value, _data: &Value) -> String {
                "<div data-test-plugin>X</div>".to_string()
            }
            fn css_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn js_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }
        register_plugin(TestPlugin);

        let spec = build_spec_unchecked(
            "root",
            vec![("root", mk_element("FerroPhase116PluginDispatchTest"))],
        );
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(
            html.contains("<div data-test-plugin>X</div>"),
            "got: {html}"
        );
    }

    #[test]
    fn walker_plugin_asset_collection_returns_plugin_types() {
        struct TestPluginB;
        impl JsonUiPlugin for TestPluginB {
            fn component_type(&self) -> &str {
                "FerroPhase116AssetCollectTestPlugin"
            }
            fn props_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            fn render(&self, _props: &Value, _data: &Value) -> String {
                String::new()
            }
            fn css_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn js_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }
        register_plugin(TestPluginB);

        let spec = build_spec_unchecked(
            "root",
            vec![
                ("root", mk_element("Text")),
                ("plug", mk_element("FerroPhase116AssetCollectTestPlugin")),
            ],
        );
        let types = collect_plugin_types(&spec);
        assert!(types.contains("FerroPhase116AssetCollectTestPlugin"));
        assert!(!types.contains("Text"));
    }

    #[test]
    fn walker_plugins_cannot_shadow_builtins() {
        // Register a plugin that claims the built-in type name "Card".
        // Per D-19 the dispatch match must still route to the built-in renderer
        // (which is a stub returning "" in Plan 02), not to the plugin.
        struct CardShadow;
        impl JsonUiPlugin for CardShadow {
            fn component_type(&self) -> &str {
                "Card"
            }
            fn props_schema(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            fn render(&self, _props: &Value, _data: &Value) -> String {
                "<div data-from-plugin>SHADOW</div>".to_string()
            }
            fn css_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn js_assets(&self) -> Vec<Asset> {
                Vec::new()
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }
        register_plugin(CardShadow);

        let spec = build_spec_unchecked("root", vec![("root", mk_element("Card"))]);
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(
            !html.contains("data-from-plugin"),
            "plugin must not shadow built-in Card; got: {html}"
        );
    }

    #[test]
    fn top_level_wrapper_present() {
        let spec = build_spec_unchecked("root", vec![("root", mk_element("Text"))]);
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(
            html.starts_with("<div class=\"flex flex-wrap gap-4"),
            "got: {html}"
        );
        assert!(html.ends_with("</div>"), "got: {html}");
    }

    #[test]
    fn html_escape_basic() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn builtin_types_count_matches_dispatch() {
        // Defense-in-depth check: BUILTIN_TYPES must be 39 entries.
        // The dispatch match in `render_element` has one arm per entry plus a
        // default arm. A compile-time mismatch would be caught by rustc; this
        // runtime check pins the invariant for future edits.
        assert_eq!(BUILTIN_TYPES.len(), 39);
    }
}
