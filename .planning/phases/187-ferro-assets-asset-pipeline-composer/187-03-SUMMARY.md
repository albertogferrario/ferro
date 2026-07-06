---
phase: 187-ferro-assets-asset-pipeline-composer
plan: "03"
subsystem: ferro-assets
tags: [new-crate, asset-pipeline, image-transcode, responsive-images, avif, rayon, lol_html, tdd, wave-3]
dependency_graph:
  requires:
    - 187-01 (Asset/ContentType/Transform/Pipeline/Error API)
    - 187-02 (HtmlMinify/CssMinify/JsMinify/InjectBeforeTag/ReplaceTokens)
  provides:
    - ImageTranscode transform (image + ravif + rayon ThreadPool, AVIF+JPEG variants, no-upscale)
    - ResponsiveImages transform (lol_html img→picture rewriter, D-12 variant discovery)
    - SC-3 integration test (no-upscale always-on; heavy AVIF encode + decode cfg-gated)
  affects:
    - ferro-assets/src/transforms/ (2 new transform modules, mod.rs updated)
    - ferro-assets/tests/image_transcode_test.rs (SC-3 integration tests)
tech_stack:
  added: []
  patterns:
    - rayon ThreadPoolBuilder::new().num_threads(max_concurrent).build() for bounded CPU parallelism
    - ravif Encoder::new().with_quality(f32).with_speed(4).encode_rgba(Img) — speed=4 default (not 1)
    - image JpegEncoder::new_with_quality + encode() for JPEG output
    - RGBA8 conversion via chunks_exact(4) — no bytemuck dep needed
    - lol_html element!("img") with el.before/el.after for <picture> wrapping
    - D-12 round-trip: rfind('-') + strip_suffix('w') to parse {stem}-{width}w.avif back to (stem, width)
    - cfg_attr(not(feature = "slow-tests"), ignore) for heavy encode tests (Phase 185/186 precedent)
key_files:
  created:
    - ferro-assets/src/transforms/image_transcode.rs
    - ferro-assets/src/transforms/responsive_images.rs
    - ferro-assets/tests/image_transcode_test.rs
  modified:
    - ferro-assets/src/transforms/mod.rs (added image_transcode + responsive_images modules)
decisions:
  - "RGBA8 pixel conversion via chunks_exact(4) instead of bytemuck: bytemuck is a transitive dep only; chunking is zero-dep and equally correct for tightly packed RGBA8 buffers"
  - "Original source asset retained in output set: fallback <img src> must still resolve; responsive_images discovers variants from the set and wraps the original in <picture>"
  - "parse_variant_name uses rfind('-') not find('-'): handles stems with hyphens (e.g. hero-banner-768w.avif → stem=hero-banner, width=768)"
  - "srcset contains asset path as emitted (no dir stripping): responsive_images uses the full path from the asset set, matching how the consumer would reference the file in URLs"
  - "SC-3 heavy tests: 2000×1500 synthetic JPEG generated in-test (no binary fixture committed)"
metrics:
  duration: "540s (~9 min)"
  completed: "2026-06-07T21:32:10Z"
  tasks: 2
  files: 4
---

# Phase 187 Plan 03: ImageTranscode + ResponsiveImages — AVIF+JPEG variants and img→picture rewriter

Pure-Rust AVIF+JPEG image transcoding with bounded rayon concurrency and lol_html responsive-images rewriting. The seven built-in transforms now ship.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | ImageTranscode — AVIF+JPEG, Lanczos3, rayon bounded pool, no-upscale | 7096ceeb | image_transcode.rs, mod.rs |
| 2 | ResponsiveImages (lol_html img→picture) + SC-3 integration test | 2d57bd63 | responsive_images.rs, mod.rs, image_transcode_test.rs |

## Acceptance Criteria Status

