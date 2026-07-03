//! Container renderers.
//!
//! Each function takes `(el, spec, data, depth)` and emits the container's
//! HTML wrapper plus the concatenated rendering of each child ID. Child
//! rendering is routed through [`super::render_element`] for ID-keyed lookup.
//!
//! Single-slot containers (`Grid`, `Collapsible`, `FormSection`,
//! `ButtonGroup`) read their children from `Element.children`. Multi-slot
//! containers (`Card`, `Modal`, `Tabs`, `KanbanBoard`, `PageHeader`) read
//! slot IDs from typed Props fields.

use serde_json::Value;

use crate::action::HttpMethod;
use crate::component::{
    ActionGroupProps, ActionItem, ButtonGroupProps, CardAppearance, CardProps, CollapsibleProps,
    DetailPageProps, DropdownMenuAction, FormMaxWidth, FormSectionLayout, FormSectionProps,
    GapSize, GridProps, KanbanBoardProps, ModalProps, PageHeaderProps, SegmentedControlProps,
    SegmentedItem, SidebarLayoutItem, SidebarLayoutProps, Size, TabsProps, Variant,
};
use crate::data::resolve_path;
use crate::spec::{Element, Spec};

use super::data::{render_inline_dropdown, resolve_row_key, template_actions};
use super::{html_escape, render_element};

// ── Multi-slot containers ────────────────────────────────────────────────

/// Renders a `Card`. Reads `CardProps` (title, description, footer,
/// max_width, variant) and wraps the rendering of each ID in `el.children`
/// in a bordered or elevated container per `variant`. Footer IDs come from
/// `CardProps.footer`. `Narrow` and `Wide` variants of `max_width` apply an
/// outer `mx-auto` width wrapper.
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

    // Body: rendered from `Element.children`.
    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    // Footer: rendered from `CardProps.footer`. Missing IDs surface via the
    // walker's missing-id diagnostic comment.
    let footer: String = props
        .footer
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    let (outer_class, inner_pad) = match props.appearance {
        CardAppearance::Bordered => (
            "rounded-lg border border-border bg-card shadow-sm overflow-visible",
            "p-4",
        ),
        CardAppearance::Elevated => ("rounded-lg bg-card shadow-md overflow-visible", "p-8"),
    };

    let mut html = format!("<div class=\"{outer_class}\"><div class=\"{inner_pad}\">");
    if let Some(ref badge) = props.badge {
        html.push_str("<div class=\"flex items-start justify-between gap-2\">");
        html.push_str(&format!(
            "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
            html_escape(&props.title)
        ));
        html.push_str(&format!(
            "<span class=\"inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-secondary/10 text-secondary-foreground shrink-0\">{}</span>",
            html_escape(badge)
        ));
        html.push_str("</div>");
    } else {
        html.push_str(&format!(
            "<h3 class=\"text-base font-semibold leading-snug text-text\">{}</h3>",
            html_escape(&props.title)
        ));
    }
    if let Some(ref subtitle) = props.subtitle {
        html.push_str(&format!(
            "<p class=\"mt-0.5 text-sm text-text-muted\">{}</p>",
            html_escape(subtitle)
        ));
    }
    if let Some(ref desc) = props.description {
        html.push_str(&format!(
            "<p class=\"mt-1 text-sm text-text-muted\">{}</p>",
            html_escape(desc)
        ));
    }
    // Body wrapper is omitted when there are no children to render. Keyed
    // on the slot list, not the rendered string — invisible children render
    // to "" but the wrapper is still emitted so the empty space is
    // structurally consistent.
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

/// Renders a `Modal` as a native `<dialog>` element with an external trigger
/// button. Body is rendered from `Element.children`; footer is rendered from
/// `ModalProps.footer`. The trigger uses `data-modal-open="{id}"`; the close
/// button uses `data-modal-close`.
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

/// Renders a `Tabs` container. Per-tab children come from
/// `Tab.children: Vec<String>`.
///
/// Two behaviors worth flagging for plugin authors:
/// 1. **Single-tab auto-hide:** when `props.tabs.len() == 1` the tab bar is
///    elided entirely and the single panel renders directly.
/// 2. **Server-driven fallback:** when no tab in the spec carries children
///    (`has_any_content == false`), or when a specific tab has empty children
///    while another has content, that tab's trigger renders as
///    `<a href="?tab=X">` instead of `<button data-tab="X">` — supporting
///    full-page reloads for SSR-only specs.
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

    // Single-tab specs render body-only (no tab strip).
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

/// Stringifies a JSON scalar for `group_by` matching and card field bindings.
/// Objects, arrays, and null yield `None`.
fn json_scalar_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Renders one data-bound card from a kanban item using the `card_*` / `row_*`
/// field bindings — the prescribed kanban card shape (title, optional
/// description, optional dropdown of `row_actions`). Mirrors `MediaCardGrid`.
fn render_kanban_card(item: &Value, props: &KanbanBoardProps, index: usize) -> String {
    let title = props
        .card_title_key
        .as_deref()
        .and_then(|k| item.get(k))
        .and_then(json_scalar_string)
        .unwrap_or_default();
    let description = props
        .card_description_key
        .as_deref()
        .and_then(|k| item.get(k))
        .and_then(json_scalar_string);

    let mut card = String::from(
        "<div data-kanban-card class=\"cursor-pointer rounded-lg border border-border bg-card p-3 flex flex-col gap-1\">",
    );
    card.push_str("<div class=\"flex items-start justify-between gap-2\">");
    card.push_str(&format!(
        "<div class=\"text-sm font-medium text-text\">{}</div>",
        html_escape(&title)
    ));
    if let Some(ref actions) = props.row_actions {
        let row_key_value = resolve_row_key(item, props.row_key.as_deref(), index);
        let templated = template_actions(actions, item, &row_key_value);
        if !templated.is_empty() {
            card.push_str(&render_inline_dropdown(
                &format!("kanban-card-{row_key_value}"),
                &templated,
            ));
        }
    }
    card.push_str("</div>");
    if let Some(desc) = description {
        card.push_str(&format!(
            "<div class=\"text-xs text-text-muted\">{}</div>",
            html_escape(&desc)
        ));
    }
    card.push_str("</div>");
    card
}

/// Pre-rendered content for a single kanban lane: header count plus the
/// concatenated card HTML, computed once and consumed by both the desktop and
/// mobile layouts.
struct LaneRender {
    id: String,
    title: String,
    count: u32,
    cards_html: String,
}

