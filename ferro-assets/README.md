# ferro-assets

Composable, content-type-aware asset pipeline for the Ferro framework.

A [`Pipeline`] runs over a heterogeneous in-memory file set — HTML, CSS, JS, images, and
any other files (JSON-UI spec bundles, SSR manifests, etc.). Each built-in transform
declares the content types it accepts; every other file passes through byte-for-byte
unchanged (the **passthrough guarantee**).

## Features

- **Content-type router** — one pipeline processes a mixed artifact set; each transform
  only touches the file types it understands.
- **Byte-identical passthrough** — files with unrecognized extensions (e.g. `.json`,
  `.wasm`) exit the pipeline with bytes identical to the input. Proved by a full
  seven-transform integration test against a JSON file.
- **Zero C system dependencies** — every codec is pure Rust (`ravif`, `image`, `lol_html`,
  `lightningcss`, `swc`). `cargo build` adds no `libvips`, `libavif`, `libwebp`, or any
  other system package to the production image. AVIF encoding uses `rav1e` without its
  assembly kernels, so no `nasm` is required. To trade that for faster encodes, enable
  `rav1e`'s SIMD path from your own manifest with `ravif = { version = "0.13", features =
  ["asm"] }` (this requires the `nasm` assembler at build time).
- **All-or-nothing execution** — any transform failure returns a structured `Error` with
  per-transform and per-file context; no partial output set is produced.
- **Synchronous API** — no async runtime dependency. The consumer wraps `pipeline.run()`
  in `tokio::task::spawn_blocking`.

## Quick Start

```rust,ignore
use ferro_assets::{Asset, Pipeline};
use ferro_assets::transforms::{
    HtmlMinify, CssMinify, JsMinify,
    ImageTranscode, ResponsiveImages,
    InjectBeforeTag, ReplaceTokens,
};
use std::collections::HashMap;

// Compose the pipeline in the canonical order.
let pipeline = Pipeline::new()
    .add(HtmlMinify::new())
    .add(CssMinify::new())
    .add(JsMinify::new())
    .add(ImageTranscode::new())
    .add(ResponsiveImages::new())
    .add(InjectBeforeTag::new("</body>", r#"<script src="/sdk.js"></script>"#))
    .add(ReplaceTokens::new(HashMap::from([
        ("%%CDN_URL%%".to_string(), "https://cdn.example.com".to_string()),
    ])));

// pipeline.run() is synchronous (blocking). Always wrap in spawn_blocking
// when calling from an async context to avoid stalling the executor.
let assets: Vec<Asset> = load_assets(); // your asset loading logic
let result = tokio::task::spawn_blocking(move || pipeline.run(assets)).await??;
```

### Building an asset set

```rust,ignore
use ferro_assets::Asset;
use bytes::Bytes;

let assets = vec![
    Asset::new("index.html",     Bytes::from(fs::read("index.html")?)),
    Asset::new("styles/app.css", Bytes::from(fs::read("styles/app.css")?)),
    Asset::new("assets/hero.jpg", Bytes::from(fs::read("assets/hero.jpg")?)),
    Asset::new("spec.json",      Bytes::from(fs::read("spec.json")?)),
    // spec.json is ContentType::Other — it passes every transform untouched.
];
```

## Built-in Transforms

| Transform | Accepted types | Key behaviour |
|-----------|---------------|---------------|
| `HtmlMinify` | `Html` | Collapses whitespace in body text; treats `<script>`/`<style>` bodies as **opaque** (template literals and inline JSON are safe) |
| `CssMinify` | `Css` | Minifies via `lightningcss =1.0.0-alpha.71` |
| `JsMinify` | `Js` | Minifies via `swc` compress+mangle |
| `ImageTranscode` | `Jpeg`, `Png`, `Avif` | Emits AVIF + JPEG variants at configurable widths (default `[480, 768, 1200, 1920]`); never upscales; bounds concurrent encodes (default ≤2) |
| `ResponsiveImages` | `Html` | Rewrites `<img src>` to `<picture>` using AVIF/JPEG variants already present in the asset set; run after `ImageTranscode` |
| `InjectBeforeTag` | `Html` | Inserts a snippet immediately before a closing tag (primary use: SDK `<script>` before `</body>`) |
| `ReplaceTokens` | All types | Raw byte substitution of `%%TOKEN%%`-style placeholders; applies to every file |

### ImageTranscode tuning

```rust,ignore
ImageTranscode::new()
    .with_widths(vec![640, 1280])  // responsive breakpoints
    .with_max_concurrent(4)         // cap peak memory on small instances
    .with_avif_quality(75.0)        // default 70.0
    .with_jpeg_quality(85)          // default 80
```

## Writing Custom Transforms

Implement the `Transform` trait and use `map_matching` for type-gated per-file work:

```rust,ignore
use ferro_assets::{Asset, ContentType, Error, Transform, map_matching};

struct StripComments;

impl Transform for StripComments {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        map_matching(assets, &[ContentType::Html], |a| {
            let stripped = remove_html_comments(&a.bytes)?;
            Ok(Asset { bytes: stripped.into(), ..a })
        })
    }
}
```

## SECURITY NOTES

**`Asset.path` is a logical key, not a filesystem path.** `ferro-assets` never reads or
writes files — it operates entirely on in-memory `Vec<Asset>`. The `path` field is an
artifact identifier used for content-type inference and error messages. If the consumer
passes an `Asset.path` to a filesystem API (e.g. to write the output), it **must sanitize
and validate the path** against path traversal attacks before doing so. The crate applies
no such validation (T-187-03).

**`ReplaceTokens` performs literal substitution with no evaluation or recursion.** The
token map values are substituted verbatim. If the map contains HTML, JavaScript, or other
structured content, the **caller is responsible for sanitizing those values** before
constructing the map (T-187-07).

**`ImageTranscode` bounds concurrent encodes to a configurable limit (default ≤2)** to cap
peak memory on small instances. When processing large image sets on memory-constrained
hosts, lower this limit or process images in batches (T-187-09).

## License

MIT — see the [ferro](https://github.com/albertogferrario/ferro) workspace README.
