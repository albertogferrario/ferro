# Phase 187: ferro-assets — Asset Pipeline Composer — Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 17 (new files for ferro-assets crate + workspace chores)
**Analogs found:** 15 / 17

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-assets/Cargo.toml` | config | — | `ferro-deployments/Cargo.toml` | exact |
| `ferro-assets/src/lib.rs` | library root | — | `ferro-deployments/src/lib.rs` | exact |
| `ferro-assets/src/error.rs` | model/error | — | `ferro-deployments/src/error.rs` | exact |
| `ferro-assets/src/asset.rs` | model | transform | `ferro-bundle/src/lib.rs` (ext_from_content_type + BundleEntry) | role-match |
| `ferro-assets/src/pipeline.rs` | service | batch/transform | `ferro-bundle/src/lib.rs` (serve_inner dispatcher, Result chain) | partial-match |
| `ferro-assets/src/transforms/mod.rs` | utility | — | `ferro-deployments/src/lib.rs` (mod declarations) | role-match |
| `ferro-assets/src/transforms/html_minify.rs` | service | transform | RESEARCH.md Pattern 5 (no codebase analog) | no-analog |
| `ferro-assets/src/transforms/css_minify.rs` | service | transform | RESEARCH.md Pattern 3 (no codebase analog) | no-analog |
| `ferro-assets/src/transforms/js_minify.rs` | service | transform | RESEARCH.md Pattern 4 (no codebase analog) | no-analog |
| `ferro-assets/src/transforms/image_transcode.rs` | service | batch/transform | RESEARCH.md Patterns 6+7 (no codebase analog) | no-analog |
| `ferro-assets/src/transforms/responsive_images.rs` | service | transform | `ferro-assets/src/transforms/html_minify.rs` (lol_html sibling) | sibling |
| `ferro-assets/src/transforms/inject_before_tag.rs` | service | transform | `ferro-assets/src/transforms/html_minify.rs` (lol_html sibling) | sibling |
| `ferro-assets/src/transforms/replace_tokens.rs` | utility | transform | `ferro-bundle/src/lib.rs` (bytes manipulation) | partial-match |
| `ferro-assets/README.md` | doc | — | `ferro-bundle/README.md` / `ferro-deployments/README.md` | exact |
| `docs/src/features/ferro-assets.md` | doc | — | `docs/src/features/deployments.md` | exact |
| Root `Cargo.toml` (members edit) | config | — | `Cargo.toml` lines 3–32 | exact |
| `.github/workflows/publish.yml` (WAVE1A edit) | config | — | publish.yml line 211 `WAVE1A_CRATES` | exact |
| `ferro-assets/tests/passthrough_proof.rs` | test | — | `ferro-bundle/tests/serve_cold.rs` | role-match |
| `ferro-assets/tests/inline_script_fixture.rs` | test | — | `ferro-json-ui/tests/round_trip.rs` (fixture-based regression) | role-match |
| `ferro-assets/tests/image_transcode_test.rs` | test | — | `ferro-bundle/tests/serve_304.rs` (property assertion pattern) | role-match |
| `ferro-assets/tests/all_or_nothing.rs` | test | — | `ferro-bundle/tests/serve_cold.rs` | role-match |

---

## Pattern Assignments

### `ferro-assets/Cargo.toml` (config)

**Analog:** `ferro-deployments/Cargo.toml` (lines 1–11 for metadata block, confirmed exact match to workspace conventions)

**Metadata block pattern** (lines 1–11):
```toml
[package]
name = "ferro-deployments"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Immutable deployment model and atomic promote for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["deployment", "atomic", "storage", "ferro"]
categories = ["web-programming", "database"]
readme = "README.md"
```

**Adapt for ferro-assets:**
```toml
[package]
name = "ferro-assets"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Composable, content-type-aware asset pipeline for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["assets", "pipeline", "minify", "avif", "ferro"]
categories = ["web-programming", "multimedia"]
readme = "README.md"

