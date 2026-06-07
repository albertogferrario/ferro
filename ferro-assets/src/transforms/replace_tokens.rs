//! Raw-bytes token substitution transform.
//!
//! Substitutes `%%TOKEN%%`-style placeholders in asset bytes via literal
//! find-and-replace. Applies to text-bearing content types only: [`ContentType::Html`],
//! [`ContentType::Css`], [`ContentType::Js`], and [`ContentType::Other`] (D-16).
//! Binary image types ([`ContentType::Avif`], [`ContentType::Jpeg`], [`ContentType::Png`])
//! are passed through unchanged — raw-byte substitution on compressed image data risks
//! silent corruption if a token pattern appears in the compressed bitstream.
//!
//! # Security note
//!
//! Substitution is literal raw-byte replacement with no eval, no recursive
//! expansion, and no regex. The **caller** is responsible for sanitizing
//! replacement values before passing them in. `ferro-assets` does not sanitize.
//!
//! [`ContentType`]: crate::ContentType
//! [`ContentType::Html`]: crate::ContentType::Html
//! [`ContentType::Css`]: crate::ContentType::Css
//! [`ContentType::Js`]: crate::ContentType::Js
//! [`ContentType::Other`]: crate::ContentType::Other
//! [`ContentType::Avif`]: crate::ContentType::Avif
//! [`ContentType::Jpeg`]: crate::ContentType::Jpeg
//! [`ContentType::Png`]: crate::ContentType::Png

use std::collections::HashMap;

use bytes::Bytes;

use crate::asset::ContentType;
use crate::{Asset, Error};

/// Substitute `%%TOKEN%%`-style placeholders in text-bearing assets.
///
/// The `map` associates token strings (e.g. `"%%API_KEY%%"`) to their
/// replacement values. Substitution is performed via literal raw-byte
/// find-and-replace on [`ContentType::Html`], [`ContentType::Css`],
/// [`ContentType::Js`], and [`ContentType::Other`] assets — tokens can
/// appear in HTML attributes, inline `<script>` bodies, text nodes, JSON, and
/// any other text file format (D-16).
///
/// Binary image types ([`ContentType::Avif`], [`ContentType::Jpeg`],
/// [`ContentType::Png`]) pass through unchanged. Raw-byte substitution on
/// compressed image data risks silent corruption if the token byte sequence
/// coincidentally appears in the compressed bitstream.
///
/// [`ContentType::Html`]: crate::ContentType::Html
/// [`ContentType::Css`]: crate::ContentType::Css
/// [`ContentType::Js`]: crate::ContentType::Js
/// [`ContentType::Other`]: crate::ContentType::Other
/// [`ContentType::Avif`]: crate::ContentType::Avif
/// [`ContentType::Jpeg`]: crate::ContentType::Jpeg
/// [`ContentType::Png`]: crate::ContentType::Png
#[derive(Debug, Clone)]
pub struct ReplaceTokens {
    map: HashMap<String, String>,
}

impl ReplaceTokens {
    /// Create a new `ReplaceTokens` transform from a token map.
    ///
    /// Keys are the token strings (e.g. `"%%API_KEY%%"`); values are their
    /// replacements. Caller is responsible for sanitizing replacement values.
    pub fn new(map: HashMap<String, String>) -> Self {
        Self { map }
    }
}

impl crate::pipeline::Transform for ReplaceTokens {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        assets
            .into_iter()
            .map(|a| {
                // Skip binary image types — raw-byte substitution would corrupt
                // compressed data if a token pattern appears in the bitstream.
                if matches!(
                    a.content_type,
                    ContentType::Jpeg | ContentType::Png | ContentType::Avif
                ) {
                    return Ok(a);
                }
                let mut bytes = a.bytes.to_vec();
                for (token, replacement) in &self.map {
                    bytes = replace_bytes(&bytes, token.as_bytes(), replacement.as_bytes());
                }
                Ok(Asset {
                    bytes: Bytes::from(bytes),
                    ..a
                })
            })
            .collect::<Result<Vec<_>, Error>>()
    }
}

/// Linear-scan byte find-and-replace.
///
/// Replaces every non-overlapping occurrence of `needle` in `haystack` with
/// `replacement`. Literal substitution only — no regex, no eval, no recursion.
fn replace_bytes(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    if needle.is_empty() {
        return haystack.to_vec();
    }

    let mut result = Vec::with_capacity(haystack.len());
    let mut i = 0;

    while i < haystack.len() {
        if haystack[i..].starts_with(needle) {
            result.extend_from_slice(replacement);
            i += needle.len();
        } else {
            result.push(haystack[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, Pipeline};

    fn make_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn replaces_token_in_html_attribute() {
        let html = br#"<script src="%%CDN_URL%%/sdk.js"></script>"#;
        let map = make_map(&[("%%CDN_URL%%", "https://cdn.example.com")]);

        let assets = vec![Asset::new("index.html", Bytes::from_static(html))];
        let result = Pipeline::new()
            .add(ReplaceTokens::new(map))
            .run(assets)
            .expect("must succeed");

        let out = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        assert!(
            out.contains("https://cdn.example.com/sdk.js"),
            "token in attribute must be replaced: {out}"
        );
        assert!(!out.contains("%%CDN_URL%%"), "token must not remain: {out}");
    }

    #[test]
    fn replaces_token_in_inline_script() {
        let html = b"<script>var key = '%%API_KEY%%';</script>";
        let map = make_map(&[("%%API_KEY%%", "secret123")]);

        let assets = vec![Asset::new("page.html", Bytes::from(html.to_vec()))];
        let result = Pipeline::new()
            .add(ReplaceTokens::new(map))
            .run(assets)
            .expect("must succeed");

        let out = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        assert!(
            out.contains("secret123"),
            "token in script must be replaced: {out}"
        );
        assert!(!out.contains("%%API_KEY%%"), "token must not remain: {out}");
    }

    #[test]
    fn replaces_token_in_other_content_type() {
        // ContentType::Other (JSON) — must still be processed.
        let json = br#"{"endpoint":"%%BASE_URL%%/api"}"#;
        let map = make_map(&[("%%BASE_URL%%", "https://app.example.com")]);

        let assets = vec![Asset::new("config.json", Bytes::from_static(json))];
        let result = Pipeline::new()
            .add(ReplaceTokens::new(map))
            .run(assets)
            .expect("must succeed");

        let out = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        assert!(
            out.contains("https://app.example.com/api"),
            "token in JSON (Other type) must be replaced: {out}"
        );
    }

    #[test]
    fn empty_map_passes_through_unchanged() {
        let html = b"<p>%%TOKEN%%</p>";
        let assets = vec![Asset::new("index.html", Bytes::from_static(html))];
        let result = Pipeline::new()
            .add(ReplaceTokens::new(HashMap::new()))
            .run(assets)
            .expect("must succeed");
        assert_eq!(result[0].bytes.as_ref(), html);
    }

    #[test]
    fn multiple_tokens_all_replaced() {
        let text = b"Hello %%FIRST%% and %%SECOND%%";
        let map = make_map(&[("%%FIRST%%", "Alice"), ("%%SECOND%%", "Bob")]);

        let assets = vec![Asset::new("msg.txt", Bytes::from_static(text))];
        let result = Pipeline::new()
            .add(ReplaceTokens::new(map))
            .run(assets)
            .expect("must succeed");

        let out = std::str::from_utf8(&result[0].bytes).expect("valid utf8");
        assert!(out.contains("Alice"), "first token must be replaced: {out}");
        assert!(out.contains("Bob"), "second token must be replaced: {out}");
    }
}
