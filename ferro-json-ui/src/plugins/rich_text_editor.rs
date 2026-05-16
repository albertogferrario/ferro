//! Rich text editor plugin for JSON-UI using Quill 2.0.3.
//!
//! Renders an interactive rich text editor backed by Quill. Each editor
//! container stores its field name in `data-ferro-field`; a single init
//! script discovers all `[data-ferro-quill]` elements, attaches Quill,
//! and mirrors HTML to the companion hidden input on every text-change.

use schemars::schema_for;
use serde_json::Value;

use crate::component::RichTextEditorProps;
use crate::data::resolve_path;
use crate::plugin::{Asset, JsonUiPlugin};
use crate::render::html_escape;

/// Quill 2.0.3 CDN asset URLs.
const QUILL_CSS_URL: &str = "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css";
const QUILL_JS_URL: &str = "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js";

// TODO(162-04): Verify SRI hashes from jsdelivr before shipping to production.
// Compute with:
//   curl -s https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css \
//     | openssl dgst -sha384 -binary | openssl base64 -A
//   curl -s https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js \
//     | openssl dgst -sha384 -binary | openssl base64 -A

/// Rich text editor plugin backed by Quill 2.0.3.
///
/// Renders a container div (`<div data-ferro-quill ...>`) and a hidden input
/// that receives the editor HTML on every text-change event. The form handler
/// receives standard `field=<html>` POST data on submit.
///
/// # Security
/// - `field` and `label` values are HTML-escaped when emitted as attributes or
///   label text (T-162-04-03).
/// - CDN assets reference Quill 2.0.3 via jsdelivr. SRI hashes are marked TODO
///   and must be verified before production deployment (T-162-04-02).
/// - The editor produces user-controlled HTML. Sanitization on submit is the
///   consumer's responsibility (T-162-04-01).
pub struct RichTextEditorPlugin;

impl JsonUiPlugin for RichTextEditorPlugin {
    fn component_type(&self) -> &str {
        "RichTextEditor"
    }

    fn props_schema(&self) -> Value {
        serde_json::to_value(schema_for!(RichTextEditorProps)).unwrap_or(Value::Null)
    }

    fn render(&self, props: &Value, data: &Value) -> String {
        let parsed: RichTextEditorProps = match serde_json::from_value(props.clone()) {
            Ok(p) => p,
            Err(e) => {
                return format!(
                    "<!-- ferro-json-ui: failed to decode RichTextEditor props: {e} -->"
                )
            }
        };

        // Resolve initial value: data_path > default_value > empty.
        let initial = parsed
            .data_path
            .as_deref()
            .and_then(|p| resolve_path(data, p))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| parsed.default_value.clone())
            .unwrap_or_default();

        // T-162-04-03: escape field, label, and initial value to prevent XSS via attributes.
        let field_esc = html_escape(&parsed.field);
        let label_esc = html_escape(&parsed.label);
        let initial_esc = html_escape(&initial);

        let mut html = String::new();
        html.push_str(&format!(
            "<label for=\"{field_esc}-editor\" class=\"text-sm font-medium text-text\">{label_esc}</label>"
        ));
        html.push_str(&format!(
            "<div id=\"{field_esc}-editor\" data-ferro-quill data-ferro-field=\"{field_esc}\">{initial_esc}</div>"
        ));
        html.push_str(&format!(
            "<input type=\"hidden\" name=\"{field_esc}\" id=\"{field_esc}-value\" value=\"{initial_esc}\">"
        ));
        if let Some(ref err) = parsed.error {
            html.push_str(&format!(
                "<p class=\"text-sm text-destructive mt-1\">{}</p>",
                html_escape(err)
            ));
        }
        html
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![
            // TODO(162-04): add SRI integrity hash once verified (T-162-04-02).
            Asset::new(QUILL_CSS_URL),
        ]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![
            // TODO(162-04): add SRI integrity hash once verified (T-162-04-02).
            Asset::new(QUILL_JS_URL),
        ]
    }

    fn init_script(&self) -> Option<String> {
        Some(
            r#"(function(){
    if (typeof Quill === 'undefined') return;
    document.querySelectorAll('[data-ferro-quill]').forEach(function(el){
        var field = el.dataset.ferroField;
        var quill = new Quill(el, { theme: 'snow' });
        var input = document.getElementById(field + '-value');
        if (input && input.value) {
            quill.root.innerHTML = input.value;
        }
        quill.on('text-change', function(){
            if (input) input.value = quill.root.innerHTML;
        });
    });
})();"#
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rich_text_editor_plugin_component_type_is_rich_text_editor() {
        let p = RichTextEditorPlugin;
        assert_eq!(p.component_type(), "RichTextEditor");
    }

    #[test]
    fn rich_text_editor_plugin_assets_include_quill_2_0_3() {
        let p = RichTextEditorPlugin;
        let js = p.js_assets();
        let css = p.css_assets();
        assert!(
            js.iter().any(|a| a.url.contains("quill@2.0.3")),
            "js asset must reference quill@2.0.3"
        );
        assert!(
            css.iter().any(|a| a.url.contains("quill@2.0.3")),
            "css asset must reference quill@2.0.3"
        );
    }

    #[test]
    fn rich_text_editor_plugin_init_script_binds_data_ferro_quill() {
        let p = RichTextEditorPlugin;
        let script = p.init_script().expect("init_script returns Some");
        assert!(script.contains("data-ferro-quill"));
        assert!(script.contains("text-change"));
    }

    #[test]
    fn rich_text_editor_plugin_render_emits_container_and_hidden_input() {
        let p = RichTextEditorPlugin;
        let out = p.render(&json!({"field": "bio", "label": "Bio"}), &json!({}));
        assert!(out.contains("data-ferro-quill"), "output must contain data-ferro-quill");
        assert!(out.contains("name=\"bio\""), "output must contain hidden input name");
        assert!(out.contains("id=\"bio-editor\""), "output must contain editor container id");
    }
}
