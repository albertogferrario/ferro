---
phase: 212-crud-handler-proc-macros
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - ferro-macros/src/resource_get.rs
  - ferro-macros/src/resource_post.rs
  - ferro-macros/src/lib.rs
  - framework/src/validation/validator.rs
  - framework/src/tenant/scoped.rs
  - framework/src/tenant/mod.rs
  - framework/src/lib.rs
  - ferro-macros/tests/resource_macro.rs
  - ferro-macros/tests/ui/resource/pass/full_crud_reference.rs
  - ferro-macros/tests/ui/resource/pass/minimal_get.rs
  - ferro-macros/tests/ui/resource/pass/minimal_post.rs
  - ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.rs
  - ferro-macros/tests/ui/resource/fail/resource_get_not_async.rs
  - ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.rs
  - Cargo.toml
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
status: issues_found
---

# Phase 212: Code Review Report

**Reviewed:** 2026-06-13T00:00:00Z
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

Phase 212 ships `#[resource_get]` / `#[resource_post]` proc macros plus `Validator::validate_or_redirect` and the `TenantScoped` trait. The load-bearing security property — the generated lookup always passes `__tenant.id` — holds on **every** code path, including the `find` and `tenant` escape hatches: both the default and the `find`-override lookup emit `(__resource_id, __tenant.id)`, and there is no branch that drops the tenant id. No critical findings.

The macro hygiene is solid: nearly every referenced symbol is emitted through the `::ferro::…` absolute path (`#ferro`), and the only bare-prelude items (`format!`, `Some`, `Ok`/`Err`, `None`, `compile_error!`) are std-prelude macros/enums that resolve identically in any consumer. Span handling, async-fn rejection, unknown-attribute rejection, and unknown-placeholder rejection all produce clean `compile_error!`s rather than panics.

The findings below are a genuine codegen bug in the `tenant = "expr"` escape hatch (the bound type is wrong, producing a confusing downstream type error), two robustness gaps in the macro that surface as runtime/silent behavior instead of compile errors, and several quality items. None block the security guarantee.

## Warnings

### WR-01: `tenant = "expr"` escape hatch binds the wrong type and double-references at the call site

**File:** `ferro-macros/src/resource_get.rs:336` and `ferro-macros/src/resource_post.rs:379`
**Issue:** The escape-hatch arm emits `let __tenant: #tenant_ty = { #expr };`, where `#tenant_ty` is the inner fn's *reference* parameter type (`tenant_param.ty.clone()`, i.e. `&TenantContext`, extracted at `resource_get.rs:261` / `resource_post.rs:301`). The default arm instead correctly binds an **owned** value: `let __tenant: #ferro::TenantContext = ...`. Two consequences in the escape-hatch path:

1. The caller-supplied expression is forced to evaluate to `&TenantContext`, not `TenantContext` — undocumented and inconsistent with the default arm.
2. The delegation call site passes `&__tenant` (`resource_get.rs:417`, `resource_post.rs:487`). With `__tenant: &TenantContext` that produces `&&TenantContext`, but the inner fn parameter is `&TenantContext` — a type mismatch the consumer sees as an opaque error far from their attribute.

The `tenant` escape hatch is exercised by no pass-fixture, so this is currently uncaught by trybuild.

**Fix:** Bind the owned type in the escape-hatch arm, mirroring the default arm, so `&__tenant` yields `&TenantContext`:
```rust
let tenant_resolution = if let Some(ref expr_str) = attrs.tenant_expr {
    let expr: proc_macro2::TokenStream = expr_str.parse().unwrap_or_else(|_| {
        quote! { compile_error!("#[resource_get]: `tenant` expression failed to parse") }
    });
    quote! {
        let __tenant: #ferro::TenantContext = { #expr };
    }
} else { /* unchanged */ };
```
Then add a `pass/` fixture that uses `tenant = "..."` so the path is covered.

### WR-02: `find = "expr"` parse failure emits `compile_error!` in expression position, not statement position