[dependencies]
lol_html = "2.6"
lightningcss = "=1.0.0-alpha.71"
swc = "66"          # verify exact version with `cargo search swc` at plan time
image = { version = "0.25", features = ["avif"] }
ravif = "0.13"
rayon = "1"
bytes = "1"
thiserror = "2"
tracing = "0.1"
```

**Critical difference from ferro-deployments:** NO `async-trait`, `tokio`, `serde`, `sea-orm` — ferro-assets is sync-only with zero ferro-* deps. This is Wave 1a (zero internal deps), unlike ferro-deployments (Wave 1b, depends on ferro-storage).

**Features block pattern** (ferro-deployments lines 31–34):
```toml
[features]
sqlx-postgres = ["sea-orm/sqlx-postgres"]
postgres-tests = ["sqlx-postgres"]
```

Adapt: If the image transcode test is too slow for default CI, gate it:
```toml
[features]
slow-tests = []
```

---

### `ferro-assets/src/lib.rs` (library root)

**Analog:** `ferro-deployments/src/lib.rs` (all 107 lines)

**Module declaration pattern** (lines 95–107):
```rust
mod config;
pub(crate) mod deployment;
mod error;
mod migration;
pub(crate) mod promote;
mod storage;

pub use config::DeploymentConfig;
pub use deployment::{Deployment, DeploymentStatus, Deployments};
pub use error::Error;
pub use migration::{CreateDeploymentPointersTable, CreateDeploymentsTable};
pub use storage::{preview_url, DeploymentStorage, StorageDeploymentStorage};
```

**Crate-level doc comment pattern** (lines 1–14):
```rust
//! # ferro-deployments
//!
//! Immutable deployment rows and atomic pointer promotion for the Ferro framework.
//!
//! ## Overview
//! ...
//! ## Quick Start
//! ...
```

**Adapt for ferro-assets:**
```rust
//! # ferro-assets
//!
//! Composable, content-type-aware asset pipeline for the Ferro framework.
//!
//! ## Overview
//!
//! A [`Pipeline`] runs over a heterogeneous [`Asset`] set (HTML, CSS, JS, images,
//! and any other files). Each [`Transform`] declares which [`ContentType`]s it
//! accepts; files outside that set pass through byte-for-byte unchanged.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_assets::{Pipeline, Asset, transforms::{HtmlMinify, CssMinify}};
//!
//! let pipeline = Pipeline::new()
//!     .add(HtmlMinify::new())
//!     .add(CssMinify::new());
//!
//! // Run on the async executor via spawn_blocking — pipeline.run() is synchronous.
//! let result = tokio::task::spawn_blocking(move || pipeline.run(assets)).await??;
//! ```

mod asset;
mod error;
mod pipeline;
pub mod transforms;

pub use asset::{Asset, ContentType};
pub use error::Error;
pub use pipeline::Pipeline;
pub use transforms::Transform;
```

---

### `ferro-assets/src/error.rs` (one thiserror Error enum)

**Analog:** `ferro-deployments/src/error.rs` (all 75 lines) — exact structural match

**Thiserror enum pattern** (lines 1–55):
```rust
//! Error types for the deployments system.

use thiserror::Error;

/// Errors that can occur in the deployments system.
#[derive(Debug, Error)]
pub enum Error {
    /// Database error.
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// Deployment is not in the ready state and cannot be promoted.
    #[error("Deployment {id} cannot be promoted: status is not ready")]
    NotReady {
        /// The deployment ID.
        id: i64,
    },

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a custom error.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}
```

**Constructor helper pattern** (lines 57–74):
```rust
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Custom(s.to_string())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}
```

**Adapt for ferro-assets** — replace domain variants with transform context:
```rust
//! Error types for the asset pipeline.

use thiserror::Error;

/// Errors that can occur during asset pipeline processing.
#[derive(Debug, Error)]
pub enum Error {
    /// A transform failed on a specific file.
    #[error("transform '{transform}' failed on '{path}': {cause}")]
    Transform {
        /// The transform that failed (e.g. "html_minify", "image_transcode").
        transform: String,
        /// The logical asset path that caused the failure.
        path: String,
        /// The underlying error message.
        cause: String,
    },

    /// Thread pool or setup error.
    #[error("setup error: {0}")]
    Setup(String),
}

impl Error {
    /// Construct a transform error with full context.
    pub fn transform(transform: impl Into<String>, path: impl Into<String>, cause: impl Into<String>) -> Self {
        Self::Transform {
            transform: transform.into(),
            path: path.into(),
            cause: cause.into(),
        }
    }

