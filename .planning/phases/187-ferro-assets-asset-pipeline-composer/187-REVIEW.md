---
phase: 187-ferro-assets-asset-pipeline-composer
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - ferro-assets/Cargo.toml
  - ferro-assets/src/lib.rs
  - ferro-assets/src/asset.rs
  - ferro-assets/src/error.rs
  - ferro-assets/src/pipeline.rs
  - ferro-assets/src/transforms/mod.rs
  - ferro-assets/src/transforms/html_minify.rs
  - ferro-assets/src/transforms/css_minify.rs
  - ferro-assets/src/transforms/js_minify.rs
  - ferro-assets/src/transforms/inject_before_tag.rs
  - ferro-assets/src/transforms/replace_tokens.rs
  - ferro-assets/src/transforms/image_transcode.rs
  - ferro-assets/src/transforms/responsive_images.rs
  - ferro-assets/tests/passthrough_proof.rs
  - ferro-assets/tests/all_or_nothing.rs
  - ferro-assets/tests/inline_script_fixture.rs
  - ferro-assets/tests/image_transcode_test.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 187: Code Review Report

**Reviewed:** 2026-06-08T00:00:00Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

`ferro-assets` is a well-structured pure-Rust asset pipeline. The core contracts
(all-or-nothing error propagation, passthrough guarantee, no-upscale, inline
`<script>`/`<style>` opaque treatment) are correctly implemented and thoroughly
tested. No panics exist on production code paths for the default configurations.
No path traversal surface exists — `Asset.path` is a logical label never
opened as a filesystem path within the crate.

Three warnings are raised:

1. The rayon concurrency bounding claim is structurally incorrect: images are
   processed sequentially inside `pool.install`, not in parallel. The `max_concurrent`
   knob has no effect on actual concurrency.
2. AVIF file paths used in the generated `srcset` attribute are interpolated
   without HTML-encoding, which would corrupt the attribute if a path ever
   contained a double-quote character.
3. `ReplaceTokens` applies raw-byte substitution to binary image assets (AVIF,
   JPEG, PNG), creating a latent risk of binary corruption if the token pattern
   appears in compressed image data. The behavior is undocumented and the
   recommended pipeline ordering places `ReplaceTokens` after `ImageTranscode`.

---

## Warnings

### WR-01: Rayon concurrency bounding is a no-op — images process serially

**File:** `ferro-assets/src/transforms/image_transcode.rs:171-176`

**Issue:** `pool.install(|| images.into_iter().map(...).collect())` runs a
standard (non-rayon) sequential iterator inside the pool's context. `pool.install`
executes the closure on one of the pool's threads, but does not parallelise the
work inside it unless a rayon parallel iterator (`par_iter()`,
`into_par_iter()`) is used. As written, all `transcode_image` calls run
sequentially on a single thread. The `max_concurrent` field has no effect on
actual concurrency. The thread pool is constructed and immediately destroyed on
every `run()` call, paying `ThreadPoolBuilder::build()` overhead for no benefit.
The doc comment on line 170 (`// pool.install bounds parallelism to max_concurrent threads (D-09)`)
and the struct-level docs (`Encodes run inside a rayon ThreadPool with num_threads set to max_concurrent`) are factually incorrect.

**Fix:** Either use rayon's parallel iterator to achieve the intended
concurrency, or remove the pool and document that processing is sequential:

```rust
// Option A — actually parallel (use rayon par_iter inside pool.install):
use rayon::prelude::*;
let variants: Result<Vec<Vec<Asset>>, Error> = pool.install(|| {
    images
        .into_par_iter()          // <-- parallel iterator
        .map(|a| self.transcode_image(a))
        .collect()
});

// Option B — admit sequential, drop the pool entirely:
let variants: Result<Vec<Vec<Asset>>, Error> = images
    .into_iter()
    .map(|a| self.transcode_image(a))
    .collect();
// Remove max_concurrent field, ThreadPoolBuilder, Error::Setup variant usage here.
```

