//! Structural HTML injection transform using lol_html.
//!
//! Inserts an HTML snippet immediately before a given closing tag. Primary use:
//! inserting an SDK `<script>` tag before `</body>` (D-15 / ASSET-F-04).
//!
//! Only applies to [`ContentType::Html`] assets; all other content types pass
//! through byte-identical via [`map_matching`].

use bytes::Bytes;
use lol_html::{
    element, end_tag, html_content::ContentType as LolContentType, HtmlRewriter, Settings,
};

use crate::{map_matching, Asset, ContentType, Error};

/// Insert an HTML snippet immediately before a specified closing tag.
///
/// The `tag` parameter should be the closing-tag form, e.g. `"</body>"` or
/// `"</head>"`. The tag name is extracted (stripped of `</` and `>`) and used
/// as the CSS selector for the lol_html element handler.
///
/// Only [`ContentType::Html`] assets are processed; others pass through unchanged.
/// Malformed HTML or an unknown tag name surfaces as [`Error::Transform`].
#[derive(Debug, Clone)]
pub struct InjectBeforeTag {
    /// Closing tag form, e.g. `"</body>"`. Used to extract the element selector.
    tag: String,
    /// HTML snippet to insert immediately before the closing tag.
    snippet: String,
}

impl InjectBeforeTag {
    /// Create a new `InjectBeforeTag` transform.
    ///
    /// `tag` should be a closing-tag string such as `"</body>"` or `"</head>"`.
    /// `snippet` is the raw HTML to insert.
    pub fn new(tag: impl Into<String>, snippet: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            snippet: snippet.into(),
        }
    }
}

impl crate::pipeline::Transform for InjectBeforeTag {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let tag = self.tag.clone();
        let snippet = self.snippet.clone();

        map_matching(assets, &[ContentType::Html], move |a| {
            let selector = closing_tag_to_selector(&tag).ok_or_else(|| {
                Error::transform(
                    "inject_before_tag",
                    &a.path,
                    format!("cannot parse tag selector from '{tag}'"),
                )
            })?;

            let snippet_clone = snippet.clone();
            let out = inject_before(&a.bytes, &selector, &snippet_clone)
                .map_err(|e| Error::transform("inject_before_tag", &a.path, e))?;
            Ok(Asset {
                bytes: Bytes::from(out),
                ..a
            })
        })
    }
}

/// Extract the element name from a closing-tag string.
///
/// `"</body>"` → `Some("body")`, `"</head>"` → `Some("head")`.
/// Returns `None` for strings that don't start with `</` and end with `>`.
fn closing_tag_to_selector(tag: &str) -> Option<String> {
    let inner = tag.strip_prefix("</")?;
    let name = inner.strip_suffix('>')?;
    let name = name.trim();
    // Name must be non-empty and contain only valid tag name characters
    // (ASCII alphanumeric or hyphen — no slashes, spaces, or other punctuation).
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        None
    } else {
        Some(name.to_string())
    }
}

/// Use lol_html to insert `snippet` immediately before the closing tag of `selector`.
fn inject_before(input: &[u8], selector: &str, snippet: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(input.len() + snippet.len());
    let snippet = snippet.to_string();

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!(selector, |el| {
                let s = snippet.clone();
                el.on_end_tag(end_tag!(move |end| {
                    end.before(&s, LolContentType::Html);
                    Ok(())
                }))?;
                Ok(())
            })],
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
    fn inject_before_body_close() {
        let html = b"<html><head></head><body><p>Hello</p></body></html>";
        let snippet = r#"<script src="sdk.js"></script>"#;

        let assets = vec![Asset::new("index.html", Bytes::from_static(html))];
        let pipeline = Pipeline::new().add(InjectBeforeTag::new("</body>", snippet));
        let result = pipeline.run(assets).expect("must succeed");

        let out = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        // Snippet must appear immediately before </body>
        assert!(
            out.contains(&format!("{snippet}</body>")),
            "snippet must appear immediately before </body>: {out}"
        );
    }

    #[test]
    fn inject_passes_other_content_type_unchanged() {
        let json = br#"{"key":"value"}"#;
        let assets = vec![Asset::new("data.json", Bytes::from_static(json))];
        let pipeline = Pipeline::new().add(InjectBeforeTag::new("</body>", "<script></script>"));
        let result = pipeline.run(assets).expect("must succeed");
        assert_eq!(result[0].bytes.as_ref(), json);
    }

    #[test]
    fn closing_tag_to_selector_parses_correctly() {
        assert_eq!(closing_tag_to_selector("</body>"), Some("body".to_string()));
        assert_eq!(closing_tag_to_selector("</head>"), Some("head".to_string()));
        assert_eq!(closing_tag_to_selector("<body>"), None);
        assert_eq!(closing_tag_to_selector("<//>"), None);
        assert_eq!(closing_tag_to_selector(""), None);
    }
}