/// Renders a `KanbanBoard`. `columns` is lane structure (always rendered);
/// card content is data-bound from `items_path` + `group_by` — each item is
/// bucketed into the column whose `id` equals the item's `group_by` value and
/// rendered as a card via the `card_*` / `row_*` bindings. Static specs that
/// set neither `items_path` nor `group_by` fall back to rendering each
/// column's `children` element IDs.
///
/// Responsive: horizontally-scrollable columns on desktop
/// (`hidden md:block`), tab-based column switching on mobile
/// (`block md:hidden`). Mobile default column honors
/// `props.mobile_default_column` when set; otherwise falls back to the first
/// column's id.
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

    // Data-bound path: bucket a flat array of items into lanes by `group_by`.
    // Falls back to static `children` rendering when no `group_by` is set.
    let lanes: Vec<LaneRender> = match props.group_by.as_deref() {
        Some(group_by) => {
            let items: Vec<Value> = props
                .items_path
                .as_deref()
                .and_then(|p| resolve_path(data, p))
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            props
                .columns
                .iter()
                .map(|col| {
                    let cards: Vec<String> = items
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| {
                            item.get(group_by).and_then(json_scalar_string).as_deref()
                                == Some(col.id.as_str())
                        })
                        .map(|(i, item)| render_kanban_card(item, &props, i))
                        .collect();
                    LaneRender {
                        id: col.id.clone(),
                        title: col.title.clone(),
                        count: cards.len() as u32,
                        cards_html: cards.concat(),
                    }
                })
                .collect()
        }
        None => props
            .columns
            .iter()
            .map(|col| {
                let cards_html: String = col
                    .children
                    .iter()
                    .map(|cid| {
                        format!(
                            "<div data-kanban-card class=\"cursor-pointer\">{}</div>",
                            render_element(cid, spec, data, depth + 1)
                        )
                    })
                    .collect();
                LaneRender {
                    id: col.id.clone(),
                    title: col.title.clone(),
                    count: col.count,
                    cards_html,
                }
            })
            .collect(),
    };

    let default_id = props
        .mobile_default_column
        .as_deref()
        .unwrap_or_else(|| &lanes[0].id);

    let mut html = String::new();

    // Desktop view: horizontal scrollable columns. The negative
    // horizontal margin cancels the parent main's `md:p-6` padding so the
    // scroller spans full content width; the matching padding restores the
    // inset for the first/last column so cards don't touch the page edge.
    // Inline styles are used because consumer Tailwind builds typically
    // only generate the literal utilities ferro renders elsewhere (no `mx-*`
    // or `md:` variants for margin/padding).
    html.push_str(
        "<div class=\"hidden md:block overflow-x-auto\" \
         style=\"margin-left: -1.5rem; margin-right: -1.5rem; padding-left: 1.5rem; padding-right: 1.5rem;\">",
    );
    html.push_str("<div class=\"flex gap-4\" style=\"min-width: min-content;\">");

    for lane in &lanes {
        // Column is a flex-column capped at viewport height. Padding lives on
        // the inner sections (header + scroll viewport) instead of the outer
        // wrapper, so the scroll clip rectangle reaches the column's rounded
        // border — card hover shadows and focus rings extend cleanly into the
        // padding zone instead of being clipped at an inner box. Scrollbars
        // are hidden via `ferro-kanban-scroll` (CSS injected by the runtime)
        // plus inline `scrollbar-width: none` for Firefox.
        // The 12rem subtraction reserves room for dashboard header + page
        // header + main padding — adjust if chrome height changes.
        html.push_str(
            "<div class=\"min-w-[260px] flex-1 flex-shrink-0 rounded-lg border border-border bg-card/50 flex flex-col\" \
             style=\"max-height: calc(100vh - 12rem);\">",
        );
        html.push_str("<div class=\"flex items-center justify-between p-3 shrink-0\">");
        html.push_str(&format!(
            "<h3 class=\"text-sm font-semibold text-text\">{}</h3>",
            html_escape(&lane.title),
        ));
        let badge_class = if lane.count > 0 {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold bg-primary text-primary-foreground"
        } else {
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium text-text-muted bg-surface"
        };
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            badge_class, lane.count,
        ));
        html.push_str("</div>");
        html.push_str(
            "<div class=\"ferro-kanban-scroll space-y-2 flex-1 overflow-y-auto px-3 pb-3\" \
             style=\"scrollbar-width: none;\">",
        );
        if lane.cards_html.is_empty() {
            if let Some(ref label) = props.empty_label {
                html.push_str(&format!(
                    "<div class=\"flex items-center justify-center h-full min-h-40 text-sm text-text-muted text-center px-3\">{}</div>",
                    html_escape(label)
                ));
            }
        } else {
            html.push_str(&lane.cards_html);
        }
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    // Mobile view: tab-based column switching.
    html.push_str("<div class=\"block md:hidden\" data-tabs>");
    html.push_str("<div class=\"flex border-b border-border mb-4\">");

    for lane in &lanes {
        let is_default = lane.id == default_id;
        let (border, text) = if is_default {
            ("border-primary", "text-primary font-semibold")
        } else {
            ("border-transparent", "text-text-muted hover:text-text")
        };
        html.push_str(&format!(
            "<button type=\"button\" data-tab=\"{}\" class=\"flex-1 px-3 py-2 text-sm border-b-2 {} {}\" aria-selected=\"{}\">{} <span class=\"ml-1 text-xs text-text-muted\">({})</span></button>",
            html_escape(&lane.id),
            border,
            text,
            is_default,
            html_escape(&lane.title),
            lane.count,
        ));
    }

    html.push_str("</div>");

    for lane in &lanes {
        let is_default = lane.id == default_id;
        let hidden = if is_default { "" } else { " hidden" };
        // Same viewport cap as desktop columns — single column scrolls
        // vertically rather than growing the page. Scrollbars hidden via
        // `ferro-kanban-scroll` + inline scrollbar-width: none.
        html.push_str(&format!(
            "<div data-tab-panel=\"{}\" class=\"ferro-kanban-scroll space-y-3 overflow-y-auto{hidden}\" \
             style=\"max-height: calc(100vh - 14rem); scrollbar-width: none;\">",
            html_escape(&lane.id),
        ));
        if lane.cards_html.is_empty() {
            if let Some(ref label) = props.empty_label {
                html.push_str(&format!(
                    "<div class=\"flex items-center justify-center min-h-40 text-sm text-text-muted text-center px-3\">{}</div>",
                    html_escape(label)
                ));
            }
        } else {
            html.push_str(&lane.cards_html);
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");

    html
}

/// Renders a `PageHeader`. Emits the title, optional breadcrumb trail with
/// chevron separators, and right-aligned action buttons.
/// `PageHeaderProps.actions: Vec<String>` lists IDs of elements (typically
/// `Button` atoms) rendered into the actions slot.
///
/// Phase 165 F11: any `Element.children` are rendered as page body below
/// the header chrome. This closes the silent-drop pathology that bit
/// JsonUi::render_file consumers using `root: PageHeader` with children —
/// previously the chrome rendered fine but the body never appeared in DOM.
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

    // Actions: IDs from `props.actions` — typically Button elements.
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

    // F11: render Element.children as page body sections below the chrome.
    // The chrome's `pb-4` provides visual separation; children stack as
    // block-level siblings. Empty children list is a no-op (back-compat).
    if !el.children.is_empty() {
        let body_html: String = el
            .children
            .iter()
            .map(|cid| render_element(cid, spec, data, depth + 1))
            .collect();
        html.push_str("<div class=\"flex flex-col gap-4\">");
        html.push_str(&body_html);
        html.push_str("</div>");
    }

    html
}