If parallelism is desired (Option A), the `Error` type must be `Send` (it is —
all fields are `String`) and `transcode_image` must be `Sync`-safe (it is — it
takes `&self` over shared immutable config). Option A is safe to use.

---

### WR-02: AVIF paths interpolated into `srcset` HTML attribute without encoding

**File:** `ferro-assets/src/transforms/responsive_images.rs:162-169`

**Issue:** The `srcset` attribute value is built by formatting asset paths
directly into a double-quoted HTML attribute:

```rust
let srcset: String = variants
    .iter()
    .map(|(w, p)| format!("{p} {w}w"))   // p is the raw asset path
    .collect::<Vec<_>>()
    .join(", ");
// ...
&format!(r#"<source type="image/avif" srcset="{srcset}">"#)
```

If any asset path contains a double-quote character (`"`), the generated HTML
will be:

```html
<source type="image/avif" srcset="path-with-"-in-it-480w.avif 480w">
```

which is malformed HTML and breaks the attribute boundary. In practice,
filesystem asset paths rarely contain `"`, but the crate accepts arbitrary
`Asset.path` strings and this is a latent correctness issue. Since `lol_html`
inserts the string as `LolContentType::Html` (raw HTML), it performs no
automatic escaping.

**Fix:** HTML-encode the path before interpolation. A minimal approach:

```rust
fn html_encode_attr(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('"', "&quot;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

let srcset: String = variants
    .iter()
    .map(|(w, p)| format!("{} {w}w", html_encode_attr(p)))
    .collect::<Vec<_>>()
    .join(", ");
```

Alternatively, add a doc invariant on `Asset.path` that prohibits `"`, `<`,
`>`, and `&` characters and validate at `Asset::new` time.

---

### WR-03: `ReplaceTokens` applies raw-byte substitution to binary image assets with no documentation

**File:** `ferro-assets/src/transforms/replace_tokens.rs:51-68`

**Issue:** `ReplaceTokens::run` iterates every asset regardless of
`ContentType`, including `Jpeg`, `Png`, and `Avif` binary image data. If a
compressed image file coincidentally contains the token byte sequence
(e.g. `%%CDN_URL%%` = `25 25 43 44 4e 5f 55 52 4c 25 25`), those bytes are
replaced with the substitution value, silently corrupting the image. The
pipeline ordering documented in `transforms/mod.rs` (line 12–13) places
`image_transcode` and `responsive_images` before `replace_tokens`, so the
generated AVIF/JPEG variants are present in the asset set when substitution
runs.

The probability of collision is low for `%%TOKEN%%`-style patterns in compressed
binary data, but the crate gives no warning and no opt-out mechanism for binary
content types. The security note in the module doc says "caller is responsible
for sanitizing replacement values" but says nothing about binary asset
corruption.

**Fix:** Add a content-type gate that skips binary image types, or document the
limitation prominently so callers know to run `ReplaceTokens` before
`ImageTranscode` if they have image assets in the set:

```rust
// Option A — skip binary types:
impl crate::pipeline::Transform for ReplaceTokens {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        assets
            .into_iter()
            .map(|a| {
                // Skip binary image types — raw-byte substitution would corrupt
                // compressed data. Tokens in image assets are not a use case.
                if matches!(a.content_type, ContentType::Jpeg | ContentType::Png | ContentType::Avif) {
                    return Ok(a);
                }
                let mut bytes = a.bytes.to_vec();
                for (token, replacement) in &self.map {
                    bytes = replace_bytes(&bytes, token.as_bytes(), replacement.as_bytes());
                }
                Ok(Asset { bytes: Bytes::from(bytes), ..a })
            })
            .collect::<Result<Vec<_>, Error>>()
    }
}

// Option B — document the ordering requirement clearly in the struct doc:
/// NOTE: Run this transform **before** `ImageTranscode` if your asset set
/// contains image files. Running after image encoding risks silent binary
/// corruption if a token pattern appears in compressed image data.
```

---

## Info

### IN-01: `with_max_concurrent(0)` silently disables concurrency bounding

