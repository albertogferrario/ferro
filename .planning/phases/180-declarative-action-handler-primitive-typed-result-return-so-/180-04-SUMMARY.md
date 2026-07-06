---
phase: 180
plan: "04"
subsystem: framework + ferro-macros
tags: [test, trybuild, integration, killer-feature-lock]
requires:
  - ferro-macros/src/action.rs (Plan 03 macro — required test target)
  - framework/src/http/action.rs (Plan 01 runtime — handle_action_result is pub #[doc(hidden)])
provides:
  - ferro-macros/tests/action_macro.rs — trybuild driver locking 6 pass + 3 fail fixtures
  - framework/tests/action_handler.rs — 10 integration tests over handle_action_result
affects:
  - ferro-macros/src/action.rs — async-block body wrapper (load-bearing fix to ? propagation)
  - ferro-macros/Cargo.toml — adds trybuild, ferro-rs, tokio, sea-orm to [dev-dependencies]
  - framework/Cargo.toml — adds hyper-util, http-body-util to [dev-dependencies]
tech-stack:
  added:
    - trybuild = "1" (ferro-macros dev-dep)
    - sea-orm = "1.0" with default-features=false (ferro-macros dev-dep, for DbErr fixture)
  patterns:
    - trybuild ui test corpus (pass + compile_fail with .stderr snapshots)
    - TCP-loopback Request constructor (canonical pattern from tenant/mod.rs:166-208)
    - Location header read via HttpResponse::headers() -> &[(String, String)] (response.rs:142)
key-files:
  created:
    - ferro-macros/tests/action_macro.rs
    - ferro-macros/tests/ui/action/pass/minimal.rs
    - ferro-macros/tests/ui/action/pass/question_mark_on_string.rs
    - ferro-macros/tests/ui/action/pass/question_mark_on_framework_error.rs
    - ferro-macros/tests/ui/action/pass/question_mark_on_db_err.rs
    - ferro-macros/tests/ui/action/pass/success_override.rs
    - ferro-macros/tests/ui/action/pass/error_override.rs
    - ferro-macros/tests/ui/action/fail/missing_redirect_to.rs
    - ferro-macros/tests/ui/action/fail/missing_redirect_to.stderr
    - ferro-macros/tests/ui/action/fail/unknown_attr_key.rs
    - ferro-macros/tests/ui/action/fail/unknown_attr_key.stderr
    - ferro-macros/tests/ui/action/fail/non_action_result_return.rs
    - ferro-macros/tests/ui/action/fail/non_action_result_return.stderr
  modified:
    - ferro-macros/Cargo.toml (+5 lines: dev-dependencies block)
    - ferro-macros/src/action.rs (~7 lines: async-block body wrapper)
    - framework/Cargo.toml (+3 lines: hyper-util + http-body-util dev-deps)
    - framework/tests/action_handler.rs (~140 lines net: full integration corpus replaces scaffold)
key-decisions:
  - "No __test_handle_action_result shim added — Plan 03 already raised handle_action_result from pub(crate) to pub #[doc(hidden)]. Integration tests reach it directly via ferro::http::action::handle_action_result. Documented as deviation from Plan 04 Option A specification."
  - "Wrapped user macro body in 'async move { #fn_block }.await' inside Plan 03's macro. Original 'let __action_result: ActionResult = { #fn_block };' did NOT change ?'s propagation target — ? exits the enclosing async fn (returning Response), not the let-bound block. Without the async-block wrap the killer feature (? on String / FrameworkError / DbErr) silently fails to compile. Caught by every pass fixture in this plan."
  - "Fixtures use 'req: Request' (per macro doc comment), not '_req: &mut Request' (which the plan text incorrectly specified). The macro's classify_param_type recognises only the unwrapped Request shape; the &mut is generated internally by the action-specific extraction in action.rs::generate_action_extraction."
  - "Added '#![allow(unused_imports)]' to each fixture. The macro rewrites the user signature so Request/ActionResult source-level imports are unused in the post-expansion code. This is a UX gap (real users will see the same warning) — flag for a Phase 181 follow-up: have the macro emit bare 'ActionResult' / 'Request' in generated code so user imports are consumed naturally."
  - "Included the DbErr fixture by adding 'sea-orm = { version = \"1.0\", default-features = false }' as a ferro-macros dev-dep. sea-orm with default-features=false avoids pulling sqlx and the runtime — only the DbErr type and trait surface are needed."
  - "non_action_result_return.stderr captures a clean 'expected Result<(), ActionError>, found Result<HttpResponse, _>' from the wrapper let-binding. The plan's escape clause ('drop if hopeless') was not exercised — Rust's diagnostic is informative enough as-is."
requirements-completed:
  - D-01
  - D-02
  - D-03
  - D-04
  - D-05
  - D-06
  - D-07
  - D-08
duration: 90 min
completed: 2026-05-30
---

# Phase 180 Plan 04: trybuild UI corpus + integration tests — Summary

Lands the test corpus that mechanically protects the `#[action]` killer-feature ergonomics from future regressions. Surfaced and fixed a load-bearing Plan 03 macro bug (`?` was propagating to the outer `Response` instead of `ActionResult`).

## Duration

- Start: 2026-05-30 (Plan 03 completion)
- End: 2026-05-30
- Total: ~90 min (inline execution per `feedback_one_test_at_a_time` and `feedback_one_cpu_op_at_a_time`)
- Tasks: 2 (trybuild corpus + integration tests + Plan 03 macro fix)
- Files modified: 4
- Files created: 13 (1 driver + 6 pass fixtures + 3 fail fixtures + 3 .stderr snapshots)

## What Was Done

### Task 1 — Trybuild UI corpus

Added `[dev-dependencies]` to `ferro-macros/Cargo.toml`:

```toml
[dev-dependencies]
trybuild = "1"
ferro-rs = { path = "../framework" }
tokio = { version = "1", features = ["macros", "rt"] }
sea-orm = { version = "1.0", default-features = false }
```

Created `ferro-macros/tests/action_macro.rs` as the trybuild driver:

```rust
#[test]
fn action_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/action/pass/*.rs");
    t.compile_fail("tests/ui/action/fail/*.rs");
}
```

Created 6 pass fixtures + 3 fail fixtures under `ferro-macros/tests/ui/action/`.

The first `TRYBUILD=overwrite` run failed all 6 pass fixtures with `error: #[action] does not yet support FormRequest parameters` — Plan 03's `classify_param_type` was mis-classifying `&mut Request` as `FormRequest`. The macro's own doc comment specifies `req: Request` (the unwrapped form) — the plan text in Plan 04 had specified `&mut Request` which collides with the macro's contract. Updated all 9 fixtures to use `_req: Request` / `req: Request`.

The same first run also surfaced `error[E0277]: ? couldn't convert the error to HttpResponse` on `?` usage — Plan 03's macro wraps the body in `{ #fn_block }` (plain block), but `?` exits the enclosing async fn (returning `Response`), not the let-binding's typed scope. **Fix to Plan 03's macro**: wrap the body in `async move { #fn_block }.await` so `?` propagates to `ActionResult` (the async block's inferred Output via the let-binding's type annotation).

The macro fix is minimal and load-bearing — without it the killer feature does not work in any consumer crate.

After the macro fix and fixture adjustments, the second `TRYBUILD=overwrite` run captured clean snapshots:

- `missing_redirect_to.stderr` — exactly Plan 03's "`#[action]: \`redirect_to\` is required, e.g. #[action(redirect_to = \"/dashboard/foo\")]`" wording.
- `unknown_attr_key.stderr` — exactly Plan 03's "`#[action]: unknown attribute \`banana\` — supported keys: \`redirect_to\`, \`method\``" wording.
- `non_action_result_return.stderr` — clean `expected \`Result<(), ActionError>\`, found \`Result<HttpResponse, _>\`` from Rust's natural type checker. The plan's escape clause (drop if hopeless) was not needed.

The third run without `TRYBUILD=overwrite` confirmed all snapshots are locked.

### Task 2 — Integration corpus

Replaced the Plan 01/03 scaffold in `framework/tests/action_handler.rs` with a 10-test corpus covering:

| # | Test | What it locks |
|---|------|---------------|
| 1 | `public_surface_compiles` | Plan 01 public API in downstream crate scope |
| 2 | `macro_generated_handler_has_correct_type` | Plan 03 macro produces `fn(Request) -> impl Future<Output = Response>` |
| 3 | `happy_path_ok_unit_redirects_303` | D-03: `Ok(())` → 303 + `?success=` query string |
| 4 | `success_override_redirect_and_flash` | D-02: `req.redirect_to(...)` + `req.flash(...)` applied on Ok path |
| 5 | `error_path_default_redirect_with_msg` | D-01 + D-06: error path produces `?error=generic&msg=<pct>` |
| 6 | `error_path_with_redirect_override` | D-08: `ActionError::unauthorized().redirect_to(...)` honored (consumer supplies auth path) |
| 7 | `t_180_02_open_redirect_error_side_falls_back` | T-180-02: external URL rejected on error path |
| 8 | `t_180_02_open_redirect_success_side_falls_back` | T-180-02: external URL rejected on success path |
| 9 | `t_180_03_log_injection_message_percent_encoded` | T-180-03: control chars survive percent-encoding into URL (sanitizer unit-tested in Plan 01) |
| 10 | `warning_flash_variant_records_303_on_error_path` | `FlashVariant::Warning` doesn't change status (variant is flash-payload-only) |

`make_request()` uses the canonical TCP-loopback pattern from `framework/src/tenant/mod.rs:166-208` (verbatim — same hyper-util `TokioIo` + http-body-util `Empty<Bytes>` body + oneshot channel). Added `hyper-util` and `http-body-util` to `framework/Cargo.toml` `[dev-dependencies]` since integration tests don't see direct `[dependencies]`.

`location_header()` reads via the verified `HttpResponse::headers() -> &[(String, String)]` getter at `response.rs:142` — not via any `.header(name)` accessor.

### Visibility reconciliation (deviation from Plan 04)

Plan 04 prescribed adding `__test_handle_action_result` as a `#[doc(hidden)] pub` shim. **Not needed** — Plan 03 already raised `handle_action_result` from `pub(crate)` to `pub #[doc(hidden)]` to make it reachable from proc-macro-generated user code. Integration tests use the same reachable path: `ferro::http::action::handle_action_result`. Test file documents this in its module rustdoc.

## Acceptance Criteria Verification

### Plan-04 Truths

| Truth | Result |
|-------|--------|
| `ferro-macros/Cargo.toml` lists trybuild, ferro-rs, tokio | PASS (and sea-orm added for DbErr fixture) |
| `tests/action_macro.rs` exists driving `t.pass()` + `t.compile_fail()` | PASS |
| `compile_pass\|compile_fail` count ≥ 2 in driver | PASS (`t.pass()` + `t.compile_fail()`) |
| 6 pass fixtures exist | PASS (`ls pass/*.rs \| wc -l` = 6) |
| 3 fail fixtures + matching `.stderr` snapshots exist | PASS (`ls fail/*.rs \| wc -l` = 3, `.stderr` = 3) |
| `framework/tests/action_handler.rs` integration corpus replaces scaffold | PASS (10 tests, ≥7 `#[tokio::test]`) |
| `grep -c '#\[tokio::test\]' framework/tests/action_handler.rs` ≥ 7 | PASS (8 tokio tests) |
| `make_request()` uses TCP loopback, not `Request::default()` / `test_default` | PASS (`TcpListener::bind`, no Default/test_default refs) |
| `Request::default()` count = 0 | PASS |
| `Request::test_default` count = 0 | PASS |
| `TcpListener::bind` count ≥ 1 | PASS |
| Location header read via `HttpResponse::headers()` | PASS (`.headers()` in `location_header()`) |
| `.headers()` count ≥ 1 in test file | PASS |
| `cargo test -p ferro-macros --test action_macro` exits 0 | PASS |
| `cargo test -p ferro-rs --test action_handler` exits 0 | PASS (10/10 passing) |
| `cargo test --all-features --all-targets` exits 0 | PASS (full workspace green) |
| No `/accedi` literal in fixtures or test files | PASS (`grep -r /accedi ferro-macros/tests framework/tests` = 0) |

### CI-parity gate (CLAUDE.md)

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | PASS (after one `cargo fmt --all` run) |
| `cargo clippy --all --all-targets -- -D warnings` | PASS |
| `cargo build --all-features` | PASS (implicit via clippy and test) |
| `cargo test --all-features --all-targets -- --test-threads=1` | PASS (every test suite reports `ok. N passed; 0 failed`) |

## Deviations from Plan

1. **No `__test_handle_action_result` shim added.** Plan 03 had already raised `handle_action_result` from `pub(crate)` to `pub #[doc(hidden)]` for macro-generated user-code use. Integration tests reach the same path directly. The plan's recommended Option A was redundant.

2. **Fixtures use `req: Request` not `_req: &mut Request`.** Plan 04's example fixtures specified `&mut Request` parameter shape, but Plan 03's macro classifies that as a non-`Request` parameter type and emits the FormRequest compile error. The macro's own module-level doc comment uses `req: Request` (the canonical form). Fixtures updated accordingly — the macro's `generate_action_extraction` emits the `&mut __ferro_req` binding internally.

3. **Plan 03 macro bug fix (load-bearing).** The plan's macro generated `let __action_result: ActionResult = { #fn_block };` — a plain block, not an async block. `?` inside the block exits the enclosing async fn (which returns `Response`), so `?` would try to convert user errors via `From<E> for HttpResponse` instead of `From<E> for ActionError`. This silently broke the killer feature. **Fix**: wrap as `async move { #fn_block }.await`. The async block defines a `?`-propagation scope; the let-binding's `: ActionResult` type annotation forces the block's `Output = ActionResult`, so `?` uses the `From<E> for ActionError` impls. Caught by every `?`-using pass fixture in this plan.

4. **`#![allow(unused_imports)]` added to fixtures.** The macro rewrites the user signature so source-level `use ferro::{Request, ActionResult}` imports are "unused" post-expansion. The lint fires on real user code too — flagged for Phase 181 follow-up to have the macro emit bare `ActionResult` / `Request` in generated code so user imports are consumed naturally.

5. **Added `hyper-util` and `http-body-util` to `framework/Cargo.toml` `[dev-dependencies]`.** Integration tests don't see direct `[dependencies]` of the crate. Both crates already direct deps of framework — duplicated to `[dev-dependencies]` for `make_request()` to compile.

6. **Added `sea-orm` to `ferro-macros/Cargo.toml` `[dev-dependencies]`.** With `default-features = false` to avoid pulling sqlx and the async runtime. Enables `question_mark_on_db_err.rs` to import `sea_orm::DbErr` directly without going through ferro-rs.

## Known Stubs

None.

## Threat Flags

All Plan-04 threats from the threat register are mechanically locked:

- **T-180-02** open-redirect mitigation: two dedicated integration tests assert `https://evil.example/` does NOT appear in the Location header on either path.
- **T-180-03** log-injection mitigation: integration test confirms percent-encoded `%0A` in the URL; sanitizer's tracing-side correctness is covered by the Plan 01 unit test `sanitize_strips_control_chars` (still passing in this workspace test run).
- **T-180-01** flash-message injection: no integration assertion (requires live consumer template); Plan 05 (docs) is the mitigation surface.

## Next Step

Wave 4 — Plans 05 (action-handlers docs) and 06 (ferro-mcp code-templates entry) can now point users to a mechanically-locked, regression-protected `#[action]` surface.

## Self-Check: PASSED

- `ferro-macros/tests/` corpus exists: 1 driver + 6 pass + 3 fail + 3 stderr = 13 files.
- `framework/tests/action_handler.rs` contains 10 tests (2 sync + 8 tokio).
- `cargo test -p ferro-macros --test action_macro` — 9/9 trybuild fixtures green.
- `cargo test -p ferro-rs --test action_handler` — 10/10 tests green.
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --all --all-targets -- -D warnings` — clean.
- `cargo test --all-features --all-targets -- --test-threads=1` — every suite reports `ok. N passed; 0 failed`.
- Commit `f6f718d3` on master.
