---
phase: 187-ferro-assets-asset-pipeline-composer
verified: 2026-06-08T08:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 187: ferro-assets — Asset Pipeline Composer — Verification Report

**Phase Goal:** New crate providing a composable, content-type-aware asset pipeline for
publish-time optimization: HTML/CSS/JS minification, pure-Rust image transcoding with
responsive variants, and generic tag injection — the Tier 1 pipeline gestiscilo's
`PublishFrontendJob` composes.
**Verified:** 2026-06-08T08:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `Pipeline::new().add(transform)...run(files)` applies transforms in order; non-matching content types pass through byte-identical | VERIFIED | `passthrough_proof.rs`: 5/5 tests green including `json_file_unchanged_by_all_seven_real_transforms` (all 7 real transforms) and `pipeline_applies_transforms_in_insertion_order` (order probe). `pipeline.rs` loop: `current = transform.run(current)?` |
| 2 | `html_minify` (lol_html), `css_minify` (lightningcss `=1.0.0-alpha.71`), `js_minify` (swc umbrella) ship as built-ins; inline `<script>`/`<style>` survive minification byte-correct | VERIFIED | `inline_script_fixture.rs`: 4/4 tests green including `inline_script_body_survives_html_minify_byte_exact` and `inline_style_body_survives_html_minify_byte_exact`. `html_minify.rs` has element handler only (NO `text!("script"/"style")` — confirmed by grep). lightningcss pin `=1.0.0-alpha.71` intact in Cargo.toml |
| 3 | `image_transcode` emits AVIF (ravif) + JPEG at configurable widths via pure-Rust codecs — ZERO new C system deps; concurrent encodes bounded (default ≤2) | VERIFIED | `image_transcode_test.rs`: 4 always-on tests green (no-upscale, D-12 naming, content types, exact-width). `into_par_iter()` inside `pool.install()` with `num_threads(max_concurrent)`. `core-foundation-sys` is a pre-existing macOS timezone dep (via `lightningcss` → `browserslist-rs` → `chrono` → `iana-time-zone`), already in `ferro-json-ui` before this phase — not a new C codec dep |
| 4 | `responsive_images` rewrites `<img>`→`<picture><source srcset>` using discovered variants; `inject_before_tag` + `%%TOKEN%%` `replace_tokens` work | VERIFIED | `responsive_images.rs`: `element!("img"` present, `image/avif` srcset built, D-12 round-trip via `parse_variant_name`. `inject_before_tag.rs`: `end_tag!` hook inserts before closing tag. `replace_tokens.rs`: raw-byte loop on text-bearing content types (Html/Css/Js/Other); skips binary (JPEG/PNG/AVIF) to prevent corruption (WR-03 fix) |
| 5 | Pipeline failure at any stage returns a structured per-file error and produces NO partial output set | VERIFIED | `all_or_nothing.rs`: 6/6 tests green including `error_mid_pipeline_produces_no_partial_output`, `error_carries_transform_and_path_context`. `pipeline.rs` `run()`: `transform.run(current)?` — any `Err` propagates immediately, no partial `Vec<Asset>` returned |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-assets/Cargo.toml` | Manifest with verified, pinned zero-C-dep deps | VERIFIED | `lightningcss = "=1.0.0-alpha.71"` (exact pin), `swc = "66"`, `version.workspace = true` |
| `ferro-assets/src/asset.rs` | Asset struct, ContentType enum, infer_content_type() | VERIFIED | 7-variant ContentType enum, Asset struct with bytes::Bytes, infer_content_type() with full extension table, 8 unit tests green |
| `ferro-assets/src/pipeline.rs` | Transform trait, map_matching helper, Pipeline builder, all-or-nothing run() | VERIFIED | `pub trait Transform`, `pub fn map_matching`, `Pipeline::run` with `current = transform.run(current)?` loop, `collect::<Result<Vec<_>,_>>()` for short-circuit |
| `ferro-assets/src/error.rs` | Single thiserror Error enum with per-file/per-transform context | VERIFIED | `#[derive(Debug, Error)]`, `Error::Transform { transform, path, cause }`, `Error::Setup(String)` with constructor helpers |
| `ferro-assets/src/transforms/html_minify.rs` | HtmlMinify (lol_html) with opaque script/style bodies | VERIFIED | `element!("script"` present, zero `text!("script"/"style")` handlers, whitespace collapse via `text!("body *", ...)` only |
| `ferro-assets/src/transforms/css_minify.rs` | CssMinify (lightningcss) | VERIFIED | `StyleSheet::parse`, `minify()`, `to_css(PrinterOptions { minify: true })` |
| `ferro-assets/src/transforms/js_minify.rs` | JsMinify (swc Compiler::minify) | VERIFIED | `Compiler::new`, `GLOBALS.set`, `try_with_handler`, `JsMinifyOptions { compress: true, mangle: true }` |
| `ferro-assets/src/transforms/inject_before_tag.rs` | InjectBeforeTag (lol_html structural injection) | VERIFIED | `pub struct InjectBeforeTag`, `element!` + `end_tag!` hook, maps `</body>` → selector `body` |
| `ferro-assets/src/transforms/replace_tokens.rs` | ReplaceTokens (raw-bytes substitution, text content types) | VERIFIED | No `map_matching` used; direct iter over assets with binary-type guard; literal `replace_bytes` loop |
| `ferro-assets/src/transforms/image_transcode.rs` | ImageTranscode (image + ravif + rayon bounded pool) | VERIFIED | `ThreadPoolBuilder`, `num_threads`, `into_par_iter()`, `w <= src_width` guard, D-12 `{stem}-{width}w.{ext}` naming |
| `ferro-assets/src/transforms/responsive_images.rs` | ResponsiveImages (lol_html img→picture rewriter) | VERIFIED | `element!("img"`, `image/avif`, `parse_variant_name` D-12 round-trip, `html_encode_attr` for safe srcset |
| `ferro-assets/tests/passthrough_proof.rs` | SC-1 byte-identical passthrough proof (mechanic + real-transform) | VERIFIED | 5 tests: stub transforms + `json_file_unchanged_by_all_seven_real_transforms` (all 7 real built-ins) |
| `ferro-assets/tests/all_or_nothing.rs` | SC-5 atomic-failure proof | VERIFIED | 6 tests: mid-pipeline, first, last, error context, setup error, non-JS passthrough |
| `ferro-assets/tests/inline_script_fixture.rs` | SC-2 inline-script/style byte-correct regression proof | VERIFIED | 4 tests: byte-exact script body, byte-exact style body, whitespace reduction, Other passthrough |
| `ferro-assets/tests/image_transcode_test.rs` | SC-3 variant/no-upscale/bounded-concurrency proof | VERIFIED | 4 always-on + 2 cfg-gated (`#[cfg_attr(not(feature = "slow-tests"), ignore)]`) |
| `docs/src/features/ferro-assets.md` | User-facing feature page | VERIFIED | Exists at `docs/src/features/ferro-assets.md` |
| `ferro-assets/README.md` | crates.io README with security caveats | VERIFIED | "Zero C" present, `spawn_blocking` present, `Asset.path`/logical-key caveat present, `sanitize` caveat present |
| `docs/src/SUMMARY.md` | Asset Pipeline entry under Features | VERIFIED | Line 53: `- [Asset Pipeline](features/ferro-assets.md)` |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Pipeline::run` | `Transform::run` | `for transform in &self.transforms { current = transform.run(current)?; }` | WIRED | `pipeline.rs:83` — exact pattern present |
| `Asset::new` | `infer_content_type` | extension inference on ingest | WIRED | `asset.rs:42` — `let content_type = infer_content_type(&path);` |
| `HtmlMinify::run` | lol_html element handler with NO text handler on script/style | opaque inline content preservation | WIRED | `html_minify.rs:70-75`: `element!("script", ...)`, `element!("style", ...)` only; zero `text!("script"/"style")` confirmed |
| `ReplaceTokens::run` | all text ContentType variants (not map_matching) | raw byte find-and-replace with binary gate | WIRED | `replace_tokens.rs:70-92` — direct iter, `as_bytes()` loop, binary-type guard returns original |
| `ImageTranscode::run` | rayon ThreadPool num_threads(max_concurrent) | `pool.install` bounds parallelism | WIRED | `image_transcode.rs:158-181` — `ThreadPoolBuilder::new().num_threads(self.max_concurrent).build()`, `into_par_iter()` inside `pool.install` |
| `ResponsiveImages::run` | variant assets named `{stem}-{width}w.{ext}` | discovers variants from asset set, no shared state | WIRED | `responsive_images.rs:75-96` — `build_avif_index` scans asset set, `parse_variant_name` parses D-12 scheme |
| `docs/src/SUMMARY.md` | `docs/src/features/ferro-assets.md` | mdbook nav link | WIRED | `SUMMARY.md:53` — `[Asset Pipeline](features/ferro-assets.md)` |

---

### Data-Flow Trace (Level 4)

Not applicable. `ferro-assets` is a pure in-memory transformation library with no database or external data source. All data flows from caller-supplied `Vec<Asset>` in through `Pipeline::run` and back out as a transformed `Vec<Asset>`. The SC-1 real-transform test proves the end-to-end flow with all 7 production transforms.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC-1 passthrough (5 tests) | `cargo test -p ferro-assets --test passthrough_proof` | 5 passed, 0 failed | PASS |
| SC-5 atomicity (6 tests) | `cargo test -p ferro-assets --test all_or_nothing` | 6 passed, 0 failed | PASS |
| SC-2 inline-script byte-exact (4 tests) | `cargo test -p ferro-assets --test inline_script_fixture` | 4 passed, 0 failed | PASS |
| SC-3 no-upscale + naming (4+2 tests) | `cargo test -p ferro-assets --test image_transcode_test` | 4 passed, 2 ignored (slow-test gated), 0 failed | PASS |
| All library unit tests (46 tests) | `cargo test -p ferro-assets --lib` | 46 passed, 0 failed | PASS |

---

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| ASSET-F-01 | Plans 01, 04 | Pipeline scaffold + passthrough guarantee + workspace chores | SATISFIED | Pipeline + Transform + all-or-nothing run(), SC-1 green, workspace member, WAVE1A registered |
| ASSET-F-02 | Plans 02, 04 | HTML/CSS/JS minification with inline safety | SATISFIED | HtmlMinify (opaque script/style), CssMinify, JsMinify shipped; SC-2 inline fixture green |
| ASSET-F-03 | Plans 03, 04 | AVIF+JPEG responsive image transcoding, zero C deps, bounded concurrency | SATISFIED | ImageTranscode with ravif+image, rayon bounded pool (into_par_iter), no new C codec deps, SC-3 green |
| ASSET-F-04 | Plans 02, 03, 04 | inject_before_tag, replace_tokens, responsive_images | SATISFIED | InjectBeforeTag, ReplaceTokens, ResponsiveImages all shipped and tested |

---

### New-Crate Chores Verification

| Chore | Status | Evidence |
|-------|--------|---------|
| `ferro-assets` in root `Cargo.toml` members | VERIFIED | `Cargo.toml:32` — `"ferro-assets"` |
| `ferro-assets` in `WAVE1A_CRATES` (not WAVE1B) | VERIFIED | `publish.yml:211` — `WAVE1A_CRATES="... ferro-assets"` |
| `docs/src/features/ferro-assets.md` exists | VERIFIED | File present |
| `docs/src/SUMMARY.md` links the page | VERIFIED | Line 53 |
| `ferro-assets/README.md` with security notes | VERIFIED | zero-C-deps, spawn_blocking, Asset.path caveat, sanitize caveat all present |
| `version.workspace = true` (no separate bump) | VERIFIED | `ferro-assets/Cargo.toml:3` |
| Manual first-publish deferred | EXPECTED | STATE.md line 6 records deferral; Plan 04 Task 3 is a checkpoint:human-action |

---

### Anti-Patterns Found

| File | Pattern | Severity | Assessment |
|------|---------|----------|------------|
| `image_transcode.rs` | `debug_assert!(src.width() > 0, ...)` | Info | Correct use of debug_assert for a defensive guard on an unreachable path with valid images. Production-safe. |
| `passthrough_proof.rs` | `log.lock().unwrap()` in test | Info | Only in `#[cfg(test)]` code inside the test file. Not production code. Acceptable. |
| `image_transcode.rs` (unit tests) | `.expect("test jpeg encode")` | Info | Inside `#[cfg(test)]` test helpers only. Not production code. |

