//! Data-display renderers: `Table` and `DataTable`.
//!
//! Each function reads typed props, escapes and substitutes cell values, and
//! emits HTML. Per-row actions resolve URL placeholders against the row's
//! data at render time. Data resolution is routed through
//! [`crate::data::resolve_path`].
//!
//! Behaviors worth flagging for spec authors:
//! - **DataTable `row_key` / `id` URL templating** — row-action URLs have
//!   the placeholders `{row_key}` and `{id}` substituted against the row's
//!   data before emission. When the `row_key` prop is unset, the fallback
//!   is the row index.
//! - **Table empty-state** — when `data_path` resolves to an array but it
//!   is empty, `props.empty_message` is emitted as a single spanning
//!   `<td>`.

use serde_json::Value;

use crate::component::{
    BadgeVariant, Column, ColumnFormat, DataTableProps, DropdownMenuAction, MediaCardGridProps,
    TableProps,
};
use crate::data::resolve_path;
use crate::spec::{Element, Spec};

use super::atoms::{badge_inline_html, render_menu_item};
use super::html_escape;

/// Renders a simple `Table` element. Reads `TableProps.columns` and
/// resolves rows from `TableProps.data_path`. When `row_actions` is set,
/// an `Azioni` header column is appended and each row receives one
/// `<a href="...">` per action.
pub(crate) fn render_table(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: TableProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Table props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let mut html = String::from(
        "<div class=\"overflow-x-auto\"><table class=\"min-w-full divide-y divide-border\">",
    );

    // Header.
    html.push_str("<thead class=\"bg-surface\"><tr>");
    for col in &props.columns {
        html.push_str(&format!(
            "<th class=\"px-4 py-2 text-left text-xs font-medium uppercase tracking-wider text-text-muted\">{}</th>",
            html_escape(&col.label)
        ));
    }
    if props.row_actions.is_some() {
        html.push_str(
            "<th class=\"px-4 py-2 text-right text-xs font-medium uppercase tracking-wider text-text-muted\">Azioni</th>"
        );
    }
    html.push_str("</tr></thead>");

    // Body.
    html.push_str("<tbody class=\"divide-y divide-border bg-background\">");

    let rows = resolve_path(data, &props.data_path);
    let row_array = rows.and_then(|v| v.as_array());

    if let Some(items) = row_array {
        if items.is_empty() {
            if let Some(ref msg) = props.empty_message {
                let col_count =
                    props.columns.len() + if props.row_actions.is_some() { 1 } else { 0 };
                html.push_str(&format!(
                    "<tr><td colspan=\"{}\" class=\"px-4 py-4 text-center text-sm text-text-muted\">{}</td></tr>",
                    col_count,
                    html_escape(msg)
                ));
            }
        } else {
            for row in items {
                html.push_str("<tr class=\"hover:bg-surface\">");
                for col in &props.columns {
                    let cell_text = cell_string(row.get(&col.key));
                    html.push_str(&format!(
                        "<td class=\"px-4 py-2 text-sm text-text whitespace-nowrap\">{}</td>",
                        html_escape(&cell_text)
                    ));
                }
                if let Some(ref actions) = props.row_actions {
                    html.push_str("<td class=\"px-4 py-2 text-right text-sm space-x-2\">");
                    for action in actions {
                        let url = action.url.as_deref().unwrap_or("#");
                        let handler_str = action.handler.as_str();
                        let label = handler_str.split('.').next_back().unwrap_or(handler_str);
                        html.push_str(&format!(
                            "<a href=\"{}\" class=\"text-primary hover:text-primary/80\">{}</a>",
                            html_escape(url),
                            html_escape(label)
                        ));
                    }
                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
        }
    } else if let Some(ref msg) = props.empty_message {
        let col_count = props.columns.len() + if props.row_actions.is_some() { 1 } else { 0 };
        html.push_str(&format!(
            "<tr><td colspan=\"{}\" class=\"px-4 py-4 text-center text-sm text-text-muted\">{}</td></tr>",
            col_count,
            html_escape(msg)
        ));
    }

    html.push_str("</tbody></table></div>");
    html
}

/// Renders a `DataTable`. Supports declarative columns with formatters,
/// per-row actions wrapped in a portal-mode dropdown (positioned with
/// `position: fixed` so it escapes the table wrapper's overflow), optional search
/// bar, and per-row visibility filters. Emits a stripe-style desktop table
/// with alternating rows plus a mobile card fallback.
///
/// URL templating: row-action URLs containing `{row_key}` or `{id}` are
/// substituted against the row's value for `props.row_key` (default: row
/// index) and the row's `id` field respectively. Any other `{column_key}`
/// placeholder is substituted against the matching column value on each
/// row before emission.
pub(crate) fn render_data_table(el: &Element, _spec: &Spec, data: &Value, _depth: usize) -> String {
    let props: DataTableProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode DataTable props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let rows = resolve_path(data, &props.data_path);
    let items: Vec<Value> = rows.and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let has_actions = props.row_actions.is_some();
    let col_count = props.columns.len() + if has_actions { 1 } else { 0 };
    let empty_msg = props
        .empty_message
        .as_deref()
        .unwrap_or("Nessun elemento trovato");

    let mut html = String::new();

    // Empty short-circuit: emit a single centered card with the description
    // text. Replaces the prior empty-table-row + mobile-empty-div split with
    // one shared, table-less visual so the empty state reads as a deliberate
    // placeholder rather than a degenerate table.
    if items.is_empty() {
        let _ = col_count; // silence unused — column count irrelevant when empty
        return format!(
            "<div class=\"rounded-lg border border-border bg-card min-h-40 py-8 px-6 flex items-center justify-center\">\
             <p class=\"text-sm text-text-muted text-center max-w-md\">{}</p>\
             </div>",
            html_escape(empty_msg)
        );
    }

    // Desktop table (hidden on mobile).
    html.push_str(
        "<div class=\"hidden md:block rounded-lg border border-border overflow-hidden\">",
    );

    {
        html.push_str("<table class=\"w-full\">");

        // Header.
        html.push_str("<thead><tr class=\"bg-surface\">");
        for col in &props.columns {
            html.push_str(&format!(
                "<th class=\"px-4 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-muted\">{}</th>",
                html_escape(&col.label)
            ));
        }
        if has_actions {
            html.push_str(
                "<th class=\"px-4 py-2 text-right text-xs font-semibold uppercase tracking-wider text-text-muted\">Azioni</th>"
            );
        }
        html.push_str("</tr></thead>");

        // Body.
        html.push_str("<tbody>");
        for (index, row) in items.iter().enumerate() {
            let row_key_value = resolve_row_key(row, props.row_key.as_deref(), index);
            let row_href = props
                .row_href
                .as_deref()
                .map(|tmpl| template_url(tmpl, row, &row_key_value));
            let (extra_class, click_attrs) = if let Some(ref href) = row_href {
                let onclick = format!(
                    " onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))window.location.assign(this.dataset.rowHref)\" data-row-href=\"{}\"",
                    html_escape(href)
                );
                (" cursor-pointer", onclick)
            } else if props.row_actions.is_some() {
                let onclick = format!(
                    " onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))document.getElementById('dt-{}').showPopover()\"",
                    html_escape(&row_key_value)
                );
                (" cursor-pointer", onclick)
            } else {
                ("", String::new())
            };
            html.push_str(&format!(
                "<tr class=\"even:bg-surface hover:bg-surface/80 transition-colors duration-150 border-t border-border{extra_class}\"{click_attrs}>"
            ));
            for col in &props.columns {
                html.push_str(&format!(
                    "<td class=\"px-4 py-2 text-sm text-text\">{}</td>",
                    render_cell(col, row.get(&col.key))
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let templated = template_actions(actions, row, &row_key_value);
                html.push_str("<td class=\"px-4 py-2 text-right\">");
                html.push_str(&render_inline_dropdown(
                    &format!("dt-{row_key_value}"),
                    &templated,
                ));
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</div>");

    // Mobile cards (visible on mobile). Empty is handled by the early
    // short-circuit above; reaching here implies at least one row.
    html.push_str("<div class=\"block md:hidden space-y-3\">");
    {
        for (index, row) in items.iter().enumerate() {
            let row_key_value = resolve_row_key(row, props.row_key.as_deref(), index);
            let row_href = props
                .row_href
                .as_deref()
                .map(|tmpl| template_url(tmpl, row, &row_key_value));
            let (open_tag, close_tag) = if let Some(ref href) = row_href {
                (
                    format!(
                        "<a href=\"{}\" class=\"block rounded-lg border border-border bg-card p-4 space-y-2 hover:bg-surface/60 cursor-pointer\">",
                        html_escape(href)
                    ),
                    "</a>".to_string(),
                )
            } else if props.row_actions.is_some() {
                (
                    format!(
                        "<div class=\"rounded-lg border border-border bg-card p-4 space-y-2 cursor-pointer\" onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))document.getElementById('dt-m-{}').showPopover()\">",
                        html_escape(&row_key_value)
                    ),
                    "</div>".to_string(),
                )
            } else {
                (
                    "<div class=\"rounded-lg border border-border bg-card p-4 space-y-2\">"
                        .to_string(),
                    "</div>".to_string(),
                )
            };
            html.push_str(&open_tag);
            for col in &props.columns {
                html.push_str(&format!(
                    "<div class=\"flex justify-between\"><span class=\"text-xs font-semibold text-text-muted uppercase\">{}</span><span class=\"text-sm text-text\">{}</span></div>",
                    html_escape(&col.label),
                    render_cell(col, row.get(&col.key))
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let templated = template_actions(actions, row, &row_key_value);
                html.push_str("<div class=\"pt-2 border-t border-border flex justify-end\">");
                html.push_str(&render_inline_dropdown(
                    &format!("dt-m-{row_key_value}"),
                    &templated,
                ));
                html.push_str("</div>");
            }
            html.push_str(&close_tag);
        }
    }
    html.push_str("</div>");

    html
}

/// Renders a single cell's value as a plain string. Handles strings,
/// numbers, booleans, and nulls directly; arrays and objects round-trip
/// through `serde_json::to_string`.
fn cell_string(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(v @ Value::Array(_)) | Some(v @ Value::Object(_)) => {
            serde_json::to_string(v).unwrap_or_default()
        }
    }
}

/// Render a single DataTable cell. Dispatches on `col.format`:
///
/// - `Some(ColumnFormat::Badge)` reads the cell as `{variant, label}` and
///   emits a Badge `<span>` (shared shape with the standalone `Badge`
///   component via [`badge_inline_html`]). Invalid values emit an HTML
///   comment diagnostic instead of falling back to text — a typed column
///   that silently rendered raw JSON would hide real spec bugs.
/// - All other formats (including absent) emit the existing
///   `html_escape(cell_string(...))` text.
///
/// The Badge cell intentionally bypasses `html_escape` on the wrapper markup
/// because the markup is server-controlled (no caller input flows through
/// the class names or `<span>` chrome). The `label` itself IS html-escaped
/// inside `badge_inline_html`.
fn render_cell(col: &Column, value: Option<&Value>) -> String {
    if let Some(ColumnFormat::Badge) = col.format {
        // Cell must be an object `{variant, label}` (or absent — empty cell).
        match value {
            None | Some(Value::Null) => return String::new(),
            Some(v @ Value::Object(_)) => {
                #[derive(serde::Deserialize)]
                struct BadgeCell {
                    variant: BadgeVariant,
                    label: String,
                }
                match serde_json::from_value::<BadgeCell>(v.clone()) {
                    Ok(cell) => return badge_inline_html(cell.variant, &cell.label),
                    Err(e) => {
                        return format!(
                            "<!-- ferro-json-ui: invalid Badge cell value: {} -->",
                            html_escape(&e.to_string())
                        );
                    }
                }
            }
            Some(_) => {
                return format!(
                    "<!-- ferro-json-ui: invalid Badge cell value: expected object {{variant, label}}, got {} -->",
                    html_escape(&cell_string(value))
                );
            }
        }
    }
    if let Some(ColumnFormat::Image) = col.format {
        // Cell value is an image URL string. Null or empty → empty cell (no broken img).
        // The URL is html-escaped before interpolation to prevent src attribute injection
        // (T-213-09).
        match value {
            None | Some(Value::Null) => return String::new(),
            Some(_) => {
                let url = cell_string(value);
                if url.is_empty() {
                    return String::new();
                }
                return format!(
                    "<img src=\"{}\" alt=\"\" class=\"w-8 h-8 rounded-full object-cover\" />",
                    html_escape(&url)
                );
            }
        }
    }
    html_escape(&cell_string(value))
}

/// Substitute placeholders in a URL template against a row.
///
/// Mirrors the placeholder resolution of `template_actions` so the row-level
/// `props.row_href` URL is computed with the same rules as
/// `row_actions[].action.url`:
///
/// 1. `{col_key}` — every key in the row object substitutes its value.
/// 2. `{row_key}` — resolved against `row_key_value`.
/// 3. `{id}` — resolved against `row["id"]` when present.
///
/// Missing placeholders are left unsubstituted.
fn template_url(template: &str, row: &Value, row_key_value: &str) -> String {
    let mut url = template.to_string();
    if let Some(obj) = row.as_object() {
        for (col_key, col_val) in obj {
            let val_str = match col_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => continue,
            };
            url = url.replace(&format!("{{{col_key}}}"), &val_str);
            url = url.replace(&format!("{{row.{col_key}}}"), &val_str);
        }
    }
    url = url.replace("{row_key}", row_key_value);
    if let Some(id) = row.get("id").and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }) {
        url = url.replace("{id}", &id);
    }
    url
}

/// Resolves the row key for a single row. Reads the value at
/// `props.row_key` from the row when present and stringifiable, otherwise
/// falls back to the row index.
pub(crate) fn resolve_row_key(row: &Value, row_key_prop: Option<&str>, index: usize) -> String {
    if let Some(rk) = row_key_prop {
        if let Some(v) = row.get(rk) {
            match v {
                Value::String(s) => return s.clone(),
                Value::Number(n) => return n.to_string(),
                _ => {}
            }
        }
    }
    index.to_string()
}

/// Templates row-action URLs for a single row.
///
/// Substitution order:
/// 1. All column keys present in the row object (`{label}`, `{slug_path}`,
///    …). Only `String` and `Number` values are substituted; booleans,
///    nulls, arrays, and objects are skipped.
/// 2. `{row_key}` — resolved against `row_key_value`.
/// 3. `{id}` — resolved against `row["id"]` when present.
///
/// Missing placeholders (no matching column, and not `{row_key}` / `{id}`)
/// are left unsubstituted — no panic, no silent removal.
/// Returns `true` iff a row-action item should appear for this row.
///
/// - No `visible_if` declared → always visible (backwards compat).
/// - `visible_if = Some(field)` and `row[field]` is truthy → visible.
///   Truthy = `true` / non-zero number / non-empty string / non-empty array
///   or object.
/// - `visible_if = Some(field)` and `row[field]` is missing, null, false,
///   `0`, or empty → hidden. **Fail-closed** so a typo in the field name
///   cannot leak an action onto every row.
fn action_visible_for_row(action: &DropdownMenuAction, row: &Value) -> bool {
    let Some(field) = action.visible_if.as_deref() else {
        return true;
    };
    match row.get(field) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(f) = n.as_f64() {
                f != 0.0
            } else {
                false
            }
        }
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

pub(crate) fn template_actions(
    actions: &[DropdownMenuAction],
    row: &Value,
    row_key_value: &str,
) -> Vec<DropdownMenuAction> {
    let id_value: Option<String> = row.get("id").and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    });

    actions
        .iter()
        .filter(|a| action_visible_for_row(a, row))
        .map(|a| {
            let mut cloned = a.clone();
            // URL fallback: when the action has no explicit `url`, use the
            // handler name as the base.
            let base_url = cloned
                .action
                .url
                .clone()
                .or_else(|| Some(cloned.action.handler.as_str().to_string()));
            if let Some(mut url) = base_url {
                // Substitute all row column keys first.
                if let Some(obj) = row.as_object() {
                    for (col_key, col_val) in obj {
                        let val_str = match col_val {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        url = url.replace(&format!("{{{col_key}}}"), &val_str);
                        url = url.replace(&format!("{{row.{col_key}}}"), &val_str);
                    }
                }
                // `{row_key}` substitutes against the value at `props.row_key`.
                url = url.replace("{row_key}", row_key_value);
                if let Some(ref id) = id_value {
                    url = url.replace("{id}", id);
                }
                cloned.action.url = Some(url);
            }
            cloned
        })
        .collect()
}

/// Minimal self-contained dropdown used for row actions. Emits HTML popover
/// markup (`popovertarget` + `popover`); `runtime/dropdowns.rs` anchors the
/// panel under its trigger on open. The browser handles dismiss and lifts
/// the panel into the top layer, so the surrounding DataTable overflow
/// context cannot clip it.
pub(crate) fn render_inline_dropdown(menu_id: &str, items: &[DropdownMenuAction]) -> String {
    let id = html_escape(menu_id);
    let mut html = String::new();
    html.push_str(&format!(
        "<button type=\"button\" popovertarget=\"{id}\" aria-haspopup=\"menu\" aria-label=\"Azioni\" class=\"cursor-pointer select-none px-2 py-1 text-text-muted hover:text-text\">\u{22EE}</button>"
    ));
    html.push_str(&format!(
        "<div popover id=\"{id}\" data-popover-menu class=\"min-w-[10rem] rounded-md border border-border bg-card shadow-md text-left p-0\" role=\"menu\">"
    ));
    for item in items {
        html.push_str(&render_menu_item(
            item,
            "block px-3 py-2 text-sm hover:bg-surface",
            "block px-3 py-2 text-sm hover:bg-surface text-destructive",
            " role=\"menuitem\"",
        ));
    }
    html.push_str("</div>");
    html
}

/// Renders a `MediaCardGrid`. Each item in `data_path` becomes a card with
/// an optional screenshot image, title, description, status badge, and
/// per-row dropdown actions. URL placeholders follow the same `{row_key}`
/// / `{id}` interpolation rules as `DataTable`.
pub(crate) fn render_media_card_grid(
    el: &Element,
    _spec: &Spec,
    data: &Value,
    _depth: usize,
) -> String {
    let props: MediaCardGridProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode MediaCardGrid props: {} -->",
                html_escape(&e.to_string())
            );
        }
    };

    let rows = resolve_path(data, &props.data_path);
    let items: Vec<Value> = rows.and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let empty_msg = props
        .empty_message
        .as_deref()
        .unwrap_or("Nessun elemento trovato");

    if items.is_empty() {
        return format!(
            "<div class=\"rounded-lg border border-border bg-card min-h-40 py-8 px-6 flex items-center justify-center\">\
             <p class=\"text-sm text-text-muted text-center max-w-md\">{}</p>\
             </div>",
            html_escape(empty_msg)
        );
    }

    let col_class = match props.columns.unwrap_or(3) {
        2 => "grid-cols-1 md:grid-cols-2",
        4 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
        _ => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
    };

    let mut html = format!("<div class=\"grid {col_class} gap-4\">");

    let aspect_ratio = props.image_aspect_ratio.as_deref().unwrap_or("4/5");
    let image_position = props.image_position.as_deref().unwrap_or("center");

    for (index, row) in items.iter().enumerate() {
        let row_key_value = resolve_row_key(row, props.row_key.as_deref(), index);

        let title = row
            .get(props.title_key.as_str())
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let description = props
            .description_key
            .as_deref()
            .and_then(|k| row.get(k))
            .and_then(|v| v.as_str());

        let image_url = props
            .image_key
            .as_deref()
            .and_then(|k| row.get(k))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let image_href = props
            .image_href_key
            .as_deref()
            .and_then(|k| row.get(k))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let badge_label = props
            .badge_key
            .as_deref()
            .and_then(|k| row.get(k))
            .and_then(|v| v.as_str());

        let badge_variant = props
            .badge_variant_key
            .as_deref()
            .and_then(|k| row.get(k))
            .and_then(|v| v.as_str())
            .unwrap_or("outline");

        html.push_str(
            "<div class=\"rounded-lg border border-border bg-card overflow-hidden flex flex-col\">",
        );

        // Image section. If image_key resolves to null/empty, render a blank
        // placeholder slot with the same aspect ratio so card heights stay
        // consistent across the grid.
        if props.image_key.is_some() {
            if let Some(img_src) = image_url {
                let img_tag = format!(
                    "<img src=\"{}\" alt=\"{}\" class=\"w-full object-cover\" style=\"aspect-ratio: {}; object-position: {};\" loading=\"lazy\">",
                    html_escape(img_src),
                    html_escape(title),
                    html_escape(aspect_ratio),
                    html_escape(image_position),
                );
                if let Some(href) = image_href {
                    html.push_str(&format!(
                        "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"block overflow-hidden bg-surface shrink-0\">{}</a>",
                        html_escape(href),
                        img_tag
                    ));
                } else {
                    html.push_str(&format!(
                        "<div class=\"overflow-hidden bg-surface shrink-0\">{img_tag}</div>"
                    ));
                }
            } else {
                html.push_str(&format!(
                    "<div class=\"w-full bg-surface shrink-0\" style=\"aspect-ratio: {};\" aria-hidden=\"true\"></div>",
                    html_escape(aspect_ratio),
                ));
            }
        }

        // Body
        html.push_str("<div class=\"p-4 flex flex-col gap-1 flex-1\">");
        html.push_str(&format!(
            "<div class=\"text-sm font-semibold text-text\">{}</div>",
            html_escape(title)
        ));
        if let Some(desc) = description {
            html.push_str(&format!(
                "<div class=\"text-xs text-text-muted truncate\">{}</div>",
                html_escape(desc)
            ));
        }
        html.push_str("</div>");

        // Footer: badge + dropdown
        let has_badge = badge_label.is_some();
        let has_actions = props.row_actions.is_some();
        if has_badge || has_actions {
            html.push_str("<div class=\"px-4 pb-4 flex items-center justify-between gap-2\">");

            if let Some(label) = badge_label {
                let badge_classes = match badge_variant {
                    "destructive" => "bg-destructive/10 text-destructive",
                    _ => "border border-border text-text",
                };
                html.push_str(&format!(
                    "<span class=\"inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {}\">{}</span>",
                    badge_classes,
                    html_escape(label)
                ));
            } else if has_actions {
                html.push_str("<span></span>");
            }

            if let Some(ref actions) = props.row_actions {
                let templated = template_actions(actions, row, &row_key_value);
                html.push_str(&render_inline_dropdown(
                    &format!("mcg-{row_key_value}"),
                    &templated,
                ));
            }

            html.push_str("</div>");
        }

        html.push_str("</div>"); // card
    }

    html.push_str("</div>"); // grid
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mk_element(type_name: &str, props: Value) -> Element {
        Element {
            type_name: type_name.to_string(),
            props,
            children: Vec::new(),
            action: None,
            visible: None,
            each: None,
            if_: None,
        }
    }

    fn mk_spec(root: &str, el: Element) -> Spec {
        let mut spec = Spec::builder()
            .element("__tmp__", Element::new("Text"))
            .build()
            .expect("builder accepts trivial spec");
        spec.root = root.to_string();
        spec.elements.clear();
        spec.elements.insert(root.to_string(), el);
        spec
    }

    // ── Table ────────────────────────────────────────────────────────────

    #[test]
    fn table_renders_rows_from_data_path() {
        let el = mk_element(
            "Table",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [{"name": "Alice"}, {"name": "Bob"}]});
        let html = render_table(&el, &spec, &data, 1);
        assert!(html.contains("Alice"), "got: {html}");
        assert!(html.contains("Bob"), "got: {html}");
        assert!(html.contains("<thead"), "got: {html}");
    }

    #[test]
    fn table_empty_rows_emits_empty_message() {
        let el = mk_element(
            "Table",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
                "empty_message": "No users found",
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": []});
        let html = render_table(&el, &spec, &data, 1);
        assert!(html.contains("No users found"), "got: {html}");
    }

    #[test]
    fn table_missing_path_emits_empty_message_when_provided() {
        let el = mk_element(
            "Table",
            json!({
                "data_path": "/absent",
                "columns": [{"key": "name", "label": "Name"}],
                "empty_message": "Nothing here",
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({});
        let html = render_table(&el, &spec, &data, 1);
        assert!(html.contains("Nothing here"), "got: {html}");
    }

    #[test]
    fn table_cell_value_is_html_escaped() {
        let el = mk_element(
            "Table",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [{"name": "<script>x</script>"}]});
        let html = render_table(&el, &spec, &data, 1);
        assert!(!html.contains("<script>x</script>"), "got: {html}");
        assert!(
            html.contains("&lt;script&gt;x&lt;/script&gt;"),
            "got: {html}"
        );
    }

    #[test]
    fn table_props_decode_failure_emits_diagnostic() {
        let el = mk_element("Table", json!(42));
        let spec = mk_spec("root", el.clone());
        let html = render_table(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode Table props"),
            "got: {html}"
        );
    }

    // ── DataTable ────────────────────────────────────────────────────────

    #[test]
    fn data_table_url_template_replaces_id() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [
                    {"label": "Edit", "action": {"handler": "edit", "url": "/users/{id}/edit", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [
            {"id": "1", "name": "Alice"},
            {"id": "2", "name": "Bob"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/users/1/edit"),
            "row 1 URL missing; got: {html}"
        );
        assert!(
            html.contains("/users/2/edit"),
            "row 2 URL missing; got: {html}"
        );
        assert!(
            !html.contains("/users/{id}/edit"),
            "{{id}} placeholder must be replaced; got: {html}"
        );
    }

    #[test]
    fn data_table_url_template_replaces_row_key() {
        // `{row_key}` substitutes against the value at `props.row_key` on
        // each row (falls back to the row index).
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "row_key": "slug",
                "columns": [{"key": "slug", "label": "Slug"}],
                "row_actions": [
                    {"label": "Open", "action": {"handler": "show", "url": "/u/{row_key}", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [
            {"slug": "alice"},
            {"slug": "bob"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(html.contains("/u/alice"), "got: {html}");
        assert!(html.contains("/u/bob"), "got: {html}");
    }

    #[test]
    fn data_table_empty_renders_empty_message() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
                "empty_message": "No rows",
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": []});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(html.contains("No rows"), "got: {html}");
    }

    #[test]
    fn data_table_default_empty_message_used_when_absent() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": []});
        let html = render_data_table(&el, &spec, &data, 1);
        // The default empty message is "Nessun elemento trovato".
        assert!(html.contains("Nessun elemento trovato"), "got: {html}");
    }

    #[test]
    fn data_table_url_template_substitution_is_escaped() {
        // Attribute-breakout via a row's id field — the templated URL must
        // pass through html_escape before emission.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [
                    {"label": "Edit", "action": {"handler": "edit", "url": "/u/{id}", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [{"id": "\"><script>", "name": "x"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            !html.contains("><script>"),
            "attribute breakout; got: {html}"
        );
        assert!(html.contains("&quot;"), "got: {html}");
    }

    #[test]
    fn data_table_renders_desktop_and_mobile_markup() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/users",
                "columns": [{"key": "name", "label": "Name"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"users": [{"name": "Alice"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("hidden md:block"),
            "desktop wrapper; got: {html}"
        );
        assert!(
            html.contains("block md:hidden"),
            "mobile wrapper; got: {html}"
        );
    }

    #[test]
    fn data_table_props_decode_failure_emits_diagnostic() {
        let el = mk_element("DataTable", json!(42));
        let spec = mk_spec("root", el.clone());
        let html = render_data_table(&el, &spec, &json!({}), 1);
        assert!(
            html.contains("<!-- ferro-json-ui: failed to decode DataTable props"),
            "got: {html}"
        );
    }

    // Extended placeholder interpolation tests.

    #[test]
    fn data_table_url_template_replaces_column_key() {
        // Any column key bound at render time is substituted in action URLs.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/pages",
                "columns": [{"key": "label", "label": "Label"}],
                "row_actions": [
                    {"label": "Edit", "action": {"handler": "edit", "url": "/p/{slug_path}/edit", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"pages": [
            {"id": "7", "label": "Home", "slug_path": "/home"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/p//home/edit"),
            "expected /p//home/edit in output; got: {html}"
        );
        assert!(
            !html.contains("{slug_path}"),
            "{{slug_path}} placeholder must be replaced; got: {html}"
        );
    }

    #[test]
    fn data_table_url_template_replaces_multiple_keys() {
        // Multiple column keys in a single URL are all substituted.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/pages",
                "columns": [{"key": "label", "label": "Label"}],
                "row_actions": [
                    {"label": "View", "action": {"handler": "view", "url": "/p/{slug_path}/{status}", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"pages": [
            {"id": "1", "label": "Home", "slug_path": "/home", "status": "draft"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/p//home/draft"),
            "expected /p//home/draft in output; got: {html}"
        );
    }

    #[test]
    fn data_table_post_row_action_emits_form_not_anchor() {
        // A row_action declaring `method: POST` must render as a real
        // `<form method="post">` with a submit `<button>`, not as an
        // `<a href>` that issues a GET request. Regression for the
        // dropped-method bug in `render_inline_dropdown`.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Delete",
                    "action": {"handler": "destroy", "url": "/items/{id}", "method": "POST"},
                    "destructive": true,
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"id": "9", "name": "x"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("<form action=\"/items/9\" method=\"post\">"),
            "POST row_action must render a form; got: {html}"
        );
        assert!(
            html.contains("<button type=\"submit\""),
            "POST row_action must include a submit button; got: {html}"
        );
        assert!(
            !html.contains("<a href=\"/items/9\""),
            "POST row_action must not render an anchor (which would GET); got: {html}"
        );
    }

    #[test]
    fn data_table_delete_row_action_spoofs_method() {
        // PUT/PATCH/DELETE row_actions use POST + a `_method` hidden input
        // for HTTP method spoofing through the form submission.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Delete",
                    "action": {"handler": "destroy", "url": "/items/{id}", "method": "DELETE"},
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"id": "9", "name": "x"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("<form action=\"/items/9\" method=\"post\">"),
            "DELETE row_action must render a POST form; got: {html}"
        );
        assert!(
            html.contains("name=\"_method\" value=\"DELETE\""),
            "DELETE row_action must spoof method via hidden input; got: {html}"
        );
    }

    #[test]
    fn data_table_get_row_action_still_emits_anchor() {
        // GET row_actions (the most common case — edit links) continue
        // to render as plain `<a href>` for navigation.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Edit",
                    "action": {"handler": "edit", "url": "/items/{id}/edit", "method": "GET"},
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"id": "9", "name": "x"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("<a href=\"/items/9/edit\""),
            "GET row_action must render an anchor; got: {html}"
        );
        assert!(
            !html.contains("<form action=\"/items/9/edit\""),
            "GET row_action must not render a form; got: {html}"
        );
    }

    #[test]
    fn data_table_url_template_missing_key_leaves_placeholder() {
        // A placeholder with no matching column key is left as-is (no panic, no silent removal).
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/pages",
                "columns": [{"key": "label", "label": "Label"}],
                "row_actions": [
                    {"label": "Edit", "action": {"handler": "edit", "url": "/p/{nonexistent}/edit", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"pages": [
            {"id": "1", "label": "Home"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        // {nonexistent} has no matching column — must remain literal in the URL.
        assert!(
            html.contains("{nonexistent}"),
            "missing-key placeholder must be left unsubstituted; got: {html}"
        );
    }

    // F6 — {row.X} prefix alias tests.

    #[test]
    fn data_table_row_prefix_placeholder_resolved() {
        // {row.delete_url} must resolve to the row's `delete_url` field value.
        // No literal curly braces or URL-encoded form must survive into the
        // rendered HTML.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Delete",
                    "action": {
                        "handler": "destroy",
                        "url": "{row.delete_url}",
                        "method": "POST"
                    },
                    "destructive": true,
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [
            {"name": "Absence", "delete_url": "/dashboard/staff/1/assenze/3/elimina"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/dashboard/staff/1/assenze/3/elimina"),
            "resolved URL must appear in rendered HTML; got: {html}"
        );
        assert!(
            !html.contains("{row.delete_url}"),
            "literal {{row.delete_url}} must not appear in rendered HTML; got: {html}"
        );
        assert!(
            !html.contains("%7Brow.delete_url%7D"),
            "URL-encoded form must not appear in rendered HTML; got: {html}"
        );
    }

    #[test]
    fn data_table_bare_placeholder_resolved() {
        // Back-compat regression guard: the bare {delete_url} form (no row.
        // prefix) must continue to resolve after the F6 alias is added.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Delete",
                    "action": {
                        "handler": "destroy",
                        "url": "{delete_url}",
                        "method": "POST"
                    },
                    "destructive": true,
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [
            {"name": "Absence", "delete_url": "/dashboard/staff/1/assenze/3/elimina"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/dashboard/staff/1/assenze/3/elimina"),
            "bare {{delete_url}} must still resolve; got: {html}"
        );
        assert!(
            !html.contains("{delete_url}"),
            "literal {{delete_url}} must not appear in rendered HTML; got: {html}"
        );
    }

    #[test]
    fn data_table_row_prefix_missing_key_leaves_placeholder() {
        // Pitfall 4 guard: when the row has no `nonexistent` field,
        // {row.nonexistent} must be left literal — not silently stripped.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [{
                    "label": "Action",
                    "action": {
                        "handler": "act",
                        "url": "/items/{row.nonexistent}",
                        "method": "GET"
                    },
                }],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [
            {"name": "Item", "delete_url": "/items/1/delete"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        // The {row.nonexistent} placeholder has no matching key in the row;
        // it must survive unsubstituted (or HTML-escaped, which is also acceptable).
        assert!(
            html.contains("{row.nonexistent}") || html.contains("&#123;row.nonexistent&#125;"),
            "missing-key {{row.nonexistent}} must be left unsubstituted; got: {html}"
        );
    }

    #[test]
    fn data_table_row_href_legacy_placeholders() {
        // Regression guard: `{row_key}` and `{id}` still resolve alongside
        // the general column-key substitution.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/pages",
                "row_key": "slug",
                "columns": [{"key": "slug", "label": "Slug"}],
                "row_actions": [
                    {"label": "View", "action": {"handler": "view", "url": "/p/{row_key}/{id}", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"pages": [
            {"id": "7", "slug": "row-3"},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("/p/row-3/7"),
            "legacy {{row_key}} and {{id}} must still be substituted; got: {html}"
        );
    }

    // ── MediaCardGrid ────────────────────────────────────────────────────

    #[test]
    fn media_card_grid_empty_state() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let html = render_media_card_grid(&el, &spec, &json!({"items": []}), 1);
        assert!(html.contains("Nessun elemento trovato"), "got: {html}");
        assert!(
            !html.contains("<img"),
            "should not render image in empty state"
        );
    }

    #[test]
    fn media_card_grid_renders_title() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"name": "Hair Factory", "id": 1}]});
        let html = render_media_card_grid(&el, &spec, &data, 1);
        assert!(html.contains("Hair Factory"), "got: {html}");
    }

    #[test]
    fn media_card_grid_renders_image_with_link() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name",
                "image_key": "screenshot_url",
                "image_href_key": "preview_url"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"name": "HF3", "id": 1, "screenshot_url": "/dashboard/pagine/1/screenshot.png", "preview_url": "/s/amaris-experience/hf3/"}]});
        let html = render_media_card_grid(&el, &spec, &data, 1);
        assert!(html.contains("<img"), "expected img tag, got: {html}");
        assert!(
            html.contains("/dashboard/pagine/1/screenshot.png"),
            "got: {html}"
        );
        assert!(
            html.contains("target=\"_blank\""),
            "expected new-tab link, got: {html}"
        );
        assert!(html.contains("/s/amaris-experience/hf3/"), "got: {html}");
    }

    #[test]
    fn media_card_grid_no_image_when_key_absent() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"name": "HF3", "id": 1}]});
        let html = render_media_card_grid(&el, &spec, &data, 1);
        assert!(!html.contains("<img"), "got: {html}");
    }

    #[test]
    fn media_card_grid_badge_destructive() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name",
                "badge_key": "status",
                "badge_variant_key": "variant"
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data =
            json!({"items": [{"name": "HF", "status": "Non visibile", "variant": "destructive"}]});
        let html = render_media_card_grid(&el, &spec, &data, 1);
        assert!(html.contains("Non visibile"), "got: {html}");
        assert!(html.contains("text-destructive"), "got: {html}");
    }

    #[test]
    fn media_card_grid_row_actions_interpolated() {
        let el = mk_element(
            "MediaCardGrid",
            json!({
                "data_path": "/items",
                "title_key": "name",
                "row_key": "id",
                "row_actions": [{"label": "Elimina", "action": {"handler": "/dashboard/pagine/{row_key}/delete", "method": "POST"}, "destructive": true}]
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"name": "HF", "id": 42}]});
        let html = render_media_card_grid(&el, &spec, &data, 1);
        assert!(html.contains("/dashboard/pagine/42/delete"), "got: {html}");
    }

    // ── ColumnFormat::Badge ──────────────────────────────────────────────

    #[test]
    fn data_table_badge_column_format_renders_pill() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "status", "label": "Status", "format": "badge"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"status": {"variant": "destructive", "label": "Mancante"}}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(html.contains("Mancante"), "label missing; got: {html}");
        assert!(
            html.contains("bg-destructive/10"),
            "missing destructive class; got: {html}"
        );
        assert!(
            html.contains("rounded-full"),
            "missing badge base class; got: {html}"
        );
        assert!(
            !html.contains("{&quot;variant&quot;"),
            "raw json leaked into cell; got: {html}"
        );
    }

    #[test]
    fn data_table_badge_column_format_invalid_value_emits_diagnostic() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "status", "label": "Status", "format": "badge"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        // String instead of {variant, label}
        let data = json!({"rows": [{"status": "Mancante"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("<!-- ferro-json-ui: invalid Badge cell value"),
            "expected diagnostic; got: {html}"
        );
        // The bad value itself should NOT be rendered as live UI.
        assert!(!html.contains(">Mancante<"), "got: {html}");
    }

    #[test]
    fn data_table_badge_column_format_null_value_renders_empty_cell() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "status", "label": "Status", "format": "badge"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"status": null}]});
        let html = render_data_table(&el, &spec, &data, 1);
        // No diagnostic, no badge — just an empty cell. Confirms null is a
        // first-class "this row has no status" rather than an error.
        assert!(
            !html.contains("<!-- ferro-json-ui: invalid"),
            "null should be valid; got: {html}"
        );
        assert!(!html.contains("rounded-full"), "got: {html}");
    }

    // ── DropdownMenuAction::visible_if ────────────────────────────────────

    #[test]
    fn data_table_visible_if_keeps_action_when_truthy() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "id", "label": "Id"}],
                "row_actions": [
                    {"label": "Scarica", "action": {"handler": "download", "url": "/d/{id}", "method": "GET"}, "visible_if": "can_download"}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"id": "1", "can_download": true}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(html.contains("Scarica"), "got: {html}");
    }

    #[test]
    fn data_table_visible_if_drops_action_when_falsy() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "id", "label": "Id"}],
                "row_actions": [
                    {"label": "Scarica", "action": {"handler": "download", "url": "/d/{id}", "method": "GET"}, "visible_if": "can_download"}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"id": "1", "can_download": false}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            !html.contains("Scarica"),
            "action should be hidden; got: {html}"
        );
    }

    #[test]
    fn data_table_visible_if_drops_action_when_field_missing() {
        // Fail-closed: missing field = hide. A typo in the spec must NOT show
        // the action everywhere (would expose privileged actions on every row).
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "id", "label": "Id"}],
                "row_actions": [
                    {"label": "Scarica", "action": {"handler": "download", "url": "/d/{id}", "method": "GET"}, "visible_if": "can_download_typo"}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"id": "1", "can_download": true}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            !html.contains("Scarica"),
            "missing field must hide action; got: {html}"
        );
    }

    #[test]
    fn data_table_visible_if_absent_keeps_action() {
        // Backwards-compat: actions with no visible_if always show.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "id", "label": "Id"}],
                "row_actions": [
                    {"label": "Open", "action": {"handler": "open", "url": "/o/{id}", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"id": "1"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("Open"),
            "action without visible_if should always show; got: {html}"
        );
    }

    #[test]
    fn data_table_visible_if_filters_per_row_independently() {
        // Two rows, one declared action with visible_if — the action shows on
        // the row whose gate is true and hides on the row whose gate is false.
        // This is the load-bearing scenario for heterogeneous-row tables.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "id", "label": "Id"}],
                "row_actions": [
                    {"label": "Send link", "action": {"handler": "send", "url": "/s/{id}", "method": "POST"}, "visible_if": "can_send"},
                    {"label": "Download", "action": {"handler": "dl", "url": "/d/{id}", "method": "GET"}, "visible_if": "can_dl"}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [
            {"id": "1", "can_send": true,  "can_dl": false},
            {"id": "2", "can_send": false, "can_dl": true},
        ]});
        let html = render_data_table(&el, &spec, &data, 1);
        // Both labels present in the full HTML (one per row).
        assert!(
            html.contains("Send link"),
            "row 1 action missing; got: {html}"
        );
        assert!(
            html.contains("Download"),
            "row 2 action missing; got: {html}"
        );
        // Each row's dropdown menu has its own popover id, so this asserts the
        // filter ran per-row by checking action urls land on the right rows.
        // Row 1's dropdown should contain /s/1 (send) but not /d/1 (download).
        // Row 2's dropdown should contain /d/2 (download) but not /s/2 (send).
        assert!(html.contains("/s/1"), "row 1 send url missing; got: {html}");
        assert!(
            !html.contains("/d/1"),
            "row 1 should not show download; got: {html}"
        );
        assert!(
            html.contains("/d/2"),
            "row 2 download url missing; got: {html}"
        );
        assert!(
            !html.contains("/s/2"),
            "row 2 should not show send; got: {html}"
        );
    }

    // ── ColumnFormat::Image ──────────────────────────────────────────────

    #[test]
    fn data_table_image_column_format_renders_img_tag() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "avatar_url", "label": "Avatar", "format": "image"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"avatar_url": "https://cdn.example/a.png"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(html.contains("<img"), "expected <img tag; got: {html}");
        assert!(
            html.contains("https://cdn.example/a.png"),
            "expected URL in src; got: {html}"
        );
    }

    #[test]
    fn data_table_image_column_format_xss_escaped() {
        // T-213-09: a URL containing `"` must not break out of the src attribute.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "avatar_url", "label": "Avatar", "format": "image"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"avatar_url": "https://x.com/a.png\" onload=\"alert(1)"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            !html.contains("\" onload=\""),
            "unescaped quote must not break src attribute; got: {html}"
        );
        assert!(
            html.contains("&quot;"),
            "quote must be html-escaped; got: {html}"
        );
    }

    #[test]
    fn data_table_image_column_format_null_renders_empty_cell() {
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/rows",
                "columns": [{"key": "avatar_url", "label": "Avatar", "format": "image"}],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"rows": [{"avatar_url": null}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            !html.contains("<img"),
            "null must render empty cell, no <img; got: {html}"
        );
    }
}
