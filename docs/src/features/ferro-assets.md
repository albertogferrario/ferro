# Asset Pipeline

`ferro-assets` provides a composable, content-type-aware asset pipeline for publish-time
optimization over an in-memory file set.

A `Pipeline` runs over a heterogeneous `Asset` collection (HTML, CSS, JS, images, and any
other files such as JSON-UI spec bundles). Each transform declares the content types it
accepts; every other file passes through byte-for-byte unchanged — the **passthrough
guarantee**. This makes the same pipeline safe for mixed artifact sets without any
per-file branching in the consumer.

`ferro-assets` uses only pure-Rust codecs. No C system packages (`libvips`, `libavif`,
`libwebp`) are required; `cargo build` adds nothing to the production image.

## Quick Start

```rust,ignore
use ferro_assets::{Asset, Pipeline};
use ferro_assets::transforms::{
    HtmlMinify, CssMinify, JsMinify,
    ImageTranscode, ResponsiveImages,
    InjectBeforeTag, ReplaceTokens,
};
use std::collections::HashMap;

let pipeline = Pipeline::new()
    .add(HtmlMinify::new())
    .add(CssMinify::new())
    .add(JsMinify::new())
    .add(ImageTranscode::new())
    .add(ResponsiveImages::new())
    .add(InjectBeforeTag::new("</body>", r#"<script src="/sdk.js"></script>"#))
    .add(ReplaceTokens::new(HashMap::new()));

// pipeline.run() is synchronous (blocking). Wrap in spawn_blocking
// when calling from an async context.
let result = tokio::task::spawn_blocking(move || pipeline.run(assets)).await??;
```

The `spawn_blocking` wrapper is required: `pipeline.run()` is CPU-bound synchronous work.
Calling it directly on the async executor stalls every concurrent HTTP request for the
duration of the pipeline.

## Asset and ContentType Model

An `Asset` is an in-memory file with a logical path, a content type, and its bytes:

```rust,ignore
use ferro_assets::{Asset, ContentType};
use bytes::Bytes;

// ContentType is inferred from the path extension.
let html  = Asset::new("index.html",  Bytes::from(html_bytes));
let css   = Asset::new("app.css",     Bytes::from(css_bytes));
let image = Asset::new("hero.jpg",    Bytes::from(jpeg_bytes));
let spec  = Asset::new("spec.json",   Bytes::from(json_bytes));
// spec has ContentType::Other — no built-in transform touches it.

// Override content type explicitly when needed.
let asset = Asset::new("file", bytes).with_content_type(ContentType::Js);
```

`infer_content_type` maps path extensions to `ContentType` variants:

| Extension(s) | ContentType |
|---|---|
| `.html`, `.htm` | `Html` |
| `.css` | `Css` |
| `.js`, `.mjs` | `Js` |
| `.jpg`, `.jpeg` | `Jpeg` |
| `.png` | `Png` |
| `.avif` | `Avif` |
| anything else | `Other` (passthrough) |

`Asset.path` is a **logical artifact key** — `ferro-assets` never reads or writes
files. If the consumer uses a path to write output to disk, it must sanitize it against
path traversal before doing so.

## Built-in Transforms

### HtmlMinify

Collapses redundant whitespace in visible body text using `lol_html`. The content of
`<script>` and `<style>` elements is treated as **opaque** — whitespace inside those
elements is never touched. Template literals, multi-line strings, and inline JSON blobs
inside `<script>` survive the transform byte-correct.

```rust,ignore
use ferro_assets::transforms::HtmlMinify;

let transform = HtmlMinify::new();
```

### CssMinify

Minifies CSS using `lightningcss` (pinned to `=1.0.0-alpha.71` — this alpha version pins
a stable API; a relaxed semver range would silently pick up breaking alpha bumps).

```rust,ignore
use ferro_assets::transforms::CssMinify;

let transform = CssMinify::new();
```

### JsMinify

Minifies JavaScript using `swc` compress+mangle. The transform operates on JS files only;
JSON and other text files that happen to contain JavaScript-like content are not touched
because their `ContentType` is `Other`.

```rust,ignore
use ferro_assets::transforms::JsMinify;

let transform = JsMinify::new();
```

### ImageTranscode

