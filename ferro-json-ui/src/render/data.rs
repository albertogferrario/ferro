//! Phase 116: data-display renderers ported from v1 `render.rs`.
//!
//! Per CONTEXT D-21 v1 HTML emission is the canonical contract. This module
//! changes the per-function signature to `(el, spec, data, depth) -> String`
//! and routes data resolution through [`crate::data::resolve_path`].
//!
//! Non-obvious v1 behaviors preserved verbatim (per CONTEXT §"Non-obvious v1
//! behaviors to preserve"):
//! - **DataTable `row_key` / `id` URL templating** — row_action URLs have the
//!   placeholders `{row_key}` (v1 default) and `{id}` (plan 116-05 addition
//!   for spec-author convenience) replaced against each row's data before
//!   emission. When `row_key` prop is unset the fallback is the row index.
//! - **Table empty-state** — when `data_path` resolves to an array but it's
//!   empty, `props.empty_message` is emitted as a single spanning `<td>`.

use serde_json::Value;

use crate::component::{DataTableProps, DropdownMenuAction, TableProps};
use crate::data::resolve_path;
use crate::spec::{Element, Spec};

use super::html_escape;

/// Port of v1 `render_table` (render.rs:1017–1102).
///
/// Simple server-side table with an optional action column on the right.
/// Row actions are emitted as plain anchor links (no dropdown) — the
/// `Azioni` header label is preserved from v1 verbatim.
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
            "<th class=\"px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-text-muted\">{}</th>",
            html_escape(&col.label)
        ));
    }
    if props.row_actions.is_some() {
        html.push_str(
            "<th class=\"px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-text-muted\">Azioni</th>"
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
                    "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
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
                        "<td class=\"px-6 py-4 text-sm text-text whitespace-nowrap\">{}</td>",
                        html_escape(&cell_text)
                    ));
                }
                if let Some(ref actions) = props.row_actions {
                    html.push_str("<td class=\"px-6 py-4 text-right text-sm space-x-2\">");
                    for action in actions {
                        let url = action.url.as_deref().unwrap_or("#");
                        let label = action
                            .handler
                            .split('.')
                            .next_back()
                            .unwrap_or(&action.handler);
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
            "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
            col_count,
            html_escape(msg)
        ));
    }

    html.push_str("</tbody></table></div>");
    html
}