    /// Construct a setup error (e.g. rayon pool build failure).
    pub fn setup(cause: impl Into<String>) -> Self {
        Self::Setup(cause.into())
    }
}
```

---

### `ferro-assets/src/asset.rs` (Asset struct, ContentType enum, infer_content_type)

**Analog:** `ferro-bundle/src/lib.rs` — `ext_from_content_type` function (lines 89–106) and `BundleEntry` struct (lines 59–67) for the content-type dispatch table pattern.

**ext_from_content_type dispatch table pattern** (ferro-bundle lines 89–106):
```rust
fn ext_from_content_type(ct: &str) -> &'static str {
    match ct.split(';').next().unwrap_or(ct).trim() {
        "application/javascript" | "text/javascript" => "js",
        "text/css" => "css",
        "text/html" => "html",
        "text/plain" => "txt",
        "application/json" => "json",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        ...
        _ => "",
    }
}
```

**Adapt for ferro-assets** — invert the mapping (path extension → enum):
```rust
use std::path::Path;

/// Content types recognized by the asset pipeline.
///
/// Variants correspond to transform-relevant media types. All other extensions
/// map to [`ContentType::Other`], which passes through every transform unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Html,
    Css,
    Js,
    Jpeg,
    Png,
    Avif,
    /// Catch-all: no transform touches this file; bytes pass through identically.
    Other,
}

/// A single in-memory artifact with a logical path and content-type tag.
#[derive(Debug, Clone)]
pub struct Asset {
    /// Logical artifact path (e.g. `assets/hero.jpg`, `index.html`).
    pub path: String,
    /// Content type, inferred from `path` extension or set explicitly.
    pub content_type: ContentType,
    /// File contents. Uses [`bytes::Bytes`] for cheap clone across transforms.
    pub bytes: bytes::Bytes,
}

impl Asset {
    /// Construct an asset, inferring content type from the path extension.
    pub fn new(path: impl Into<String>, bytes: bytes::Bytes) -> Self {
        let path = path.into();
        let content_type = infer_content_type(&path);
        Self { path, content_type, bytes }
    }

    /// Construct an asset with an explicit content type override.
    pub fn with_content_type(mut self, ct: ContentType) -> Self {
        self.content_type = ct;
        self
    }
}

/// Infer content type from path extension.
///
/// Unknown or absent extensions return [`ContentType::Other`].
pub fn infer_content_type(path: &str) -> ContentType {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("html" | "htm") => ContentType::Html,
        Some("css")          => ContentType::Css,
        Some("js" | "mjs")   => ContentType::Js,
        Some("jpg" | "jpeg") => ContentType::Jpeg,
        Some("png")          => ContentType::Png,
        Some("avif")         => ContentType::Avif,
        _                    => ContentType::Other,
    }
}
```

---

### `ferro-assets/src/pipeline.rs` (Pipeline builder + Transform trait + run())

**Analog:** `ferro-bundle/src/lib.rs` — `serve_inner` dispatcher (lines 228–257) for the single-pass result-chain concept; adapted heavily since ferro-bundle is a registry lookup, not a transform chain.

**Builder pattern reference** (ferro-deployments/src/config.rs lines 29–34):
```rust
pub fn with_preview_domain(mut self, domain: impl Into<String>) -> Self {
    self.preview_domain = Some(domain.into());
    self
}
```

**Core pattern for pipeline.rs** — from RESEARCH.md Pattern 2 and 8 (confirmed correct by CONTEXT.md D-07/D-17):
```rust
use crate::{Asset, Error};

/// A transform operates over the entire asset collection.
///
/// Files whose [`ContentType`] is not in the transform's accepted set must be
/// returned unchanged (byte-identical passthrough — the crate's core guarantee).
pub trait Transform: Send + Sync {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error>;
}

/// Convenience helper for transforms that work file-by-file.
///
/// Files not in `accepted` are passed through with no allocation. The returned
/// iterator short-circuits on the first `Err` — no partial output is emitted.
pub fn map_matching<F>(
    assets: Vec<Asset>,
    accepted: &[crate::ContentType],
    mut f: F,
) -> Result<Vec<Asset>, Error>
where
    F: FnMut(Asset) -> Result<Asset, Error>,
{
    assets.into_iter().map(|a| {
        if accepted.contains(&a.content_type) {
            f(a)
        } else {
            Ok(a)
        }
    }).collect()   // collect::<Result<Vec<_>, _>>() — first Err short-circuits
}