/// Renders a `DetailPage`. Emits the canonical resource-detail layout:
/// PageHeader chrome (title + breadcrumb + actions), then an info Card
/// (rendered only when `info` is non-empty) wrapping the listed slot IDs,
/// then `Element.children` stacked below the card as page sections
/// (Tabs, related-resource Cards, etc.). The HTML matches what a
/// hand-composed `PageHeader + Card + Tabs` triple would emit, so the
/// component centralizes the contract without changing the visual output
/// downstream pages were already producing.
pub(crate) fn render_detail_page(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    let props: DetailPageProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode DetailPage props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let mut html = String::new();

    // Header chrome — mirrors render_page_header's structure verbatim so the
    // DetailPage skeleton stays visually identical to manually-composed pages.
    html.push_str("<div class=\"flex flex-wrap items-center justify-between gap-3 pb-4\">");
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

    if !props.actions.is_empty() {
        let actions_html: String = props
            .actions
            .iter()
            .map(|cid| render_element(cid, spec, data, depth + 1))
            .collect();
        html.push_str("<div class=\"flex flex-wrap items-center gap-2\">");
        html.push_str(&actions_html);
        html.push_str("</div>");
    }
    html.push_str("</div>");

    // Body: info Card + children sections, stacked vertically with gap-4.
    let has_body = !props.info.is_empty() || !el.children.is_empty();
    if has_body {
        html.push_str("<div class=\"flex flex-col gap-4\">");

        // Info Card — mirrors render_card's Bordered+Default output with an
        // empty title (suppressed via the same path as Card uses for empty
        // titles in practice). The wrapper is omitted entirely when `info`
        // is empty so pages without a primary info block get just the
        // header + children.
        if !props.info.is_empty() {
            let info_body: String = props
                .info
                .iter()
                .map(|cid| render_element(cid, spec, data, depth + 1))
                .collect();
            html.push_str(
                "<div class=\"rounded-lg border border-border bg-card shadow-sm overflow-visible\">\
                 <div class=\"p-4\">",
            );
            html.push_str(
                "<div class=\"flex flex-wrap gap-3 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto overflow-visible\">",
            );
            html.push_str(&info_body);
            html.push_str("</div></div></div>");
        }

        // Element.children render as below-the-fold sections (Tabs,
        // related-resource Cards, action panels).
        if !el.children.is_empty() {
            let body_html: String = el
                .children
                .iter()
                .map(|cid| render_element(cid, spec, data, depth + 1))
                .collect();
            html.push_str(&body_html);
        }

        html.push_str("</div>");
    }

    html
}

// ── Single-slot containers ────────────────────────────────────────────────

/// Renders a `Grid` container. Emits a CSS-grid `<div>` whose column count
/// comes from `GridProps.columns` (clamped to `1..=12`), with optional
/// `md_columns` / `lg_columns` breakpoint overrides. When `scrollable` is
/// set, emits a horizontally-scrollable `grid-flow-col` layout instead.
/// Children are rendered from `Element.children`.
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

    // Children body via `render_element`.
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

// SVG chevron used to indicate the collapsed/expanded state.
const CHEVRON_DOWN: &str = concat!(
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg>"
);

/// Renders a `Collapsible` section as a `<details>`/`<summary>` pair. The
/// title sits in `<summary>`; the body is rendered from `Element.children`.
/// `expanded = true` adds the HTML `open` attribute.
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

/// Renders a `FormSection` as a `<fieldset>` with an optional legend and
/// description. Two layout variants are supported: stacked (single column)
/// and `TwoColumn` (5-column grid with the title/description in the first 2
/// and the body in the remaining 3). The body is rendered from
/// `Element.children`.
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

/// Horizontal button row. Renders each child ID in `el.children` and wraps
/// the resulting HTML in a `<div class="flex items-center gap-2 flex-wrap">`
/// container. `ButtonGroupProps.gap` is decoded for prop-shape diagnostics
/// but does not influence the emitted CSS (the gap is fixed at `gap-2`).
pub(crate) fn render_button_group(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    // Decode-check: malformed props surface via HTML comment rather than crash.
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

/// Returns the Tailwind button classes for a given `Variant` at default size.
/// Matches the `render_button_inner` variant table in `atoms.rs`.
fn button_variant_classes(variant: &Variant) -> &'static str {
    match variant {
        Variant::Primary => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 bg-primary text-primary-foreground hover:bg-primary/90 \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
        Variant::Secondary => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 bg-secondary text-secondary-foreground hover:bg-secondary/90 \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
        Variant::Outline => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 border border-border bg-background text-text hover:bg-surface \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
        Variant::Ghost => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 text-text hover:bg-surface \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
        Variant::Destructive => {
            "inline-flex items-center justify-center rounded-md font-medium text-sm px-4 py-2 \
             transition-colors duration-150 bg-destructive text-primary-foreground hover:bg-destructive/90 \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
        }
    }
}

/// Fail-closed visibility gate for `ActionItem.visible_if`.
///
/// Mirrors the semantics of `action_visible_for_row` in `render/data.rs`:
/// - `visible_if` absent → always visible.
/// - `visible_if` set and `data[field]` is missing, null, false, `0`, or
///   empty string/array/object → hidden.
fn action_item_visible(item: &ActionItem, data: &serde_json::Value) -> bool {
    let Some(field) = item.visible_if.as_deref() else {
        return true;
    };
    match data.get(field) {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(f) = n.as_f64() {
                f != 0.0
            } else {
                false
            }
        }
        Some(serde_json::Value::String(s)) => !s.is_empty(),
        Some(serde_json::Value::Array(a)) => !a.is_empty(),
        Some(serde_json::Value::Object(o)) => !o.is_empty(),
    }
}

