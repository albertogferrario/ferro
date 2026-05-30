---
phase: 180
plan: 01
subsystem: framework/http
tags: [action-handler, error-types, flash, redirect, security]
requires: []
provides:
  - framework/src/http/action.rs (ActionError, ActionOk, ActionResult, ActionKind, FlashVariant, IntoActionError, handle_action_result)
  - ferro crate root re-exports for the six public types
affects:
  - framework/src/http/mod.rs (pub mod action + pub use action::)
  - framework/src/lib.rs (pub use http::{...} extended)
tech-stack:
  added: []
  patterns:
    - thiserror::Error derive on ActionError struct
    - blanket impl<E: Display> IntoActionError for E (extension trait for long-tail errors)
    - concrete From impls for FrameworkError/String/&'static str/sea_orm::DbErr (no blanket From per E0119 resolution)
    - is_safe_redirect gate: starts_with('/') && !starts_with("//") (T-180-02)
    - sanitize_log_message: c.is_control() -> ' ' replacement (T-180-03)
    - handle_action_result: session.flash("_action") + ?error=...&msg=... back-compat query string + 303
key-files:
  created:
    - framework/src/http/action.rs
  modified:
    - framework/src/http/mod.rs
    - framework/src/lib.rs
key-decisions:
  - "NO blanket impl<T: IntoActionError> From<T> for ActionError — E0119 coherence conflict with concrete From impls; IntoActionError stays as extension trait only (RESEARCH §4.7 OQ-A)"
  - "ActionError::unauthorized() carries redirect_override = None (project-agnostic crates rule; no /accedi literal in framework source)"
  - "Test helpers use headers() iterator pattern (HttpResponse has no header_value() accessor; status_code() + headers() are the public API)"
  - "handle_action_result NOT re-exported at crate root — internal runtime helper called by #[action] macro via ::ferro::http::handle_action_result"
requirements-completed: [D-01, D-02, D-03, D-04, D-06, D-07, D-08]
duration: 5 min
completed: 2026-05-30
---

# Phase 180 Plan 01: Runtime Types Summary

Runtime types for the declarative action handler primitive — `ActionError`, `ActionOk`, `ActionResult`, `ActionKind`, `FlashVariant`, `IntoActionError`, and the `handle_action_result` dispatcher that writes session flash, builds back-compat query strings, and returns a 303 redirect — all with three security mitigations (T-180-01 rustdoc, T-180-02 open-redirect gate, T-180-03 log injection sanitizer) verified by 19 unit tests.

## Stats

- Duration: 5 min
- Started: 2026-05-30T00:56:15Z
- Completed: 2026-05-30T01:01:23Z
- Tasks: 2/2
- Files created: 1 (`framework/src/http/action.rs`)
- Files modified: 2 (`framework/src/http/mod.rs`, `framework/src/lib.rs`)
- Unit tests added: 19

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create framework/src/http/action.rs | f2950ccf | framework/src/http/action.rs, framework/src/http/mod.rs |
| 2 | Wire into mod.rs and re-export from lib.rs | 7dcb18ac | framework/src/lib.rs |

## Key Files Modified

### `framework/src/http/mod.rs` — lines added at top and after response re-exports
```rust
pub mod action;  // new module declaration

pub use action::{
    handle_action_result, ActionError, ActionKind, ActionOk, ActionResult, FlashVariant,
    IntoActionError,
};
```

### `framework/src/lib.rs` — final form of the `pub use http::{...}` block
```rust
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, ActionError, ActionKind,
    ActionOk, ActionResult, Cookie, CookieOptions, FlashVariant, FormRequest, FromParam,
    FromRequest, HttpResponse, InertiaRedirect, IntoActionError, MultipartForm, PaginationLinks,
    PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap, Response,
    ResponseExt, SameSite, UploadedFile,
};
```

Note: `handle_action_result` is NOT re-exported at the crate root. It is an internal runtime helper called by `#[action]`-generated code via `::ferro::http::handle_action_result`.

