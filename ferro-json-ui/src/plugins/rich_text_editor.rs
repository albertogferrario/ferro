//! Asset adapter for the first-class `RichTextEditor` component.
//!
//! `Component::RichTextEditor` is dispatched directly by `render_component`
//! (it is NOT a plugin component in the routing sense). However, it requires
//! Quill 2.0.3 JS and CSS loaded from jsDelivr, SRI-pinned, exactly once per
//! page across multiple editor instances. The cleanest way to deliver that
//! contract without inventing a parallel asset pipeline is to expose Quill's
//! CDN/SRI declarations through the existing `JsonUiPlugin` interface and
//! register them in `global_plugin_registry()` like any other plugin. The
//! plugin's `render()` is unreachable (first-class variant is dispatched
//! first); only `css_assets()` / `js_assets()` are consumed, via
//! `collect_plugin_assets`, when `collect_plugin_types_node` enrolls
//! `"RichTextEditor"` into the per-page plugin-types set (see
//! `render::collect_plugin_types_node`).
//!
//! Conceptual coherence (D-02): first-class components reuse the plugin asset
//! pipeline rather than introducing a parallel CDN path. The surface evolves
//! to absorb the requirement.
//!
//! Bumping Quill is a deliberate phase: update the constants in
//! `crate::assets::quill` (and re-run the SRI computation). This module is
//! mechanical glue and does not need to change on a version bump.

use serde_json::Value;

use crate::assets::quill::{QUILL_CSS_SRI, QUILL_CSS_URL, QUILL_JS_SRI, QUILL_JS_URL};
use crate::component::RichTextEditorProps;
use crate::plugin::{Asset, JsonUiPlugin};

/// Asset-only plugin adapter for `Component::RichTextEditor`.
///
/// `render()` is unreachable in normal operation — it returns an explicit
/// server-side error sentinel string for the unreachable path so any future
/// regression that routes a `Plugin{plugin_type:"RichTextEditor"}` through
/// here produces a debuggable signal rather than silent success.
pub struct RichTextEditorPlugin;

impl JsonUiPlugin for RichTextEditorPlugin {
    fn component_type(&self) -> &str {
        "RichTextEditor"
    }

    fn props_schema(&self) -> Value {
        // Generated from the props derive — keeps the schema in lock-step
        // with the Rust struct without manual maintenance.
        serde_json::to_value(schemars::schema_for!(RichTextEditorProps)).unwrap_or_else(|_| {
            serde_json::json!({
                "type": "object",
                "description": "RichTextEditor props (schema derivation failed at runtime)",
            })
        })
    }

    fn render(&self, _props: &Value, _data: &Value) -> String {
        // Unreachable in normal operation: Component::RichTextEditor is
        // dispatched directly by render_component. If this fires, a future
        // regression routed a Plugin{plugin_type:"RichTextEditor"} through
        // the plugin pipeline — surface it loudly.
        String::from(
            "<div class=\"p-4 bg-red-50 text-red-600 rounded\">\
             RichTextEditorPlugin.render unreachable: Component::RichTextEditor \
             is dispatched directly by render_component. \
             Did a regression route a generic Plugin variant here?\
             </div>",
        )
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![Asset::new(QUILL_CSS_URL)
            .integrity(QUILL_CSS_SRI)
            .crossorigin("anonymous")]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![Asset::new(QUILL_JS_URL)
            .integrity(QUILL_JS_SRI)
            .crossorigin("anonymous")]
    }

    fn init_script(&self) -> Option<String> {
        // The runtime IIFE for RichTextEditor lives in FERRO_RUNTIME_JS
        // (runtime/rich_text_editor.rs, Plan 04) — emitted as part of the
        // single page-wide bundle, not a per-plugin init.
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_type_is_rich_text_editor() {
        let p = RichTextEditorPlugin;
        assert_eq!(p.component_type(), "RichTextEditor");
    }

    #[test]
    fn css_assets_have_sha384_sri_and_anonymous_crossorigin() {
        let p = RichTextEditorPlugin;
        let css = p.css_assets();
        assert_eq!(css.len(), 1, "exactly one CSS asset");
        assert_eq!(css[0].url, QUILL_CSS_URL);
        let integrity = css[0].integrity.as_deref().expect("integrity required");
        assert!(
            integrity.starts_with("sha384-"),
            "integrity must use sha384"
        );
        assert_eq!(css[0].crossorigin.as_deref(), Some("anonymous"));
    }

    #[test]
    fn js_assets_have_sha384_sri_and_anonymous_crossorigin() {
        let p = RichTextEditorPlugin;
        let js = p.js_assets();
        assert_eq!(js.len(), 1, "exactly one JS asset");
        assert_eq!(js[0].url, QUILL_JS_URL);
        let integrity = js[0].integrity.as_deref().expect("integrity required");
        assert!(
            integrity.starts_with("sha384-"),
            "integrity must use sha384"
        );
        assert_eq!(js[0].crossorigin.as_deref(), Some("anonymous"));
    }

    #[test]
    fn init_script_is_none() {
        let p = RichTextEditorPlugin;
        assert!(p.init_script().is_none());
    }

    #[test]
    fn props_schema_describes_rich_text_editor() {
        let p = RichTextEditorPlugin;
        let schema = p.props_schema();
        // schema must reference the RichTextEditorProps shape (specifically
        // the `name` and `formats` fields). Assert on substrings of the
        // serialized form since the exact schemars output structure varies.
        let s = serde_json::to_string(&schema).expect("schema must serialize");
        assert!(
            s.contains("name"),
            "schema must reference the name field: {s}"
        );
        assert!(
            s.contains("formats"),
            "schema must reference the formats field: {s}"
        );
    }
}
