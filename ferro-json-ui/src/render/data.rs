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

use crate::component::{DataTableProps, DropdownMenuAction, TableProps};
use crate::data::resolve_path;
use crate::spec::{Element, Spec};

use super::atoms::render_menu_item;
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
            let (extra_class, click_attrs) = if has_actions {
                let menu_id = format!("dt-{row_key_value}");
                let onclick = format!(
                    " onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))document.getElementById('{}').showPopover()\"",
                    html_escape(&menu_id)
                );
                (" cursor-pointer", onclick)
            } else if let Some(ref href) = row_href {
                let onclick = format!(
                    " onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))window.location.assign(this.dataset.rowHref)\" data-row-href=\"{}\"",
                    html_escape(href)
                );
                (" cursor-pointer", onclick)
            } else {
                ("", String::new())
            };
            html.push_str(&format!(
                "<tr class=\"even:bg-surface hover:bg-surface/80 transition-colors duration-150 border-t border-border{extra_class}\"{click_attrs}>"
            ));
            for col in &props.columns {
                let cell_text = cell_string(row.get(&col.key));
                html.push_str(&format!(
                    "<td class=\"px-4 py-2 text-sm text-text\">{}</td>",
                    html_escape(&cell_text)
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
            let (open_tag, close_tag) = if has_actions {
                let menu_id = format!("dt-m-{row_key_value}");
                (
                    format!(
                        "<div class=\"rounded-lg border border-border bg-card p-4 space-y-2 cursor-pointer hover:bg-surface/60\" onclick=\"if(!event.target.closest('button,a,[popovertarget],[popover]'))document.getElementById('{}').showPopover()\">",
                        html_escape(&menu_id)
                    ),
                    "</div>".to_string(),
                )
            } else if let Some(ref href) = row_href {
                (
                    format!(
                        "<a href=\"{}\" class=\"block rounded-lg border border-border bg-card p-4 space-y-2 hover:bg-surface/60 cursor-pointer\">",
                        html_escape(href)
                    ),
                    "</a>".to_string(),
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
                let cell_text = cell_string(row.get(&col.key));
                html.push_str(&format!(
                    "<div class=\"flex justify-between\"><span class=\"text-xs font-semibold text-text-muted uppercase\">{}</span><span class=\"text-sm text-text\">{}</span></div>",
                    html_escape(&col.label),
                    html_escape(&cell_text)
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
fn resolve_row_key(row: &Value, row_key_prop: Option<&str>, index: usize) -> String {
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
fn template_actions(
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
fn render_inline_dropdown(menu_id: &str, items: &[DropdownMenuAction]) -> String {
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

    #[test]
    fn data_table_row_with_actions_emits_show_popover_onclick() {
        // When row_actions is set, clicking the row (desktop <tr> and mobile card)
        // must open the dropdown via showPopover(), not navigate.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "row_key": "id",
                "columns": [{"key": "name", "label": "Name"}],
                "row_actions": [
                    {"label": "Edit", "action": {"handler": "edit", "url": "/items/{id}/edit", "method": "GET"}}
                ],
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"id": "42", "name": "Foo"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("showPopover()"),
            "rows with actions must open dropdown on click; got: {html}"
        );
        assert!(
            html.contains("dt-42"),
            "popover id must include row key; got: {html}"
        );
        assert!(
            html.contains("dt-m-42"),
            "mobile popover id must include row key; got: {html}"
        );
        assert!(
            !html.contains("window.location.assign"),
            "row with actions must not navigate on click; got: {html}"
        );
    }

    #[test]
    fn data_table_row_href_only_still_navigates() {
        // When only row_href is set (no row_actions), clicking the row navigates.
        let el = mk_element(
            "DataTable",
            json!({
                "data_path": "/items",
                "row_key": "id",
                "columns": [{"key": "name", "label": "Name"}],
                "row_href": "/items/{id}",
            }),
        );
        let spec = mk_spec("root", el.clone());
        let data = json!({"items": [{"id": "5", "name": "Bar"}]});
        let html = render_data_table(&el, &spec, &data, 1);
        assert!(
            html.contains("window.location.assign"),
            "href-only row must still navigate on click; got: {html}"
        );
        assert!(
            !html.contains("showPopover()"),
            "href-only row must not emit showPopover; got: {html}"
        );
    }
}
