---
phase: 180-declarative-action-handler-primitive-typed-result-return-so-
verified: 2026-05-30T18:00:00Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
---

# Phase 180: Declarative Action Handler Primitive — Verification Report

**Phase Goal:** Deliver a declarative `#[action]` proc-macro primitive that wraps the framework's POST-style redirect-on-success handler pattern. Handler bodies return `ActionResult = Result<(), ActionError>`, `?` works on `String`, `&'static str`, `FrameworkError`, and `sea_orm::DbErr`, success-side overrides are recorded via `req.flash(...)` / `req.redirect_to(...)` setters, and the macro produces a 303-redirecting `Response`-returning handler with flash writes, percent-encoded back-compat query string, open-redirect mitigation (T-180-02), and log-injection sanitization (T-180-03).

**Verified:** 2026-05-30T18:00:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | `ActionResult = Result<(), ActionError>` (D-03): user body returns `Ok(())`, `?` propagates via concrete `From` impls | VERIFIED | `grep -c 'pub type ActionResult = Result<(), ActionError>' framework/src/http/action.rs` = 1; 4 `From` impls confirmed (String, &'static str, FrameworkError, sea_orm::DbErr); `From<sea_orm::DbErr>` is unconditional (no feature gate, matching framework/Cargo.toml); `async move { #fn_block }.await` wrap in macro (Plan 04 fix) ensures `?` exits ActionResult scope |
| 2 | Success-side overrides via `Request` setters (D-02): `req.flash(...)` / `req.redirect_to(...)` | VERIFIED | `grep -c 'pub fn flash' framework/src/http/request.rs` = 1; `grep -c 'pub fn redirect_to' framework/src/http/request.rs` = 1; `pub(crate) fn action_overrides` present; `action_overrides` field in Request (6 references) |
| 3 | `#[action]` proc-macro exists, parses `redirect_to` (required) and `method` (optional), emits compile_error on missing/unknown attrs (D-05) | VERIFIED | `ferro-macros/src/action.rs` exists (confirmed); `pub fn action_impl` present; `fn parse_action_attrs` present; `to_compile_error` calls = 7; trybuild corpus: `missing_redirect_to.rs` and `unknown_attr_key.rs` fail fixtures pass; `concat!(module_path!()` = 2 (1 in doc + 1 in generated code, functional requirement met); `handle_action_result` referenced 7 times |
| 4 | Runtime helper `handle_action_result` is `pub #[doc(hidden)]`, writes session flash, builds 303 response, applies same-origin validation, sanitizes logs (D-06, D-07, T-180-02, T-180-03) | VERIFIED | `pub fn handle_action_result` at line 265 with `#[doc(hidden)]` at 264; `.status(303)` count = 2 (one per arm); `.header("Location"` count = 2; `is_control` present; `redirect_override` count = 16; `session_mut.*flash` pattern present; 10/10 integration tests pass including T-180-02 and T-180-03 behavioral assertions |
| 5 | D-08 project-agnostic rule: no `/accedi` literal in `framework/`, `ferro-macros/`, `ferro-mcp/`, or `docs/` sources | VERIFIED | `grep -rn '/accedi' framework/src/ ferro-macros/src/ docs/src/` = 0; `ActionError::unauthorized()` has `redirect_override = None` (confirmed by unit test `unauthorized_constructor_no_default_redirect`); `/accedi` appears only in `code_templates.rs` inside smoke-test assertion string literals (negation checks), not in template body |
| 6 | Test corpus: trybuild 6 pass + 3 fail fixtures with `.stderr` snapshots; integration tests >= 7 `#[tokio::test]` using TCP loopback `make_request()` | VERIFIED | `ls ferro-macros/tests/ui/action/pass/` = 6 files; `ls ferro-macros/tests/ui/action/fail/` = 6 files (3 `.rs` + 3 `.stderr`); `grep -c '#[tokio::test]' framework/tests/action_handler.rs` = 8 (>= 7); `TcpListener::bind` count = 1; `Request::default()` count = 0; `Request::test_default` count = 0; `.headers()` count = 2; `cargo test -p ferro-macros --test action_macro` = 1/1 passed; `cargo test -p ferro-rs --test action_handler` = 10/10 passed |
| 7 | Docs surface: `docs/src/the-basics/action-handlers.md` exists with `#[action(` >= 3, `req.flash`/`req.redirect_to` >= 2, security section, no `/accedi`, TOC entry, cross-link from controllers.md | VERIFIED | File exists; `#[action(` = 5 (>= 3); `req.flash|req.redirect_to` = 5 (>= 2); `## Security` = 1; `/accedi` = 0; `action-handlers.md` in SUMMARY.md = 1; `action-handlers` in controllers.md = 1 (handlers.md does not exist — cross-link correctly placed in controllers.md, the page that shows `#[handler]` usage) |
| 8 | MCP catalog: `ferro-mcp/src/tools/code_templates.rs` contains `action_handler` template under `"handler"` category with `#[action(`, `ActionResult`, `Ok(())`, imports listing `action`/`ActionError`/`ActionResult`/`Request`; no `ActionOk` in template body/imports; no `/accedi` in template body | VERIFIED | `action_handler` occurrences = 7 (>= 2); `#[action(redirect_to` in template body = present; `ActionResult` = 6; `ActionOk` appears only in smoke-test assertion string checking its absence; `/accedi` appears only in smoke-test assertion string checking its absence; `cargo test -p ferro-mcp --all-targets` = 227/227 passed including `action_handler_template_registered` |

**Score:** 8/8 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/http/action.rs` | `ActionError`, `ActionKind`, `FlashVariant`, `ActionResult`, `IntoActionError`, `ActionResultExt`, `handle_action_result`, `ActionOverrides` | VERIFIED | 458 lines; all public types confirmed; `handle_action_result` is `pub #[doc(hidden)]` (raised from `pub(crate)` in Plan 03 for macro-generated user-site visibility); 14 inline unit tests passing |
| `framework/src/http/request.rs` | `Request::flash`, `Request::redirect_to`, `pub(crate) action_overrides`, `action_overrides: ActionOverrides` field | VERIFIED | 713 lines; 3 methods confirmed; `action_overrides` referenced 6 times (field decl + 2 setters + getter + constructor(s)) |
| `framework/src/http/mod.rs` | `pub mod action;` registered | VERIFIED | `grep -c 'pub mod action' framework/src/http/mod.rs` = 1; `pub use action::` present |
| `framework/src/lib.rs` | Re-exports `ActionError`, `ActionKind`, `ActionResult`, `ActionResultExt`, `FlashVariant`, `IntoActionError`, `ferro_macros::action`; excludes `ActionOk`, `ActionOverrides` | VERIFIED | `pub use http::action::` = 1; `ActionError` = 1; `ActionOk` = 0; `ActionOverrides` = 0; `pub use ferro_macros::action` = 1 |
| `ferro-macros/src/action.rs` | `action_impl`, `parse_action_attrs`, `use crate::utils::`, `handle_action_result` dispatch, `concat!(module_path!())`, compile_error paths | VERIFIED | 272 lines; all criteria met; async-move body wrap present (load-bearing Plan 04 fix) |
| `ferro-macros/src/lib.rs` | `mod action;` and `#[proc_macro_attribute] pub fn action` registered | VERIFIED | `mod action` = 1; `pub fn action` = 1 |
| `ferro-macros/src/utils.rs` | `pub(crate) enum ParamKind`, `pub(crate) fn classify_param_type`, `pub(crate) fn generate_extraction`, `pub(crate) fn extract_param_name`, `pub(crate) fn is_primitive_type_name`, `pub(crate) fn ferro` | VERIFIED | All 6 items confirmed |
| `ferro-macros/src/handler.rs` | Uses `use crate::utils::` (no inlined helper copies) | VERIFIED | `fn classify_param_type` = 0 in handler.rs; `use crate::utils::` = 1 |
| `ferro-macros/tests/action_macro.rs` | Trybuild driver with `t.pass()` + `t.compile_fail()` | VERIFIED | Exists; 1/1 test passed |
| `ferro-macros/tests/ui/action/pass/` | 6 pass fixtures | VERIFIED | 6 `.rs` files confirmed |
| `ferro-macros/tests/ui/action/fail/` | 3 fail fixtures + 3 `.stderr` snapshots | VERIFIED | 3 `.rs` + 3 `.stderr` confirmed |
| `framework/tests/action_handler.rs` | 10 integration tests (8 tokio + 2 sync), TCP loopback, no `Request::default` | VERIFIED | 10/10 passing; `TcpListener::bind` present; `Request::default()` = 0; `Request::test_default` = 0 |
| `docs/src/the-basics/action-handlers.md` | User guide with security section, no `/accedi`, `#[action(` >= 3 | VERIFIED | 197 lines; all acceptance greps pass |
| `docs/src/SUMMARY.md` | TOC entry for `action-handlers.md` | VERIFIED | Count = 1 |
| `docs/src/the-basics/controllers.md` | Cross-link to `action-handlers.md` | VERIFIED | Cross-link present (handlers.md does not exist; controllers.md is the correct analog page) |
| `ferro-mcp/src/tools/code_templates.rs` | `action_handler` CodeTemplate under `"handler"` category | VERIFIED | All acceptance criteria met; 227/227 ferro-mcp tests pass |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `framework/src/lib.rs` | `framework/src/http/action.rs` | `pub use http::action::{...}` | VERIFIED | 6 types re-exported at crate root |
| `framework/src/lib.rs` | `ferro-macros::action` | `pub use ferro_macros::action` | VERIFIED | Confirmed present |
| `ferro-macros/src/action.rs::action_impl` | `ferro-macros/src/utils.rs::{ferro, classify_param_type, ...}` | `use crate::utils::` | VERIFIED | Single import line present |
| `generated handler body` | `framework/src/http/action.rs::handle_action_result` | `::ferro::http::action::handle_action_result(...)` emitted by macro | VERIFIED | 7 occurrences of `handle_action_result` in action.rs; `pub #[doc(hidden)]` makes it visible from user crates |
| `framework/src/http/action.rs::handle_action_result` | `framework/src/session/store.rs::flash` | `session_mut(|s| s.flash("_action", ...))` | VERIFIED | Pattern present in action.rs |
| `framework/src/http/action.rs::handle_action_result` | `framework/src/http/request.rs::Request::action_overrides` | reads success-side overrides after body returns | VERIFIED | `action_overrides()` called from `handle_action_result` |
| `ferro-macros/src/handler.rs` | `ferro-macros/src/utils.rs` | `use crate::utils::` (Plan 02 refactor) | VERIFIED | No helper duplication in handler.rs |

---

### Data-Flow Trace (Level 4)

Not applicable — this phase delivers a framework primitive (macro + runtime types), not a component that renders data from a store. The integration tests directly exercise the data flow: `ActionResult -> handle_action_result -> 303 Response with Location header`.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| trybuild corpus: 6 pass + 3 fail fixtures | `cargo test -p ferro-macros --test action_macro` | 1/1 passed (all 9 fixtures green) | PASS |
| Integration runtime: 10 tests covering happy path, overrides, T-180-02, T-180-03 | `cargo test -p ferro-rs --test action_handler --all-features` | 10/10 passed | PASS |
| Unit tests in action.rs: 14 tests covering constructors, From impls, sanitizer, is_same_origin | `cargo test -p ferro-rs --lib http::action` | 14/14 passed | PASS |
| ferro-mcp tests including action_handler_template_registered smoke test | `cargo test -p ferro-mcp --all-targets` | 227/227 passed | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| D-01 | Plans 01, 04 | `ActionError` with `message`, `kind`, `flash_variant`, `redirect_override`; constructors `msg`, `not_found`, `forbidden`, `unauthorized` | SATISFIED | All 4 fields confirmed in struct; all 4 constructors confirmed; builder methods `.with_flash()` and `.redirect_to()` present; unit tests cover all constructors |
| D-02 | Plans 01, 04 | Success-side overrides via `Request::flash` / `Request::redirect_to` setters; `ActionOk` dropped | SATISFIED | Both setters present on Request; `ActionOk` = 0 in action.rs; `ActionOk` = 0 in lib.rs; integration test `success_override_redirect_and_flash` passes |
| D-03 | Plans 01, 04 | `ActionResult = Result<(), ActionError>` | SATISFIED | Type alias confirmed; `Ok(())` is the user-facing success expression; 0 references to `ActionOk` anywhere in public surface |
| D-04 | Plans 01, 04 | `IntoActionError` wrapper trait + blanket `impl<E: Display>` for long-tail types; no blanket `From<E: Display>` (avoids orphan conflicts) | SATISFIED | `pub trait IntoActionError` = 1; `impl<E: std::fmt::Display> IntoActionError for E` = 1; `ActionResultExt::action_err()` extension present; question_mark_on_string pass fixture confirms `?` on `Result<_, String>` |
| D-05 | Plans 02, 03, 04 | `#[action(redirect_to = "...", method = "POST")]`; required attr compile_error, unknown key compile_error; reuses Plan 02 utils helpers | SATISFIED | Macro exists; attr parser confirmed; compile_error paths confirmed by trybuild fail fixtures; `use crate::utils::` = 1 in action.rs |
| D-06 | Plans 01, 04 | Session flash via `session.flash("_action", ...)` + back-compat query string `?error=...&msg=...` / `?success=...` | SATISFIED | `session_mut.*flash.*_action` pattern in handle_action_result; back-compat query string in both arms; integration test `error_path_default_redirect_with_msg` confirms `?error=generic&msg=<pct>` format |
| D-07 | Plans 01, 04 | `tracing::error!(handler = %name, msg = %safe_msg, kind = ?err.kind, ...)` with control-char sanitization | SATISFIED | `is_control` = 1 in action.rs; `sanitize_for_log` applied before tracing calls; unit test `sanitize_strips_control_chars` confirms `\n`, `\t`, `\x00` → spaces |
| D-08 | Plans 01, 04, 05, 06 | No `/accedi` literal in `framework/`, `ferro-macros/`, `ferro-mcp/`, `docs/` sources; `unauthorized()` has `redirect_override = None` by default | SATISFIED | `grep -rn '/accedi' framework/src/ ferro-macros/src/ docs/src/` = 0 matches; two occurrences in `code_templates.rs` are inside negation-assertion strings in the smoke test; unit test `unauthorized_constructor_no_default_redirect` asserts `redirect_override.is_none()` |

---

### Anti-Patterns Found

No blockers or stubs found. Noted items:

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-macros/tests/ui/action/pass/*.rs` | `#![allow(unused_imports)]` in all fixtures | Info | Documented in Plan 04 as a UX gap: the macro rewrites the user signature so source-level `Request`/`ActionResult` imports become unused post-expansion. Flagged for Phase 181 follow-up (have the macro emit bare `ActionResult`/`Request` in generated code so user imports are consumed naturally). Not a blocker. |
| `ferro-macros/src/action.rs` | `FormRequest` params emit `compile_error!` | Info | Documented behavior, not a stub. `FromRequest::from_request` takes `Request` by move; consuming `__ferro_req` inside extraction would create use-after-move at the `handle_action_result` dispatch. The `compile_error!` guides users to `req.form().await?` inside the body instead. The killer-feature handler shape (`req: Request` + path params) is unaffected. A follow-up phase can add `FromRequest::from_request_mut(&mut Request)`. |

---

### Human Verification Required

None. All acceptance criteria are mechanically verifiable and confirmed by the spot-check runs above.

---

### Gaps Summary

No gaps found. All 8 observable truths are verified, all required artifacts exist and are substantive and wired, all key links are confirmed, all requirement IDs (D-01 through D-08) are satisfied, and two focused behavioral spot-check suites (trybuild + integration) confirm runtime correctness.

The one intentional deviation from the original plan language — cross-link placed in `controllers.md` instead of a non-existent `handlers.md` — is correctly resolved: `handlers.md` does not exist in the repository, and `controllers.md` is the natural landing page for `#[handler]` users discovering `#[action]`.

The Phase 180 CI-parity gate was last confirmed green in the Plan 06 SUMMARY (`cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features --all-targets -- --test-threads=1` all exit 0, per commit `641df5a6`). The working tree is clean with no uncommitted changes. Spot-check runs in this verification session confirm no drift: trybuild 1/1, integration 10/10, unit 14/14, mcp 227/227.

---

_Verified: 2026-05-30T18:00:00Z_
_Verifier: Claude (gsd-verifier)_