**File:** `ferro-assets/src/transforms/image_transcode.rs:71-73`

**Issue:** `with_max_concurrent(0)` is accepted without validation. With rayon's
`ThreadPoolBuilder`, `num_threads(0)` means "use the number of logical CPUs" —
not "use zero threads". The API contract of the method ("maximum number of
concurrent image encodes") is violated silently. This only matters if WR-01 is
fixed to use actual parallelism; in the current sequential implementation it is
benign.

**Fix:** Add a bounds check or document the rayon `0 = logical CPUs` semantics:

```rust
pub fn with_max_concurrent(mut self, n: usize) -> Self {
    // rayon treats 0 as "number of logical CPUs"; enforce at least 1.
    self.max_concurrent = n.max(1);
    self
}
```

---

### IN-02: Pool constructed and destroyed on every `run()` call

**File:** `ferro-assets/src/transforms/image_transcode.rs:154-157`

**Issue:** `ThreadPoolBuilder::new().num_threads(...).build()` is called on
every invocation of `ImageTranscode::run`. Thread pool creation involves OS-level
thread spawning and is not cheap. For single-shot publish jobs this is
immaterial, but if `run()` is called in a loop (e.g. for incremental builds or
test scenarios), the overhead compounds. This is related to WR-01: once the pool
actually does parallel work, reusing it across calls would be more efficient.

**Fix:** If WR-01 is resolved with `par_iter`, consider making the pool a field
constructed in `ImageTranscode::new()` (requires removing `Clone` from the
struct, or wrapping the pool in `Arc`):

```rust
pub struct ImageTranscode {
    pool: Arc<rayon::ThreadPool>,
    // ... other fields
}
```

This is a quality improvement, not a correctness issue.

---

### IN-03: `resize_to_width` uses floating-point division with no guard for zero-width source

**File:** `ferro-assets/src/transforms/image_transcode.rs:188`

**Issue:**

```rust
let height = ((src.height() as f64 * width as f64) / src.width() as f64).round() as u32;
```

If `src.width()` is 0 (theoretically possible if a decoder returns a
degenerate image), this is `f64::NAN` after `0.0 / 0.0`, which casts to `0u32`
(Rust's `as` truncation for NaN), recovered to `1` by `height.max(1)`. The
subsequent `resize_exact(width=0, height=1, ...)` would depend on `image`'s
behavior for a zero-dimension resize. With the default configured widths
`[480, 768, 1200, 1920]`, a zero-width source image would never pass the
`w <= src_width` filter (480 <= 0 is false), so `resize_to_width` is
unreachable in practice with default widths. The only reachable path is with
a caller-configured `with_widths(vec![0, ...])`, which is an unusual input.

**Fix:** Add an assertion or explicit guard:

```rust
fn resize_to_width(src: &DynamicImage, width: u32) -> DynamicImage {
    debug_assert!(src.width() > 0, "resize_to_width called on zero-width image");
    let src_w = src.width().max(1); // defensive; should never be 0 with valid input
    let height = ((src.height() as f64 * width as f64) / src_w as f64).round() as u32;
    let height = height.max(1);
    src.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
}
```

---

### IN-04: `Error::transform` test lives in `asset.rs` rather than `error.rs`

**File:** `ferro-assets/src/asset.rs:123-129`

**Issue:** The test `error_transform_to_string_contains_all_three_fields` is in
`asset.rs` (inside `mod tests`) despite testing `crate::error::Error`. This is
a minor organisation issue — it does not affect correctness, but a reader
looking for `Error` tests would search `error.rs` first and not find them.

**Fix:** Move the test to `error.rs`:

```rust
// In ferro-assets/src/error.rs, at the bottom:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_transform_to_string_contains_all_three_fields() {
        let e = Error::transform("html_minify", "index.html", "boom");
        let s = e.to_string();
        assert!(s.contains("html_minify"));
        assert!(s.contains("index.html"));
        assert!(s.contains("boom"));
    }
}
```

---

_Reviewed: 2026-06-08T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