**File:** `ferro-macros/src/resource_get.rs:348-350` and `ferro-macros/src/resource_post.rs:390-392`
**Issue:** On a malformed `find` path the fallback is `quote! { compile_error!("…") }`, which is then interpolated into `let __resource_opt = #find_path(__resource_id, __tenant.id).await…`. The result is `let __resource_opt = compile_error!("…")(__resource_id, __tenant.id).await…` — `compile_error!` is invoked, so the build does fail, but the surrounding tokens (`( … ).await.map_err(…)?`) are still parsed first, which can produce a second, noisier syntax error that masks the intended message. The same shape applies to the `tenant` expr fallback (`resource_get.rs:332`, `resource_post.rs:375`), though there it sits in a cleaner statement-ish position.

**Fix:** Validate the `find` / `tenant` strings with `syn::parse_str::<syn::Path>` / `syn::parse_str::<syn::Expr>` at attribute-parse time and return `e.to_compile_error()` with a precise span, rather than embedding `compile_error!` mid-expression:
```rust
let find_path: syn::Path = syn::parse_str(find_path_str)
    .map_err(|e| syn::Error::new(Span::call_site(),
        format!("#[resource_get]: `find` is not a valid path: {e}")))?;
```
(Requires threading the error out of `resource_get_impl`, e.g. by doing the parse in `parse_resource_get_attrs`.)

### WR-03: unterminated `{placeholder` is silently ignored instead of flagged

**File:** `ferro-macros/src/resource_get.rs:161-186` (`validate_url_placeholders`) and `resource_post.rs:206-231`
**Issue:** The scanner only acts when it finds a matching `}` (`if let Some(end_off) = …position(|&b| b == b'}')`). A template such as `on_miss = "/x/{id"` (missing close brace) falls through to `i += 1` and is accepted as a literal. `build_url_format` (`resource_get.rs:192`, `resource_post.rs:234`) has the identical structure, so the literal `{id` is then handed to `format!` at runtime — `format!("/x/{id")` is itself a malformed format string and would fail to compile, but the diagnostic points at generated code, not the attribute. A bare `{` with no close brace is a likely typo and should be rejected at the macro boundary with a clear message.

