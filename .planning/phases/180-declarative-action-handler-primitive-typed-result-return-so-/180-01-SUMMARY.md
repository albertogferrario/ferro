---
phase: 180
plan: "01"
subsystem: framework/http
tags: [action, error-handling, http, runtime-types, request]
dependency_graph:
  requires: []
  provides:
    - ActionError
    - ActionKind
    - FlashVariant
    - ActionResult
    - IntoActionError
    - ActionResultExt
    - handle_action_result
    - ActionOverrides
    - Request::flash
    - Request::redirect_to
    - Request::action_overrides
  affects:
    - framework/src/http/mod.rs
    - framework/src/lib.rs
tech_stack:
  added:
    - thiserror (ActionError derive — already present in framework Cargo.toml)
    - form_urlencoded (percent-encode flash messages — already present)
    - tracing (log emission — already present)
  patterns:
    - pub(crate) runtime helper pattern (handle_action_result)
    - builder consuming pattern (with_flash, redirect_to on ActionError)
    - IntoActionError wrapper trait + blanket impl for long-tail Display types
    - same-origin URL validation (T-180-02)
    - log-injection sanitizer via is_control() (T-180-03)
key_files:
  created:
    - framework/src/http/action.rs (458 lines)
    - framework/tests/action_handler.rs (22 lines)
  modified:
    - framework/src/http/mod.rs (76 lines — added pub mod action + pub use)
    - framework/src/http/request.rs (713 lines — added action_overrides field + 3 methods)
    - framework/src/lib.rs (444 lines — added pub use http::action::{...})
decisions:
  - "sea_orm::DbErr From impl is unconditional (sea-orm is always a dependency in framework/Cargo.toml — no database feature gate exists)"
  - "is_same_origin rejects scheme-relative URLs (// prefix) in addition to absolute URLs, matching the pattern in response.rs::same_origin_path_from_referer"
  - "is_same_origin and sanitize_for_log promoted to pub(crate) to suppress dead_code lint (they are called from handle_action_result which itself has #[allow(dead_code)] until Plan 03)"
  - "FrameworkError constructor used in unit test: FrameworkError::internal() (no ::other() constructor exists)"
metrics:
  duration: "~25 minutes"
  completed: "2026-05-30"
  tasks: 2
  files: 5
---

# Phase 180 Plan 01: Action Runtime Types Summary

One-liner: `ActionResult = Result<(), ActionError>` with `From` impls for `String`, `&'static str`, `FrameworkError`, `sea_orm::DbErr`, `IntoActionError` blanket for everything else, and `Request::flash` / `Request::redirect_to` for success-side overrides.

## CI-Parity Gate Results

All four CI-parity commands exited 0 on the full workspace:

```
cargo fmt --all -- --check            ✓
cargo clippy --all --all-targets -- -D warnings  ✓
cargo build --all-features            ✓ (implied by test pass)
cargo test --all-features --all-targets -- --test-threads=1  ✓
```

Unit tests in `framework/src/http/action.rs`: **14 passing**
Integration test in `framework/tests/action_handler.rs`: **1 passing**

## Files Created / Modified

| File | Status | Lines |
|------|--------|-------|
| `framework/src/http/action.rs` | Created | 458 |
| `framework/tests/action_handler.rs` | Created | 22 |
| `framework/src/http/mod.rs` | Modified | 76 |
| `framework/src/http/request.rs` | Modified | 713 |
| `framework/src/lib.rs` | Modified | 444 |

## HttpResponse Builder Confirmation

The chain `crate::http::HttpResponse::new().status(303).header("Location", &location)` compiled directly:

- `HttpResponse::new() -> Self` at `response.rs:18`
- `.status(status: u16) -> Self` at `response.rs:94`
- `.header(name: impl Into<String>, value: impl Into<String>) -> Self` at `response.rs:121`

No substitution was needed. The plan's verified chain is exactly what compiled.

## FrameworkError Constructor Used

`FrameworkError::internal("framework boom")` was used in the `from_framework_error_impl` unit test. The plan suggested `FrameworkError::other(...)` which does not exist; `internal(message)` is the equivalent free-form constructor.

## Request Method-Name Clash Status

**No clash.** Neither `flash` nor `redirect_to` existed on `Request` before Plan 01. Both were introduced as new methods with no naming conflict.

