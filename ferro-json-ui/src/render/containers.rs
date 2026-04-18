//! Phase 116 container renderers ported from v1 render.rs.
//!
//! Per CONTEXT D-21 v1 HTML emission is the canonical contract; this module
//! changes only the function signature (now `(el, spec, data, depth)`) and
//! routes child rendering through `super::render_element` for ID-keyed lookup.
//!
//! Per CONTEXT D-05 single-slot containers (Grid, Collapsible, FormSection,
//! ButtonGroup) read their children from `Element.children`. Multi-slot
//! containers (Card, Modal, Tabs, KanbanBoard, PageHeader) read slot IDs
//! from typed Props fields per D-06.

use serde_json::Value;

use crate::component::{
    ButtonGroupProps, CollapsibleProps, FormSectionLayout, FormSectionProps, GapSize, GridProps,
};
use crate::spec::{Element, Spec};

use super::{html_escape, render_element};

// ── Container component renderers ────────────────────────────────────────
//
// Multi-slot containers (stubs until Task 2 of Plan 116-04):

pub(crate) fn render_card(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    String::new()
}

pub(crate) fn render_modal(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    String::new()
}

pub(crate) fn render_tabs(_el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String {
    String::new()
}

pub(crate) fn render_kanban_board(
    _el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    String::new()
}

pub(crate) fn render_page_header(
    _el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    String::new()
}

// ── Single-slot containers ────────────────────────────────────────────────

/// Port of v1 `render_grid` (render.rs L2123-2155). Renders a CSS-grid wrapper;
/// children come from `Element.children` per D-05.
pub(crate) fn render_grid(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: GridProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Grid props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let gap = match props.gap {
        GapSize::None => "gap-0",
        GapSize::Sm => "gap-2",
        GapSize::Md => "gap-4",
        GapSize::Lg => "gap-6",
        GapSize::Xl => "gap-8",
    };

    // Children body via render_element (D-05).
    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    if props.scrollable == Some(true) {
        return format!(
            "<div class=\"overflow-x-auto\"><div class=\"grid grid-flow-col auto-cols-[minmax(280px,1fr)] {gap}\">{body}</div></div>"
        );
    }

    let cols = props.columns.clamp(1, 12);
    let mut col_classes = format!("grid-cols-{cols}");
    if let Some(md) = props.md_columns {
        col_classes.push_str(&format!(" md:grid-cols-{}", md.clamp(1, 12)));
    }
    if let Some(lg) = props.lg_columns {
        col_classes.push_str(&format!(" lg:grid-cols-{}", lg.clamp(1, 12)));
    }
    format!("<div class=\"grid w-full {col_classes} {gap}\">{body}</div>")
}

// ── Collapsible SVG chevron (v1 render.rs L2159-2163) ────────────────────
const CHEVRON_DOWN: &str = concat!(
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg>"
);

/// Port of v1 `render_collapsible` (render.rs L2165-2184). `<details>`/`<summary>`
/// pair with the body coming from `Element.children` per D-05.
pub(crate) fn render_collapsible(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: CollapsibleProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Collapsible props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let mut html = String::from("<details class=\"group\"");
    if props.expanded {
        html.push_str(" open");
    }
    html.push('>');
    let aria_expanded = if props.expanded { "true" } else { "false" };
    html.push_str(&format!(
        "<summary class=\"flex items-center justify-between cursor-pointer px-4 py-3 text-sm font-medium text-text bg-surface rounded-lg hover:bg-card\" aria-expanded=\"{}\">{}<span class=\"text-text-muted group-open:rotate-180 transition-transform\">{CHEVRON_DOWN}</span></summary>",
        aria_expanded,
        html_escape(&props.title)
    ));
    html.push_str("<div class=\"px-4 py-3 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">");
    html.push_str(&body);
    html.push_str("</div></details>");
    html
}

/// Port of v1 `render_form_section` (render.rs L2214-2259). Two layout variants
/// (stacked, two-column); body comes from `Element.children` per D-05.
pub(crate) fn render_form_section(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: FormSectionProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode FormSection props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let is_two_column = matches!(props.layout.as_ref(), Some(FormSectionLayout::TwoColumn));

    if is_two_column {
        let mut html = String::from("<fieldset class=\"md:grid md:grid-cols-5 md:gap-8\">");
        html.push_str(&format!(
            "<div class=\"md:col-span-2\"><legend class=\"text-base font-semibold text-text\">{}</legend>",
            html_escape(&props.title)
        ));
        if let Some(ref desc) = props.description {
            html.push_str(&format!(
                "<p class=\"text-sm text-text-muted mt-1\">{}</p>",
                html_escape(desc)
            ));
        }
        html.push_str("</div>");
        html.push_str("<div class=\"md:col-span-3 space-y-4 mt-4 md:mt-0\">");
        html.push_str(&body);
        html.push_str("</div></fieldset>");
        html
    } else {
        let mut html = String::from(
            "<fieldset class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
        );
        html.push_str(&format!(
            "<legend class=\"text-base font-semibold text-text\">{}</legend>",
            html_escape(&props.title)
        ));
        if let Some(ref desc) = props.description {
            html.push_str(&format!(
                "<p class=\"text-sm text-text-muted\">{}</p>",
                html_escape(desc)
            ));
        }
        html.push_str("<div class=\"space-y-4\">");
        html.push_str(&body);
        html.push_str("</div></fieldset>");
        html
    }
}

/// Port of v1 `render_button_group` (render.rs L758-765). Horizontal button row;
/// children come from `Element.children` per D-05.
///
/// Note: v1 iterated `props.buttons: Vec<ComponentNode>`; v2 takes children from
/// `Element.children` (generic `ButtonGroupProps` retains only the `gap` field).
pub(crate) fn render_button_group(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    // Decode-check for D-12 diagnostic discipline; `gap` value isn't consumed
    // in v1's emission (v1 hard-codes `gap-2`), but a malformed props payload
    // still surfaces via HTML comment per D-10.
    if !el.props.is_null() {
        if let Err(e) = serde_json::from_value::<ButtonGroupProps>(el.props.clone()) {
            return format!(
                "<!-- ferro-json-ui: failed to decode ButtonGroup props: {} -->",
                html_escape(&e.to_string())
            );
        }
    }

    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    format!("<div class=\"flex items-center gap-2 flex-wrap\">{body}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Element, ElementBuilder, Spec};
    use serde_json::json;

    fn build_spec(elements: Vec<(&str, ElementBuilder)>) -> Spec {
        let mut b = Spec::builder();
        for (id, el) in elements {
            b = b.element(id, el);
        }
        b.build().expect("ok")
    }

    #[test]
    fn grid_recurses_children() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Grid")
                    .prop("columns", 2)
                    .child("a")
                    .child("b"),
            ),
            ("a", Element::new("Text").prop("content", "AAA")),
            ("b", Element::new("Text").prop("content", "BBB")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_grid(el, &spec, &json!({}), 1);
        assert!(html.contains("grid-cols-2"), "got: {html}");
        assert!(html.starts_with("<div class=\"grid"), "got: {html}");
    }

    #[test]
    fn grid_scrollable_emits_flow_col() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Grid").prop("scrollable", true),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_grid(el, &spec, &json!({}), 1);
        assert!(html.contains("grid-flow-col"), "got: {html}");
        assert!(html.contains("overflow-x-auto"), "got: {html}");
    }

    #[test]
    fn collapsible_emits_details_summary() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Collapsible").prop("title", "More"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_collapsible(el, &spec, &json!({}), 1);
        assert!(html.starts_with("<details"), "got: {html}");
        assert!(html.contains("<summary"), "got: {html}");
        assert!(html.contains("More"), "title missing; got: {html}");
        assert!(html.contains("aria-expanded=\"false\""), "got: {html}");
    }

    #[test]
    fn collapsible_expanded_sets_open_attribute() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Collapsible")
                .prop("title", "Open me")
                .prop("expanded", true),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_collapsible(el, &spec, &json!({}), 1);
        assert!(
            html.starts_with("<details class=\"group\" open>"),
            "got: {html}"
        );
        assert!(html.contains("aria-expanded=\"true\""), "got: {html}");
    }

    #[test]
    fn form_section_emits_title_escaped() {
        let spec = build_spec(vec![(
            "root",
            Element::new("FormSection").prop("title", "<b>X</b>"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_form_section(el, &spec, &json!({}), 1);
        assert!(
            html.contains("&lt;b&gt;X&lt;/b&gt;"),
            "title must be escaped; got: {html}"
        );
        assert!(
            !html.contains("<b>X</b>"),
            "raw HTML must not appear; got: {html}"
        );
    }

    #[test]
    fn form_section_two_column_layout() {
        let spec = build_spec(vec![(
            "root",
            Element::new("FormSection")
                .prop("title", "Profile")
                .prop("description", "Update your info")
                .prop("layout", "two_column"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_form_section(el, &spec, &json!({}), 1);
        assert!(html.contains("md:grid-cols-5"), "got: {html}");
        assert!(html.contains("md:col-span-2"), "got: {html}");
        assert!(html.contains("md:col-span-3"), "got: {html}");
        assert!(html.contains("Update your info"), "got: {html}");
    }

    #[test]
    fn button_group_wraps_in_flex_row() {
        let spec = build_spec(vec![(
            "root",
            Element::new("ButtonGroup").prop("gap", "sm"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_button_group(el, &spec, &json!({}), 1);
        assert_eq!(
            html, "<div class=\"flex items-center gap-2 flex-wrap\"></div>",
            "got: {html}"
        );
    }
}