/// Renders an `ActionGroup` — an ordered action list that emits inline buttons
/// up to `max_inline` and a trailing overflow kebab for the rest. Destructive
/// items are always in the kebab, rendered last.
///
/// ## Partition rules (D-01..D-04)
/// - Non-destructive items up to `max_inline` (default 2) → inline.
/// - Non-destructive items beyond the cap → overflow kebab in input order.
/// - Destructive items → overflow kebab appended **last**, regardless of their
///   position in `items`. They never count toward `max_inline`.
/// - The overflow kebab is hidden when nothing overflows (D-03).
///
/// ## Form-wrapping (D-15)
/// Non-GET inline actions wrap in `<form method="post">`. GET actions render as
/// `<a href>` links. Matches `render_menu_item`'s non-GET branch in `atoms.rs`.
///
/// ## `visible_if` gate
/// Fail-closed: items whose `visible_if` field is absent or falsy in `data` are
/// removed before partitioning.
// Dead-code suppressed until plan 02 wires this into the dispatch table.
#[allow(dead_code)]
pub(crate) fn render_action_group(
    el: &Element,
    _spec: &Spec,
    data: &serde_json::Value,
    _depth: usize,
) -> String {
    let props: ActionGroupProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode ActionGroup props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let overflow_label = props.overflow_label.as_deref().unwrap_or("Azioni");
    let max_inline = props.max_inline.unwrap_or(2) as usize;

    // Apply visible_if fail-closed gate.
    let visible_items: Vec<&ActionItem> = props
        .items
        .iter()
        .filter(|item| action_item_visible(item, data))
        .collect();

    // Partition into normal (non-destructive) and destructive.
    let (normal, destructive): (Vec<&ActionItem>, Vec<&ActionItem>) =
        visible_items.iter().partition(|i| !i.destructive);

    let inline_items: Vec<&&ActionItem> = normal.iter().take(max_inline).collect();
    let mut overflow: Vec<&&ActionItem> = normal.iter().skip(max_inline).collect();
    // Destructive items go last in the overflow.
    overflow.extend(destructive.iter());

    let mut html = String::new();

    // Render inline items.
    for item in &inline_items {
        let url = match item.action.url.as_deref().filter(|s| !s.is_empty()) {
            Some(u) => u.to_string(),
            None => {
                let h = item.action.handler.as_str();
                if h.is_empty() {
                    "#".to_string()
                } else {
                    h.to_string()
                }
            }
        };

        let btn_classes =
            button_variant_classes(item.variant.as_ref().unwrap_or(&Variant::Primary));
        let label = html_escape(&item.label);

        match item.action.method {
            HttpMethod::Get => {
                html.push_str(&format!(
                    "<a href=\"{}\" class=\"{btn_classes}\">{label}</a>",
                    html_escape(&url),
                ));
            }
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => {
                let method_spoof = match item.action.method {
                    HttpMethod::Put => Some("PUT"),
                    HttpMethod::Patch => Some("PATCH"),
                    HttpMethod::Delete => Some("DELETE"),
                    _ => None,
                };
                html.push_str(&format!(
                    "<form action=\"{}\" method=\"post\">",
                    html_escape(&url),
                ));
                if let Some(m) = method_spoof {
                    html.push_str(&format!(
                        "<input type=\"hidden\" name=\"_method\" value=\"{m}\">"
                    ));
                }
                html.push_str(&format!(
                    "<button type=\"submit\" class=\"{btn_classes}\">{label}</button>"
                ));
                html.push_str("</form>");
            }
        }
    }

    // Render overflow kebab only when there is overflow.
    if !overflow.is_empty() {
        let trigger_icon = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" \
             viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" \
             stroke-linecap=\"round\" stroke-linejoin=\"round\">\
             <circle cx=\"12\" cy=\"5\" r=\"1\"/>\
             <circle cx=\"12\" cy=\"12\" r=\"1\"/>\
             <circle cx=\"12\" cy=\"19\" r=\"1\"/></svg>";

        html.push_str(&format!(
            "<button type=\"button\" popovertarget=\"{}\" aria-label=\"{}\" \
             class=\"inline-flex items-center justify-center rounded-md p-1.5 \
             text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 \
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
             focus-visible:ring-offset-2\">{trigger_icon}</button>",
            html_escape(&props.menu_id),
            html_escape(overflow_label),
        ));

        html.push_str(&format!(
            "<div popover id=\"{}\" data-popover-menu \
             class=\"w-48 rounded-md border border-border bg-card shadow-md text-left p-0\">",
            html_escape(&props.menu_id),
        ));

        for item in &overflow {
            // Convert ActionItem to DropdownMenuAction at the call boundary
            // so we can reuse render_menu_item without duplicating its logic.
            let dma = DropdownMenuAction {
                label: item.label.clone(),
                action: item.action.clone(),
                destructive: item.destructive,
                visible_if: item.visible_if.clone(),
            };
            html.push_str(&super::atoms::render_menu_item(
                &dma,
                "block px-4 py-2 text-sm text-text hover:bg-surface transition-colors duration-150",
                "block px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors duration-150",
                "",
            ));
        }

        html.push_str("</div>"); // close popover panel
    }

    format!("<div class=\"flex items-center gap-2 flex-wrap\">{html}</div>")
}