## Threat Model Verification

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-180-01 (flash message injection) | Rustdoc on `ActionError` warns templates must HTML-escape `message` field | Present — `ActionError` struct doc and module-level doc both carry the warning |
| T-180-02 (open redirect) | `is_same_origin` applied to both `err.redirect_override` (error path) and `overrides.redirect_override` (success path) inside `handle_action_result`; scheme-relative `//` URLs are also rejected | Present — `is_same_origin` checks `starts_with('/') && !starts_with("//")`, unit test `is_same_origin_rejects_absolute` covers the `//evil.example/` case |
| T-180-03 (log injection) | `sanitize_for_log` strips `is_control()` chars; applied before all `tracing::error!` and `tracing::warn!` calls | Present — unit test `sanitize_strips_control_chars` verifies `\n`, `\t`, `\x00` are replaced with spaces |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `is_same_origin` accepted `//evil.example/` (scheme-relative URLs)**
- **Found during:** Task 1 unit test run
- **Issue:** The plan's sample implementation `url.starts_with('/')` passes the `is_same_origin_rejects_absolute` test assertion `!is_same_origin("//evil.example/")` because `//` starts with `/`.
- **Fix:** Changed to `url.starts_with('/') && !url.starts_with("//")`, matching the established pattern in `framework/src/http/response.rs::same_origin_path_from_referer`.
- **Files modified:** `framework/src/http/action.rs`
- **Commit:** `d733a8fb`

**2. [Rule 2 - Missing critical functionality] `#[allow(dead_code)]` and `pub(crate)` visibility for compile-time correctness**
- **Found during:** Task 1 clippy run
- **Issue:** `handle_action_result`, `is_same_origin`, `sanitize_for_log`, and `action_overrides` on Request were flagged as dead code since Plan 03 (the macro) doesn't exist yet. Clippy `-D warnings` treats these as errors.
- **Fix:** Promoted `is_same_origin` and `sanitize_for_log` to `pub(crate)` (they are legitimately reusable within the crate); added `#[allow(dead_code)]` to `handle_action_result` and `action_overrides` with inline comments citing Plan 03 as the future call site.
- **Files modified:** `framework/src/http/action.rs`, `framework/src/http/request.rs`
- **Commit:** `d733a8fb`

**3. [Rule 1 - Bug] `sea_orm::DbErr` From impl is unconditional (no `#[cfg(feature = "database")]` gate)**
- **Found during:** Task 1 implementation — reading framework/Cargo.toml
- **Issue:** The plan specifies `#[cfg(feature = "database")]` on the `From<sea_orm::DbErr>` impl, but the `database` feature does not exist in `framework/Cargo.toml` (`sea-orm` is an unconditional dependency). The gate in `error.rs:454` is also unconditional.
- **Fix:** Implemented the impl unconditionally, matching `error.rs`.
- **Files modified:** `framework/src/http/action.rs`
- **Commit:** `d733a8fb`

**4. [Rule 1 - Bug] `FrameworkError::other(...)` constructor does not exist**
- **Found during:** Task 1 unit test authoring
- **Issue:** The plan's unit test uses `FrameworkError::other("framework boom")` which is not a constructor in `framework/src/error.rs`. The available free-form constructor is `FrameworkError::internal(message)`.
- **Fix:** Used `FrameworkError::internal("framework boom")` in `from_framework_error_impl` test.
- **Files modified:** `framework/src/http/action.rs`
- **Commit:** `d733a8fb`

## Known Stubs

None. All public symbols are fully implemented. `handle_action_result` is complete but has no call site until Plan 03 ships the `#[action]` proc-macro.

## Commits

| Commit | Message |
|--------|---------|
| `d733a8fb` | feat(180-01): add ActionError, ActionResult, ActionOverrides, and handle_action_result |
| `2be6a414` | feat(180-01): add Request::flash, redirect_to, action_overrides field; re-export action types |

## Self-Check: PASSED

- `framework/src/http/action.rs` exists ✓
- `framework/tests/action_handler.rs` exists ✓
- Commit `d733a8fb` exists ✓
- Commit `2be6a414` exists ✓
- 14 unit tests pass in `http::action` ✓
- 1 integration test passes in `action_handler` ✓
- Full workspace `cargo test --all-features --all-targets` green ✓
