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
    ButtonGroupProps, CardProps, CollapsibleProps, FormMaxWidth, FormSectionLayout,
    FormSectionProps, GapSize, GridProps, KanbanBoardProps, ModalProps, PageHeaderProps, TabsProps,
};
use crate::spec::{Element, Spec};

use super::{html_escape, render_element};

// ── Multi-slot containers ────────────────────────────────────────────────

/// Port of v1 `render_card` (render.rs L769-813). Body = `Element.children`
/// (D-05); footer = `CardProps.footer` IDs (D-06). Preserves v1's `max_width`
/// outer wrapper for Narrow/Wide variants.
pub(crate) fn render_card(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: CardProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Card props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Body: Element.children (D-05).
    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    // Footer: props.footer (D-06). Slot-borne IDs are NOT graph-validated at
    // parse time (D-07); missing IDs surface via render_element's D-10 comment.
    let footer: String = props
        .footer
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let mut html = String::from(
        "<div class=\"rounded-lg border border-border bg-card shadow-sm overflow-visible\"><div class=\"p-4\">",
    );
    html.push_str(&format!(
        "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.title)
    ));
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    // v1 gated the body wrapper on `!props.children.is_empty()`; v2 gates on
    // Element.children having at least one ID (child rendering may itself
    // emit "" when the atom is stubbed or an element is invisible, so we key
    // the wrapper off the slot list — not the rendered string).
    if !el.children.is_empty() {
        html.push_str(
            "<div class=\"mt-3 flex flex-wrap gap-3 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto overflow-visible\">",
        );
        html.push_str(&body);
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close inner p-4 region
    if !props.footer.is_empty() {
        html.push_str(
            "<div class=\"border-t border-border px-6 py-4 flex items-center justify-between gap-2\">",
        );
        html.push_str(&footer);
        html.push_str("</div>");
    }
    html.push_str("</div>"); // close outer card

    match props.max_width.as_ref().unwrap_or(&FormMaxWidth::Default) {
        FormMaxWidth::Default => {}
        FormMaxWidth::Narrow => {
            html = format!("<div class=\"max-w-2xl mx-auto\">{html}</div>");
        }
        FormMaxWidth::Wide => {
            html = format!("<div class=\"max-w-4xl mx-auto\">{html}</div>");
        }
    }

    html
}

/// Port of v1 `render_modal` (render.rs L815-863). Native `<dialog>` element;
/// body = `Element.children` (D-05); footer = `ModalProps.footer` IDs (D-06).
pub(crate) fn render_modal(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: ModalProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Modal props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();
    let footer: String = props
        .footer
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let trigger = props.trigger_label.as_deref().unwrap_or("Open");
    let mut html = String::new();
    // Trigger button (sibling of dialog, not inside it)
    html.push_str(&format!(
        "<button type=\"button\" class=\"inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium cursor-pointer\" data-modal-open=\"{}\">{}</button>",
        html_escape(&props.id),
        html_escape(trigger)
    ));
    // Native <dialog> element
    html.push_str(&format!(
        "<dialog id=\"{}\" aria-modal=\"true\" aria-labelledby=\"{}-title\" class=\"bg-card rounded-lg shadow-lg max-w-lg w-full mx-4 p-6 backdrop:bg-black/50\">",
        html_escape(&props.id),
        html_escape(&props.id)
    ));
    // Header row: title + close button
    html.push_str("<div class=\"flex items-center justify-between mb-4\">");
    html.push_str(&format!(
        "<h3 id=\"{}-title\" class=\"text-lg font-semibold leading-snug text-text\">{}</h3>",
        html_escape(&props.id),
        html_escape(&props.title)
    ));
    html.push_str(
        "<button type=\"button\" data-modal-close aria-label=\"Chiudi\" class=\"text-text-muted hover:text-text p-2 rounded transition-colors duration-150\">\u{00d7}</button>",
    );
    html.push_str("</div>");
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"text-sm text-text-muted mb-4\">{}</p>",
            html_escape(desc)
        ));
    }
    html.push_str(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
    );
    html.push_str(&body);
    html.push_str("</div>");
    if !props.footer.is_empty() {
        html.push_str("<div class=\"mt-6 flex items-center justify-end gap-2\">");
        html.push_str(&footer);
        html.push_str("</div>");
    }
    html.push_str("</dialog>");
    html
}