- [x] `grep -q 'ThreadPoolBuilder' ferro-assets/src/transforms/image_transcode.rs`
- [x] `grep -q 'num_threads' ferro-assets/src/transforms/image_transcode.rs`
- [x] `grep -qE 'w <= src_width' ferro-assets/src/transforms/image_transcode.rs` (no-upscale guard)
- [x] `grep -q 'Lanczos3' ferro-assets/src/transforms/image_transcode.rs`
- [x] `! grep -qE '\.unwrap\(\)' ferro-assets/src/transforms/image_transcode.rs` (no panic on decode)
- [x] no tokio import in image_transcode.rs (only in doc comment "No tokio runtime is required")
- [x] `cargo test -p ferro-assets --lib image_transcode` exits 0 (13 unit tests)
- [x] `grep -q 'element!("img"' ferro-assets/src/transforms/responsive_images.rs`
- [x] `grep -q 'image/avif' ferro-assets/src/transforms/responsive_images.rs`
- [x] `grep -qE 'w\.(avif|jpg)|-.*w\.' ferro-assets/src/transforms/responsive_images.rs`
- [x] `grep -q 'cfg_attr(not(feature = "slow-tests"), ignore)' ferro-assets/tests/image_transcode_test.rs`
- [x] `cargo test -p ferro-assets --lib responsive_images` exits 0 (8 unit tests)
- [x] `cargo test -p ferro-assets --test image_transcode_test` exits 0 (4 light pass, 2 heavy ignored)
- [x] `cargo clippy -p ferro-assets --all-targets -- -D warnings` clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ravif with_quality takes f32 not f64**
- **Found during:** Task 1 first compile
- **Issue:** RESEARCH.md Pattern 6 showed `with_quality(quality as f64)` but ravif 0.13 API takes `f32`
- **Fix:** Passed `quality` (already f32) directly without cast
- **Files modified:** ferro-assets/src/transforms/image_transcode.rs
- **Commit:** 7096ceeb

**2. [Rule 3 - Blocking] bytemuck not a direct dependency**
- **Found during:** Task 1 first compile
- **Issue:** RESEARCH.md suggested `bytemuck::cast_slice` for RGBA8 conversion, but bytemuck is a transitive dep not a direct one; `use bytemuck` fails without `cargo add bytemuck`
- **Fix:** Replaced with `chunks_exact(4).map(|c| RGBA8 { r, g, b, a })` — zero-dependency, equivalent correctness for tightly packed RGBA8 data
- **Files modified:** ferro-assets/src/transforms/image_transcode.rs
- **Commit:** 7096ceeb

**3. [Rule 1 - Bug] rgb::FromSlice import caused unused import warning**
- **Found during:** Task 1 first compile
- **Issue:** Initial code imported `use rgb::FromSlice` which then became unused after the bytemuck fix
- **Fix:** Removed the import entirely; pixel conversion done inline
- **Files modified:** ferro-assets/src/transforms/image_transcode.rs
- **Commit:** 7096ceeb

**4. [Rule 2 - Missing] Transform trait not in scope in integration test**
- **Found during:** Task 2 integration test compile
- **Issue:** `t.run(...)` fails without `use ferro_assets::transforms::Transform` in scope
- **Fix:** Added `Transform` to the imports in image_transcode_test.rs
- **Files modified:** ferro-assets/tests/image_transcode_test.rs
- **Commit:** 2d57bd63

## Known Stubs

None. Both transforms are fully implemented. The passthrough_proof.rs integration test references `ResponsiveImages::new()` which now resolves correctly — the stub from Plan 02 is resolved.

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced.

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-187-09 | mitigated: image::load_from_memory returns Result; rayon pool bounds concurrent encodes to default ≤2; no-upscale filter caps output dimensions |
| T-187-10 | mitigated: decode + ravif encode both return Result; errors surface as Error::transform; no .unwrap() on any encode path |
| T-187-11 | mitigated: rewriter.write/end return Result, mapped to Error::transform; no .unwrap() |
| T-187-12 | mitigated: w <= src_width guard runs first; SC-3 light test proves 400px source emits zero variants |

## Self-Check: PASSED

Files exist:
- ferro-assets/src/transforms/image_transcode.rs ✓
- ferro-assets/src/transforms/responsive_images.rs ✓
- ferro-assets/tests/image_transcode_test.rs ✓

Commits exist:
- 7096ceeb ✓
- 2d57bd63 ✓