No blockers. No warnings. All `.unwrap()` / `.expect()` uses in production transform code paths have been confirmed absent (per plan acceptance criteria and confirmed by review).

---

### Code Review Fixes Verification

All three warnings from the 187-REVIEW.md were resolved in 187-REVIEW-FIX.md and confirmed in the final source:

| Finding | Fix | Status |
|---------|-----|--------|
| WR-01: Rayon concurrency was a no-op (sequential iter inside pool.install) | Changed to `into_par_iter()` inside `pool.install()` | FIXED — commit `e75b66f4`, `use rayon::prelude::*` + `into_par_iter()` at line 178 |
| WR-02: AVIF paths interpolated into srcset without HTML encoding | Added `html_encode_attr()` helper | FIXED — commit `36f0d32d`, `html_encode_attr` at line 192 of `responsive_images.rs` |
| WR-03: ReplaceTokens applied raw bytes to binary image assets | Added binary-type guard to skip JPEG/PNG/AVIF | FIXED — commit `339e1182`, `matches!(a.content_type, ContentType::Jpeg | ContentType::Png | ContentType::Avif)` guard at line 75 |

---

### Human Verification Required

None. All success criteria are verifiable programmatically. Tests are green. The manual first-publish to crates.io is a deferred operational step (not a code verification item) — explicitly recorded in STATE.md and Plan 04 as a user-action checkpoint deferred to the milestone push.

---

### Gaps Summary

None. All five ROADMAP success criteria are verified against the actual codebase. All four requirement IDs (ASSET-F-01 through ASSET-F-04) are satisfied. All new-crate chores are complete (workspace member, WAVE1A publish registration, docs page, README with security notes, SUMMARY link). All code review findings were fixed before verification.

---

_Verified: 2026-06-08T08:00:00Z_
_Verifier: Claude (gsd-verifier)_
