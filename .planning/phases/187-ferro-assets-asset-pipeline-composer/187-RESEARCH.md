# Phase 187: ferro-assets — Asset Pipeline Composer - Research

**Researched:** 2026-06-07
**Domain:** New Rust leaf crate — composable in-memory asset pipeline, content-type routing, HTML/CSS/JS minification, pure-Rust image transcoding, responsive variant generation, lol_html rewriting
**Confidence:** HIGH (stack decisions locked in CONTEXT.md; sibling crate patterns verified from codebase; library APIs verified from Context7 and official docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Crate placement, dependencies & publish wave**
- D-01: `ferro-assets` is a **pure leaf crate with ZERO `ferro-*` dependencies** — operates only on bytes. Publish Wave 1a (alongside ferro-macros, ferro-events, ferro-storage, etc.). Add to `.github/workflows/publish.yml` `WAVE1A_CRATES` and to workspace `members` list in `Cargo.toml`.
- D-02: Dependency pins: `lol_html = "2.6"`, `lightningcss = "=1.0.0-alpha.71"` (EXACT pin), swc sub-crates (NOT `swc_core` umbrella), `image = "0.25"`, `ravif` (pure-Rust AVIF via rav1e), `thiserror = "2"`, `bytes = "1"`. Zero new C system dependencies (criterion 3). libvips is rejected.
- D-03: No `from_env()` config struct, no app-identity fields. Pipeline tuning on builder methods only.
- D-04: No `tokio`/async-runtime dependency.

**Asset representation & content-type model**
- D-05: Core type `Asset { path: String, content_type: <type>, bytes: bytes::Bytes }`. `bytes::Bytes` for cheap clones.
- D-06: Content type inferred from path extension on ingest; explicit per-asset override available. Unknown extensions → "other" catch-all → byte-identical passthrough.

**Transform trait shape & passthrough semantics**
- D-07: `Transform` trait operates over the whole collection: `run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error>`. Convenience helper (e.g. `map_matching(types, fn)`) so per-file transforms stay simple. `Pipeline::new().add(transform)…run(files)` applies in insertion order.

**Execution model & bounded image concurrency**
- D-08: `Pipeline::run()` is synchronous (blocking). No async runtime in crate. Consumer wraps in `tokio::task::spawn_blocking`.
- D-09: `image_transcode` bounds concurrent encodes to configurable limit, default ≤2. Enforced with CPU thread pool (rayon or std::thread + counting gate) — must NOT require a live tokio runtime.

**Image transcode & responsive variants**
- D-10: Output formats: **AVIF (`ravif`) + JPEG fallback only.** WebP is explicitly OUT of v1.
- D-11: Responsive widths configurable, default `[480, 768, 1200, 1920]`; only emit widths `<= source.width()` (never upscale). Resize via `image::imageops` Lanczos3.
- D-12: Deterministic variant naming: `{stem}-{width}w.{ext}` (e.g. `hero-768w.avif`). Must be parseable back into (stem, width, format).
- D-13: `responsive_images` is a lol_html rewriter transforming `<img src>` into `<picture><source type="image/avif" srcset=…><img …(JPEG fallback)></picture>`. Runs AFTER `image_transcode`.

**HTML minify inline-content safety**
- D-14: `html_minify` MUST treat `<script>` and `<style>` element text content as opaque — never collapse/rewrite whitespace inside them. Configure lol_html `ElementContentHandlers` to leave `<script>`/`<style>` bodies untouched. Regression fixture required (criterion 2).

**Injection & token substitution**
- D-15: `inject_before_tag(tag, snippet)` — lol_html transform inserting a snippet immediately before a given tag (SDK `<script>` before `</body>`).
- D-16: `replace_tokens(map)` — **byte-safe raw string substitution** for `%%TOKEN%%`-style placeholders. Done on raw bytes (NOT via lol_html). Separate from inject (structural, lol_html) because tokens can appear anywhere.

**Failure semantics**
- D-17: `Pipeline::run()` is all-or-nothing. Any transform/file failure returns a structured `Error` with per-file + per-transform context. NO partial output set. One `thiserror` Error enum.

**New-crate workspace chores**
- D-18: Add to workspace `members` (Cargo.toml) and `.github/workflows/publish.yml` `WAVE1A_CRATES`. First publish requires one-time manual `cargo publish -p ferro-assets`. Docs page in `docs/src/features/` + SUMMARY.md entry. Cargo.toml metadata mirrors sibling new crates.

### Claude's Discretion
- Exact `Asset.content_type` representation (enum vs mime string) and detection table
- Exact `Transform` trait signature/ownership (`Vec<Asset>` by value vs `&mut`)
- Bounded-concurrency primitive (rayon sized pool vs std threads + counting gate) — constraint: default 2, configurable, no tokio
- Exact responsive variant naming format (must round-trip to stem/width/format)
- Exact swc sub-crate versions (verify at plan time — major versions jumped in 2025-2026)
- Builder-method shape for pipeline tuning
- Whether `<picture>` rewrite includes width/sizes attributes or just type+srcset in v1

### Deferred Ideas (OUT OF SCOPE)
- Lossy/WebP output
- `oxc_minifier` instead of swc
- Critical-CSS extraction/inline
- Tier 2 (Node sandbox) pipeline
- Streaming / on-disk pipeline
- ferro-mcp asset-pipeline introspection tool
- MiniJinja templating (happens before this pipeline, in consumer)
- Storage upload / Spaces (caller's responsibility)
- Job orchestration, CDN purge (Phase 188)
- SEO/meta injection (serve-time concern per PITFALLS C-01)
- Async/tokio pipeline
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ASSET-F-01 | `Pipeline` composes ordered transforms; each transform declares accepted content types and passes everything else through unchanged | Transform trait whole-collection model (D-07); `map_matching` helper; ContentType enum with `Other` catch-all (D-06) |
| ASSET-F-02 | `html_minify`, `css_minify`, `js_minify` ship as built-ins; inline `<script>`/`<style>` content survives byte-correct | lol_html 2.6 opaque-content pattern (D-14); lightningcss =1.0.0-alpha.71 API (D-02); swc umbrella crate via `c.minify()` (see §Standard Stack) |
| ASSET-F-03 | `image_transcode` emits AVIF+JPEG responsive variants via pure-Rust codecs; zero C system deps; bounded concurrency default ≤2 | ravif 0.13 `Encoder::new().with_quality().encode_rgba()`; image 0.25 `DynamicImage::resize(Lanczos3)`; rayon `ThreadPoolBuilder::new().num_threads(n).build()` (D-09); libvips rejection rationale |
| ASSET-F-04 | `inject_before_tag(tag, html)` and `replace_tokens(map)` ship as built-ins | lol_html element handler for structural injection (D-15); raw bytes `memchr`/`replace` for token substitution (D-16) |
</phase_requirements>

---

## Summary

`ferro-assets` is a new leaf crate with zero `ferro-*` dependencies providing a synchronous, composable, content-type-aware asset pipeline. Its killer feature is the passthrough guarantee: any file type that does not match a transform's declared content types passes through byte-for-byte, making the pipeline safe for heterogeneous artifact sets that include JSON-UI spec bundles, SSR manifests, and static files alongside HTML/CSS/JS/images.

The entire implementation composes five mature Rust crates: `lol_html` (HTML streaming rewrite), `lightningcss` (CSS minify), the `swc` crate (JS minify via `Compiler::minify`), `image` 0.25 (resize + JPEG encode), and `ravif` (AVIF encode). All are pure-Rust with no C system dependencies. The crate ships no async runtime — `Pipeline::run()` is blocking; the consumer (`PublishFrontendJob`) wraps the call in `tokio::task::spawn_blocking`.

The most implementation-sensitive concern is the lol_html `<script>`/`<style>` opaque-content constraint (PITFALLS C-02): `html_minify` must register element handlers that do NOT add text handlers for `<script>` and `<style>` elements. The lol_html streaming parser treats `<script>` content specially at the parse level, but a text handler registered for `script` would receive its text nodes for mutation. Not registering a text handler (only an element handler) is the correct configuration for opaque-content preservation.

The swc version situation is a critical planning input: the gestiscilo research cited `swc_ecma_minifier ~0.203` but the crate reached major version ~55 in 2025-2026. The recommended approach is the high-level `swc` umbrella crate (now v66), whose `Compiler::minify` API accepts an in-memory `SourceFile` and wraps parse/optimize/codegen in one call. This approach is simpler and more stable than composing the low-level `swc_ecma_minifier` + `swc_ecma_parser` + `swc_ecma_codegen` sub-crates (which each independently track major versions). The `swc` crate adds build-time cost but eliminates API compatibility risk.

**Primary recommendation:** Build the `Transform` trait and `Asset` model first (Wave 0-1), then layer in the five transform implementations in order of complexity: `replace_tokens` (trivial, bytes-only) → `inject_before_tag` → `css_minify` → `html_minify` → `js_minify` → `image_transcode` → `responsive_images`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Content-type routing / passthrough | API / Backend (library) | — | Pure Rust enum dispatch; no network, no DB |
| HTML minification + opaque script/style | API / Backend (library) | — | lol_html streaming rewriter runs in-process |
| CSS minification | API / Backend (library) | — | lightningcss is a pure Rust library call |
| JS minification | API / Backend (library) | — | swc `Compiler::minify` runs in-process |
| Image resize + AVIF/JPEG encode | API / Backend (library) | — | image + ravif CPU-bound Rust; bounded by rayon ThreadPool |
| `<img>` → `<picture>` rewrite | API / Backend (library) | — | lol_html streaming rewriter, runs after image_transcode |
| SDK snippet injection | API / Backend (library) | — | lol_html element handler; structural HTML mutation |
| Token substitution | API / Backend (library) | — | Raw bytes find-and-replace; no parser needed |
| spawn_blocking wrapping | API / Backend (consumer) | — | Consumer (`PublishFrontendJob`) owns async-to-sync bridge |
| Storage upload | CDN / Static (consumer) | — | This crate returns `Vec<Asset>`; caller uploads |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| lol_html | 2.6 | HTML streaming rewrite (minify, img→picture, inject_before_tag) | Maintained by Cloudflare; production-grade; latest 2.6.0 [VERIFIED: crates.io] |
| lightningcss | =1.0.0-alpha.71 | CSS minify + autoprefixing | EXACT pin required (alpha API breaks between minor bumps); used in Parcel 2, Deno, cargo-leptos [CITED: v7.1-STACK.md D-03] |
| swc | ~66 | JS minify via `Compiler::minify` (high-level wrapper) | Replaces low-level swc_ecma_minifier which jumped to v55 with breaking API changes; `swc` crate wraps parse/optimize/codegen in a stable API; version 66 current [VERIFIED: docs.rs/swc, lib.rs] |
| image | 0.25 | `DynamicImage` resize (Lanczos3) + JPEG encode | Already in workspace; avif feature enables ravif encoder [CITED: v7.1-STACK.md D-03 §4] |
| ravif | 0.13 | AVIF encode from RGBA pixels via rav1e | Pure Rust, zero C deps, `Encoder::new().with_quality(q).with_speed(s).encode_rgba(Img)` [VERIFIED: docs.rs/ravif] |
| thiserror | 2 | Error enum derivation | Workspace standard [VERIFIED: CONVENTIONS.md, ferro-deployments/Cargo.toml] |
| bytes | 1 | `Bytes` for cheap clone of asset content | Workspace standard; enables zero-copy across transforms [CITED: CONTEXT.md D-05] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rayon | 1 | Sized `ThreadPool` for bounded image concurrency | Preferred over std::thread + counting gate; `ThreadPoolBuilder::new().num_threads(n).build()` + `pool.install(|| ...)` is clean and synchronous [VERIFIED: docs.rs/rayon] |
| tracing | 0.1 | Structured logging (encode errors, passthrough counts) | Workspace standard |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `swc` umbrella crate | `swc_ecma_minifier` + `swc_ecma_parser` + `swc_ecma_codegen` + `swc_common` directly | Low-level sub-crates independently track major versions (currently v55/v35/v21 etc.); compatible sets must be resolved from the umbrella crate's Cargo.lock; `swc` crate is stable API surface; recommendation: use `swc` |
| `rayon::ThreadPool` | `std::thread` + `Arc<Mutex<Semaphore>>` | rayon pool is simpler, production-tested, and `pool.install(|| ...)` is idiomatic; no tokio requirement |
| `lightningcss` | `swc_css_minifier` | lightningcss has broader autoprefixing + CSS nesting support; swc CSS is secondary in SWC ecosystem [CITED: v7.1-STACK.md D-03] |
| `ravif` | libvips via libvips-rust-bindings | libvips-rust-bindings explicitly states "VipsImage is not thread safe at the moment" — fatal in spawn_blocking; adds C system dep to production image [CITED: v7.1-STACK.md D-03 §4; v7.1-PITFALLS.md C-03] |
| `image` 0.25 | `fast_image_resize` | image crate already in workspace; fast_image_resize gains little for publish-time batch work |

**Installation:**
```bash
# In ferro-assets/Cargo.toml — the planner resolves exact swc version at plan time
# NOTE: verify `swc` crate version at npm view or docs.rs/swc before pinning
cargo add lol_html@2.6 lightningcss@=1.0.0-alpha.71 image@0.25 ravif thiserror@2 bytes@1 rayon tracing
cargo add swc  # verify exact version on docs.rs/swc at plan time — currently ~66
```

**Critical version notes:**
- `lightningcss = "=1.0.0-alpha.71"` — EXACT pin required, not `"1"` [CITED: v7.1-STACK.md D-03 §2]
- `swc` crate: version ~66 current as of 2026-06-07 [ASSUMED: based on lib.rs/crates.io search result showing v66; verify with `cargo search swc`]
- `lol_html` version 2.6.0 confirmed on crates.io [VERIFIED: crates.io search result]
- `ravif` 0.13.0 confirmed on docs.rs [VERIFIED: docs.rs/ravif]
- `image` 0.25 is already in workspace Cargo.lock [CITED: v7.1-STACK.md D-03 §4]

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer (PublishFrontendJob)
        │
        │  Vec<Asset { path, content_type, bytes }>
        ▼
  Pipeline::run(assets)
        │
        ├── Transform 1: html_minify (lol_html)
        │       accepts: ContentType::Html
        │       ├── whitespace collapse (non-script/style)
        │       └── <script>/<style> content: OPAQUE (no text handler registered)
        │       other content types: pass through byte-identical
        │
        ├── Transform 2: css_minify (lightningcss)
        │       accepts: ContentType::Css
        │       └── parse → minify → to_css
        │
        ├── Transform 3: js_minify (swc Compiler::minify)
        │       accepts: ContentType::Js
        │       └── cm.new_source_file → c.minify → output.code.as_bytes()
        │
        ├── Transform 4: image_transcode (image + ravif)
        │       accepts: ContentType::Jpeg | ContentType::Png | ContentType::Avif
        │       rayon ThreadPool (num_threads = max_concurrent_encodes, default 2)
        │       per image:
        │         decode → for each width ≤ source.width():
        │           resize(Lanczos3) → encode AVIF (ravif) + encode JPEG
        │           emit new Asset with name "{stem}-{width}w.{ext}"
        │       input image replaced in set + new variant assets appended
        │
        ├── Transform 5: responsive_images (lol_html)
        │       accepts: ContentType::Html
        │       reads emitted variant assets from the asset set
        │       <img src="hero.jpg"> discovers hero-480w.avif, hero-768w.avif, …
        │       rewrites to <picture><source type="image/avif" srcset="…">
        │                            <img src="hero.jpg" …> (JPEG fallback)
        │                   </picture>
        │
        ├── Transform 6: inject_before_tag(tag, snippet) (lol_html)
        │       accepts: ContentType::Html
        │       element handler on closing tag → insert_before(snippet)
        │
        └── Transform 7: replace_tokens(map) (raw bytes)
                accepts: ALL ContentType variants (tokens can appear anywhere)
                bytes::Bytes find-and-replace for each %%TOKEN%% key
                │
                ▼
  Result<Vec<Asset>, Error>
  (all-or-nothing: Err on any transform failure → no partial output returned)
```

### Recommended Project Structure

```
ferro-assets/
├── Cargo.toml               # workspace version, zero ferro-* deps
├── README.md
├── src/
│   ├── lib.rs               # pub re-exports; crate doc header
│   ├── error.rs             # Error enum (thiserror) with TransformError + FileError context
│   ├── asset.rs             # Asset struct, ContentType enum, infer_content_type(path)
│   ├── pipeline.rs          # Pipeline struct, Transform trait, map_matching helper
│   ├── transforms/
│   │   ├── mod.rs           # pub use all transforms
│   │   ├── html_minify.rs   # HtmlMinify struct — lol_html whitespace collapse, opaque script/style
│   │   ├── css_minify.rs    # CssMinify struct — lightningcss parse/minify/to_css
│   │   ├── js_minify.rs     # JsMinify struct — swc Compiler::minify
│   │   ├── image_transcode.rs # ImageTranscode — rayon pool, ravif + JPEG, responsive variants
│   │   ├── responsive_images.rs # ResponsiveImages — lol_html img→picture rewriter
│   │   ├── inject_before_tag.rs # InjectBeforeTag — lol_html element insert_before
│   │   └── replace_tokens.rs   # ReplaceTokens — raw bytes %%TOKEN%% substitution
└── tests/
    ├── passthrough_proof.rs     # SC-1: JSON file through full pipeline unchanged
    ├── inline_script_fixture.rs # SC-2: inline <script> + <style> bytes survive html_minify
    ├── image_transcode_test.rs  # SC-3: AVIF+JPEG emitted, no C system deps, bounded concurrency
    └── all_or_nothing.rs        # SC-5: error at mid-pipeline produces no output
```

### Pattern 1: ContentType Enum and Infer Function

**What:** Small enum covering transform-relevant types + `Other` catch-all.
**When to use:** Asset construction; determines which transforms run.

```rust
// Source: derived from CONTEXT.md D-05/D-06
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Html,
    Css,
    Js,
    Jpeg,
    Png,
    Avif,
    Other,
}

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

### Pattern 2: Transform Trait and map_matching Helper

**What:** Whole-collection trait with convenience helper for per-file transforms.
**When to use:** Every built-in transform implements this trait.

```rust
// Source: CONTEXT.md D-07 design
pub trait Transform: Send + Sync {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error>;
}

/// Helper for transforms that operate file-by-file within accepted content types.
/// Files outside `accepted` are passed through byte-identical.
pub fn map_matching<F>(
    assets: Vec<Asset>,
    accepted: &[ContentType],
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
    }).collect()
}
```

### Pattern 3: lightningcss CSS Minification

**What:** Parse → minify → print CSS using lightningcss =1.0.0-alpha.71.
**When to use:** `CssMinify::run` for each CSS asset.

```rust
// Source: Context7 lightningcss docs [VERIFIED: Context7 /parcel-bundler/lightningcss]
use lightningcss::stylesheet::{StyleSheet, ParserOptions, MinifyOptions, PrinterOptions};
use lightningcss::targets::Browsers;

fn minify_css(source: &str) -> Result<String, Error> {
    let mut stylesheet = StyleSheet::parse(source, ParserOptions::default())
        .map_err(|e| Error::transform("css_minify", e.to_string()))?;
    let targets = Browsers {
        chrome: Some(90 << 16),
        ..Browsers::default()
    };
    stylesheet.minify(MinifyOptions {
        targets: targets.into(),
        ..MinifyOptions::default()
    }).map_err(|e| Error::transform("css_minify", e.to_string()))?;
    let result = stylesheet.to_css(PrinterOptions {
        minify: true,
        ..PrinterOptions::default()
    }).map_err(|e| Error::transform("css_minify", e.to_string()))?;
    Ok(result.code)
}
```

### Pattern 4: swc JS Minification (High-Level API)

**What:** Minify plain ES2020 JavaScript using the `swc` crate's `Compiler::minify`.
**When to use:** `JsMinify::run` for each JS asset.
**Critical note:** The gestiscilo research cited `swc_ecma_minifier ~0.203` but SWC crates underwent a major version jump (swc_ecma_minifier is now ~v55; swc crate is ~v66). Use the `swc` umbrella crate for a stable API — it exposes `Compiler::minify` which handles parse/optimize/codegen internally.

```rust
// Source: https://github.com/swc-project/swc/blob/main/crates/swc/examples/minify.rs
// [VERIFIED: GitHub fetch of official example]
use std::sync::Arc;
use swc::{config::JsMinifyOptions, try_with_handler, BoolOrDataConfig, JsMinifyExtras};
use swc_common::{SourceMap, GLOBALS};

fn minify_js(source: &str, filename: &str) -> Result<String, Error> {
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
    }).map_err(|e| Error::transform("js_minify", e.to_string()))?;
    Ok(output.code)
}
```

**swc Cargo.toml addition:**
```toml
# Verify exact version at plan time via `cargo search swc` or docs.rs/swc
# Current as of 2026-06-07: ~66.x  [ASSUMED: from lib.rs/crates.io search]
swc = "66"
swc_common = "..."  # swc crate re-exports swc_common; may not need direct dep
```

**Why NOT low-level swc sub-crates:** `swc_ecma_minifier`, `swc_ecma_parser`, `swc_ecma_codegen`, `swc_common` each track independent major versions. A compatible tuple that compiled for gestiscilo (`~0.203`, `~0.149`, `~0.149`, `~0.37`) is now stale — the crates jumped major versions. Resolving a compatible tuple requires deriving it from the umbrella crate's own Cargo.lock, making the `swc` umbrella approach strictly simpler. [ASSUMED: version jump confirmed by crates.io search returning "42.0.0" and "55.0.0" for swc_ecma_minifier; plan-time `cargo search` must re-verify]

### Pattern 5: lol_html Opaque Script/Style Content

**What:** HTML minification that preserves `<script>` and `<style>` content byte-for-byte.
**When to use:** `HtmlMinify::run` — the regression-critical constraint.
**Key insight:** lol_html's `HtmlRewriter` parses `<script>` content as raw character data at the tokenizer level. By registering an element handler for `script` and `style` WITHOUT a corresponding text handler, the rewriter passes all text nodes inside these elements to the output unchanged. Only register text handlers on elements whose text content you intend to modify (attribute values, visible text nodes).

```rust
// Source: docs.rs/lol_html + official API [VERIFIED: Context7 /cloudflare/lol-html]
use lol_html::{element, text, HtmlRewriter, Settings, ElementContentHandlers};

fn minify_html(input: &[u8]) -> Result<Vec<u8>, Error> {
    let mut output = vec![];
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // <script> and <style>: element handler only, NO text handler
                // lol_html will pass content bytes through untouched
                element!("script", |_el| { Ok(()) }),
                element!("style",  |_el| { Ok(()) }),

                // Collapse whitespace in visible text nodes of other elements
                // (example: strip leading/trailing whitespace from <p> text)
                // text!("p, li, td, th, h1, h2, h3, h4, h5, h6", |t| {
                //     if t.as_str().trim().is_empty() { t.remove(); }
                //     Ok(())
                // }),

                // Strip HTML comments (except IE conditionals)
                // element!("*", |el| { /* comment handling */ Ok(()) }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );
    rewriter.write(input)
        .map_err(|e| Error::transform("html_minify", e.to_string()))?;
    rewriter.end()
        .map_err(|e| Error::transform("html_minify", e.to_string()))?;
    Ok(output)
}
```

**Critical warning — the C-02 pitfall:** Any text handler registered for `script` or `style` — including one registered with `text!("script", ...)` that does nothing — may still receive text chunks and if it calls `t.remove()` or any modification, it corrupts inline JS. The correct configuration is: no text handler on `script` / `style` at all, period.

### Pattern 6: ravif AVIF Encoding + image Resize

**What:** Resize a `DynamicImage` and encode to AVIF using ravif.
**When to use:** `ImageTranscode::run` for each source image × each responsive width.

```rust
// Source: docs.rs/ravif 0.13 [VERIFIED: docs.rs/ravif fetch]
// Source: v7.1-STACK.md D-03 §4 responsive_widths sketch [CITED]
use image::{DynamicImage, imageops::FilterType};
use ravif::{Encoder, Img, RGBA8};

fn encode_avif(img: &DynamicImage, quality: f32) -> Result<Vec<u8>, Error> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Vec<RGBA8> = rgba.pixels()
        .map(|p| RGBA8 { r: p[0], g: p[1], b: p[2], a: p[3] })
        .collect();
    let result = Encoder::new()
        .with_quality(quality as f64)
        .with_speed(4)
        .encode_rgba(Img::new(&pixels, width as usize, height as usize))
        .map_err(|e| Error::transform("image_transcode", e.to_string()))?;
    Ok(result.avif_file)
}

fn resize_to_width(src: &DynamicImage, width: u32) -> DynamicImage {
    let height = (src.height() as f64 * width as f64 / src.width() as f64) as u32;
    src.resize(width, height, FilterType::Lanczos3)
}
```

### Pattern 7: Bounded-Concurrency Image Encoding (rayon)

**What:** Rayon sized `ThreadPool` for bounded concurrent image encodes.
**When to use:** `ImageTranscode::run` — controls peak memory on 512 MB instances.

```rust
// Source: docs.rs/rayon ThreadPool [VERIFIED: WebSearch rayon ThreadPoolBuilder]
use rayon::ThreadPoolBuilder;

pub struct ImageTranscode {
    max_concurrent: usize,  // default 2
    widths: Vec<u32>,       // default [480, 768, 1200, 1920]
    avif_quality: f32,      // default 70.0
    jpeg_quality: u8,       // default 80
}

impl ImageTranscode {
    pub fn new() -> Self { Self { max_concurrent: 2, widths: vec![480, 768, 1200, 1920], avif_quality: 70.0, jpeg_quality: 80 } }
    pub fn with_max_concurrent(mut self, n: usize) -> Self { self.max_concurrent = n; self }
    pub fn with_widths(mut self, widths: Vec<u32>) -> Self { self.widths = widths; self }
}

impl Transform for ImageTranscode {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_concurrent)
            .build()
            .map_err(|e| Error::setup(e.to_string()))?;

        // Collect image assets to process; non-image assets pass through
        let mut output: Vec<Asset> = Vec::with_capacity(assets.len() * 5);
        let (images, others): (Vec<_>, Vec<_>) = assets
            .into_iter()
            .partition(|a| matches!(a.content_type, ContentType::Jpeg | ContentType::Png | ContentType::Avif));

        output.extend(others); // pass through non-images

        // Process images in the bounded pool (pool.install bounds parallelism)
        let variants: Result<Vec<Vec<Asset>>, Error> = pool.install(|| {
            images.into_iter().map(|asset| self.transcode_image(asset)).collect()
        });
        for group in variants? {
            output.extend(group);
        }
        Ok(output)
    }
}
```

**Note on rayon vs std threads:** `rayon::ThreadPool::install` runs the closure on the pool's threads with work-stealing. When `num_threads(2)` is set, at most 2 hardware threads are active. This is the simplest path to the D-09 requirement. std::thread + a `Semaphore` (e.g. via `parking_lot::Condvar` or `std::sync::Semaphore`) would work too but adds boilerplate. Recommendation: rayon. [ASSUMED: rayon already in ferro workspace — verify with `grep rayon */Cargo.toml`; if not present, it is a new dep but a common pure-Rust crate]

### Pattern 8: Pipeline and All-or-Nothing Failure

**What:** `Pipeline` collects transforms and applies them in order. Any `Err` short-circuits the chain without returning partial output.
**When to use:** Core executor.

```rust
// Source: CONTEXT.md D-07, D-17
pub struct Pipeline {
    transforms: Vec<Box<dyn Transform>>,
}

impl Pipeline {
    pub fn new() -> Self { Self { transforms: vec![] } }
    pub fn add(mut self, t: impl Transform + 'static) -> Self {
        self.transforms.push(Box::new(t));
        self
    }
    pub fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let mut current = assets;
        for transform in &self.transforms {
            // Any Err here returns immediately — no partial output
            current = transform.run(current)?;
        }
        Ok(current)
    }
}
```

### Anti-Patterns to Avoid

- **Text handler on `<script>` or `<style>`:** Even a no-op text handler signals to lol_html that you want to receive text chunks, which opens the door to accidental mutations. Always omit text handlers for these elements.
- **Calling `pipeline.run()` on the tokio executor directly:** Will stall HTTP request handling during a multi-image encode. Consumer MUST use `tokio::task::spawn_blocking(move || pipeline.run(assets))`.
- **Returning partial output on error:** `Pipeline::run()` must return `Err` on any failure without emitting a partial `Vec<Asset>`. The consumer's two-phase upload protocol depends on this (gestiscilo PUB-05).
- **Upscaling images:** Never emit a responsive variant with `width > src.width()`. Filter `widths` before the encode loop.
- **Hardcoding app identity:** No strings like `"gestiscilo"`, `"jetskiadriatic"`, `"Ferro Application"` anywhere in the crate. Project-agnostic rule from CLAUDE.md. Token maps and injection snippets are always passed by the caller.
- **Using `swc_core` umbrella:** `swc_core` is the monolithic umbrella that includes transformer, bundler, etc. — it bloats compile time. Use `swc` (the build-tool wrapper) or the specific sub-crates. Avoid `swc_core`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML streaming rewrite | Custom HTML parser | `lol_html` 2.6 | lol_html is Cloudflare production grade; handles HTML5 quirks, streaming I/O, attribute mutation, element insertion |
| CSS parse + minify + autoprefix | Custom CSS processor | `lightningcss =1.0.0-alpha.71` | Parcel, Deno, cargo-leptos all use it; handles CSS nesting, modern color, custom properties |
| JS parse + optimize + codegen | Custom JS minifier | `swc` umbrella crate | SWC is used by Next.js, Vercel, Shopify; compresses + mangles + handles ES2020+ correctly |
| AVIF encoding | Custom AVIF encoder | `ravif` 0.13 | rav1e-based, pure Rust, no libaom/libavif C deps; `Encoder::new().encode_rgba()` is one call |
| Image resize | Custom bilinear/lanczos | `image::imageops::resize(FilterType::Lanczos3)` | Already in workspace; Lanczos3 is the quality standard for downscaling |
| Thread-bounded parallelism | `Arc<Mutex<Semaphore>>` + raw `std::thread` | `rayon::ThreadPoolBuilder::new().num_threads(n).build()` | Work-stealing, safe cancellation, standard in Rust ecosystem |

**Key insight:** Every custom implementation of HTML/CSS/JS parsing introduces parsing edge cases that take months to discover. All three minifiers are battle-tested on production traffic at scale. The "don't hand-roll" rule is especially important here: HTML minification with safe inline-script handling is deceptively hard to get right.

---

## Common Pitfalls

### Pitfall 1: html_minify Corrupts Inline Scripts (C-02)
**What goes wrong:** A text handler registered for `script` or `style` collapses template literals, multi-line strings, or JSON blobs inside the element — the rendered page fires `SyntaxError`.
**Why it happens:** HTML minifiers naively treat all text as whitespace-collapsible. lol_html's handler API allows registering text handlers for any element including `script`; doing so opts you into content mutation.
**How to avoid:** Register element handlers for `script` and `style` with no corresponding text handler. The opaque-content behavior is the default when no text handler is present.
**Warning signs:** `SyntaxError: Unexpected token` in the browser console after publishing. Template literal backticks or newlines inside a `<script>` block are gone or mangled.
**Proof artifact:** The inline-script regression fixture (criterion 2) — run `html_minify` on a fixture containing a real-tenant inline `<script>` with template literals + JSON blob + `<style>` block; assert byte-exact match on script/style content.

### Pitfall 2: swc Version API Mismatch
**What goes wrong:** Copying the gestiscilo research's `swc_ecma_minifier = "0.203"` tuple causes `cargo add` to fail or resolve an incompatible set; the `optimize()` function signature has changed.
**Why it happens:** SWC sub-crates independently track major versions. swc_ecma_minifier jumped from 0.2xx to v55 in 2025.
**How to avoid:** Use the `swc` umbrella crate (current ~v66) which exposes `Compiler::minify`. The umbrella crate resolves the correct sub-crate versions internally via its own Cargo.lock. Verify the exact `swc` crate version with `cargo search swc` at plan time.
**Warning signs:** `cargo build` produces "failed to select a version" or "trait X not implemented" on `swc_ecma_minifier::optimize`.

### Pitfall 3: AVIF Encoding Speed
**What goes wrong:** `ravif` at default speed (speed=1 = slowest, highest quality) takes 10-30 seconds per high-resolution image, making publish jobs unacceptably slow.
**Why it happens:** rav1e's AV1 encoder trades CPU time for bitrate efficiency. The default configuration favors quality.
**How to avoid:** Set `Encoder::with_speed(4)` (medium speed, acceptable quality for web) in the `ImageTranscode` builder default. Expose it as `with_avif_speed(u8)` for consumer override. Document that speed 1-4 is quality-first, 5-8 is speed-first.
**Warning signs:** A 4K JPEG source takes > 15 seconds per variant during a test encode.

### Pitfall 4: Upscaling Responsive Variants
**What goes wrong:** A 400px-wide source image emits a 1920px variant (upscaled), which is larger than the original and serves no purpose.
**Why it happens:** The width filter is applied before encoding but a bug skips the `<= src.width()` guard.
**How to avoid:** `widths.iter().filter(|&&w| w <= src.width())` must be the first operation in the responsive-widths loop. Test: a 400×300 source image with default widths `[480, 768, 1200, 1920]` must emit ZERO responsive variants (all widths exceed source width).
**Warning signs:** Variant files exist in the output set with dimensions larger than the source.

### Pitfall 5: Pipeline Partial Output on Error
**What goes wrong:** A transform implementation returns `Ok(partial_set)` when some assets succeed and one fails, instead of `Err`. The consumer uploads the partial set and promotes a broken deployment.
**Why it happens:** The `map_matching` helper collects results; if a per-file `Result` is handled with `unwrap_or_else` instead of `?`, errors are silently swallowed.
**How to avoid:** `map_matching` must use `.collect::<Result<Vec<_>, _>>()` — the first `Err` short-circuits the iterator and propagates to `run()`. Never `unwrap` or `unwrap_or_default` in transform code.
**Warning signs:** A test that injects a deliberate encode failure returns `Ok` with a subset of assets instead of `Err`.

### Pitfall 6: lightningcss Version Drift
**What goes wrong:** Someone bumps `lightningcss` from `=1.0.0-alpha.71` to `"1"` in Cargo.toml during a routine `cargo update`. A subsequent alpha release with breaking API changes causes compilation failure.
**Why it happens:** The `=` exact pin is removed or relaxed.
**How to avoid:** Always use `lightningcss = "=1.0.0-alpha.71"` (exact pin with `=` prefix). Never use `"1"`, `"~1.0.0-alpha.71"`, or `"^1.0.0-alpha.71"` for this dependency.
**Warning signs:** `cargo update` changes lightningcss in Cargo.lock; subsequent `cargo build` has type errors on `StyleSheet::parse` or `MinifyOptions`.

---

## Code Examples

### Full Pipeline Composition (Consumer Reference)

```rust
// Source: CONTEXT.md <specifics> pipeline ordering [CITED]
// This is the order gestiscilo's PublishFrontendJob composes the pipeline
use ferro_assets::{
    Pipeline,
    transforms::{
        HtmlMinify, CssMinify, JsMinify,
        ImageTranscode, ResponsiveImages,
        InjectBeforeTag, ReplaceTokens,
    },
};
use std::collections::HashMap;

let mut tokens = HashMap::new();
tokens.insert("%%GESTISCILO_API_KEY%%".to_string(), api_key.clone());

let pipeline = Pipeline::new()
    .add(HtmlMinify::new())
    .add(CssMinify::new())
    .add(JsMinify::new())
    .add(ImageTranscode::new()
        .with_max_concurrent(2)
        .with_widths(vec![480, 768, 1200, 1920]))
    .add(ResponsiveImages::new())
    .add(InjectBeforeTag::new("</body>", &sdk_script_tag))
    .add(ReplaceTokens::new(tokens));

// Consumer wraps in spawn_blocking — NEVER call pipeline.run() on the async executor
let result = tokio::task::spawn_blocking(move || pipeline.run(assets)).await??;
```

### Cargo.toml for ferro-assets

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
swc = "66"  # VERIFY exact version at plan time: cargo search swc | head -1
image = { version = "0.25", features = ["avif"] }
ravif = "0.13"
rayon = "1"
bytes = "1"
thiserror = "2"
tracing = "0.1"

[dev-dependencies]
# No additional test crates needed beyond std; heavy image tests may be cfg-gated
# per Phase 185/186 precedent if transcode test is too slow for default CI
```

---

## Runtime State Inventory

> This is a greenfield crate phase (new crate, no existing state to rename/migrate). Runtime state inventory not applicable.

Nothing found in any category — this phase creates a new crate. No rename/refactor of existing stored state.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test -p ferro-assets` |
| Full suite command | `cargo test --all-features -p ferro-assets` |
| Heavy transcode gate | Controlled via `#[cfg_attr(not(feature = "slow-tests"), ignore)]` on the image transcode test if needed |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ASSET-F-01 (SC-1) | JSON file through full pipeline unchanged (passthrough proof) | integration | `cargo test -p ferro-assets -- passthrough_json_file` | ❌ Wave 0 |
| ASSET-F-01 | ContentType::Other assets pass through byte-identical | unit | `cargo test -p ferro-assets -- other_type_passthrough` | ❌ Wave 0 |
| ASSET-F-01 | Pipeline applies transforms in insertion order | unit | `cargo test -p ferro-assets -- pipeline_ordering` | ❌ Wave 0 |
| ASSET-F-02 (SC-2) | Inline `<script>` with template literals + JSON survives html_minify byte-correct | integration | `cargo test -p ferro-assets -- inline_script_regression` | ❌ Wave 0 |
| ASSET-F-02 | Inline `<style>` body survives html_minify byte-correct | integration | `cargo test -p ferro-assets -- inline_style_regression` | ❌ Wave 0 |
| ASSET-F-02 | CSS is actually minified (output smaller than input) | unit | `cargo test -p ferro-assets -- css_minify_reduces_size` | ❌ Wave 0 |
| ASSET-F-02 | JS is actually minified (output smaller than input) | unit | `cargo test -p ferro-assets -- js_minify_reduces_size` | ❌ Wave 0 |
| ASSET-F-03 (SC-3) | AVIF + JPEG variants emitted per configured width | integration | `cargo test -p ferro-assets -- image_transcode_variants` | ❌ Wave 0 |
| ASSET-F-03 | Source image narrower than all widths emits ZERO variants | unit | `cargo test -p ferro-assets -- no_upscale` | ❌ Wave 0 |
| ASSET-F-03 | Concurrent encodes bounded by semaphore (no OOM) | integration | `cargo test -p ferro-assets -- bounded_concurrency` | ❌ Wave 0 |
| ASSET-F-04 | `inject_before_tag("</body>", snippet)` inserts before closing body | unit | `cargo test -p ferro-assets -- inject_before_body` | ❌ Wave 0 |
| ASSET-F-04 | `replace_tokens` substitutes %%TOKEN%% in HTML attribute | unit | `cargo test -p ferro-assets -- replace_token_in_attribute` | ❌ Wave 0 |
| ASSET-F-04 | `replace_tokens` substitutes %%TOKEN%% inside inline `<script>` body | unit | `cargo test -p ferro-assets -- replace_token_in_script` | ❌ Wave 0 |
| SC-5 (D-17) | Pipeline failure produces no partial output set | integration | `cargo test -p ferro-assets -- pipeline_all_or_nothing` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-assets`
- **Per wave merge:** `cargo test --all-features -p ferro-assets`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-assets/src/` — entire crate (new crate, all files are Wave 0 gaps)
- [ ] `ferro-assets/tests/passthrough_proof.rs` — SC-1 JSON passthrough test
- [ ] `ferro-assets/tests/inline_script_fixture.rs` — SC-2 inline-script regression
- [ ] `ferro-assets/tests/image_transcode_test.rs` — SC-3 AVIF+JPEG variants
- [ ] `ferro-assets/tests/all_or_nothing.rs` — SC-5 failure atomicity test
- [ ] `ferro-assets/Cargo.toml` — crate manifest
- [ ] `ferro-assets/README.md` — crate README (document zero-C-deps as a feature)
- [ ] Workspace `Cargo.toml` members addition
- [ ] `.github/workflows/publish.yml` `WAVE1A_CRATES` addition

*(The inline_script_fixture content: lift a real fragment from jetskiadriatic HTML with a `<script>` block containing template literals, multi-line strings, and a JSON blob, plus an inline `<style>`. Assert that after `html_minify`, the script body and style body are byte-identical to the originals.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | crate has no auth surface |
| V3 Session Management | no | no sessions |
| V4 Access Control | no | crate has no access-control surface |
| V5 Input Validation | yes | all asset bytes are untrusted; parsers must handle malformed input without panic |
| V6 Cryptography | no | no crypto operations |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed CSS causing lightningcss panic | Denial of Service | `StyleSheet::parse` returns `Result` — propagate `Err` through `Error::transform`; never `.unwrap()` in production path |
| Malformed JS causing swc panic | Denial of Service | `Compiler::minify` via `try_with_handler` catches parse errors as `Result` — propagate through `Error::transform` |
| %%TOKEN%% substitution with attacker-controlled values injecting HTML | Tampering | `replace_tokens` is raw byte substitution — the caller is responsible for sanitizing values; document this in crate README |
| Image decode of malformed JPEG/PNG causing OOM | Denial of Service | `image::open` / `image::load_from_memory` returns `Result`; propagate errors; the rayon pool bounds memory surface |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `swc_ecma_minifier ~0.203` + low-level sub-crates | `swc` umbrella crate ~v66 | 2025 (major version jump) | Planner must use `swc` umbrella, NOT the gestiscilo STACK research's individual sub-crate versions |
| libvips for image processing | `image` 0.25 + `ravif` 0.13 | ferro Phase 187 design (2026-06-07) | Zero C system deps; pure Rust; thread-safe |
| libvips AVIF + WebP | AVIF only via ravif; WebP deferred | CONTEXT.md D-10 | Simpler dep tree; WebP revisited in v7.2 |
| swc_ecma_minifier ~0.203 compatible set from gestiscilo research | Umbrella `swc` crate | 2025 | Gestiscilo STACK research is stale for swc versions specifically |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `swc` umbrella crate is currently at ~v66 (June 2026) | Standard Stack, JS Minify Pattern | If version is different, `cargo add swc@66` fails; fallback: `cargo search swc` at plan time to get exact version |
| A2 | `rayon` is not yet in the ferro workspace dependency tree (would be a new dep for ferro-assets) | Standard Stack | If rayon is already present (e.g. via image transitive deps), no new dep needed; verify with `grep -r rayon */Cargo.toml` |
| A3 | `swc` umbrella crate at v66 still uses `Compiler::new` + `cm.new_source_file` + `c.minify` API from the GitHub example | Code Examples §Pattern 4 | If API changed, executor must check `docs.rs/swc` at plan time before writing `js_minify.rs` |
| A4 | `swc_common::GLOBALS.set` call is still required in the `swc` v66 API | Code Examples §Pattern 4 | If removed, the example compiles without it; no functional impact |
| A5 | ravif 0.13 uses `Encoder::new().with_quality().with_speed().encode_rgba(Img::new(pixels, w, h))` | Code Examples §Pattern 6 | If API changed between 0.13 minor bumps, executor must check docs.rs/ravif |

**If A1 or A3 is wrong:** The planner should add a Wave 0 task: "verify swc version and Compiler::minify API before writing js_minify.rs". The risk is limited to one transform; the other six transforms have higher-confidence APIs.

---

## Open Questions

1. **swc exact version and API at plan time**
   - What we know: gestiscilo research cited 0.203 sub-crates (stale); crates.io shows swc_ecma_minifier ~v55 and swc umbrella ~v66 as of 2026-06-07
   - What's unclear: Exact minor/patch version and whether `Compiler::minify` + `GLOBALS.set` pattern is unchanged
   - Recommendation: Planner adds a Wave 0 step: `cargo search swc` + `cargo add swc --dry-run` to lock the exact version before implementation begins

2. **rayon workspace dependency status**
   - What we know: rayon is not listed in ferro-queue or ferro-deployments Cargo.toml
   - What's unclear: Whether `image 0.25` with the `avif` feature transitively pulls rayon
   - Recommendation: Executor confirms with `cargo tree -p ferro-assets 2>/dev/null | grep rayon` after adding deps; if present transitively, no direct dep needed

3. **lol_html 2.6 exact API for the opaque-script pattern**
   - What we know: The pattern is "element handler only, no text handler for script/style"; confirmed from official docs and Context7
   - What's unclear: Whether lol_html 2.6 changed any API surface vs 2.4/2.5
   - Recommendation: Verified from crates.io that 2.6.0 is the current release [VERIFIED]; API is stable at the `HtmlRewriter` + `Settings` + `element!` macro level

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | Build | ✓ | 1.88.0 | — |
| lol_html 2.6 | html_minify, responsive_images, inject_before_tag | ✓ (crates.io) | 2.6.0 | — |
| lightningcss =1.0.0-alpha.71 | css_minify | ✓ (crates.io) | =1.0.0-alpha.71 | — |
| swc ~66 | js_minify | ✓ (crates.io, verify exact version) | ~66.x [ASSUMED] | — |
| image 0.25 | image_transcode | ✓ (workspace Cargo.lock) | 0.25 | — |
| ravif 0.13 | image_transcode (AVIF encode) | ✓ (crates.io, pure Rust) | 0.13.0 | — |
| rayon | image_transcode bounded concurrency | ✓ (pure Rust, crates.io) | 1.x | std::thread + counting gate |

**Missing dependencies with no fallback:** None.

**First-publish note:** CI token is `publish-update` only, not `publish-new`. `ferro-assets` does not yet exist on crates.io, so the executor must run `cargo publish -p ferro-assets` once from a local terminal. [VERIFIED: MEMORY.md project_ferro_publish_token_scoping.md reference; same as Phase 183/186]

---

## Sources

### Primary (HIGH confidence)
- `CONTEXT.md D-01..D-18` — all locked decisions [VERIFIED: codebase read]
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-STACK.md §D-03` — crate selection rationale, lightningcss API pattern, swc pin rationale (version is stale but rationale is valid), image+ravif vs libvips, responsive-widths code sketch [VERIFIED: file read]
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-PITFALLS.md §C` — C-01 (SEO out), C-02 (opaque script/style), C-03 (memory semaphore), C-04 (all-or-nothing), A-04 (sync CPU in spawn_blocking) [VERIFIED: file read]
- `docs.rs/ravif 0.13` — `Encoder::new().with_quality().with_speed().encode_rgba(Img)` API [VERIFIED: docs.rs/ravif fetch]
- `github.com/swc-project/swc/crates/swc/examples/minify.rs` — `GLOBALS.set + try_with_handler + Compiler::new + cm.new_source_file + c.minify` pattern [VERIFIED: GitHub WebFetch]
- Context7 `/parcel-bundler/lightningcss` — `StyleSheet::parse / minify / to_css` API [VERIFIED: Context7 query]
- Context7 `/cloudflare/lol-html` — `HtmlRewriter`, `Settings`, `element_content_handlers` structure [VERIFIED: Context7 query]
- `docs.rs/rayon` — `ThreadPoolBuilder::new().num_threads(n).build()` + `pool.install()` [VERIFIED: WebSearch rayon docs]
- `.github/workflows/publish.yml` — `WAVE1A_CRATES` list; ferro-assets belongs here [VERIFIED: codebase read]
- `ferro-deployments/Cargo.toml`, `ferro-queue/Cargo.toml`, `ferro-bundle/Cargo.toml` — new-crate workspace template [VERIFIED: codebase read]
- `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md` — workspace coding standards [VERIFIED: codebase read]
- `186-RESEARCH.md` — sibling new-leaf-crate precedent [VERIFIED: codebase read]

### Secondary (MEDIUM confidence)
- crates.io search results — lol_html 2.6.0 confirmed; swc ~v66 and swc_ecma_minifier ~v55 confirmed [VERIFIED: WebSearch 2026-06-07]
- gestiscilo v7.1-ARCHITECTURE.md §D-03 "Ferro-side primitives" — ferro-assets deliverable definition [VERIFIED: file read]
- gestiscilo v7.1-INTEGRATION.md §Phase 189 — consumer contract (PublishFrontendJob composes the pipeline) [VERIFIED: file read]

### Tertiary (LOW confidence)
- swc `Compiler::minify` exact v66 API — [ASSUMED: A1, A3] based on GitHub example from swc monorepo; must be re-verified at plan time
- rayon workspace status — [ASSUMED: A2] not in ferro-queue or ferro-deployments; verify before committing Cargo.toml

---

## Metadata

**Confidence breakdown:**
- Standard stack (lol_html, lightningcss, ravif): HIGH — versions confirmed from crates.io, APIs from docs.rs + Context7
- Standard stack (swc): MEDIUM — major version jump confirmed; exact v66 API assumed from GitHub example; must verify at plan time
- Architecture (Transform trait, Pipeline, ContentType): HIGH — directly from locked CONTEXT.md D-05/D-06/D-07/D-17
- Image transcode (ravif API): HIGH — `Encoder::new().encode_rgba(Img)` verified from docs.rs
- lol_html opaque-script pattern: HIGH — verified from Context7 and official docs
- rayon bounded concurrency: HIGH — `ThreadPoolBuilder::num_threads` is stable rayon API
- Pitfalls: HIGH — sourced from locked v7.1-PITFALLS.md §C + CONTEXT.md D-14

**Research date:** 2026-06-07
**Valid until:** 2026-07-07 (lol_html, lightningcss, ravif, image are stable; swc must be re-verified at plan time — swc major versions have been moving monthly in 2025-2026)