**Fix:** In `validate_url_placeholders`, when a `{` is seen but no `}` follows, return `Err(format!("#[resource_get]: unterminated `{{` placeholder in `{context}`"))`. Mirror in `resource_post`.

### WR-04: `on_miss` redirect uses 302; `redirect_to` success uses 303 — inconsistent and 302 is the wrong semantics for a POST miss

**File:** `ferro-macros/src/resource_post.rs:413-414` and `resource_post.rs:422-423`
**Issue:** The POST miss arm returns a **302** redirect (`.status(302)`), while the success/error envelope returns **303** (`handle_action_result`, `action.rs:340`). For a POST, a 302 invites the client to repeat the request as POST against the new location; 303 (See Other) is the correct status that forces a GET on the redirect target. The GET macro using 302 (`resource_get.rs:370`) is acceptable, but the POST miss arm should be 303 for consistency with the rest of the POST envelope and correct method semantics. The module rustdoc (`resource_post.rs:9`) and `lib.rs:702` both claim the miss path returns `ActionError::not_found` / a 404, which does not match the emitted 302/404 code either — see IN-03.

**Fix:** Change the POST `on_miss` arm to `.status(303)`.

## Info

### IN-01: `__ferro_params` is bound but never used in the generated body

**File:** `ferro-macros/src/resource_get.rs:397` and `resource_post.rs:461`
**Issue:** `let __ferro_params = __ferro_req.params().clone();` is emitted but no generated statement references it (id extraction uses `__ferro_req.param_as("id")` directly, and URL synthesis uses `__resource_id`). The clone is dead work and would trip `unused_variables` were it not for the leading `__`. It mirrors the `#[handler]`/`#[action]` shape (`utils.rs:167`) but those macros actually read `__ferro_params`.

**Fix:** Remove the binding from both generated wrappers unless a future param-binding feature needs it.

### IN-02: `fn_attrs` are replayed onto both the wrapper and the inner fn, duplicating `#[doc]` / arbitrary attributes

**File:** `ferro-macros/src/resource_get.rs:394,420` and `resource_post.rs:458` (inner fn at `498` omits them)
**Issue:** `resource_get` emits `#(#fn_attrs)*` on **both** the outer wrapper (line 394) and the inner fn (line 420). `resource_post` emits them only on the outer wrapper (line 458), not the inner. Beyond the inconsistency between the two macros, replaying user attributes such as `#[doc = "…"]`, `#[cfg(...)]`, or a second route attribute onto a generated inner fn can have surprising effects (duplicated docs, an inner fn conditionally compiled out while the wrapper that calls it is not). Doc comments belong on the public wrapper only.

**Fix:** Decide one policy and apply it to both macros — emit `#(#fn_attrs)*` on the public wrapper only; drop it from the inner fn in `resource_get`.

### IN-03: module rustdoc and `lib.rs` expansion docs claim a `404` / `ActionError::not_found` POST miss, but the code emits `302`/`404` `HttpResponse`

**File:** `ferro-macros/src/resource_post.rs:9`, `resource_post.rs:474`, `ferro-macros/src/lib.rs:663,702`
**Issue:** The `resource_post` module header (line 9) says "On miss: redirect (if `on_miss` given) or return `ActionError::not_found`." `lib.rs:663` says "omitted → `ActionError::not_found`" and the abridged expansion at `lib.rs:702` shows `None => return Err(::ferro::HttpResponse::new().status(404))`. The actual code (correctly, per the comment at `resource_post.rs:404-407`) emits an `HttpResponse` (302 or 404), never an `ActionError`. The inline comment is accurate; the module/`lib.rs` prose is stale.

**Fix:** Update the rustdoc in `resource_post.rs:9` and `lib.rs:663` to state the miss arm returns a 302 redirect (or 404 when `on_miss` is absent), matching the emitted code and IN-04 once the status is corrected.

### IN-04: duplicated helper code across the two macro modules

**File:** `ferro-macros/src/resource_get.rs:161-280` vs `ferro-macros/src/resource_post.rs:206-318`
**Issue:** `validate_url_placeholders`, `build_url_format`, `InnerParams`, and `extract_inner_params` are near-identical copies in both files (the only differences are the `#[resource_get]` vs `#[resource_post]` string in error messages). Fixing WR-03 — or any future placeholder/param-extraction bug — requires editing both copies and risks them drifting (they already differ trivially in the `InnerParams` type alias spelling: fully-qualified `syn::Pat` in `resource_post.rs:262` vs imported `Pat` in `resource_get.rs:220`).

**Fix:** Hoist the shared helpers into `utils.rs` (or a new `resource_common.rs`), parameterizing the macro-name string used in diagnostics.

### IN-05: `validate_or_redirect` takes `data` as a second argument that is almost always the same `&Value` already passed to `Validator::new`

**File:** `framework/src/validation/validator.rs:169-176`
**Issue:** The signature is `validate_or_redirect(self, data: &Value, url: …)`. The validator already holds `self.data: &'a Value` (line 31). Requiring the caller to pass `data` again (as in `full_crud_reference.rs:62-64`, where the same `&data` is passed to both `new` and `validate_or_redirect`) invites a mismatch where a caller passes a *different* value as old-input than was validated, silently flashing the wrong old input. This is a minor API-shape smell, not a bug.

**Fix:** Drop the parameter and reuse `self.data` for `with_old_input`:
```rust
pub fn validate_or_redirect(self, url: impl Into<String>)
    -> Result<(), crate::http::action::ActionError>
{
    let data = self.data.clone();
    self.validate().map_err(|e| e.with_old_input(&data).into_action_error(url))
}
```
If a deliberate "validate X, flash Y" case exists, keep the parameter but document why.

---

_Reviewed: 2026-06-13T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
