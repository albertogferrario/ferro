---
phase: 214-scaffold-library-parity
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - framework/src/lib.rs
  - framework/src/validation/validator.rs
  - ferro-cli/src/commands/make_scaffold.rs
  - ferro-cli/src/templates/auth.rs
  - ferro-cli/src/templates/make.rs
  - ferro-cli/src/templates/scaffold.rs
  - ferro-cli/src/templates/files/backend/bootstrap.rs.tpl
  - ferro-cli/src/templates/files/backend/controllers/auth.rs.tpl
  - ferro-cli/src/templates/files/backend/controllers/profile.rs.tpl
  - ferro-cli/src/templates/files/backend/controllers/settings.rs.tpl
  - ferro-cli/tests/benchmark_new_project.rs
  - ferro-cli/tests/fixtures/benchmark/Dockerfile
  - ferro-mcp/src/tools/code_templates.rs
  - .github/workflows/ci.yml
  - .github/workflows/publish.yml
  - docs/src/the-basics/action-handlers.md
  - docs/src/features/database.md
findings:
  critical: 1
  warning: 2
  info: 3
  total: 6
status: issues_found
---

# Phase 214: Code Review Report

**Reviewed:** 2026-06-13
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 214 fixes scaffold-template↔published-library API drift and adds a per-PR
compile smoke test (`scaffold_builds_against_workspace_ferro`) plus a
post-publish Docker smoke test. The `error_response!` macro and
`Validator::with_error` additions are both correct.

However, the new smoke test only exercises the **`--api`** scaffold path
(`api_controller_template` / `api_controller_with_fk_template`), which is the
path the executor hardened with `ferro::error_response!`. The **full-stack**
(non-`--api`) controller templates — `scaffold_controller_template` and
`scaffold_controller_with_fk_template` in `ferro-cli/src/templates/scaffold.rs`
— still emit four `HttpResponse` constructor methods that do **not** exist on
the published `ferro` facade. A user running `ferro make:scaffold` without
`--api` on a full-stack project generates a controller that does not compile.
This is the exact class of drift the phase set out to eliminate, left uncaught
because the guard test does not cover this code path.

The `error_response!` macro itself is well-formed: it expands to a bare
`HttpResponse` value (no `Ok(...)` wrapper), making it valid in both
`.map_err(|e| ...)` and `.ok_or_else(|| ...)` arms where `?` unwraps the `Err`.
`.status($status as u16)` chains correctly against the real
`HttpResponse::status(self, u16) -> Self`. `$crate::HttpResponse` /
`$crate::serde_json` paths are hygienic. The documented info-disclosure aspect
(`$msg` often being `e.to_string()`) is developer-controlled and acceptable.

`Validator::with_error` is consistent with the rest of the builder API
(`mut self -> Self`, `impl Into<String>`), and its `pre_errors` are correctly
drained in `validate()`.

## Critical Issues

### CR-01: Full-stack scaffold controller templates emit non-existent `HttpResponse` methods

**File:** `ferro-cli/src/templates/scaffold.rs` (multiple lines in
`scaffold_controller_template` ~836-932 and
`scaffold_controller_with_fk_template` ~661-760)

**Issue:** The full-stack (non-`--api`) controller templates emit calls to four
`HttpResponse` constructors that do not exist on the published facade:

- `HttpResponse::internal_server_error(e.to_string())` — used in every handler
  (index/show/create/store/edit/update/destroy)
- `HttpResponse::not_found("{name} not found")` — used in `show`, `edit`,
  `update`
- `HttpResponse::bad_request(format!("Invalid form data: {{}}", e))` — used in
  `store`, `update`
- `HttpResponse::redirect(...)` / `HttpResponse::redirect("/{plural}")` — used
  in `store`, `update`, `destroy`

Verified against `framework/src/http/response.rs`: the `HttpResponse` impl
exposes `new/text/json/bytes/download/set_body/status/header/append_header/
cookie/ok/...` — none of `internal_server_error`, `not_found`, `bad_request`,
or `redirect`. (`not_found`/`bad_request` exist only on `FrameworkError` and
`ActionError`; `redirect` exists only on `Inertia` and `Redirect`.)

`make_scaffold.rs::generate_controller` dispatches to these full-stack
templates whenever `!api_only` (and to the FK variant when foreign keys are
present), so this is reachable production output, not dead code. A generated
full-stack controller will fail `cargo build` with `no function or associated
item named 'internal_server_error' found for struct 'HttpResponse'` (and the
three siblings).

The Phase 214 guard test does not catch this: `benchmark_new_project.rs`
invokes `make:scaffold` only with `--api`, so the full-stack templates are
never compiled by CI.

**Fix:** Bring the full-stack templates to parity with the `--api` templates,
which already use the correct symbols. For the error arms, mirror the API path:

