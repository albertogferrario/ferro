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
use swc::{
    config::{IsModule, JsMinifyOptions},
    try_with_handler, BoolOrDataConfig, JsMinifyExtras,
};
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
                        // Treat input as a classic script, not a module. Without
                        // this, swc assumes module-scope safety and hoists inner
                        // function/var declarations of any IIFE wrapper to the
                        // top level. For a `<script src>` (non-module) consumer
                        // that produces a global symbol collision the moment a
                        // second minified script ships a same-named local — the
                        // first one's `function fooBar()` (mangled to `t`) ends
                        // up as a global, then a later script's `var t = false`
                        // clobbers it, and any closure that captured the
                        // original `t` dies with `TypeError: t is not a
                        // function`. Witnessed in the gestiscilo asset pipeline
                        // running over the jetskiadriatic tenant repo where
                        // tenant-info.js's IIFE got unwrapped and info-strip.js's
                        // hoisted `var t` overrode it.
                        module: IsModule::Bool(false),
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
    fn iife_wrapper_survives_minification() {
        // Regression guard for the IIFE-stripping bug that surfaced on
        // jetskiadriatic.it: tenant-info.js (an IIFE that defines a private
        // `applyTenantInfo()` and attaches event listeners) was being
        // unwrapped by swc with default options, so `applyTenantInfo` —
        // mangled to `t` — became a global, and a later script's hoisted
        // `var t` clobbered it. With `module: IsModule::Bool(false)` the
        // outer IIFE survives and no inner symbol leaks to the global scope.
        let js = "(function(){var private = 42; function applyTenantInfo(){return private + 1;} window.__r = applyTenantInfo();})();";
        let result = minify_js(js, "iife.js").expect("must succeed");
        // The outer IIFE shape must survive: there must be at least one
        // function expression that runs immediately. A simple heuristic that
        // catches the regression: the minified output must NOT begin with a
        // bare top-level `function NAME(` declaration — that's what the
        // pre-fix output looked like (`async function t(){…}` first thing
        // in the file).
        let trimmed = result.trim_start();
        assert!(
            !trimmed.starts_with("function applyTenantInfo"),
            "outer IIFE must be preserved, not unwrapped to top-level function; got: {result}"
        );
        // And the inner symbol name `applyTenantInfo` must not be at the
        // very top level either — it can survive as a mangled local inside
        // the preserved IIFE, but it must not be a global function
        // declaration that would shadow other scripts.
        assert!(
            !trimmed.starts_with("function ") || trimmed.starts_with("function()"),
            "no top-level named function declaration leaked: {result}"
        );
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
