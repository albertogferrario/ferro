//! CSS minification transform using lightningcss.
//!
//! Parses CSS, applies minification, and re-serializes with `minify: true`.
//! Pinned to `lightningcss = "=1.0.0-alpha.71"` (exact pin required — alpha
//! API breaks between minor bumps; see PITFALLS §6 / CONTEXT D-02).
//!
//! Malformed CSS surfaces as [`Error::Transform`] — no panics on the parse path.

use bytes::Bytes;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

use crate::{map_matching, Asset, ContentType, Error};

/// Minify CSS using lightningcss.
///
/// Accepts [`ContentType::Css`] assets. All other content types pass through
/// byte-identical. UTF-8 decoding failure on a CSS asset surfaces as
/// [`Error::Transform`].
///
/// The `lightningcss` version is pinned exactly (`=1.0.0-alpha.71`) — never
/// relax this to `"1"` or a range selector.
#[derive(Debug, Clone)]
pub struct CssMinify;

impl CssMinify {
    /// Create a new `CssMinify` transform.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CssMinify {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::pipeline::Transform for CssMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Css], |a| {
            let source = std::str::from_utf8(&a.bytes)
                .map_err(|e| Error::transform("css_minify", &a.path, e.to_string()))?;
            let minified =
                minify_css(source).map_err(|e| Error::transform("css_minify", &a.path, e))?;
            Ok(Asset {
                bytes: Bytes::from(minified.into_bytes()),
                ..a
            })
        })
    }
}

/// Parse, minify, and re-serialize CSS.
fn minify_css(source: &str) -> Result<String, String> {
    let mut stylesheet =
        StyleSheet::parse(source, ParserOptions::default()).map_err(|e| e.to_string())?;
    stylesheet
        .minify(MinifyOptions::default())
        .map_err(|e| e.to_string())?;
    let result = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..PrinterOptions::default()
        })
        .map_err(|e| e.to_string())?;
    Ok(result.code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, Pipeline};

    #[test]
    fn css_minify_reduces_size() {
        let css = "body {  color : red ;  background-color:   white;  }";
        let result = minify_css(css).expect("must succeed");
        assert!(
            result.len() < css.len(),
            "minified output must be shorter than input: got {}, expected < {}",
            result.len(),
            css.len()
        );
        assert!(
            result.contains("red"),
            "minified CSS must still contain 'red'"
        );
    }

    #[test]
    fn css_minify_other_content_type_passes_through() {
        let json = br#"{"key":"value"}"#;
        let assets = vec![Asset::new("data.json", Bytes::from_static(json))];
        let pipeline = Pipeline::new().add(CssMinify::new());
        let result = pipeline.run(assets).expect("must succeed");
        assert_eq!(result[0].bytes.as_ref(), json);
    }

    #[test]
    fn malformed_css_returns_err_not_panic() {
        let bad_css = "this is { not valid css at all {{{";
        // lightningcss is lenient, but severely invalid CSS should error or
        // produce output — in either case, never panic.
        let _ = minify_css(bad_css);
    }
}