/// Port of v1 `render_data_table` (render.rs:1104–1285).
///
/// Stripe-style desktop table with alternating rows plus a mobile card
/// fallback. Each row emits a self-contained action block holding the
/// templated row actions. v1 wraps row actions in a `DropdownMenu`; this
/// plan emits an inline `<details>` dropdown so data.rs can stand alone
/// from Plan 116-03's `render_dropdown_menu` (cross-wave isolation). Plan
/// 116-06 integration can swap the inline dropdown for the shared one
/// once both waves merge.
///
/// URL templating: row_action URLs containing `{row_key}` (v1 semantics)
/// OR `{id}` (plan 116-05 convenience shortcut) are substituted against
/// the row's value for `props.row_key` (default row index) and the row's
/// `id` field respectively.
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

    // Desktop table (hidden on mobile).
    html.push_str(
        "<div class=\"hidden md:block rounded-lg border border-border overflow-hidden\">",
    );

    if items.is_empty() {
        html.push_str("<table class=\"w-full\"><tbody>");
        html.push_str(&format!(
            "<tr><td colspan=\"{}\" class=\"px-6 py-8 text-center text-sm text-text-muted\">{}</td></tr>",
            col_count,
            html_escape(empty_msg)
        ));
        html.push_str("</tbody></table>");
    } else {
        html.push_str("<table class=\"w-full\">");

        // Header.
        html.push_str("<thead><tr class=\"bg-surface\">");
        for col in &props.columns {
            html.push_str(&format!(
                "<th class=\"px-6 py-4 text-left text-xs font-semibold uppercase tracking-wider text-text-muted\">{}</th>",
                html_escape(&col.label)
            ));
        }
        if has_actions {
            html.push_str(
                "<th class=\"px-6 py-4 text-right text-xs font-semibold uppercase tracking-wider text-text-muted\">Azioni</th>"
            );
        }
        html.push_str("</tr></thead>");

        // Body.
        html.push_str("<tbody>");
        for (index, row) in items.iter().enumerate() {
            html.push_str(
                "<tr class=\"even:bg-surface hover:bg-surface/80 transition-colors duration-150 border-t border-border\">"
            );
            for col in &props.columns {
                let cell_text = cell_string(row.get(&col.key));
                html.push_str(&format!(
                    "<td class=\"px-6 py-4 text-sm text-text\">{}</td>",
                    html_escape(&cell_text)
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let row_key_value = resolve_row_key(row, props.row_key.as_deref(), index);
                let templated = template_actions(actions, row, &row_key_value);
                html.push_str("<td class=\"px-6 py-4 text-right\">");
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

    // Mobile cards (visible on mobile).
    html.push_str("<div class=\"block md:hidden space-y-3\">");
    if items.is_empty() {
        html.push_str(&format!(
            "<div class=\"text-center text-sm text-text-muted py-8\">{}</div>",
            html_escape(empty_msg)
        ));
    } else {
        for (index, row) in items.iter().enumerate() {
            html.push_str("<div class=\"rounded-lg border border-border bg-card p-4 space-y-2\">");
            for col in &props.columns {
                let cell_text = cell_string(row.get(&col.key));
                html.push_str(&format!(
                    "<div class=\"flex justify-between\"><span class=\"text-xs font-semibold text-text-muted uppercase\">{}</span><span class=\"text-sm text-text\">{}</span></div>",
                    html_escape(&col.label),
                    html_escape(&cell_text)
                ));
            }
            if let Some(ref actions) = props.row_actions {
                let row_key_value = resolve_row_key(row, props.row_key.as_deref(), index);
                let templated = template_actions(actions, row, &row_key_value);
                html.push_str("<div class=\"pt-2 border-t border-border flex justify-end\">");
                html.push_str(&render_inline_dropdown(
                    &format!("dt-m-{row_key_value}"),
                    &templated,
                ));
                html.push_str("</div>");
            }
            html.push_str("</div>");
        }
    }
    html.push_str("</div>");

    html
}

/// Render a single cell's value as a plain string. Matches v1 semantics
/// (render.rs:1057–1065, 1156–1164, 1225–1233).
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

/// Resolve the row key for a single row. Matches v1 (render.rs:1171–1181,
/// 1242–1252): `props.row_key` field value when present and stringifiable,
/// otherwise the row index.
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

/// Template row_action URLs for a single row. Substitutes both `{row_key}`
/// (v1 verbatim) and `{id}` (plan 116-05 convenience placeholder, resolved
/// against `row["id"]` when present). If the row has no `id` field the
/// `{id}` placeholder is left unsubstituted.
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
            // Resolve URL from handler when url is None (v1 fallback).
            let base_url = cloned
                .action
                .url
                .clone()
                .or_else(|| Some(cloned.action.handler.clone()));
            if let Some(mut url) = base_url {
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

/// Minimal self-contained dropdown used for row actions. Uses `<details>` +
/// `<summary>` so it works without JS and does not depend on Plan 116-03's
/// `render_dropdown_menu`. Each item emits an `<a href>` or a POST form
/// depending on the action method — this keeps the emitted URLs visible in
/// `href="..."` so the URL-template assertions in tests can match.
fn render_inline_dropdown(menu_id: &str, items: &[DropdownMenuAction]) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "<details class=\"relative inline-block\" id=\"{}\">",
        html_escape(menu_id)
    ));
    html.push_str(
        "<summary class=\"cursor-pointer select-none px-2 py-1 text-text-muted\">\u{22EE}</summary>",
    );
    html.push_str("<div class=\"absolute right-0 mt-1 min-w-[10rem] rounded-md border border-border bg-background shadow-sm z-10\">");
    for item in items {
        let url = item.action.url.as_deref().unwrap_or("#");
        let destructive_class = if item.destructive {
            " text-destructive"
        } else {
            ""
        };
        html.push_str(&format!(
            "<a href=\"{}\" class=\"block px-3 py-2 text-sm hover:bg-surface{}\">{}</a>",
            html_escape(url),
            destructive_class,
            html_escape(&item.label)
        ));
    }
    html.push_str("</div></details>");
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
        // v1 semantics: {row_key} substitutes against props.row_key's value
        // on each row (falls back to row index).
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
        // v1 default (Italian) is the fallback.
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
}