## Verification Results

```
cargo test -p ferro-rs --lib http::action

running 19 tests
test http::action::tests::action_error_msg_defaults ... ok
test http::action::tests::action_error_constructors_set_kind_and_keep_no_redirect ... ok
test http::action::tests::action_error_builders_consume_self ... ok
test http::action::tests::action_ok_from_unit_is_default ... ok
test http::action::tests::action_ok_builders ... ok
test http::action::tests::from_impls_message_round_trip ... ok
test http::action::tests::into_action_error_blanket_works ... ok
test http::action::tests::sanitize_log_message_strips_control_chars ... ok
test http::action::tests::handle_action_result_ok_default_303_to_redirect_to_with_success_1 ... ok
test http::action::tests::handle_action_result_ok_with_flash_key ... ok
test http::action::tests::handle_action_result_ok_with_safe_override ... ok
test http::action::tests::handle_action_result_err_default_kind ... ok
test http::action::tests::handle_action_result_err_kinds_in_query ... ok
test http::action::tests::handle_action_result_percent_encodes_message ... ok
test http::action::tests::handle_action_result_err_rejects_offsite_redirect_override ... ok
test http::action::tests::handle_action_result_err_rejects_scheme_relative_redirect_override ... ok
test http::action::tests::handle_action_result_err_rejects_javascript_redirect_override ... ok
test http::action::tests::handle_action_result_err_accepts_safe_override ... ok
test http::action::tests::handle_action_result_ok_rejects_offsite_redirect_override ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured
```

```
cargo clippy -p ferro-rs --lib --all-features -- -D warnings
→ Finished (no warnings)

cargo fmt -p ferro-rs -- --check
→ Clean
```

Project-agnostic audit:
```
grep -rnE '"/accedi"' framework/src/http/action.rs framework/src/http/mod.rs framework/src/lib.rs
→ zero matches (PASS)
```

## Deviations from Plan

**1. [Rule 1 - Bug] Test helpers adapted for actual HttpResponse API**

- **Found during:** Task 1 (before writing tests)
- **Issue:** Plan's test helpers assumed `HttpResponse::header_value(&str) -> Option<&str>`, which does not exist. The actual public API exposes `headers() -> &[(String, String)]` and `status_code() -> u16`.
- **Fix:** Replaced `resp.header_value("Location")` with `resp.headers().iter().find(|(k, _)| k.eq_ignore_ascii_case("Location")).map(|(_, v)| v.clone())` in both `location_of` and `status_of` test helpers. The plan explicitly anticipated this and instructed the executor to use whatever accessors exist.
- **Files modified:** `framework/src/http/action.rs` (tests only)
- **Verification:** All 19 tests pass

No other deviations — plan executed as written.

## Threat Mitigation Status

| Threat | Mitigation | Verification |
|--------|-----------|--------------|
| T-180-01 (flash XSS) | `# Security` rustdoc block on `ActionError` | `grep -c '# Security' framework/src/http/action.rs` → 1 |
| T-180-02 (open redirect) | `is_safe_redirect`: `starts_with('/') && !starts_with("//")` | 5 tests covering https://, //, javascript:, safe /path |
| T-180-03 (log injection) | `sanitize_log_message`: `c.is_control() -> ' '` | `sanitize_log_message_strips_control_chars` test passes |

## Next

Ready for Plan 02 — `#[action]` proc-macro attribute in `ferro-macros`.

## Self-Check: PASSED

- [x] `framework/src/http/action.rs` exists on disk
- [x] `framework/src/http/mod.rs` has `pub mod action;` and `pub use action::` block
- [x] `framework/src/lib.rs` has all 6 types in `pub use http::` block
- [x] Commits f2950ccf and 7dcb18ac exist in git log
- [x] 19 tests pass
- [x] Clippy clean
- [x] No `/accedi` literal in any modified file