Transcodes JPEG, PNG, and AVIF source images into AVIF and JPEG variants at configurable
responsive widths. Variants are added to the asset set alongside the original source (the
original is retained as the JPEG fallback for `<img src>`).

Default configuration:

- Widths: `[480, 768, 1200, 1920]` — only widths `≤ source.width()` are emitted (no upscaling).
- Max concurrent encodes: 2 — bounds peak memory on small instances.
- AVIF quality: 70.0, AVIF speed: 4 (faster than the slowest setting without quality loss).
- JPEG quality: 80.

```rust,ignore
use ferro_assets::transforms::ImageTranscode;

let transform = ImageTranscode::new()
    .with_widths(vec![640, 1280])
    .with_max_concurrent(4)
    .with_avif_quality(75.0)
    .with_jpeg_quality(85);
```

Variant naming is deterministic: `{stem}-{width}w.avif` and `{stem}-{width}w.jpg` (e.g.
`assets/hero-768w.avif`). `ResponsiveImages` discovers these by name — run `ImageTranscode`
before `ResponsiveImages` in the pipeline.

### ResponsiveImages

Rewrites every `<img src="…">` element in HTML files to a `<picture>` element referencing
the AVIF and JPEG variants already present in the asset set. Relies on the deterministic
variant naming from `ImageTranscode`. Run after `ImageTranscode`.

```rust,ignore
use ferro_assets::transforms::ResponsiveImages;

let transform = ResponsiveImages::new();
```

Example output (for `<img src="assets/hero.jpg">`):

```html
<picture>
  <source type="image/avif" srcset="assets/hero-480w.avif 480w, assets/hero-768w.avif 768w, assets/hero-1200w.avif 1200w">
  <img src="assets/hero.jpg" ...>
</picture>
```

### InjectBeforeTag

Inserts a literal snippet immediately before a given closing tag in every HTML file. The
primary use is injecting an SDK `<script>` before `</body>`.

```rust,ignore
use ferro_assets::transforms::InjectBeforeTag;

let transform = InjectBeforeTag::new("</body>", r#"<script src="/sdk.js"></script>"#);
```

The tag name is validated — only alphanumeric and hyphen characters are accepted. An
invalid tag name is a no-op (the transform returns the asset unchanged).

### ReplaceTokens

Performs raw byte substitution of `%%TOKEN%%`-style placeholders across every file,
regardless of content type. Tokens can appear in HTML attributes, inline scripts, CSS
custom properties, or JSON values — the transform operates on raw bytes, not a parsed DOM.

```rust,ignore
use ferro_assets::transforms::ReplaceTokens;
use std::collections::HashMap;

let tokens = HashMap::from([
    ("%%CDN_URL%%".to_string(),  "https://cdn.example.com".to_string()),
    ("%%APP_VERSION%%".to_string(), "1.0.3".to_string()),
]);
let transform = ReplaceTokens::new(tokens);
```

Substitution is literal with no evaluation or recursion. **The caller is responsible for
sanitizing token values** before constructing the map — values are written verbatim into
the output bytes.

## Writing Custom Transforms

Implement `Transform` and use `map_matching` to gate work by content type:

```rust,ignore
use ferro_assets::{Asset, ContentType, Error, Transform, map_matching};

pub struct StripHtmlComments;

impl Transform for StripHtmlComments {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Html], |a| {
            let out = remove_comments(&a.bytes)
                .map_err(|e| Error::transform("strip_html_comments", &a.path, e))?;
            Ok(Asset { bytes: out.into(), ..a })
        })
    }
}
```

`map_matching` iterates the asset set: files in `accepted` are passed to the closure;
everything else passes through unchanged. The collection short-circuits on the first `Err`
— no partial output is produced.

For transforms that operate on the whole asset set (e.g. cross-referencing variant names),
implement `run` directly without `map_matching`:

```rust,ignore
impl Transform for MyBatchTransform {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        // process the full set and return a new Vec<Asset>
        Ok(assets)
    }
}
```

## Error Reference

| Variant | Meaning |
|---------|---------|
| `Error::Transform { transform, path, cause }` | A transform failed on a specific asset. `transform` names the stage (e.g. `"html_minify"`), `path` is the logical asset path, `cause` is the underlying message. |
| `Error::Setup(msg)` | Thread pool or initialisation failure (e.g. rayon `ThreadPoolBuilder` error). |
