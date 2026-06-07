//! JavaScript minification transform using the swc umbrella crate.
//!
//! Uses `swc::Compiler::minify` (high-level API) — simpler and more stable
//! than composing the low-level `swc_ecma_minifier` + `swc_ecma_parser` +
//! `swc_ecma_codegen` sub-crates, which each track independent major versions.
//!
//! Pinned to `swc = "66"` (verified 66.0.0 via `cargo search swc` at plan time;
//! see 187-01-SUMMARY.md swc Version section).
//!
//! Malformed JS surfaces as [`Error::Transform`] — no panics on the parse path.

use std::sync::Arc;

use bytes::Bytes;
use swc::{config::JsMinifyOptions, try_with_handler, BoolOrDataConfig, JsMinifyExtras};
use swc_common::{FileName, SourceMap, GLOBALS};

use crate::{map_matching, Asset, ContentType, Error};

/// Minify JavaScript using `swc::Compiler::minify`.
///
/// Accepts [`ContentType::Js`] assets. All other content types pass through
/// byte-identical. Minification applies both compression and mangling.
///
/// Malformed JavaScript (parse errors) is propagated via [`Error::Transform`] —
/// the `try_with_handler` wrapper converts swc parse errors to `Result::Err`.
#[derive(Debug, Clone)]
pub struct JsMinify;

impl JsMinify {
    /// Create a new `JsMinify` transform.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsMinify {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::pipeline::Transform for JsMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Js], |a| {
            let source = std::str::from_utf8(&a.bytes)
                .map_err(|e| Error::transform("js_minify", &a.path, e.to_string()))?;
            let minified = minify_js(source, &a.path)
                .map_err(|e| Error::transform("js_minify", &a.path, e))?;
            Ok(Asset {
                bytes: Bytes::from(minified.into_bytes()),
                ..a
            })
        })
    }
}

/// Parse and minify JavaScript source using swc's high-level API.
///
/// `filename` is used only for error messages — it is not read from disk.
fn minify_js(source: &str, filename: &str) -> Result<String, String> {
    let cm = Arc::<SourceMap>::default();
    let c = swc::Compiler::new(cm.clone());

    let output = GLOBALS
        .set(&Default::default(), || {
            try_with_handler(cm.clone(), Default::default(), |handler| {
                let fm = cm.new_source_file(
                    Arc::new(FileName::Custom(filename.to_string())),
                    source.to_string(),
                );
                c.minify(
                    fm,
                    handler,
                    &JsMinifyOptions {
                        compress: BoolOrDataConfig::from_bool(true),
                        mangle: BoolOrDataConfig::from_bool(true),
                        ..Default::default()
                    },
                    JsMinifyExtras::default(),
                )
            })
        })
        .map_err(|e| format!("{e:?}"))?;

    Ok(output.code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, Pipeline};

    #[test]
    fn js_minify_reduces_size() {
        let js = "function add(a, b) { return a + b; }";
        let result = minify_js(js, "test.js").expect("must succeed");
        assert!(
            result.len() < js.len(),
            "minified output must be shorter than input: got {}, expected < {}",
            result.len(),
            js.len()
        );
    }

    #[test]
    fn js_minify_other_content_type_passes_through() {
        let json = br#"{"key":"value"}"#;
        let assets = vec![Asset::new("data.json", Bytes::from_static(json))];
        let pipeline = Pipeline::new().add(JsMinify::new());
        let result = pipeline.run(assets).expect("must succeed");
        assert_eq!(result[0].bytes.as_ref(), json);
    }

    #[test]
    fn malformed_js_returns_err_not_panic() {
        // Severely malformed JS — expect either an error or lenient output, never panic.
        let bad_js = "function {{{{{ completely invalid";
        let result = minify_js(bad_js, "bad.js");
        // Must return Err (parse error), not panic.
        assert!(
            result.is_err(),
            "malformed JS must return Err, got: {result:?}"
        );
    }
}
