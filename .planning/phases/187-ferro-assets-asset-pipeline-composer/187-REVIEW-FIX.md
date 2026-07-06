---
phase: 187-ferro-assets-asset-pipeline-composer
fixed_at: 2026-06-08T00:00:00Z
review_path: .planning/phases/187-ferro-assets-asset-pipeline-composer/187-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 187: Code Review Fix Report

**Fixed at:** 2026-06-08
**Source review:** `.planning/phases/187-ferro-assets-asset-pipeline-composer/187-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (WR-01, WR-02, WR-03, IN-01, IN-03)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: Rayon concurrency bounding was a no-op

**Files modified:** `ferro-assets/src/transforms/image_transcode.rs`
**Commit:** e75b66f4
**Applied fix:** Changed `images.into_iter()` to `images.into_par_iter()` inside
`pool.install(...)`. Added `use rayon::prelude::*;`. Updated struct-level doc comment to
accurately describe `into_par_iter()` usage. Also bundled IN-01 and IN-03 into this commit
(same file).

### IN-01: `with_max_concurrent(0)` silently mapped to "all CPUs"

**Files modified:** `ferro-assets/src/transforms/image_transcode.rs`
**Commit:** e75b66f4 (bundled with WR-01)
**Applied fix:** Added `n.max(1)` in `with_max_concurrent` so 0 is clamped to 1. Added doc
note explaining that rayon treats `num_threads(0)` as "all logical CPUs".

### IN-03: `resize_to_width` had no guard for zero-width source image

**Files modified:** `ferro-assets/src/transforms/image_transcode.rs`
**Commit:** e75b66f4 (bundled with WR-01)
**Applied fix:** Added `debug_assert!(src.width() > 0, ...)` and `let src_w =
src.width().max(1)` as defensive guard before the aspect-ratio division, preventing NaN
propagation from a degenerate zero-width image.

### WR-02: AVIF paths interpolated into `srcset` without HTML encoding

**Files modified:** `ferro-assets/src/transforms/responsive_images.rs`
**Commit:** 36f0d32d
**Applied fix:** Added `html_encode_attr(s: &str) -> String` helper that escapes `&`, `"`,
`<`, `>` (in that order). Used it in the srcset builder: `format!("{} {w}w",
html_encode_attr(p))`.

### WR-03: `ReplaceTokens` corrupted binary image assets

**Files modified:** `ferro-assets/src/transforms/replace_tokens.rs`
**Commit:** 339e1182
**Applied fix:** Added `use crate::asset::ContentType;` and a `matches!(a.content_type,
ContentType::Jpeg | ContentType::Png | ContentType::Avif)` guard at the top of the `map`
closure that returns `Ok(a)` unchanged for binary types. Updated module doc and struct doc to
document the text-only scope (D-16 intent preserved for Html/Css/Js/Other).

## Skipped Issues

None — all in-scope findings were successfully fixed.

---

_Fixed: 2026-06-08_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