/// Ordered composition of [`Transform`]s over a heterogeneous asset set.
///
/// Transforms are applied in insertion order. Any `Err` from a transform
/// immediately returns — no partial output set is produced.
pub struct Pipeline {
    transforms: Vec<Box<dyn Transform>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { transforms: vec![] }
    }

    /// Add a transform to the end of the chain.
    pub fn add(mut self, t: impl Transform + 'static) -> Self {
        self.transforms.push(Box::new(t));
        self
    }

    /// Run all transforms in order. All-or-nothing: returns `Err` on any failure.
    ///
    /// This is a blocking call. Wrap in `tokio::task::spawn_blocking` when
    /// calling from an async context.
    pub fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let mut current = assets;
        for transform in &self.transforms {
            current = transform.run(current)?;
        }
        Ok(current)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
```

---

### `ferro-assets/src/transforms/mod.rs`

**Analog:** `ferro-deployments/src/lib.rs` module declaration pattern (lines 95–107) — copy the `pub mod` + `pub use` shape.

```rust
// Declare all transform modules and re-export transform structs so consumers
// can write `use ferro_assets::transforms::HtmlMinify` etc.
mod html_minify;
mod css_minify;
mod js_minify;
mod image_transcode;
mod responsive_images;
mod inject_before_tag;
mod replace_tokens;

pub use html_minify::HtmlMinify;
pub use css_minify::CssMinify;
pub use js_minify::JsMinify;
pub use image_transcode::ImageTranscode;
pub use responsive_images::ResponsiveImages;
pub use inject_before_tag::InjectBeforeTag;
pub use replace_tokens::ReplaceTokens;
```

Also re-export `Transform` from `pipeline` so `use ferro_assets::transforms::Transform` works:
```rust
pub use crate::pipeline::Transform;
```

---

### `ferro-assets/src/transforms/html_minify.rs` (lol_html, opaque script/style)

**Analog:** No codebase analog. Use RESEARCH.md Pattern 5 as the reference — it is verified against lol_html 2.6 docs.

**Key pattern from RESEARCH.md (Pattern 5):**
```rust
use lol_html::{element, HtmlRewriter, Settings};

pub struct HtmlMinify;

impl HtmlMinify {
    pub fn new() -> Self { Self }
}

impl Transform for HtmlMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Html], |a| {
            let out = minify_html(&a.bytes)
                .map_err(|e| Error::transform("html_minify", &a.path, e))?;
            Ok(Asset { bytes: bytes::Bytes::from(out), ..a })
        })
    }
}

fn minify_html(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = vec![];
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // CRITICAL (D-14 / PITFALLS C-02):
                // element handler ONLY for script and style — NO text handler.
                // A text handler would receive text chunks for mutation and
                // corrupts template literals / JSON blobs / multi-line strings.
                element!("script", |_el| { Ok(()) }),
                element!("style",  |_el| { Ok(()) }),
                // Add whitespace-collapse text handlers for visible elements here.
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );
    rewriter.write(input).map_err(|e| e.to_string())?;
    rewriter.end().map_err(|e| e.to_string())?;
    Ok(output)
}
```

**Builder shape (from ferro-deployments/src/config.rs):**
```rust
impl HtmlMinify {
    pub fn new() -> Self { Self }
}

impl Default for HtmlMinify {
    fn default() -> Self { Self::new() }
}
```

---

### `ferro-assets/src/transforms/css_minify.rs` (lightningcss)

**Analog:** No codebase analog. Use RESEARCH.md Pattern 3.

**Core lightningcss pattern:**
```rust
use lightningcss::stylesheet::{StyleSheet, ParserOptions, MinifyOptions, PrinterOptions};

pub struct CssMinify;

impl Transform for CssMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Css], |a| {
            let source = std::str::from_utf8(&a.bytes)
                .map_err(|e| Error::transform("css_minify", &a.path, e.to_string()))?;
            let minified = minify_css(source)
                .map_err(|e| Error::transform("css_minify", &a.path, e))?;
            Ok(Asset { bytes: bytes::Bytes::from(minified.into_bytes()), ..a })
        })
    }
}

fn minify_css(source: &str) -> Result<String, String> {
    let mut stylesheet = StyleSheet::parse(source, ParserOptions::default())
        .map_err(|e| e.to_string())?;
    stylesheet.minify(MinifyOptions::default())
        .map_err(|e| e.to_string())?;
    let result = stylesheet.to_css(PrinterOptions { minify: true, ..Default::default() })
        .map_err(|e| e.to_string())?;
    Ok(result.code)
}
```

**Critical version note:** `lightningcss = "=1.0.0-alpha.71"` EXACT pin in Cargo.toml. Never relax to `"1"`.

---

### `ferro-assets/src/transforms/js_minify.rs` (swc umbrella crate)

**Analog:** No codebase analog. Use RESEARCH.md Pattern 4 (verified against swc GitHub example).

**Core swc pattern:**
```rust
use std::sync::Arc;
use swc::{config::JsMinifyOptions, try_with_handler, BoolOrDataConfig, JsMinifyExtras};
use swc_common::{SourceMap, GLOBALS};

