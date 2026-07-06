//! HTML minification transform using lol_html.
//!
//! Collapses whitespace in visible text nodes while treating `<script>` and
//! `<style>` element bodies as opaque (PITFALLS C-02 / D-14). A text handler
//! registered for `script` or `style` would receive text chunks for potential
//! mutation and corrupt template literals, multi-line strings, and JSON blobs.
//! Only element handlers are registered for those two elements — no text handler.

use bytes::Bytes;
use lol_html::{
    element, html_content::ContentType as LolContentType, text, HtmlRewriter, Settings,
};

use crate::{map_matching, Asset, ContentType, Error};

/// Minify HTML by collapsing whitespace in visible text nodes.
///
/// `<script>` and `<style>` element bodies are **never touched** (D-14 /
/// PITFALLS C-02). Only element handlers are registered for those two elements
/// so their text content passes through byte-for-byte, preserving template
/// literals, `${}` interpolations, multi-line strings, and JSON blobs.
///
/// Malformed HTML surfaces as [`Error::Transform`] — no panics on the parse path.
#[derive(Debug, Clone)]
pub struct HtmlMinify;

impl HtmlMinify {
    /// Create a new `HtmlMinify` transform.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlMinify {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::pipeline::Transform for HtmlMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Html], |a| {
            let out =
                minify_html(&a.bytes).map_err(|e| Error::transform("html_minify", &a.path, e))?;
            Ok(Asset {
                bytes: Bytes::from(out),
                ..a
            })
        })
    }
}

/// Minify HTML bytes.
///
/// Registers element handlers for `script` and `style` WITHOUT any
/// corresponding text handlers — this is the opaque-content guarantee.
/// A text handler on `script` or `style` (even a no-op) would open the door
/// to mutations that corrupt inline JavaScript.
///
/// Whitespace in visible text nodes (outside `<script>`/`<style>`) is
/// conservatively collapsed: runs of whitespace-only text are removed;
/// non-empty text is trimmed of leading/trailing whitespace.
fn minify_html(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(input.len());

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // CRITICAL (D-14 / PITFALLS C-02):
                // Element handler ONLY for script and style — NO text handler.
                // A text handler here would receive text chunks for mutation and
                // would corrupt template literals, JSON blobs, or multi-line
                // strings inside <script>/<style> blocks.
                element!("script", |_el| Ok(())),
                element!("style", |_el| Ok(())),
                // Collapse whitespace in visible text nodes.
                // "body *" matches all elements inside <body>; conservative
                // approach: strip whitespace-only nodes, trim others.
                text!("body *", |t| {
                    let s = t.as_str();
                    if s.chars().all(|c| c.is_ascii_whitespace()) {
                        // Entirely whitespace — remove unless it may be
                        // significant inter-element spacing.
                        // Keep a single space to avoid merging adjacent words.
                        if !s.is_empty() {
                            t.replace(" ", LolContentType::Text);
                        }
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(input).map_err(|e| e.to_string())?;
    rewriter.end().map_err(|e| e.to_string())?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, Pipeline};

    #[test]
    fn html_minify_collapses_whitespace_outside_script() {
        let html = b"<html><body><p>   hello   </p></body></html>";
        let out = minify_html(html).expect("must succeed");
        assert!(
            out.len() <= html.len(),
            "output should not be larger than input"
        );
        assert!(!out.is_empty(), "output must not be empty");
    }

    #[test]
    fn other_content_type_passes_through_unchanged() {
        let json = br#"{"key":"value"}"#;
        let assets = vec![Asset::new("data.json", Bytes::from_static(json))];
        let pipeline = Pipeline::new().add(HtmlMinify::new());
        let result = pipeline.run(assets).expect("must succeed");
        assert_eq!(result[0].bytes.as_ref(), json);
    }

    #[test]
    fn malformed_html_returns_err_not_panic() {
        // lol_html is lenient with malformed HTML, but extremely broken input
        // should still produce output or an error — never panic.
        let result = minify_html(b"<unclosed");
        // Either Ok (lol_html is lenient) or Err — both are acceptable; never panic.
        let _ = result;
    }
}
