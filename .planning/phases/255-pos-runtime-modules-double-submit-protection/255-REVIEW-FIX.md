---
phase: 255-pos-runtime-modules-double-submit-protection
fixed_at: 2026-07-05T13:55:17Z
review_path: .planning/phases/255-pos-runtime-modules-double-submit-protection/255-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 255: Code Review Fix Report

**Fixed at:** 2026-07-05T13:55:17Z
**Source review:** .planning/phases/255-pos-runtime-modules-double-submit-protection/255-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (fix_scope: critical_warning — 0 Critical, 3 Warning; 4 Info findings out of scope)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `disable_on_submit` demo in cassa.json cannot bind — button has no enclosing form

**Files modified:** `app/src/views/cassa.json`, `docs/src/features/write-kernel.md`
**Commit:** 31299b56
**Applied fix:** Wrapped `btn_confirm` in a new `confirm_form` Form element (`action: { handler: "/cassa/conferma", method: "POST" }`, button as child with `button_type: "submit"`), and pointed `cart_pane.children` at `confirm_form`. The button now both POSTs to the existing `/cassa/conferma` handler and satisfies `btn.closest('form')`, so the double-submit guard binds. The write-kernel.md layer-1 example was aligned to show the enclosing Form and gained an explicit sentence: the button must participate in a form (Form child or HTML5 `form` attribute via `ButtonProps.form`); a standalone Button with an element-level POST action is not form-wrapped. Locked decisions preserved: guard still binds on the form submit event, flag stays on `btn._submitted`, no new dispatcher setup function, `data-disable-on-submit` contract unchanged.

### WR-02: Double-submit latch fires even when the first submission was cancelled

**Files modified:** `ferro-json-ui/src/runtime/form_guards.rs`
**Commit:** 68a8958d
**Applied fix:** Added `if (e.defaultPrevented) return;` as the first statement of the submit listener in `initDisableOnSubmit`, per the review's suggested fix, with a comment explaining why (an earlier handler such as an inline `confirm()` cancelled the submission — no request was sent, so latching would leave the form unsubmittable until reload).

### WR-03: Unescaped attribute value in selector string can throw and abort the entire runtime

**Files modified:** `ferro-json-ui/src/runtime/mod.rs`, `ferro-json-ui/src/runtime/numpad.rs`, `ferro-json-ui/src/runtime/tiles.rs`
**Commit:** b27e4a78
**Applied fix:** Structural fix: rewrote the `ferroRuntime()` dispatcher to iterate a `setups` array and invoke each setup inside `try { setups[i](); } catch (err) { ... }` (ES5 style, manual for-loop), so one throwing concern can no longer abort the remaining setups. Updated the `dispatcher_invokes_every_setup` drift-guard test to assert every setup name is registered in the array and that the per-concern try/catch invocation is present. Also applied the review's optional hardening at both selector sites: `field = field.replace(/["\\\]]/g, '')` after `getAttribute` in `initNumpad` (numpad.rs) and `initQtyButton` (tiles.rs), degrading a malformed field name to a no-match early return instead of a `SyntaxError` throw.

## Verification

- `app/src/views/cassa.json` re-read and parsed with `JSON.parse` — valid.
- Assembled runtime JS bundle (all 15 modules + new dispatcher) syntax-checked via node `new Function` — valid.
- `cargo fmt --all -- --check` — clean.
- `cargo test -p ferro-json-ui --all-features` — 783 tests across 5 targets (753 unit + 5 + 11 + 8 integration + 6 doc-tests), 0 failed.

---

_Fixed: 2026-07-05T13:55:17Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