pub struct JsMinify;

impl Transform for JsMinify {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Js], |a| {
            let source = std::str::from_utf8(&a.bytes)
                .map_err(|e| Error::transform("js_minify", &a.path, e.to_string()))?;
            let minified = minify_js(source, &a.path)
                .map_err(|e| Error::transform("js_minify", &a.path, e))?;
            Ok(Asset { bytes: bytes::Bytes::from(minified.into_bytes()), ..a })
        })
    }
}

fn minify_js(source: &str, filename: &str) -> Result<String, String> {
    let cm = Arc::<SourceMap>::default();
    let c = swc::Compiler::new(cm.clone());
    let output = GLOBALS.set(&Default::default(), || {
        try_with_handler(cm.clone(), Default::default(), |handler| {
            let fm = cm.new_source_file(
                swc_common::FileName::Custom(filename.into()),
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
    }).map_err(|e| e.to_string())?;
    Ok(output.code)
}
```

**Planning note:** Verify exact `swc` crate version with `cargo search swc` before writing this file. API may have changed from RESEARCH.md's assumed v66.

---

### `ferro-assets/src/transforms/image_transcode.rs` (image + ravif + rayon)

**Analog:** No codebase analog for image processing. Use RESEARCH.md Patterns 6, 7.

**Builder shape** (copy from ferro-deployments/src/config.rs `with_*` pattern):
```rust
pub struct ImageTranscode {
    max_concurrent: usize,       // default 2 (D-09)
    widths: Vec<u32>,            // default [480, 768, 1200, 1920] (D-11)
    avif_quality: f32,           // default 70.0
    avif_speed: u8,              // default 4 (see PITFALLS §3: speed=1 is 30s/image)
    jpeg_quality: u8,            // default 80
}

impl ImageTranscode {
    pub fn new() -> Self {
        Self {
            max_concurrent: 2,
            widths: vec![480, 768, 1200, 1920],
            avif_quality: 70.0,
            avif_speed: 4,
            jpeg_quality: 80,
        }
    }
    pub fn with_max_concurrent(mut self, n: usize) -> Self { self.max_concurrent = n; self }
    pub fn with_widths(mut self, widths: Vec<u32>) -> Self { self.widths = widths; self }
    pub fn with_avif_quality(mut self, q: f32) -> Self { self.avif_quality = q; self }
    pub fn with_avif_speed(mut self, s: u8) -> Self { self.avif_speed = s; self }
    pub fn with_jpeg_quality(mut self, q: u8) -> Self { self.jpeg_quality = q; self }
}
```

**Core rayon-bounded pattern** (RESEARCH.md Pattern 7):
```rust
use rayon::ThreadPoolBuilder;

impl Transform for ImageTranscode {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_concurrent)
            .build()
            .map_err(|e| Error::setup(e.to_string()))?;

        let (images, mut output): (Vec<_>, Vec<_>) = assets
            .into_iter()
            .partition(|a| matches!(a.content_type,
                ContentType::Jpeg | ContentType::Png | ContentType::Avif));

        // pool.install bounds parallelism to max_concurrent threads
        let variants: Result<Vec<Vec<Asset>>, Error> = pool.install(|| {
            images.into_iter()
                  .map(|asset| self.transcode_image(asset))
                  .collect()
        });
        for group in variants? {
            output.extend(group);
        }
        Ok(output)
    }
}
```

**Deterministic naming** (D-12): `{stem}-{width}w.{ext}`, e.g. `hero-768w.avif`. The `responsive_images` transform parses variants back out by matching `{stem}-{n}w.{avif|jpg}` in the asset set.

---

### `ferro-assets/src/transforms/responsive_images.rs` (lol_html img→picture)

**Analog:** `ferro-assets/src/transforms/html_minify.rs` (sibling lol_html transform — same `HtmlRewriter` / `Settings` / `element_content_handlers` setup).

**Core pattern** (RESEARCH.md D-13 + CONTEXT.md D-12):
```rust
// Runs AFTER image_transcode. Discovers AVIF+JPEG variants already in the
// asset set by matching deterministic naming scheme {stem}-{width}w.{ext}.
// Uses lol_html element handler on "img" — NOT on script/style.
element!("img", |el| {
    if let Some(src) = el.get_attribute("src") {
        // look up variants in asset set, build srcset strings
        // emit <picture><source type="image/avif" srcset="..."><img ...></picture>
        el.before("<picture>", ContentType::Html);
        el.after("</picture>", ContentType::Html);
    }
    Ok(())
})
```

---

### `ferro-assets/src/transforms/inject_before_tag.rs` (lol_html structural injection)

**Analog:** `ferro-assets/src/transforms/html_minify.rs` (sibling lol_html transform).

**Core pattern** (RESEARCH.md D-15):
```rust
pub struct InjectBeforeTag {
    tag: String,
    snippet: String,
}

impl InjectBeforeTag {
    pub fn new(tag: impl Into<String>, snippet: impl Into<String>) -> Self {
        Self { tag: tag.into(), snippet: snippet.into() }
    }
}

impl Transform for InjectBeforeTag {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let tag = self.tag.clone();
        let snippet = self.snippet.clone();
        map_matching(assets, &[ContentType::Html], |a| {
            // lol_html element handler: insert snippet immediately before the
            // element matching `tag` (e.g. `</body>` → target = `body` end tag).
            // ...
            Ok(a)
        })
    }
}
```

---

### `ferro-assets/src/transforms/replace_tokens.rs` (raw bytes substitution)

**Analog:** `ferro-bundle/src/lib.rs` — raw bytes operations (lines 134–160 SHA-256 byte processing) for the bytes-manipulation idiom. Structural pattern is simple iteration.

**Core pattern** (RESEARCH.md D-16):
```rust
use std::collections::HashMap;