/// Port of v1 `render_tabs` (render.rs L865-959).
///
/// Two preserved non-obvious behaviors from v1 (CONTEXT "Non-obvious v1
/// behaviors to preserve"):
/// 1. **Single-tab auto-hide:** when `props.tabs.len() == 1` the tab bar is
///    elided entirely and the single panel renders directly.
/// 2. **Server-driven fallback:** when no tab in the spec carries children
///    (`has_any_content == false`) or a specific tab has empty children while
///    another has content, that tab's trigger renders as `<a href="?tab=X">`
///    instead of `<button data-tab="X">` — supporting full-page reloads for
///    SSR-only specs.
///
/// Per-tab children come from `Tab.children: Vec<String>` (D-06).
pub(crate) fn render_tabs(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: TabsProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Tabs props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Single-tab auto-hide (v1 L867-877).
    if props.tabs.len() == 1 {
        let tab = &props.tabs[0];
        let mut html = String::from(
            "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
        );
        for cid in &tab.children {
            html.push_str(&render_element(cid, spec, data, depth + 1));
        }
        html.push_str("</div>");
        return html;
    }

    let has_any_content = props.tabs.iter().any(|t| !t.children.is_empty());

    let mut html = String::from("<div data-tabs>");
    html.push_str("<div class=\"border-b border-border\">");
    html.push_str("<nav class=\"flex -mb-px space-x-4\" role=\"tablist\">");

    for tab in &props.tabs {
        let is_active = tab.value == props.default_tab;
        let border = if is_active {
            "border-primary"
        } else {
            "border-transparent"
        };
        let text = if is_active {
            "text-primary font-semibold"
        } else {
            "text-text-muted hover:text-text"
        };

        if has_any_content && (is_active || !tab.children.is_empty()) {
            // Client-side tab trigger
            html.push_str(&format!(
                "<button type=\"button\" role=\"tab\" id=\"tab-btn-{}\" aria-controls=\"tab-panel-{}\" data-tab=\"{}\" \
                 class=\"border-b-2 {} {} px-3 py-2 text-sm font-medium cursor-pointer transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
                 aria-selected=\"{}\">{}</button>",
                html_escape(&tab.value),
                html_escape(&tab.value),
                html_escape(&tab.value),
                border,
                text,
                is_active,
                html_escape(&tab.label),
            ));
        } else {
            // Server-driven tab: link with ?tab= query param
            html.push_str(&format!(
                "<a href=\"?tab={}\" role=\"tab\" id=\"tab-btn-{}\" aria-controls=\"tab-panel-{}\" \
                 class=\"border-b-2 {} {} px-3 py-2 text-sm font-medium transition-colors duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\" \
                 aria-selected=\"{}\">{}</a>",
                html_escape(&tab.value),
                html_escape(&tab.value),
                html_escape(&tab.value),
                border,
                text,
                is_active,
                html_escape(&tab.label),
            ));
        }
    }

    html.push_str("</nav></div>");

    // Render all tab panels — inactive panels are hidden via CSS.
    for tab in &props.tabs {
        if tab.children.is_empty() && tab.value != props.default_tab {
            continue;
        }
        let hidden = if tab.value != props.default_tab {
            " hidden"
        } else {
            ""
        };
        html.push_str(&format!(
            "<div role=\"tabpanel\" id=\"tab-panel-{}\" aria-labelledby=\"tab-btn-{}\" data-tab-panel=\"{}\" class=\"pt-4 flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto{}\">",
            html_escape(&tab.value),
            html_escape(&tab.value),
            html_escape(&tab.value),
            hidden,
        ));
        for cid in &tab.children {
            html.push_str(&render_element(cid, spec, data, depth + 1));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html
}

/// Port of v1 `render_kanban_board` (render.rs L499-587).
///
/// Responsive: horizontally-scrollable columns on desktop (`hidden md:block`),
/// tab-based column switching on mobile (`block md:hidden`). Per-column
/// children come from `KanbanColumnProps.children: Vec<String>` (D-06). Mobile
/// default column honors `props.mobile_default_column` when set, otherwise
/// falls back to the first column's id.
pub(crate) fn render_kanban_board(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: KanbanBoardProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode KanbanBoard props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    if props.columns.is_empty() {
        return String::new();
    }

    let default_id = props
        .mobile_default_column
        .as_deref()
        .unwrap_or_else(|| &props.columns[0].id);

    let mut html = String::new();

    // Desktop view: horizontal scrollable columns.
    html.push_str("<div class=\"hidden md:block overflow-x-auto\">");
    html.push_str("<div class=\"flex gap-4\" style=\"min-width: min-content;\">");

    for col in &props.columns {
        html.push_str(
            "<div class=\"min-w-[260px] flex-1 flex-shrink-0 rounded-lg border border-border bg-card/50 p-3\">",
        );
        html.push_str("<div class=\"flex items-center justify-between mb-3\">");
        html.push_str(&format!(
            "<h3 class=\"text-sm font-semibold text-text\">{}</h3>",
            html_escape(&col.title),
        ));
        let badge_class = if col.count > 0 {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold bg-primary text-primary-foreground"
        } else {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium text-text-muted bg-surface"
        };
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            badge_class, col.count,
        ));
        html.push_str("</div>");
        html.push_str("<div class=\"space-y-2\">");
        for cid in &col.children {
            html.push_str("<div data-kanban-card class=\"cursor-pointer\">");
            html.push_str(&render_element(cid, spec, data, depth + 1));
            html.push_str("</div>");
        }
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    // Mobile view: tab-based column switching.
    html.push_str("<div class=\"block md:hidden\" data-tabs>");
    html.push_str("<div class=\"flex border-b border-border mb-4\">");

    for col in &props.columns {
        let is_default = col.id == default_id;
        let (border, text) = if is_default {
            ("border-primary", "text-primary font-semibold")
        } else {
            ("border-transparent", "text-text-muted hover:text-text")
        };
        html.push_str(&format!(
            "<button type=\"button\" data-tab=\"{}\" class=\"flex-1 px-3 py-2 text-sm border-b-2 {} {}\" aria-selected=\"{}\">{} <span class=\"ml-1 text-xs text-text-muted\">({})</span></button>",
            html_escape(&col.id),
            border,
            text,
            is_default,
            html_escape(&col.title),
            col.count,
        ));
    }

    html.push_str("</div>");

    for col in &props.columns {
        let is_default = col.id == default_id;
        let hidden = if is_default { "" } else { " hidden" };
        html.push_str(&format!(
            "<div data-tab-panel=\"{}\" class=\"space-y-3{hidden}\">",
            html_escape(&col.id),
        ));
        for cid in &col.children {
            html.push_str("<div data-kanban-card class=\"cursor-pointer\">");
            html.push_str(&render_element(cid, spec, data, depth + 1));
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");

    html
}

/// Port of v1 `render_page_header` (render.rs L708-756). Title + breadcrumb
/// inline, with chevron separators. Actions are `PageHeaderProps.actions: Vec<String>`
/// IDs per D-06 — typically Button elements rendered to the right.
pub(crate) fn render_page_header(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: PageHeaderProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode PageHeader props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    // Actions: props.actions (D-06) — typically Button elements.
    let actions_html: String = props
        .actions
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let mut html =
        String::from("<div class=\"flex flex-wrap items-center justify-between gap-3 pb-4\">");

    // Title block — breadcrumb and title fused into one inline flow
    html.push_str("<div class=\"flex items-center gap-2 min-w-0\">");

    if !props.breadcrumb.is_empty() {
        for item in &props.breadcrumb {
            if let Some(ref url) = item.url {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"text-sm text-text-muted hover:text-text whitespace-nowrap\">{}</a>",
                    html_escape(url),
                    html_escape(&item.label)
                ));
            } else {
                html.push_str(&format!(
                    "<span class=\"text-sm text-text-muted whitespace-nowrap\">{}</span>",
                    html_escape(&item.label)
                ));
            }
            // Chevron separator between breadcrumb and title
            html.push_str(
                "<span aria-hidden=\"true\" class=\"text-text-muted flex-shrink-0\">\
                 <svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">\
                 <path fill-rule=\"evenodd\" d=\"M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z\" clip-rule=\"evenodd\"/>\
                 </svg></span>"
            );
        }
    }

    html.push_str(&format!(
        "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text truncate\">{}</h2>",
        html_escape(&props.title)
    ));
    html.push_str("</div>");

    // Actions (optional)
    if !props.actions.is_empty() {
        html.push_str("<div class=\"flex flex-wrap items-center gap-2\">");
        html.push_str(&actions_html);
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html
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

    // ── Multi-slot container tests (Task 2) ─────────────────────────────
    //
    // Atom renderers (Text, Button) are stubs in this worktree until Plan
    // 116-03 lands in the sibling worktree; tests therefore check the
    // container's wrapper markup, slot recursion sites, and diagnostic
    // behavior rather than atom child content.

    #[test]
    fn card_emits_wrapper_and_title_escaped() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card").prop("title", "<b>T</b>"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("rounded-lg border border-border bg-card"),
            "got: {html}"
        );
        assert!(
            html.contains("&lt;b&gt;T&lt;/b&gt;"),
            "title must be escaped; got: {html}"
        );
        assert!(
            !html.contains("<b>T</b>"),
            "raw HTML must not appear; got: {html}"
        );
    }

    #[test]
    fn card_renders_body_wrapper_when_children_present() {
        // With child IDs in Element.children, Card emits the body wrapper div
        // even though the atom stub returns "". Verifies Element.children is
        // the body slot per D-05.
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Card").prop("title", "Hi").child("body1"),
            ),
            ("body1", Element::new("Text").prop("content", "BODY")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<div class=\"mt-3 flex flex-wrap gap-3"),
            "body wrapper missing; got: {html}"
        );
    }

    #[test]
    fn card_renders_footer_wrapper_from_props() {
        // Footer slot lives in CardProps.footer per D-06. Footer wrapper is
        // emitted whenever props.footer is non-empty, regardless of whether
        // the referenced atoms resolve.
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Card")
                    .prop("title", "Hi")
                    .prop("footer", json!(["foot1"])),
            ),
            ("foot1", Element::new("Button").prop("label", "FOOT")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("border-t border-border px-6 py-4"),
            "footer wrapper missing; got: {html}"
        );
    }

    #[test]
    fn card_missing_footer_id_emits_diagnostic() {
        // Per CONTEXT D-07: Phase 115's parser validates Element.children IDs
        // but NOT slot-borne IDs (footer). The walker's D-10 missing-child
        // comment is the render-time safety net.
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "T")
                .prop("footer", json!(["ghost"])),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: element references missing id 'ghost' -->"),
            "got: {html}"
        );
    }

    #[test]
    fn card_max_width_narrow_wraps_in_mx_auto() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "T")
                .prop("max_width", "narrow"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 1);
        assert!(
            html.starts_with("<div class=\"max-w-2xl mx-auto\">"),
            "narrow wrapper missing; got: {html}"
        );
    }

    #[test]
    fn modal_emits_trigger_and_dialog() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Modal")
                .prop("id", "m1")
                .prop("title", "Confirm"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_modal(el, &spec, &json!({}), 1);
        assert!(html.contains("data-modal-open=\"m1\""), "got: {html}");
        assert!(html.contains("<dialog id=\"m1\""), "got: {html}");
        assert!(html.contains("aria-labelledby=\"m1-title\""), "got: {html}");
        assert!(html.contains("Confirm"), "title missing; got: {html}");
    }

    #[test]
    fn modal_renders_footer_wrapper_from_props() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Modal")
                    .prop("id", "m1")
                    .prop("title", "Confirm")
                    .prop("footer", json!(["f1"])),
            ),
            ("f1", Element::new("Button").prop("label", "Yes")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_modal(el, &spec, &json!({}), 1);
        assert!(
            html.contains("mt-6 flex items-center justify-end gap-2"),
            "footer wrapper missing; got: {html}"
        );
    }

    #[test]
    fn tabs_renders_per_tab_panels() {
        // Both tabs carry children → both tabpanels render. Uses Tab.children
        // per D-06. Atom stub returns "" but tabpanel wrappers still emit.
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Tabs").prop("default_tab", "a").prop(
                    "tabs",
                    json!([
                        {"value": "a", "label": "A", "children": ["t1"]},
                        {"value": "b", "label": "B", "children": ["t2"]},
                    ]),
                ),
            ),
            ("t1", Element::new("Text").prop("content", "PANEL_A")),
            ("t2", Element::new("Text").prop("content", "PANEL_B")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("data-tabs"),
            "tab container missing; got: {html}"
        );
        assert!(
            html.contains("data-tab-panel=\"a\""),
            "panel a missing; got: {html}"
        );
        assert!(
            html.contains("data-tab-panel=\"b\""),
            "panel b missing; got: {html}"
        );
        assert!(
            html.contains("data-tab=\"a\""),
            "client-side tab trigger a missing; got: {html}"
        );
        assert!(
            html.contains("data-tab=\"b\""),
            "client-side tab trigger b missing; got: {html}"
        );
    }

    #[test]
    fn tabs_single_tab_auto_hides_bar() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Tabs").prop("default_tab", "a").prop(
                    "tabs",
                    json!([{"value": "a", "label": "A", "children": ["t1"]}]),
                ),
            ),
            ("t1", Element::new("Text").prop("content", "ONLY")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_tabs(el, &spec, &json!({}), 1);
        // Single-tab auto-hide: no tab bar markup, no data-tab attribute.
        assert!(
            !html.contains("data-tab=\"a\""),
            "tab bar should be hidden for single-tab; got: {html}"
        );
        assert!(
            !html.contains("data-tabs"),
            "tab container wrapper should be skipped; got: {html}"
        );
        // Single-tab still emits the panel wrapper div.
        assert!(
            html.starts_with("<div class=\"flex flex-wrap gap-4"),
            "got: {html}"
        );
    }

    #[test]
    fn tabs_empty_children_uses_server_driven_link() {
        // All tabs have empty children → has_any_content is false → every tab
        // renders as <a href="?tab=X"> (server-driven fallback, CONTEXT
        // non-obvious behavior).
        let spec = build_spec(vec![(
            "root",
            Element::new("Tabs").prop("default_tab", "a").prop(
                "tabs",
                json!([
                    {"value": "a", "label": "A", "children": []},
                    {"value": "b", "label": "B", "children": []},
                ]),
            ),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_tabs(el, &spec, &json!({}), 1);
        assert!(
            html.contains("href=\"?tab=a\""),
            "expected server-driven link for tab a; got: {html}"
        );
        assert!(
            html.contains("href=\"?tab=b\""),
            "expected server-driven link for tab b; got: {html}"
        );
        // No client-side data-tab attributes on the triggers.
        assert!(
            !html.contains("data-tab=\"a\""),
            "server-driven fallback must not emit data-tab; got: {html}"
        );
    }

    #[test]
    fn kanban_renders_columns_desktop_and_mobile() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("KanbanBoard").prop(
                    "columns",
                    json!([
                        {"id": "todo", "title": "Todo", "count": 1, "children": ["c1"]},
                        {"id": "done", "title": "Done", "count": 0, "children": ["c2"]},
                    ]),
                ),
            ),
            ("c1", Element::new("Text").prop("content", "FIRST_CARD")),
            ("c2", Element::new("Text").prop("content", "SECOND_CARD")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_kanban_board(el, &spec, &json!({}), 1);
        // Desktop horizontal-scroll wrapper.
        assert!(
            html.contains("hidden md:block overflow-x-auto"),
            "desktop wrapper missing; got: {html}"
        );
        // Mobile tab-based wrapper.
        assert!(
            html.contains("block md:hidden"),
            "mobile wrapper missing; got: {html}"
        );
        // Column titles escaped and present.
        assert!(html.contains("Todo"), "got: {html}");
        assert!(html.contains("Done"), "got: {html}");
        // Per-column data-tab-panel markers.
        assert!(
            html.contains("data-tab-panel=\"todo\""),
            "todo panel missing; got: {html}"
        );
        assert!(
            html.contains("data-tab-panel=\"done\""),
            "done panel missing; got: {html}"
        );
        // Active count badge vs. zero count badge.
        assert!(
            html.contains("bg-primary text-primary-foreground"),
            "active count badge class missing; got: {html}"
        );
    }

    #[test]
    fn kanban_honors_mobile_default_column() {
        let spec = build_spec(vec![(
            "root",
            Element::new("KanbanBoard")
                .prop("mobile_default_column", "done")
                .prop(
                    "columns",
                    json!([
                        {"id": "todo", "title": "Todo", "count": 0, "children": []},
                        {"id": "done", "title": "Done", "count": 0, "children": []},
                    ]),
                ),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_kanban_board(el, &spec, &json!({}), 1);
        // The "done" mobile panel is the visible one (no " hidden" suffix);
        // "todo" is hidden on mobile.
        assert!(
            html.contains("data-tab-panel=\"done\" class=\"space-y-3\">"),
            "done should be the visible mobile panel; got: {html}"
        );
        assert!(
            html.contains("data-tab-panel=\"todo\" class=\"space-y-3 hidden\">"),
            "todo should be hidden on mobile; got: {html}"
        );
    }

    #[test]
    fn kanban_empty_columns_returns_empty() {
        let spec = build_spec(vec![(
            "root",
            Element::new("KanbanBoard").prop("columns", json!([])),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_kanban_board(el, &spec, &json!({}), 1);
        assert_eq!(html, "", "empty columns must render empty; got: {html}");
    }

    #[test]
    fn page_header_renders_title_and_breadcrumb() {
        let spec = build_spec(vec![(
            "root",
            Element::new("PageHeader").prop("title", "Dashboard").prop(
                "breadcrumb",
                json!([
                    {"label": "Home", "url": "/"},
                    {"label": "Reports"},
                ]),
            ),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_page_header(el, &spec, &json!({}), 1);
        assert!(html.contains("Dashboard"), "title missing; got: {html}");
        assert!(
            html.contains("<a href=\"/\""),
            "first breadcrumb should be an anchor; got: {html}"
        );
        assert!(html.contains("Home"), "got: {html}");
        assert!(html.contains("Reports"), "got: {html}");
        // Second breadcrumb has no URL → rendered as <span>.
        assert!(
            html.contains(
                "<span class=\"text-sm text-text-muted whitespace-nowrap\">Reports</span>"
            ),
            "urlless breadcrumb should be a span; got: {html}"
        );
    }

    #[test]
    fn page_header_renders_actions_wrapper_from_props() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("PageHeader")
                    .prop("title", "Dashboard")
                    .prop("actions", json!(["b1"])),
            ),
            ("b1", Element::new("Button").prop("label", "Create")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_page_header(el, &spec, &json!({}), 1);
        assert!(html.contains("Dashboard"), "got: {html}");
        assert!(
            html.contains("flex flex-wrap items-center gap-2"),
            "actions wrapper missing; got: {html}"
        );
    }

    #[test]
    fn page_header_missing_action_id_emits_diagnostic() {
        // Parallel to Card.footer: PageHeader.actions IDs are not
        // graph-validated at parse time (D-07). Walker's D-10 catches them.
        let spec = build_spec(vec![(
            "root",
            Element::new("PageHeader")
                .prop("title", "T")
                .prop("actions", json!(["ghost"])),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_page_header(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: element references missing id 'ghost' -->"),
            "got: {html}"
        );
    }
}