/// Renders a `SegmentedControl` — a connected button cluster used for date
/// scrollers, view toggles, and other "pick one of N" navigation primitives.
///
/// Items come from `props.items` (literal) or from `props.data_path` resolved
/// against `data`. Literal wins when both are present. Each item becomes an
/// `<a href>` so the control works without JS; the active item carries
/// `aria-selected="true"` and is styled distinctly. The outer container
/// emits `role="tablist"` when an `aria_label` is supplied.
pub(crate) fn render_segmented_control(
    el: &Element,
    _spec: &Spec,
    data: &Value,
    _depth: usize,
) -> String {
    let props: SegmentedControlProps = if el.props.is_null() {
        SegmentedControlProps::default()
    } else {
        match serde_json::from_value(el.props.clone()) {
            Ok(p) => p,
            Err(e) => {
                return format!(
                    "<!-- ferro-json-ui: failed to decode SegmentedControl props: {} -->",
                    html_escape(&e.to_string())
                );
            }
        }
    };

    let items: Vec<SegmentedItem> = if !props.items.is_empty() {
        props.items
    } else if let Some(path) = props.data_path.as_deref() {
        match resolve_path(data, path) {
            Some(v) => match serde_json::from_value::<Vec<SegmentedItem>>(v.clone()) {
                Ok(list) => list,
                Err(e) => {
                    return format!(
                        "<!-- ferro-json-ui: SegmentedControl data_path '{}' did not resolve to Vec<SegmentedItem>: {} -->",
                        html_escape(path),
                        html_escape(&e.to_string())
                    );
                }
            },
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };

    if items.is_empty() {
        return String::new();
    }

    // Size-driven padding/text classes. Keep typography compact across sizes —
    // segmented controls are nav chrome, not primary CTAs.
    let segment_pad = match props.size {
        Size::Sm => "px-2.5 py-1 text-xs",
        Size::Md => "px-3 py-1.5 text-sm",
        Size::Lg => "px-4 py-2 text-base",
    };

    let mut group = String::from(
        "<div class=\"inline-flex rounded-md border border-input bg-background overflow-hidden\"",
    );
    if let Some(label) = props.aria_label.as_deref() {
        group.push_str(" role=\"tablist\" aria-label=\"");
        group.push_str(&html_escape(label));
        group.push('"');
    }
    group.push('>');

    let last_idx = items.len().saturating_sub(1);
    for (idx, item) in items.iter().enumerate() {
        let border_cls = if idx < last_idx {
            "border-r border-input"
        } else {
            ""
        };
        let state_cls = if item.active {
            "bg-surface text-text font-semibold"
        } else {
            "text-text-muted hover:bg-surface hover:text-text font-medium"
        };
        let aria_label_attr = item
            .aria_label
            .as_deref()
            .map(|l| format!(" aria-label=\"{}\"", html_escape(l)))
            .unwrap_or_default();
        let role_attr = if props.aria_label.is_some() {
            format!(
                " role=\"tab\" aria-selected=\"{}\"",
                if item.active { "true" } else { "false" }
            )
        } else {
            String::new()
        };
        group.push_str(&format!(
            "<a href=\"{href}\"{aria_label_attr}{role_attr} class=\"{pad} {state} {border} transition-colors\">{label}</a>",
            href = html_escape(&item.href),
            aria_label_attr = aria_label_attr,
            role_attr = role_attr,
            pad = segment_pad,
            state = state_cls,
            border = border_cls,
            label = html_escape(&item.label),
        ));
    }
    group.push_str("</div>");
    group
}

/// Renders a `SidebarLayout` — a two-column page layout with a sticky vertical
/// nav (left) and a main content slot (right). Children render inside the
/// main slot; each child is expected to manage its own `visible` rule against
/// `props.active` so only the matching section is in the DOM.
///
/// On mobile the layout collapses to a single column: the sidebar becomes a
/// horizontally scrollable strip above the main content.
///
/// Replaces the opener/closer `RawHtml` pattern previously used by consumers
/// to fake an asymmetric grid.
pub(crate) fn render_sidebar_layout(
    el: &Element,
    spec: &Spec,
    data: &Value,
    depth: usize,
) -> String {
    let props: SidebarLayoutProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode SidebarLayout props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let items: Vec<SidebarLayoutItem> = if !props.items.is_empty() {
        props.items
    } else if let Some(path) = props.data_path.as_deref() {
        match resolve_path(data, path) {
            Some(v) => match serde_json::from_value::<Vec<SidebarLayoutItem>>(v.clone()) {
                Ok(list) => list,
                Err(e) => {
                    return format!(
                        "<!-- ferro-json-ui: SidebarLayout data_path '{}' did not resolve to Vec<SidebarLayoutItem>: {} -->",
                        html_escape(path),
                        html_escape(&e.to_string())
                    );
                }
            },
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // Build the sidebar nav HTML.
    let mut nav = String::from(
        "<nav class=\"flex md:flex-col gap-1 overflow-x-auto md:overflow-x-visible whitespace-nowrap md:whitespace-normal\"",
    );
    if let Some(label) = props.aria_label.as_deref() {
        nav.push_str(" aria-label=\"");
        nav.push_str(&html_escape(label));
        nav.push('"');
    }
    nav.push('>');
    for item in &items {
        let is_active = item.slug == props.active;
        let cls = if is_active {
            "block px-3 py-2 text-sm font-semibold rounded-md bg-surface text-text"
        } else {
            "block px-3 py-2 text-sm font-medium rounded-md text-text-muted hover:bg-surface hover:text-text transition-colors"
        };
        let aria = if is_active {
            " aria-current=\"page\""
        } else {
            ""
        };
        nav.push_str(&format!(
            "<a href=\"{}\"{} class=\"{}\">{}</a>",
            html_escape(&item.url),
            aria,
            cls,
            html_escape(&item.label),
        ));
    }
    nav.push_str("</nav>");

    // Main content — children render in DOM order; each child controls its own
    // visibility via `visible` so only the active section materialises.
    let main_content: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    format!(
        "<div class=\"flex flex-col gap-6 md:grid md:grid-cols-[220px_minmax(0,1fr)] md:gap-6\">\
          <aside class=\"md:sticky md:top-4 md:self-start min-w-0\">{nav}</aside>\
          <main class=\"min-w-0 flex flex-col gap-4\">{main_content}</main>\
        </div>",
    )
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
    fn grid_renders_when_visible_true() {
        use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
        let spec = build_spec(vec![(
            "root",
            Element::new("Grid")
                .prop("columns", 1)
                .visible(Visibility::Condition(VisibilityCondition {
                    path: "/flag".into(),
                    operator: VisibilityOperator::Eq,
                    value: Some(json!(true)),
                })),
        )]);
        let html = crate::render::render_spec_to_html(&spec, &json!({"flag": true}));
        assert!(
            html.contains("<div class=\"grid"),
            "Grid must render when visible-true; got: {html}"
        );
    }

    #[test]
    fn grid_hidden_when_visible_false() {
        use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
        let spec = build_spec(vec![(
            "root",
            Element::new("Grid")
                .prop("columns", 1)
                .visible(Visibility::Condition(VisibilityCondition {
                    path: "/flag".into(),
                    operator: VisibilityOperator::Eq,
                    value: Some(json!(true)),
                })),
        )]);
        let html = crate::render::render_spec_to_html(&spec, &json!({"flag": false}));
        assert!(
            !html.contains("<div class=\"grid"),
            "Grid must be absent when visible-false; got: {html}"
        );
    }

    #[test]
    fn grid_visible_consumer_reproduction() {
        // Mirrors the consumer's chip-strip spec from
        // gestiscilo-it/app/src/views/calendario/calendar_day.json:73-85.
        // The chip-strip Grid is nested inside a root Grid, with visibility gated
        // on `/has_staff`. The consumer reports the inner Grid does not render
        // when has_staff=true; RESEARCH predicts this fails to reproduce against
        // current ferro master (the visibility evaluator architecture is correct).
        //
        // Outcome A (predicted): both branches pass — F9 closes as no-repro.
        // Outcome B (alternate): one or both branches fail — Task 3 investigates.
        use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
        let chip_strip_visible = Visibility::Condition(VisibilityCondition {
            path: "/has_staff".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!(true)),
        });
        let spec = build_spec(vec![
            (
                "root",
                Element::new("Grid")
                    .prop("columns", 1)
                    .child("staff_chips_row"),
            ),
            (
                "staff_chips_row",
                Element::new("Grid")
                    .prop("columns", 1)
                    .prop("gap", "sm")
                    .visible(chip_strip_visible.clone()),
            ),
        ]);

        let html_visible = crate::render::render_spec_to_html(&spec, &json!({"has_staff": true}));
        assert!(
            html_visible.matches("<div class=\"grid").count() >= 2,
            "inner Grid must render when has_staff=true (outer + inner = at least 2 grid divs); got: {html_visible}"
        );

        let html_hidden = crate::render::render_spec_to_html(&spec, &json!({"has_staff": false}));
        assert_eq!(
            html_hidden.matches("<div class=\"grid").count(),
            1,
            "only outer Grid renders when has_staff=false (inner Grid hidden, leaving exactly 1 grid div); got: {html_hidden}"
        );
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

    // ── Multi-slot container tests ──────────────────────────────────────
    //
    // These tests check the container's wrapper markup, slot recursion
    // sites, and diagnostic behavior rather than the atom child content.

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
        // With child IDs in Element.children, Card emits the body wrapper div.
        // Verifies Element.children is the body slot.
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
        // Footer slot lives in CardProps.footer. Footer wrapper is emitted
        // whenever props.footer is non-empty, regardless of whether the
        // referenced atoms resolve.
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
    fn card_missing_footer_id_rejected_at_parse_time() {
        // Dangling footer IDs are caught at spec build/parse time rather
        // than at render time; `SpecError::FooterMissing` is the parse-time
        // rejection variant.
        let err = Spec::builder()
            .element(
                "root",
                Element::new("Card")
                    .prop("title", "T")
                    .prop("footer", json!(["ghost"])),
            )
            .build()
            .unwrap_err();
        match err {
            crate::spec::SpecError::FooterMissing {
                element_id,
                footer_id,
            } => {
                assert_eq!(element_id, "root");
                assert_eq!(footer_id, "ghost");
            }
            other => panic!("expected FooterMissing, got {other:?}"),
        }
    }

    #[test]
    fn render_card_bordered_default() {
        let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            html.contains("border border-border"),
            "expected Bordered class, got: {html}"
        );
        assert!(
            html.contains("shadow-sm"),
            "expected shadow-sm, got: {html}"
        );
        assert!(html.contains("p-4"), "expected p-4, got: {html}");
    }

    #[test]
    fn render_card_elevated_no_border() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "X")
                .prop("appearance", "elevated"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            html.contains("shadow-md"),
            "expected shadow-md, got: {html}"
        );
        assert!(html.contains("p-8"), "expected p-8, got: {html}");
        assert!(
            !html.contains("border-border"),
            "Elevated must NOT contain border-border, got: {html}"
        );
    }

    #[test]
    fn render_card_omitted_variant_is_bordered() {
        let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            html.contains("border border-border"),
            "missing variant defaults to Bordered, got: {html}"
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
    fn render_card_emits_badge_when_present() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "Booking")
                .prop("badge", "Scade tra 9m"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            html.contains("Scade tra 9m"),
            "badge label must appear in DOM; got: {html}"
        );
        assert!(
            html.contains("bg-secondary/10"),
            "badge must use Secondary chrome; got: {html}"
        );
        assert!(
            html.contains("flex items-start justify-between"),
            "title-row wrapper must be emitted when badge present; got: {html}"
        );
    }

    #[test]
    fn render_card_omits_badge_when_absent() {
        let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            !html.contains("bg-secondary/10"),
            "no badge chrome when badge absent; got: {html}"
        );
        assert!(
            !html.contains("flex items-start justify-between"),
            "no title-row wrapper when badge absent; got: {html}"
        );
    }

    #[test]
    fn render_card_emits_subtitle_when_present() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "Booking")
                .prop("subtitle", "Marco Rossi"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(
            html.contains("Marco Rossi"),
            "subtitle text must appear in DOM; got: {html}"
        );
        assert!(
            html.contains("mt-0.5 text-sm text-text-muted"),
            "subtitle must use muted styling with mt-0.5 spacing; got: {html}"
        );
    }

    #[test]
    fn render_card_omits_subtitle_when_absent() {
        let spec = build_spec(vec![("root", Element::new("Card").prop("title", "X"))]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        // The description block uses `mt-1 text-sm text-text-muted`; the subtitle
        // would use `mt-0.5 text-sm text-text-muted`. With neither subtitle nor
        // description, no muted-text paragraph should emit at all.
        assert!(
            !html.contains("text-sm text-text-muted"),
            "no muted text when both subtitle and description absent; got: {html}"
        );
    }

    #[test]
    fn render_card_emits_title_subtitle_description_badge_together() {
        let spec = build_spec(vec![(
            "root",
            Element::new("Card")
                .prop("title", "Booking #1")
                .prop("subtitle", "Marco Rossi")
                .prop("description", "Customer detail")
                .prop("badge", "Scade tra 9m"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_card(el, &spec, &json!({}), 0);
        assert!(html.contains("Booking #1"), "title; got: {html}");
        assert!(html.contains("Marco Rossi"), "subtitle; got: {html}");
        assert!(html.contains("Customer detail"), "description; got: {html}");
        assert!(html.contains("Scade tra 9m"), "badge; got: {html}");
        // Slot order: title-row (with badge) → subtitle → description.
        let title_pos = html.find("Booking #1").expect("title present");
        let subtitle_pos = html.find("Marco Rossi").expect("subtitle present");
        let desc_pos = html.find("Customer detail").expect("description present");
        assert!(
            title_pos < subtitle_pos,
            "title must precede subtitle; got: {html}"
        );
        assert!(
            subtitle_pos < desc_pos,
            "subtitle must precede description; got: {html}"
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
        // Both tabs carry children → both tabpanels render. Children come
        // from Tab.children.
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
        // "todo" is hidden on mobile. Panels are scrollable so the kanban
        // never grows the page past the viewport.
        assert!(
            html.contains(
                "data-tab-panel=\"done\" class=\"ferro-kanban-scroll space-y-3 overflow-y-auto\""
            ),
            "done should be the visible mobile panel; got: {html}"
        );
        assert!(
            html.contains(
                "data-tab-panel=\"todo\" class=\"ferro-kanban-scroll space-y-3 overflow-y-auto hidden\""
            ),
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
    fn page_header_renders_children_as_body_below_chrome() {
        // Phase 165 F11: previously PageHeader silently dropped
        // Element.children. The body block now renders below the chrome.
        let spec = build_spec(vec![
            (
                "root",
                Element::new("PageHeader")
                    .prop("title", "Customers")
                    .child("body_card"),
            ),
            (
                "body_card",
                Element::new("Card").prop("title", "Customer list"),
            ),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_page_header(el, &spec, &json!({}), 1);
        assert!(
            html.contains("Customers"),
            "header title missing; got: {html}"
        );
        assert!(
            html.contains("Customer list"),
            "child Card body missing — F11 regression; got: {html}"
        );
        // Chrome closes before body container opens.
        let chrome_close = html
            .find("</div><div class=\"flex flex-col gap-4\">")
            .or_else(|| html.find("</div>\n<div class=\"flex flex-col gap-4\">"));
        assert!(
            chrome_close.is_some(),
            "body wrapper must follow the chrome div; got: {html}"
        );
    }

    #[test]
    fn page_header_with_empty_children_is_unchanged() {
        // Back-compat: no children → no body wrapper, same HTML shape as before.
        let spec = build_spec(vec![(
            "root",
            Element::new("PageHeader").prop("title", "Static"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_page_header(el, &spec, &json!({}), 1);
        assert!(html.contains("Static"));
        assert!(
            !html.contains("flex flex-col gap-4"),
            "body wrapper must not appear when children list is empty; got: {html}"
        );
    }

    #[test]
    fn page_header_missing_action_id_emits_diagnostic() {
        // Parallel to Card.footer: PageHeader.actions IDs are not
        // graph-validated at parse time. The walker's missing-id diagnostic
        // catches them at render time.
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

    // ── KanbanBoard structure/content tests ─────────────────────────────

    fn kanban_data_bound(extra: Vec<(&str, Value)>) -> ElementBuilder {
        let mut el = Element::new("KanbanBoard")
            .prop(
                "columns",
                json!([
                    {"title": "Open", "id": "open"},
                    {"title": "Done", "id": "done"}
                ]),
            )
            .prop("items_path", "/items")
            .prop("group_by", "status")
            .prop("card_title_key", "name");
        for (k, v) in extra {
            el = el.prop(k, v);
        }
        el
    }

    #[test]
    fn render_kanban_board_buckets_items_into_columns() {
        let spec = build_spec(vec![("root", kanban_data_bound(vec![]))]);
        let el = spec.elements.get("root").unwrap();
        let data = json!({"items": [
            {"id": 1, "name": "Alpha", "status": "open"},
            {"id": 2, "name": "Beta", "status": "done"},
            {"id": 3, "name": "Gamma", "status": "open"}
        ]});
        let html = render_kanban_board(el, &spec, &data, 0);
        // Both lanes render (structure always present).
        assert!(html.contains(">Open<"), "lane Open missing: {html}");
        assert!(html.contains(">Done<"), "lane Done missing: {html}");
        // Cards land in their lanes.
        assert!(html.contains("Alpha"), "Alpha card missing: {html}");
        assert!(html.contains("Beta"), "Beta card missing: {html}");
        assert!(html.contains("Gamma"), "Gamma card missing: {html}");
        // Per-lane count badge: Open has 2 (desktop + mobile = 2 occurrences each).
        assert_eq!(
            html.matches(">2<").count(),
            1,
            "Open lane desktop count badge should read 2: {html}"
        );
    }

    #[test]
    fn render_kanban_board_static_columns_always_render() {
        // Columns present but no matching items → lanes still render (not blank).
        let spec = build_spec(vec![("root", kanban_data_bound(vec![]))]);
        let el = spec.elements.get("root").unwrap();
        let data = json!({"items": []});
        let html = render_kanban_board(el, &spec, &data, 0);
        assert!(
            html.contains(">Open<"),
            "empty lane Open must render: {html}"
        );
        assert!(
            html.contains(">Done<"),
            "empty lane Done must render: {html}"
        );
    }

    #[test]
    fn render_kanban_board_empty_columns_renders_empty() {
        let spec = build_spec(vec![(
            "root",
            Element::new("KanbanBoard").prop("items_path", "/items"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_kanban_board(el, &spec, &json!({"items": []}), 0);
        assert_eq!(html, "", "no columns → empty board; got: {html}");
    }

    #[test]
    fn render_kanban_board_card_description_and_actions() {
        let spec = build_spec(vec![(
            "root",
            kanban_data_bound(vec![
                ("card_description_key", json!("customer")),
                (
                    "row_actions",
                    json!([{"label": "View", "action": {"handler": "/orders/{row_key}"}}]),
                ),
                ("row_key", json!("id")),
            ]),
        )]);
        let el = spec.elements.get("root").unwrap();
        let data = json!({"items": [
            {"id": 7, "name": "Alpha", "status": "open", "customer": "Acme"}
        ]});
        let html = render_kanban_board(el, &spec, &data, 0);
        assert!(html.contains("Acme"), "card description missing: {html}");
        assert!(html.contains("View"), "row action missing: {html}");
        assert!(
            html.contains("/orders/7"),
            "row_key interpolation missing: {html}"
        );
    }

    #[test]
    fn segmented_control_literal_items_render_with_active_highlight() {
        let spec = build_spec(vec![(
            "root",
            Element::new("SegmentedControl").prop(
                "items",
                json!([
                    {"label": "Giorno", "href": "?view=day", "active": true},
                    {"label": "Mese", "href": "?view=month", "active": false}
                ]),
            ),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_segmented_control(el, &spec, &json!({}), 1);
        assert!(html.contains("Giorno"), "label missing: {html}");
        assert!(html.contains("Mese"), "second label missing: {html}");
        assert!(html.contains("?view=day"), "href missing: {html}");
        assert!(
            html.contains("inline-flex rounded-md border"),
            "outer container missing: {html}"
        );
        assert!(
            html.contains("bg-surface text-text font-semibold"),
            "active styling missing: {html}"
        );
    }

    #[test]
    fn segmented_control_aria_label_adds_tablist_role() {
        let spec = build_spec(vec![(
            "root",
            Element::new("SegmentedControl")
                .prop("aria_label", "Vista calendario")
                .prop(
                    "items",
                    json!([
                        {"label": "G", "href": "?d", "active": true},
                        {"label": "M", "href": "?m", "active": false}
                    ]),
                ),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_segmented_control(el, &spec, &json!({}), 1);
        assert!(
            html.contains("role=\"tablist\""),
            "tablist role missing: {html}"
        );
        assert!(
            html.contains("aria-label=\"Vista calendario\""),
            "aria-label missing: {html}"
        );
        assert!(
            html.contains("aria-selected=\"true\""),
            "active aria-selected missing: {html}"
        );
        assert!(
            html.contains("aria-selected=\"false\""),
            "inactive aria-selected missing: {html}"
        );
    }

    #[test]
    fn segmented_control_data_path_resolves_items() {
        let spec = build_spec(vec![(
            "root",
            Element::new("SegmentedControl").prop("data_path", "/nav/items"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let data = json!({
            "nav": {
                "items": [
                    {"label": "←", "href": "?prev", "active": false, "aria_label": "Precedente"},
                    {"label": "Oggi", "href": "?today", "active": false},
                    {"label": "→", "href": "?next", "active": false, "aria_label": "Successivo"}
                ]
            }
        });
        let html = render_segmented_control(el, &spec, &data, 1);
        assert!(html.contains("?prev"), "prev href missing: {html}");
        assert!(html.contains("?next"), "next href missing: {html}");
        assert!(
            html.contains("aria-label=\"Precedente\""),
            "per-item aria_label missing: {html}"
        );
    }

    #[test]
    fn segmented_control_empty_renders_nothing() {
        let spec = build_spec(vec![("root", Element::new("SegmentedControl"))]);
        let el = spec.elements.get("root").unwrap();
        let html = render_segmented_control(el, &spec, &json!({}), 1);
        assert!(
            html.is_empty(),
            "empty segmented control should emit nothing: {html}"
        );
    }

    #[test]
    fn sidebar_layout_renders_nav_and_main_with_active_highlight() {
        let spec = build_spec(vec![
            (
                "root",
                Element::new("SidebarLayout")
                    .prop(
                        "items",
                        json!([
                            {"slug": "generale", "label": "Generale", "url": "?tab=generale"},
                            {"slug": "servizi", "label": "Servizi", "url": "?tab=servizi"}
                        ]),
                    )
                    .prop("active", "servizi")
                    .child("body"),
            ),
            ("body", Element::new("Text").prop("content", "MAIN-BODY")),
        ]);
        let el = spec.elements.get("root").unwrap();
        let html = render_sidebar_layout(el, &spec, &json!({}), 1);
        assert!(
            html.contains("MAIN-BODY"),
            "main slot child missing: {html}"
        );
        assert!(html.contains("Generale"), "inactive item missing: {html}");
        assert!(html.contains("Servizi"), "active item missing: {html}");
        assert!(
            html.contains("md:grid md:grid-cols-[220px"),
            "asymmetric grid layout missing: {html}"
        );
        assert!(
            html.contains("aria-current=\"page\""),
            "active aria-current missing: {html}"
        );
    }

    #[test]
    fn sidebar_layout_active_swaps_highlight() {
        let items = json!([
            {"slug": "a", "label": "A", "url": "?tab=a"},
            {"slug": "b", "label": "B", "url": "?tab=b"}
        ]);
        let spec_a = build_spec(vec![(
            "root",
            Element::new("SidebarLayout")
                .prop("items", items.clone())
                .prop("active", "a"),
        )]);
        let spec_b = build_spec(vec![(
            "root",
            Element::new("SidebarLayout")
                .prop("items", items)
                .prop("active", "b"),
        )]);
        let html_a =
            render_sidebar_layout(spec_a.elements.get("root").unwrap(), &spec_a, &json!({}), 1);
        let html_b =
            render_sidebar_layout(spec_b.elements.get("root").unwrap(), &spec_b, &json!({}), 1);
        // Active item carries aria-current="page"; inactive does not. We check
        // that the marker lands on the right text in each spec.
        let active_a_idx = html_a.find("aria-current=\"page\"").unwrap();
        let active_b_idx = html_b.find("aria-current=\"page\"").unwrap();
        // The substring after the marker up to the next "</a>" must contain
        // the active label.
        let tail_a = &html_a[active_a_idx
            ..html_a[active_a_idx..]
                .find("</a>")
                .map(|p| active_a_idx + p)
                .unwrap_or(html_a.len())];
        let tail_b = &html_b[active_b_idx
            ..html_b[active_b_idx..]
                .find("</a>")
                .map(|p| active_b_idx + p)
                .unwrap_or(html_b.len())];
        assert!(
            tail_a.ends_with('A'),
            "active=a should highlight A; tail_a={tail_a}"
        );
        assert!(
            tail_b.ends_with('B'),
            "active=b should highlight B; tail_b={tail_b}"
        );
    }

    // ── ActionGroup tests ─────────────────────────────────────────────────

    fn action_item(label: &str, url: &str, method: &str) -> serde_json::Value {
        json!({
            "label": label,
            "action": {"url": url, "method": method, "handler": ""}
        })
    }

    fn action_item_destructive(label: &str, url: &str) -> serde_json::Value {
        json!({
            "label": label,
            "action": {"url": url, "method": "DELETE", "handler": ""},
            "destructive": true
        })
    }

    #[test]
    fn render_action_group_inline_and_overflow() {
        // 5 GET items, max_inline default (2), none destructive:
        // → 2 inline anchors + kebab holding 3 overflow items
        let items = json!([
            action_item("A", "/a", "GET"),
            action_item("B", "/b", "GET"),
            action_item("C", "/c", "GET"),
            action_item("D", "/d", "GET"),
            action_item("E", "/e", "GET"),
        ]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m1"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        // inline: A and B appear as links
        assert!(html.contains(">A<"), "A inline missing: {html}");
        assert!(html.contains(">B<"), "B inline missing: {html}");
        // overflow kebab present
        assert!(
            html.contains("popovertarget=\"m1\""),
            "kebab trigger missing: {html}"
        );
        assert!(
            html.contains("popover id=\"m1\" data-popover-menu"),
            "popover panel missing: {html}"
        );
        // C, D, E in overflow
        assert!(html.contains(">C<"), "C in overflow missing: {html}");
        assert!(html.contains(">D<"), "D in overflow missing: {html}");
        assert!(html.contains(">E<"), "E in overflow missing: {html}");
    }

    #[test]
    fn action_group_no_overflow_hides_kebab() {
        // 2 GET items, max_inline 2, none destructive → NO kebab
        let items = json!([action_item("A", "/a", "GET"), action_item("B", "/b", "GET"),]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m2"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        assert!(
            !html.contains("popovertarget"),
            "kebab should be absent: {html}"
        );
        assert!(
            !html.contains("popover id="),
            "popover panel should be absent: {html}"
        );
        assert!(html.contains(">A<"), "A missing: {html}");
        assert!(html.contains(">B<"), "B missing: {html}");
    }

    #[test]
    fn action_group_destructive_ordering() {
        // items: [A (normal), DEL (destructive), B (normal)], max_inline 2
        // → A and B inline, DEL in kebab and LAST; DEL never inline
        let items = json!([
            action_item("A", "/a", "GET"),
            action_item_destructive("DEL", "/del"),
            action_item("B", "/b", "GET"),
        ]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m3"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        // A and B inline
        assert!(html.contains(">A<"), "A inline missing: {html}");
        assert!(html.contains(">B<"), "B inline missing: {html}");
        // kebab exists (destructive overflows)
        assert!(
            html.contains("popovertarget=\"m3\""),
            "kebab missing: {html}"
        );
        // DEL in overflow (in popover panel)
        assert!(html.contains(">DEL<"), "DEL missing: {html}");
        // DEL must appear AFTER the popover panel opening tag
        let popover_start = html.find("popover id=\"m3\"").expect("popover panel");
        let del_pos = html.find(">DEL<").expect("DEL pos");
        assert!(
            del_pos > popover_start,
            "DEL should be inside overflow panel: {html}"
        );
        // DEL must be last in the panel (no inline button named DEL before popover)
        let a_pos = html.find(">A<").expect("A pos");
        let b_pos = html.find(">B<").expect("B pos");
        assert!(
            a_pos < popover_start,
            "A should be inline (before popover): {html}"
        );
        assert!(
            b_pos < popover_start,
            "B should be inline (before popover): {html}"
        );
    }

    #[test]
    fn action_group_non_get_wraps_form() {
        // 1 POST inline item → output contains <form method="post"> + <button type="submit"
        let items = json!([action_item("Save", "/save", "POST")]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m4"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        assert!(
            html.contains("<form action=\"/save\" method=\"post\">"),
            "form missing: {html}"
        );
        assert!(
            html.contains("<button type=\"submit\""),
            "submit button missing: {html}"
        );
        assert!(
            !html.contains("popovertarget"),
            "no overflow expected: {html}"
        );
    }

    #[test]
    fn action_group_get_renders_link() {
        // 1 GET inline item → <a href → no <form
        let items = json!([action_item("View", "/view", "GET")]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m5"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        assert!(html.contains("<a href=\"/view\""), "anchor missing: {html}");
        assert!(
            !html.contains("<form"),
            "form should not appear for GET: {html}"
        );
    }

    #[test]
    fn action_group_data_binding_parity() {
        // Items decoded from a literal array render identically to the same
        // items declared directly — $data binding is resolved before render_action_group
        // is called; this test asserts that decoding an items array is transparent.
        let items = json!([action_item("X", "/x", "GET"), action_item("Y", "/y", "GET"),]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items.clone())
                .prop("menu_id", "parity"),
        )]);
        let el = spec.elements.get("root").unwrap();
        let html = render_action_group(el, &spec, &json!({}), 1);
        assert!(html.contains(">X<"), "X missing: {html}");
        assert!(html.contains(">Y<"), "Y missing: {html}");
        // same render from a second spec with identical props — deterministic
        let spec2 = build_spec(vec![(
            "root2",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "parity"),
        )]);
        let el2 = spec2.elements.get("root2").unwrap();
        let html2 = render_action_group(el2, &spec2, &json!({}), 1);
        assert_eq!(html, html2, "render should be deterministic/identical");
    }

    #[test]
    fn action_group_visible_if() {
        // item with visible_if pointing to an absent field → hidden (fail-closed)
        // item with visible_if pointing to a truthy field → shown
        let items = json!([
            {
                "label": "Shown",
                "action": {"url": "/shown", "method": "GET", "handler": ""},
                "visible_if": "active"
            },
            {
                "label": "Hidden",
                "action": {"url": "/hidden", "method": "GET", "handler": ""},
                "visible_if": "nonexistent_field"
            },
        ]);
        let spec = build_spec(vec![(
            "root",
            Element::new("ActionGroup")
                .prop("items", items)
                .prop("menu_id", "m6"),
        )]);
        let el = spec.elements.get("root").unwrap();
        // data row with "active": true but no "nonexistent_field"
        let data = json!({"active": true});
        let html = render_action_group(el, &spec, &data, 1);
        assert!(
            html.contains(">Shown<"),
            "Shown item should be visible: {html}"
        );
        assert!(
            !html.contains(">Hidden<"),
            "Hidden item should be absent (fail-closed): {html}"
        );
    }
}
