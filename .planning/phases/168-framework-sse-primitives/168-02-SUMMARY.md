---
phase: 168-framework-sse-primitives
plan: "02"
subsystem: framework/http
tags: [sse, streaming, ferro-body, field-injection, security]
requirements: [AISSE-01]

dependency_graph:
  requires:
    - FerroBody enum (168-01, framework/src/http/body.rs)
    - SseEvent/SseStream skeleton (168-01, framework/src/http/sse.rs)
    - HttpResponse::sse()/sse_channel() factories (168-01, framework/src/http/response.rs)
  provides:
    - SSE field injection mitigation (event/id newline stripping)
    - FerroBody::is_streaming() helper
    - T-168-SEC field injection test
    - T-168-07 SSE factory headers assertion
    - T-168-08 FerroBody::Stream structural assertion
  affects:
    - framework/src/http/sse.rs (field injection stripping + tests)
    - framework/src/http/body.rs (is_streaming helper)

tech_stack:
  added: []
  patterns:
    - "String::replace(['\n', '\r'], '') for single-line SSE field sanitization"
    - "matches!(self, FerroBody::Stream(_)) for variant discrimination"
    - "#[tokio::test] required for any test that constructs SseStream (interval_at needs runtime)"

key_files:
  created: []
  modified:
    - framework/src/http/sse.rs
    - framework/src/http/body.rs

decisions:
  - "Strip \\n/\\r in builder setters (not caller responsibility) — primitive-level guarantee per T-168-SEC-01 threat register"
  - "T-168-07/08 placed in sse.rs test module (natural location for factory/integration assertions)"
  - "Tests changed from #[test] to #[tokio::test] — interval_at requires a Tokio runtime"

metrics:
  duration: "~10 minutes"
  completed: "2026-06-08T14:23:14Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 2
---

# Phase 168 Plan 02: SSE field injection mitigation + factory tests Summary

Complete-and-reconcile pass on top of Plan 01's SSE skeleton: SSE field injection stripped at the builder level (`event`/`id` setters), `FerroBody::is_streaming()` helper added, and tests T-168-07/08/SEC that Plan 01 deferred are now present and passing.

## What Was Built

### Task 1: SSE field injection mitigation + T-168-SEC test (commit 0b06f685)

**`framework/src/http/sse.rs`** — security hardening:
- `SseEvent::event()` setter strips `\n` and `\r` via `s.replace(['\n', '\r'], "")` before storing
- `SseEvent::id()` setter same stripping logic
- Module-level security note rewritten: from "application must sanitize" to "primitive strips in setters"
- Rustdoc on both setters explains the stripping and why `data` is exempt (multi-line = multiple `data:` lines, not injection)
- `T-168-SEC` test: `SseEvent::data("x").event("a\nb")` produces exactly one `event:` line (`event: ab`), not two; `id` with `\r` similarly sanitized

### Task 2: FerroBody::is_streaming + T-168-07/08 (commit 0acfe120)

**`framework/src/http/body.rs`** — `FerroBody::is_streaming()`:
- `pub fn is_streaming(&self) -> bool { matches!(self, FerroBody::Stream(_)) }`
- Rustdoc explains D-06 structural compression rule: future compression layers must check this and pass `Stream` through untouched

**`framework/src/http/sse.rs`** — T-168-07 and T-168-08 tests:
- `sse_factory_headers`: asserts all 4 required headers on `HttpResponse::sse_channel(16)` — `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `Connection: keep-alive`, `X-Accel-Buffering: no`
- `sse_response_is_stream_variant`: asserts `sse_channel(16)` body `is_streaming() == true`; `HttpResponse::text("hello").into_hyper()` body `is_streaming() == false`

## Test Results

| Test | Name | Status |
|------|------|--------|
| T-168-01 | `sse_event_wire_format` | pass |
| T-168-02 | `sse_event_multi_line_data` | pass |
| T-168-03 | `sse_stream_poll_delivers_event` | pass |
| T-168-04 | `sse_stream_keep_alive_ping` | pass |
| T-168-05 | `ferro_body_full_variant` | pass |
| T-168-06 | `ferro_body_stream_variant` | pass |
| T-168-07 | `sse_factory_headers` | pass |
| T-168-08 | `sse_response_is_stream_variant` | pass |
| T-168-09 | `sse_stream_incremental_delivery` | pass |
| T-168-SEC | `sse_field_injection_newline_stripped` | pass |
| T-168-10 | response::tests (15 tests, buffered-path regression) | pass |
| Total http:: | 105 tests | all pass |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tests T-168-07/08 needed #[tokio::test], not #[test]**
- **Found during:** Task 2 first test run
- **Issue:** `HttpResponse::sse_channel()` internally calls `SseStream::channel()` which calls `tokio::time::interval_at` — this panics ("there is no reactor running") when called outside a Tokio runtime context.
- **Fix:** Changed `#[test] fn sse_factory_headers` and `#[test] fn sse_response_is_stream_variant` to `#[tokio::test] async fn`.
- **Files modified:** `framework/src/http/sse.rs`
- **Commit:** included in 0acfe120

**2. [Rule 1 - Lint] Unused import `crate::http::body::FerroBody` in test module**
- **Found during:** Task 2 clippy run
- **Issue:** Added `use crate::http::body::FerroBody` to the test module but `is_streaming()` is accessed via the return value of `resp.body()` — no explicit `FerroBody` name needed.
- **Fix:** Removed the unused import.
- **Files modified:** `framework/src/http/sse.rs`
- **Commit:** included in 0acfe120

## Reconciliation Notes

Plan 01's executor implemented substantially more than its skeleton task described — `SseEvent`, `SseStream`, `HttpResponse::sse()`/`sse_channel()`, and tests T-168-01..04/09 were all present. Plan 02 executed as a pure complete-and-reconcile pass:

- Verified each Plan 02 acceptance criterion against existing code
- Added only what was absent: field injection stripping (T-168-SEC-01 threat), `is_streaming()` (D-06), T-168-07/08 tests
- No duplicate types or implementations created
- All 10 test identifiers (T-168-01..09 + SEC) now have assertions

## Known Stubs

None. All SSE types are fully functional. No placeholder data, hardcoded empty values, or deferred wiring.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. This plan reduces the attack surface by moving field injection prevention from the application layer into the primitive.

## Self-Check: PASSED

- `framework/src/http/sse.rs` — FOUND
- `framework/src/http/body.rs` — FOUND
- Task 1 commit 0b06f685 — FOUND
- Task 2 commit 0acfe120 — FOUND
- `cargo test -p ferro-rs --lib -- http::` — 105/105 pass
- `cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- All T-168-01..09 + T-168-SEC assertions present and passing
- No new dependencies added
- No `SseEvent`/`SseStream`/`FerroBody` duplicates