pub struct ReplaceTokens {
    map: HashMap<String, String>,
}

impl ReplaceTokens {
    pub fn new(map: HashMap<String, String>) -> Self { Self { map } }
}

impl Transform for ReplaceTokens {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        // Accepts ALL ContentType variants — tokens can appear anywhere.
        assets.into_iter().map(|a| {
            let mut bytes = a.bytes.to_vec();
            for (token, replacement) in &self.map {
                // Raw bytes find-and-replace — NOT via lol_html.
                // Safe for tokens in attribute values, inline JS, text nodes.
                let token_bytes = token.as_bytes();
                let replacement_bytes = replacement.as_bytes();
                let mut i = 0;
                let mut result = Vec::with_capacity(bytes.len());
                while i < bytes.len() {
                    if bytes[i..].starts_with(token_bytes) {
                        result.extend_from_slice(replacement_bytes);
                        i += token_bytes.len();
                    } else {
                        result.push(bytes[i]);
                        i += 1;
                    }
                }
                bytes = result;
            }
            Ok(Asset { bytes: bytes::Bytes::from(bytes), ..a })
        }).collect()
    }
}
```

**Note:** `replace_tokens` intentionally does NOT use `map_matching` — it applies to ALL content types, including `ContentType::Other`.

---

### `docs/src/features/ferro-assets.md` (docs page)

**Analog:** `docs/src/features/deployments.md` (all 190 lines) — exact structural model.

**Page structure pattern:**
```markdown
# Deployments

`ferro-deployments` provides [one-line summary].

[2-3 sentence overview paragraph]

## Setup

### [First setup step]

[code block]

## [Core API Section]

[Subsections with code blocks]

## Error Reference

