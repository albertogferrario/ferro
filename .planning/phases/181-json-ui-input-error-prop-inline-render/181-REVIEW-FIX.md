---
phase: 181-json-ui-input-error-prop-inline-render
fixed_at: 2026-06-01T00:00:00Z
review_path: .planning/phases/181-json-ui-input-error-prop-inline-render/181-REVIEW.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 181: Code Review Fix Report

**Fixed at:** 2026-06-01
**Source review:** .planning/phases/181-json-ui-input-error-prop-inline-render/181-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: render_json / render_json_with_errors skip merge_data — asymmetry with HTML paths

**Files modified:** `framework/src/json_ui/mod.rs`
**Commit:** 423d78bc
**Applied fix:** Added `let spec_with_data = spec.clone().merge_data(data.clone());` before the resolve call in both `render_json` (line 215) and `render_json_with_errors` (line 283), mirroring the identical pattern already present in `render_with_config` (line 84) and `render_with_errors_config` (line 269). The `effective_data` null-fallback logic is unchanged — it still references the caller's original `data` argument, not the merged copy, so the returned `"data"` field is not polluted with merged spec data.

Verified: `cargo build -p ferro-rs` completes cleanly (`Finished dev profile` in 27.74s).

_(Fixed in iteration 1.)_

---

### IN-01: Blessed-path doc example misleads on session-flash automation

**Files modified:** `docs/src/json-ui/forms.md`
**Commit:** f2e4b2cd
**Applied fix:** Replaced the placeholder `ValidationError::default()` GET handler example with a real working pattern. The new example reads the errors map from the session flash via `session().and_then(|s| s.get("_flash.old._validation_errors")).unwrap_or_default()` and calls `JsonUi::render_with_errors` directly. The subsequent prose was reworded to accurately describe the contract: the handler reads and passes the errors map explicitly; no automatic session reading occurs inside `render_with_errors` or `render_validation_error`.

Verified: `req.validation_errors()` does not exist on `Request` (confirmed by grepping `framework/src/http/request.rs`); `ValidationError` has no public constructor from `HashMap` (no `From` impl); `session()` is exported from `ferro` and `render_with_errors` accepts `&HashMap<String, Vec<String>>` — all APIs used in the example are real.

---

### IN-02: CheckboxList description paragraph uses wrong muted-text class

**Files modified:** `ferro-json-ui/src/render/form.rs`
**Commit:** 38cccc58
**Applied fix:** Changed `text-muted-foreground` to `text-text-muted` at line 594 in `render_checkbox_list`, making it consistent with every other description paragraph in the file (`render_input` line 316, `render_select` line 425, `render_checkbox` line 512, `render_switch` line 730).

Verified: `cargo build -p ferro-json-ui` completes cleanly (`Finished dev profile` in 10.43s).

---

_Fixed: 2026-06-01_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
