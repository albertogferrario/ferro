---
phase: 212-crud-handler-proc-macros
fixed_at: 2026-06-13T00:00:00Z
review_path: .planning/phases/212-crud-handler-proc-macros/212-REVIEW.md
iteration: 1
findings_in_scope: 9
fixed: 7
skipped: 2
status: partial
---

# Phase 212: Code Review Fix Report

**Fixed at:** 2026-06-13
**Source review:** `.planning/phases/212-crud-handler-proc-macros/212-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 9
- Fixed: 7 (WR-01, WR-02, WR-03, WR-04, IN-02, IN-03, IN-05)
- Skipped: 2 (IN-01, IN-04)

## Fixed Issues

### WR-01: `tenant = "expr"` escape hatch binds wrong type

**Files modified:** `ferro-macros/src/resource_get.rs`, `ferro-macros/src/resource_post.rs`, `ferro-macros/tests/ui/resource/pass/tenant_expr.rs`
**Commit:** c51330f0
**Applied fix:** Both escape-hatch arms now bind `let __tenant = { #expr };` without an explicit type annotation, so type inference yields the owned `TenantContext` the expression returns. The delegation `&__tenant` then correctly produces `&TenantContext`. The `unwrap_or_else` parse fallback was also upgraded to an early `return e.to_compile_error().into()` (consistent with WR-02 fix). Added `tests/ui/resource/pass/tenant_expr.rs` pass fixture exercising `tenant = "..."` on both macros — previously uncovered.

### WR-02: `find = "expr"` parse failure emits `compile_error!` in expression position

**Files modified:** `ferro-macros/src/resource_get.rs`, `ferro-macros/src/resource_post.rs`
**Commit:** c51330f0
**Applied fix:** Both `find` and `tenant` parse-failure paths now return `syn::Error::new(...).to_compile_error().into()` directly from the macro entry point, instead of embedding `compile_error!` mid-expression. This produces a clean, single diagnostic rather than secondary type errors from the surrounding tokens.

### WR-03: Unterminated `{placeholder` silently accepted

**Files modified:** `ferro-macros/src/resource_get.rs`, `ferro-macros/src/resource_post.rs`, `ferro-macros/tests/ui/resource/fail/resource_get_unterminated_placeholder.rs`, `ferro-macros/tests/ui/resource/fail/resource_get_unterminated_placeholder.stderr`
**Commit:** c51330f0
**Applied fix:** Both `validate_url_placeholders` implementations now detect a `{` with no closing `}` and return an `Err` with message `"unterminated '{' placeholder in '{context}' — missing closing '}'"`. Added fail fixture + `.stderr` snapshot (generated via `TRYBUILD=overwrite`, confirmed green without overwrite).

### WR-04: POST miss uses 302, should be 303

**Files modified:** `ferro-macros/src/resource_post.rs`
**Commit:** c51330f0
**Applied fix:** Both POST `on_miss` arms (static URL and interpolated URL) changed from `.status(302)` to `.status(303)`. GET macro redirects were left unchanged (302 is acceptable for GET miss redirects).

### IN-02: `fn_attrs` duplicated onto inner fn in `resource_get`

**Files modified:** `ferro-macros/src/resource_get.rs`
**Commit:** c51330f0
**Applied fix:** Removed `#(#fn_attrs)*` from the inner fn in `resource_get`. Attrs (doc comments, `#[cfg(...)]`, etc.) now only appear on the public outer wrapper, matching the existing `resource_post` behavior. Policy is consistent: outer wrapper only.

### IN-05: `validate_or_redirect` takes redundant `data` argument

**Files modified:** `framework/src/validation/validator.rs`, `ferro-macros/tests/ui/resource/pass/full_crud_reference.rs`, `ferro-macros/src/lib.rs`, `.planning/phases/212-crud-handler-proc-macros/212-CONTEXT.md`
**Commit:** afca4f46
**Applied fix:** Signature changed from `validate_or_redirect(self, data: &Value, url: …)` to `validate_or_redirect(self, url: …)`. The data reference is captured from `self.data` before `validate()` consumes `self`. All call sites updated: unit tests in `validator.rs`, `full_crud_reference.rs` trybuild fixture, `lib.rs` doc example. CONTEXT.md D-07 updated with a note explaining the refinement.

### IN-03: Stale rustdoc claims `ActionError::not_found` but code emits `HttpResponse`

**Files modified:** `ferro-macros/src/resource_post.rs`, `ferro-macros/src/lib.rs`
**Commit:** 42108bb4
**Applied fix:** Updated `resource_post.rs` module header (line 9) to say "303 redirect or 404 `HttpResponse`". Updated `lib.rs` optional-args doc for `on_miss` to say "303 redirect on lookup miss; omitted → 404 `HttpResponse`". Fixed inline generated-code comment "POST shape: ActionError" → "emits HttpResponse (303 redirect or 404)".

## Skipped Issues

### IN-01: `__ferro_params` bound but never used in generated body

**File:** `ferro-macros/src/resource_get.rs:397` and `resource_post.rs:461`
**Reason:** Not listed in the priority fixes. The binding uses `__` prefix (suppresses `unused_variables` warning) and mirrors the `#[handler]`/`#[action]` shape from `utils.rs:167`. Removing it is a safe future cleanup but has no correctness or diagnostic impact today.
**Original issue:** Dead `let __ferro_params = __ferro_req.params().clone()` clone emitted in both generated wrappers.

### IN-04: ~120-line helper duplication across the two macro modules

**File:** `ferro-macros/src/resource_get.rs:161-280` vs `ferro-macros/src/resource_post.rs:206-318`
**Reason:** Deferred — extraction would touch both macro files plus add a new `resource_common.rs` module, inflating the diff with no correctness gain. The WR-03 fix was applied symmetrically to both copies. The helpers are now correct; deduplication is a maintenance improvement for a future cleanup phase.
**Original issue:** `validate_url_placeholders`, `build_url_format`, `InnerParams`, `extract_inner_params` are near-identical copies in both files.

---

_Fixed: 2026-06-13_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