| Variant | Meaning |
|---------|---------|
| `Error::Variant` | Description |
```

**Adapt for ferro-assets** — sections:
1. Overview (passthrough guarantee, zero-C-deps fact from criterion 3)
2. Quick Start (Pipeline composition + spawn_blocking consumer pattern)
3. Asset and ContentType model
4. Built-in Transforms (one subsection each: HtmlMinify, CssMinify, JsMinify, ImageTranscode, ResponsiveImages, InjectBeforeTag, ReplaceTokens)
5. Writing Custom Transforms (Transform trait)
6. Error Reference

---

### `docs/src/SUMMARY.md` (one-line entry addition)

**Analog:** `docs/src/SUMMARY.md` lines 52 (current last Features entry):
```markdown
- [Deployments](features/deployments.md)
```

**Add after Deployments:**
```markdown
- [Asset Pipeline](features/ferro-assets.md)
```

---

### Root `Cargo.toml` (workspace members addition)

**Analog:** `Cargo.toml` lines 3–32 (current members list)

**Current list ends at line 32:**
```toml
    "ferro-bundle",
    "ferro-deployments",
]
```

**Add `"ferro-assets"` as the next entry** (alphabetical order within Wave 1a group is not enforced; append after ferro-deployments):
```toml
    "ferro-bundle",
    "ferro-deployments",
    "ferro-assets",
]
```

---

### `.github/workflows/publish.yml` (WAVE1A_CRATES edit)

**Analog:** `publish.yml` line 211:
```bash
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration"
```

**Append `ferro-assets` to the end of the WAVE1A_CRATES string** (zero ferro-* deps — confirmed by D-01):
```bash
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration ferro-assets"
```

---

## Integration Tests

### `ferro-assets/tests/passthrough_proof.rs` (SC-1)

**Analog:** `ferro-bundle/tests/serve_cold.rs` — doc comment convention, single focused assertion, no test framework deps beyond std.

**Test file structure pattern** (ferro-bundle/tests/serve_cold.rs lines 1–11):
```rust
//! BUNDLE-02 cold path integration test.
//!
//! Verifies that a registered bundle dispatched via `serve_inner` returns:
//! - status 200
//! [bullet list of assertions]
//!
//! Each integration test file is compiled into its own binary by cargo; OS-level
//! process isolation prevents registry leakage to other test files.

use ferro_bundle::Bundle;
```

**Adapt for passthrough_proof.rs:**
```rust
//! SC-1: passthrough proof — a JSON file through the full HTML/CSS/JS/image
//! pipeline exits byte-identical.
//!
//! This test is the artifact-agnostic guarantee: ContentType::Other files
//! are never touched by any built-in transform.

use ferro_assets::{Asset, Pipeline, transforms::{HtmlMinify, CssMinify, JsMinify, ImageTranscode, ResponsiveImages, InjectBeforeTag, ReplaceTokens}};
use bytes::Bytes;

#[test]
fn json_file_passes_through_full_pipeline_unchanged() {
    let json_bytes = Bytes::from_static(br#"{"intent":"browse","fields":[]}"#);
    let assets = vec![Asset::new("spec.json", json_bytes.clone())];

    let pipeline = Pipeline::new()
        .add(HtmlMinify::new())
        .add(CssMinify::new())
        .add(JsMinify::new())
        .add(ImageTranscode::new())
        .add(ResponsiveImages::new())
        .add(InjectBeforeTag::new("</body>", "<script>test</script>"))
        .add(ReplaceTokens::new(std::collections::HashMap::new()));

    let result = pipeline.run(assets).expect("pipeline must succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].bytes, json_bytes, "JSON bytes must be byte-identical after full pipeline");
}
```

---

### `ferro-assets/tests/inline_script_fixture.rs` (SC-2)

**Analog:** `ferro-json-ui/tests/round_trip.rs` — fixture file loading pattern (`fs::read_to_string("tests/fixtures/...")`) and `assert_round_trip` helper style.

**Fixture loading pattern** (ferro-json-ui/tests/round_trip.rs lines 12–15):
```rust
fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|e| panic!("failed to read tests/fixtures/{path}: {e}"))
}
```

**Adapt for inline_script_fixture.rs:**
```rust
//! SC-2: regression fixture — inline <script> with template literals + JSON blob
//! and inline <style> body survive html_minify byte-correct.
//!
//! Fixture: tests/fixtures/inline_script.html
//! Expected: tests/fixtures/inline_script_expected_script.txt
//!           tests/fixtures/inline_script_expected_style.txt

use ferro_assets::{Asset, Pipeline, transforms::HtmlMinify};
use bytes::Bytes;
use std::fs;

fn fixture(path: &str) -> Vec<u8> {
    fs::read(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|e| panic!("failed to read tests/fixtures/{path}: {e}"))
}

#[test]
fn inline_script_body_survives_html_minify_byte_exact() {
    let html = fixture("inline_script.html");
    let expected_script_body = fixture("inline_script_expected_script.txt");
    // ... extract script body from result, assert equality
}
```

**Fixtures to create** in `ferro-assets/tests/fixtures/`:
- `inline_script.html` — HTML page with `<script>` containing template literals, multi-line strings, JSON blob, plus an inline `<style>` block (lifted from real-tenant content per D-14)
- `inline_script_expected_script.txt` — the exact script body bytes that must survive unchanged
- `inline_script_expected_style.txt` — the exact style body bytes that must survive unchanged

---

### `ferro-assets/tests/image_transcode_test.rs` (SC-3)

**Analog:** `ferro-bundle/tests/serve_cold.rs` — property assertion pattern (assert specific output properties, not byte equality).

```rust
//! SC-3: image_transcode emits AVIF + JPEG variants per configured width.
//! Asserts: correct variant count, no upscaling, file naming scheme.

