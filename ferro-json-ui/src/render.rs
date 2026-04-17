//! Phase 115 placeholder renderer.
//!
//! Emits the Spec as pretty-printed JSON inside a `<pre>` block. The real
//! flat-element walker lands in Phase 116. This placeholder exists solely
//! to keep the workspace green during the v1 -> v2 flip.

use serde_json::Value;

use crate::spec::Spec;

/// Plugin-asset bundle returned by `render_spec_to_html_with_plugins`.
///
/// In Phase 115 the placeholder does not walk elements, so `css_head` and
/// `scripts` are always empty. Phase 116 reintroduces real plugin asset
/// collection by walking `spec.elements` against the plugin registry.
pub struct RenderResult {
    pub html: String,
    pub css_head: String,
    pub scripts: String,
}

/// Placeholder renderer. Pretty-prints the Spec inside `<pre><code>`.
pub fn render_spec_to_html(spec: &Spec, _data: &Value) -> String {
    let pretty = serde_json::to_string_pretty(spec)
        .unwrap_or_else(|e| format!("{{\"error\": \"serialize failed: {e}\"}}"));
    let escaped = html_escape(&pretty);
    format!(
        "<!-- ferro-json-ui v2 render pipeline arrives in Phase 116 -->\n\
         <div class=\"ferro-json-ui\" data-spec-version=\"v2\">\n\
         <pre style=\"font-family:monospace;white-space:pre-wrap;\"><code>{escaped}</code></pre>\n\
         </div>"
    )
}

/// Plugin-aware variant. In Phase 115 no plugins are detected because the
/// placeholder does not walk the element graph.
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    RenderResult {
        html: render_spec_to_html(spec, data),
        css_head: String::new(),
        scripts: String::new(),
    }
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Element, Spec};
    use serde_json::json;

    #[test]
    fn placeholder_emits_marker_comment() {
        let spec = Spec::builder()
            .element("root", Element::new("Text").prop("content", "Hi"))
            .build()
            .unwrap();
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(html.contains("ferro-json-ui v2 render pipeline arrives in Phase 116"));
        assert!(html.contains("data-spec-version=\"v2\""));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn placeholder_escapes_html_in_props() {
        let spec = Spec::builder()
            .element("root", Element::new("Text").prop("content", "<script>"))
            .build()
            .unwrap();
        let html = render_spec_to_html(&spec, &json!({}));
        assert!(html.contains("&lt;script&gt;"));
        // The raw `<script>` from props must be escaped, not echoed verbatim.
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn with_plugins_has_empty_asset_fields() {
        let spec = Spec::builder()
            .element("root", Element::new("Text"))
            .build()
            .unwrap();
        let res = render_spec_to_html_with_plugins(&spec, &json!({}));
        assert!(res.css_head.is_empty());
        assert!(res.scripts.is_empty());
        assert!(!res.html.is_empty());
    }
}
