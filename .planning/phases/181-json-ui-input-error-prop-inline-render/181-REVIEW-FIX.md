---
phase: 181-json-ui-input-error-prop-inline-render
fixed_at: 2026-06-01T00:00:00Z
review_path: .planning/phases/181-json-ui-input-error-prop-inline-render/181-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 181: Code Review Fix Report

**Fixed at:** 2026-06-01
**Source review:** .planning/phases/181-json-ui-input-error-prop-inline-render/181-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: render_json / render_json_with_errors skip merge_data — asymmetry with HTML paths

**Files modified:** `framework/src/json_ui/mod.rs`
**Commit:** 423d78bc
**Applied fix:** Added `let spec_with_data = spec.clone().merge_data(data.clone());` before the resolve call in both `render_json` (line 215) and `render_json_with_errors` (line 283), mirroring the identical pattern already present in `render_with_config` (line 84) and `render_with_errors_config` (line 269). The `effective_data` null-fallback logic is unchanged — it still references the caller's original `data` argument, not the merged copy, so the returned `"data"` field is not polluted with merged spec data.

Verified: `cargo build -p ferro-rs` completes cleanly (`Finished dev profile` in 27.74s).

---

_Fixed: 2026-06-01_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