use ferro_assets::{Asset, ContentType, Pipeline, transforms::ImageTranscode};
```

**Optional slow-test gate** (per Phase 185/186 pattern and RESEARCH.md Validation Architecture):
```rust
#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore)]
fn avif_and_jpeg_variants_emitted_for_all_configured_widths() {
    // Heavy encode test — only runs with --features slow-tests
}

#[test]
fn no_upscale_when_source_narrower_than_all_widths() {
    // Lightweight: 400px source, default widths [480,768,1200,1920] → 0 variants
}
```

---

### `ferro-assets/tests/all_or_nothing.rs` (SC-5)

**Analog:** `ferro-bundle/tests/serve_cold.rs` — result assertion pattern.

```rust
//! SC-5: pipeline failure atomicity — an error at mid-pipeline returns Err
//! with no partial output.

#[test]
fn error_mid_pipeline_produces_no_partial_output() {
    // Inject a Transform that always errors on JS assets.
    // Assert that pipeline.run() returns Err, not Ok(partial_vec).
}
```

---

## Shared Patterns

### Builder `with_*` Methods (consuming)

**Source:** `ferro-deployments/src/config.rs` lines 29–34
**Apply to:** `ImageTranscode`, `Pipeline` (`.add()`), `InjectBeforeTag`

```rust
pub fn with_preview_domain(mut self, domain: impl Into<String>) -> Self {
    self.preview_domain = Some(domain.into());
    self
}
```

Rule: consuming `mut self → Self`, prefix `with_`, accept `impl Into<T>` where the field is `String`.

---

### thiserror Error Enum

**Source:** `ferro-deployments/src/error.rs` lines 1–75
**Apply to:** `ferro-assets/src/error.rs`

One `Error` enum per crate, `thiserror::Error` derive, named struct variants with `///` doc on each field, `impl Error { fn constructor() }` helpers for ergonomic construction at call sites.

---

### Module Doc Comment

**Source:** `ferro-deployments/src/error.rs` line 1, `ferro-deployments/src/config.rs` line 1
**Apply to:** every `src/*.rs` and `src/transforms/*.rs` file

Each file opens with a `//! One-line description of what this module provides.` comment.

---

### Integration Test File Header

**Source:** `ferro-bundle/tests/serve_cold.rs` lines 1–11, `ferro-deployments/tests/race_promote_sqlite.rs` lines 1–10
**Apply to:** all four `ferro-assets/tests/*.rs` files

```rust
//! [Short test ID]: [what the test proves — stated as an invariant, not a procedure].
//!
//! [Bullet list of assertions / success criteria covered]
//!
//! Run: `cargo test -p ferro-assets --test [filename]`
```

---

### `Default` impl for zero-field structs

**Source:** `ferro-deployments/src/config.rs` line 6 (`#[derive(Debug, Clone, Default)]`)
**Apply to:** `HtmlMinify`, `CssMinify`, `JsMinify`, `ResponsiveImages`, `Pipeline`

Derive or manually impl `Default` for all stateless/defaultable structs so callers can write `.add(HtmlMinify::default())`.

---

## No Analog Found

Files with no close match in the codebase (planner uses RESEARCH.md patterns):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `ferro-assets/src/transforms/html_minify.rs` | service | transform | No lol_html usage anywhere in the workspace |
| `ferro-assets/src/transforms/css_minify.rs` | service | transform | No lightningcss usage anywhere in the workspace |
| `ferro-assets/src/transforms/js_minify.rs` | service | transform | No swc usage anywhere in the workspace |
| `ferro-assets/src/transforms/image_transcode.rs` | service | batch | No image/ravif/rayon usage in workspace (image 0.25 is in workspace Cargo.lock but not used by any crate directly) |

For these four, RESEARCH.md Patterns 3–7 are the reference. The planner should add a Wave 0 verification task: `cargo search swc` to confirm exact version before writing `js_minify.rs`.

---

## Metadata

**Analog search scope:** `ferro-deployments/`, `ferro-bundle/`, `ferro-json-ui/tests/`, `ferro-cli/tests/`, `docs/src/features/`, root `Cargo.toml`, `.github/workflows/publish.yml`
**Files scanned:** 22
**Pattern extraction date:** 2026-06-07
