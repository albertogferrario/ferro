---
phase: 212-crud-handler-proc-macros
plan: "02"
subsystem: ferro-macros + framework/facade
tags: [proc-macro, tenant-scoped, crud, codegen, trybuild]
dependency_graph:
  requires: [212-01]
  provides: [resource_get, resource_post]
  affects:
    - ferro-macros/src/resource_get.rs
    - ferro-macros/src/resource_post.rs
    - ferro-macros/src/lib.rs
    - framework/src/lib.rs
    - ferro-macros/tests/resource_macro.rs
    - ferro-macros/tests/ui/resource/pass/minimal_get.rs
    - ferro-macros/tests/ui/resource/pass/minimal_post.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_not_async.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_not_async.stderr
    - ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.stderr
    - ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.rs
    - ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.stderr
tech_stack:
  added: []
  patterns:
    - inlined-handler-boilerplate (D-06 — no nested #[handler]/#[action])
    - named-inner-fn pattern (CRUD-05 — IDE jump-to-def preserved)
    - tenant-scoped-lookup-always-passes-tenant-id (T-212-01)
    - {param}-placeholder-compile-error (D-08)
    - trybuild-pass-fail-harness
key_files:
  created:
    - ferro-macros/src/resource_get.rs
    - ferro-macros/src/resource_post.rs
    - ferro-macros/tests/resource_macro.rs
    - ferro-macros/tests/ui/resource/pass/minimal_get.rs
    - ferro-macros/tests/ui/resource/pass/minimal_post.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_not_async.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_not_async.stderr
    - ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.rs
    - ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.stderr
    - ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.rs
    - ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.stderr
  modified:
    - ferro-macros/src/lib.rs
    - framework/src/lib.rs
decisions:
  - "Inline handler/action boilerplate (D-06) — no nested #[ferro::handler]/#[ferro::action] attribute; avoids double-extraction (Pitfall 1)"
  - "resource_post drops async move wrapping for inner fn call — direct .await preserves &mut __ferro_req alive for handle_action_result (Pitfall 3)"
  - "POST miss arm emits Err(HttpResponse) (not ActionError::into_response) — outer wrapper returns Response not ActionResult"
  - "InnerParams type alias added to satisfy clippy::type_complexity on extract_inner_params return type"
metrics:
  duration: "~17 minutes"
  completed: "2026-06-13"
requirements: [CRUD-01, CRUD-02, CRUD-05]
---

# Phase 212 Plan 02: Resource Macro Codegen Summary

**One-liner:** `#[resource_get]` and `#[resource_post]` fold the tenant-scoped CRUD prelude into a single attribute while keeping tenant and resource as real typed parameters in a named inner fn.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | trybuild harness + pass/fail fixtures | 9b6848e6 | resource_macro.rs, 2 pass + 3 fail fixtures |
| 2 | Implement #[resource_get] (CRUD-01) + facade + fail .stderr | 06d295dd | resource_get.rs, resource_post.rs (stub), lib.rs ×2, 2 .stderr |
| 3 | Complete #[resource_post] (CRUD-02) + fail .stderr snapshot | 773a6bde | resource_post_missing_redirect_to.stderr |

## Verification

- `cargo test -p ferro-macros --test resource_macro` — 5/5 green (2 pass + 3 compile-fail)
- `grep find_for_tenant(__resource_id, __tenant.id)` present in BOTH macro files
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-macros --all-targets -- -D warnings` — clean
- `cargo doc -p ferro-rs --no-deps` — no warnings
- `ferro::resource_get` and `ferro::resource_post` re-exported from facade

## CRUD Requirements Met

| Req | Description | Evidence |
|-----|-------------|---------|
| CRUD-01 | `#[resource_get]` folds typed-param + tenant + lookup + 404-on-miss | pass/minimal_get.rs compiles; inner fn `__edit_inner` with typed params |
| CRUD-02 | `#[resource_post]` folds prelude + validation-redirect envelope | pass/minimal_post.rs compiles; handle_action_result called |
| CRUD-05 | IDE experience: typed params, named inner fn | `format_ident!("__{}_inner", fn_name)` generates named inner fn; user params preserved as typed arguments |

## Security — T-212-01 (IDOR prevention)

Generated lookup in both macros:

```
<#resource_ty as #ferro::TenantScoped>::find_for_tenant(__resource_id, __tenant.id)
```

There is no code path in either file that emits an un-scoped lookup. The `find =` override still receives `(__resource_id, __tenant.id)` as arguments. Grep-verified in both files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Borrow-after-move in resource_post async move block (Pitfall 3)**
- **Found during:** Task 2 — trybuild `minimal_post.rs` failed with `E0382: borrow of moved value`
- **Issue:** The plan sketch used `async move { __inner_fn(&mut __ferro_req, ...).await }`, which moved `__ferro_req` into the async block; `handle_action_result(&mut __ferro_req)` then failed after the move.
- **Fix:** Replaced `async move { ... }.await` with a direct `.await` call on the inner fn: `let __action_result = __inner_fn(&mut __ferro_req, ...).await;`. The `&mut` borrow ends when the inner fn returns, before `handle_action_result` borrows again.
- **Files modified:** ferro-macros/src/resource_post.rs
- **Commit:** 06d295dd

**2. [Rule 1 - Bug] POST miss arm used ActionError::into_response() which does not exist**
- **Found during:** Task 2 design review (before compilation)
- **Issue:** Initial miss arm emitted `ActionError::not_found(...).into_response()`, but `ActionError` has no `into_response()` method — the outer wrapper returns `Response = Result<HttpResponse, HttpResponse>`, not `ActionResult`.
- **Fix:** Replaced with `Err(HttpResponse::new().status(302|404))` same as `resource_get` — the `ActionError` shape only applies inside `ActionResult`-returning bodies.
- **Files modified:** ferro-macros/src/resource_post.rs
- **Commit:** 06d295dd (pre-compilation fix)

**3. [Rule 1 - Bug] clippy::type_complexity on extract_inner_params return type**
- **Found during:** Task 2 clippy run
- **Issue:** Tuple return `(Box<Pat>, Box<Type>, Box<Pat>, Box<Type>, String)` triggered `clippy::type_complexity`.
- **Fix:** Added `type InnerParams = (...);` alias in both files.
- **Files modified:** ferro-macros/src/resource_get.rs, ferro-macros/src/resource_post.rs
- **Commit:** 06d295dd

**4. [Rule 1 - Bug] clippy::needless_borrows_for_generic_args on fn_token**
- **Found during:** Task 2 clippy run
- **Issue:** `syn::Error::new_spanned(&input_fn.sig.fn_token, ...)` flagged — `fn_token` already implements the required trait without borrowing.
- **Fix:** Removed the `&` in both files.
- **Files modified:** ferro-macros/src/resource_get.rs, ferro-macros/src/resource_post.rs
- **Commit:** 06d295dd

## Known Stubs

None. Both macros are fully implemented. The `minimal_post.rs` fixture uses `__form_url` in the body; the macro synthesizes and injects it as a hidden inner-fn parameter.

## Threat Flags

None beyond the T-212-01 surface documented in the plan's threat model. Both macro files enforce the tenant-scoped lookup by construction — no new surface introduced.

## Self-Check: PASSED

- ferro-macros/src/resource_get.rs — FOUND
- ferro-macros/src/resource_post.rs — FOUND
- ferro-macros/tests/resource_macro.rs — FOUND
- ferro-macros/tests/ui/resource/pass/minimal_get.rs — FOUND
- ferro-macros/tests/ui/resource/pass/minimal_post.rs — FOUND
- ferro-macros/tests/ui/resource/fail/resource_get_not_async.stderr — FOUND
- ferro-macros/tests/ui/resource/fail/resource_get_unknown_placeholder.stderr — FOUND
- ferro-macros/tests/ui/resource/fail/resource_post_missing_redirect_to.stderr — FOUND
- commit 9b6848e6 — FOUND
- commit 06d295dd — FOUND
- commit 773a6bde — FOUND