```rust
// instead of:  .map_err(|e| HttpResponse::internal_server_error(e.to_string()))?
.map_err(|e| ferro::error_response!(500, e.to_string()))?
// instead of:  .ok_or_else(|| HttpResponse::not_found("{name} not found"))?
.ok_or_else(|| ferro::error_response!(404, "{name} not found"))?
// instead of:  .map_err(|e| HttpResponse::bad_request(format!("Invalid form data: {}", e)))?
.map_err(|e| ferro::error_response!(400, format!("Invalid form data: {}", e)))?
```

For the success redirects, emit an Inertia redirect (these are Inertia
controllers): `Inertia::redirect(&req, &format!("/{plural}/{{}}", result.id))`
(note `Inertia::redirect` takes `&Request`, so the handlers must keep `req` in
scope — `store`/`update`/`destroy` currently consume it only for
`SavedInertiaContext::from(&req)`; use `Inertia::redirect_ctx(&ctx, ...)` which
the auth/profile templates already use). Then extend
`benchmark_new_project.rs` to also scaffold one resource **without** `--api`
so the full-stack path is compiled by the guard test — otherwise this drift
class can silently regress again.

## Warnings

### WR-01: Compile smoke test covers only the `--api` template family

**File:** `ferro-cli/tests/benchmark_new_project.rs:45-105`

**Issue:** All three `make:scaffold` invocations in
`scaffold_builds_against_workspace_ferro` pass `--api`. The full-stack
templates (`scaffold_controller_template`,
`scaffold_controller_with_fk_template`) and the Inertia page templates are
never exercised by the per-PR guard. The phase's stated goal is "every PR
catches template↔library API drift before publish," but half the controller
template surface is outside the guard — which is why CR-01 survived.

**Fix:** Add at least one non-`--api` scaffold step (and ideally one with a
validated FK to exercise the `_with_fk` variant) before the `cargo build` step.
Because full-stack scaffolding also writes `frontend/src/pages/...`, ensure the
generated project either tolerates those files at `cargo build` time (they are
not Rust) or that the test asserts only the Rust crate compiles.

### WR-02: `Validator::passes()` / `fails()` ignore pre-seeded errors

**File:** `framework/src/validation/validator.rs:208-237`

**Issue:** `with_error` records into `self.pre_errors`, which `validate()`
correctly drains (lines 161-164). But `passes()` and `fails()` re-run only the
rule loop and never consult `pre_errors`. A validator that has a cross-field
error injected via `with_error` but no failing rule will report
`passes() == true` / `fails() == false`, contradicting `validate()` which would
return `Err`. The auth templates use `.validate()`, so this does not break the
scaffold, but the three methods now disagree about validity.

**Fix:** Make `passes()` account for pre-seeded errors, e.g. seed them into the
local `errors` map before the rule loop:

```rust
pub fn passes(&self) -> bool {
    let mut errors = ValidationError::new();
    for (field, message) in &self.pre_errors {
        errors.add(field, message.clone());
    }
    // ... existing rule loop ...
    errors.is_empty()
}
```

## Info

### IN-01: `error_response!` doc example uses `ferro::error_response!` while the doc comment lives in `ferro-rs`

**File:** `framework/src/lib.rs:390-409`

**Issue:** The doctests/examples reference `ferro::error_response!`, but the
crate is published as `ferro-rs` and the doc-comment examples elsewhere in this
file use `ferro_rs::`. This is purely cosmetic — consumers alias the dep to
`ferro`, and `#[macro_export]` exposes the macro at the crate root regardless —
but the example is technically inconsistent with the surrounding `ferro_rs::`
examples (e.g. `json_response!` at line 358). No functional impact.

**Fix:** None required; optionally align the example prefix with the others for
internal consistency.

### IN-02: Dockerfile `ARG FERRO_VERSION=0.2.55` default can drift from the workspace version

**File:** `ferro-cli/tests/fixtures/benchmark/Dockerfile:34`

**Issue:** The default `0.2.55` is overridden by CI
(`--build-arg FERRO_VERSION="$VERSION"` in `publish.yml:367`), so the CI path
is correct. But a local `docker build` with no `--build-arg` silently tests a
hardcoded old version that will diverge from the workspace as it bumps. The
comment documents the override but the stale default remains a footgun for
local runs.

**Fix:** Acceptable as-is given the comment; optionally drop the default so a
local build without `--build-arg` fails fast rather than testing a stale
version.

### IN-03: `make_scaffold.rs` doc comment duplicated verbatim

**File:** `ferro-cli/src/commands/make_scaffold.rs:1983-1990`

**Issue:** The doc block "Apply smart defaults for scaffold generation based on
project structure. / Returns (api_only, with_tests, with_factory) tuple ..." is
repeated twice consecutively above `apply_smart_defaults`. Harmless, but it is
dead duplication.

**Fix:** Delete the duplicated 4-line block.

---

_Reviewed: 2026-06-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
