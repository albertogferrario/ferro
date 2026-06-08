---
phase: 168-framework-sse-primitives
plan: "01"
subsystem: framework/http
tags: [sse, streaming, ferro-body, response-pipeline, hyper]
requirements: [AISSE-01]

dependency_graph:
  requires: []
  provides:
    - FerroBody enum (framework/src/http/body.rs)
    - SseEvent wire serializer (framework/src/http/sse.rs)
    - SseStream Body impl (framework/src/http/sse.rs)
    - HttpResponse::sse() / sse_channel() factories (framework/src/http/response.rs)
    - hyper::Response<FerroBody> as the unified response body type across all 6 serve-path files
  affects:
    - framework/src/server.rs (WsInterceptor type, handle_request, health_response, serve_ferro_base_css)
    - framework/src/websocket.rs (handle_ws_upgrade)
    - framework/src/static_files.rs (try_serve_from_dir, try_serve_static_file)
    - framework/src/middleware/pre_route.rs (PreRouteResult type alias — pre-1.0 breaking change)
    - framework/src/debug/mod.rs (json_response + 6 pub fns)
    - framework/src/http/response.rs (into_hyper)
    - framework/src/lib.rs (public re-exports)

tech_stack:
  added: []
  patterns:
    - "Enum body (FerroBody) over BoxBody: zero heap allocation on buffered hot path"
    - "hyper::body::{Body,Frame,SizeHint} re-exports used instead of direct http-body dep"
    - "interval_at(Instant::now() + period, period) to defer first ping tick"
    - "response.map(|_incoming| FerroBody::Full(...)) for tungstenite WS 101 body mapping"

key_files:
  created:
    - framework/src/http/sse.rs
  modified:
    - framework/src/http/body.rs
    - framework/src/http/mod.rs
    - framework/src/http/response.rs
    - framework/src/server.rs
    - framework/src/websocket.rs
    - framework/src/static_files.rs
    - framework/src/middleware/pre_route.rs
    - framework/src/debug/mod.rs
    - framework/src/lib.rs
    - framework/src/json_ui/mod.rs

decisions:
  - "Used hyper::body::{Body,Frame,SizeHint} re-exports rather than adding http-body as a direct dep (no new deps)"
  - "FerroBody::Debug implemented manually (SseStream has no Debug)"
  - "writeln! used instead of write!(f, '...\\n') to satisfy clippy -D warnings"
  - "json_ui/mod.rs test helper has_content_type updated to FerroBody (blast radius from into_hyper change)"
  - "SseStream::channel_with_interval test-only constructor added for keep-alive ping test without tokio test-util feature"
  - "sse_channel() factory added to HttpResponse alongside sse(stream) for ergonomic handler use"

metrics:
  duration: "~35 minutes"
  completed: "2026-06-08T14:13:41Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 10
---

# Phase 168 Plan 01: FerroBody generalization + SSE backing types Summary

`FerroBody` enum replacing `Full<Bytes>` across all 6 hyper serve-path files, with `SseEvent`/`SseStream` skeleton types enabling streaming responses without dynamic dispatch on the buffered hot path.

## What Was Built

### Task 1: FerroBody + SseEvent + SseStream (commit 9bf5fe5e)

**`framework/src/http/body.rs`** — `FerroBody` enum:
- `Full(Full<Bytes>)` — buffered path, zero-cost
- `Stream(SseStream)` — streaming SSE path
- `impl http_body::Body` via `hyper::body::{Body,Frame,SizeHint}` re-exports (no new dep)
- `impl From<Full<Bytes>> for FerroBody`
- Manual `Debug` impl (SseStream has no Debug)
- Unit tests T-168-05 and T-168-06

**`framework/src/http/sse.rs`** (new file):
- `SseEvent` with WHATWG wire format: `data`, `event`, `id`, `retry` fields
- `Display`/`to_wire()` using `writeln!` (clippy-clean)
- `SseStream` wrapping `mpsc::Receiver<SseEvent>` + 15s `Interval` keep-alive
- `interval_at(Instant::now() + period, period)` — first tick deferred (avoids immediate ping pitfall)
- `poll_frame` via `tokio::select!`-equivalent: receiver first, then interval
- Both `Receiver` and `Interval` are `Unpin` — no `pin-project-lite` needed
- Unit tests: T-168-01 through T-168-04, T-168-09 (9 tests total)

### Task 2: 17-site blast radius refactor (commit 7a630667)

All 6 serve-path files updated from `hyper::Response<Full<Bytes>>` to `hyper::Response<FerroBody>`:

| File | Sites changed |
|------|--------------|
| `http/response.rs` | 1 (into_hyper) + sse()/sse_channel() factories added |
| `server.rs` | 5 (WsInterceptor type alias, ws_interceptor bound, handle_request, health_response, serve_ferro_base_css) |
| `websocket.rs` | 4 edit points (return type + 2 error bodies + `response.map(|_| FerroBody::Full(...))`) |
| `static_files.rs` | 2 (try_serve_from_dir + try_serve_static_file signatures + 1 body construction) |
| `pre_route.rs` | 1 (PreRouteResult type alias — documented pre-1.0 breaking change) |
| `debug/mod.rs` | 7 (json_response + 6 pub fns) |

Re-exports added: `http/mod.rs` and `lib.rs` expose `FerroBody`, `SseEvent`, `SseStream`.

## Test Results

| Test suite | Count | Status |
|-----------|-------|--------|
| `http::sse::tests` | 9 | all pass |
| `http::body::ferro_body_tests` | 2 | all pass |
| `http::response::tests` (T-168-10 regression) | 15 | all pass |
| `static_files::tests` | 12 | all pass |
| `server::ferro_base_css_route_tests` | 3 | all pass |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `http_body` not a direct dependency**
- **Found during:** Task 1 first build
- **Issue:** `use http_body::{Body, Frame, SizeHint}` failed — `http-body` is a transitive dep of `http-body-util` but not a direct dep of `framework/`
- **Fix:** Used `hyper::body::{Body, Frame, SizeHint}` re-exports (hyper 1 re-exports all three from `http-body 1.0.1`). No new dep added.
- **Files modified:** `framework/src/http/body.rs`, `framework/src/http/sse.rs`

**2. [Rule 2 - Missing Debug] `FerroBody` needs Debug for existing test helpers**
- **Found during:** Task 2 clippy run
- **Issue:** `json_ui/mod.rs` tests use `format!("{:?}", hyper.into_body())` which requires `FerroBody: Debug`. `SseStream` has no `Debug` impl so `#[derive(Debug)]` fails.
- **Fix:** Added manual `Debug` impl on `FerroBody` (Full variant delegates, Stream variant prints `"FerroBody::Stream(..)"`)
- **Files modified:** `framework/src/http/body.rs`

**3. [Rule 1 - Bug] `json_ui/mod.rs` test helper `has_content_type` type mismatch**
- **Found during:** Task 2 clippy run
- **Issue:** `has_content_type` parameter was `hyper::Response<Full<Bytes>>` — became a type error after `into_hyper()` was changed to return `FerroBody`
- **Fix:** Updated parameter type to `hyper::Response<crate::http::FerroBody>`
- **Files modified:** `framework/src/json_ui/mod.rs`

**4. [Rule 1 - Lint] `write!(f, "...\n")` clippy lint in SseEvent Display**
- **Found during:** Task 2 clippy run
- **Issue:** Clippy `-D warnings` rejects `write!` with format strings ending in `\n`; wants `writeln!`
- **Fix:** Replaced all `write!(f, "...\n")` with `writeln!(f, "...")` and bare `write!(f, "\n")` with `writeln!(f)`. Wire format output is identical.
- **Files modified:** `framework/src/http/sse.rs`

**5. [Rule 2 - Missing test helper] tokio `test-util` feature not available for `pause/advance`**
- **Found during:** Task 1 test run
- **Issue:** T-168-04 keep-alive ping test used `tokio::time::pause()` + `advance()` which require the `test-util` feature not enabled in `framework/Cargo.toml`
- **Fix:** Added `SseStream::channel_with_interval(buffer, duration)` test-only constructor; rewrote the test with a 10ms real interval + real sleep + `BodyExt::frame()` future
- **Files modified:** `framework/src/http/sse.rs`

**6. [Rule 2 - Missing module visibility] `body` module is private**
- **Found during:** Task 2 first build
- **Issue:** External modules (`server.rs`, `debug/mod.rs`, etc.) imported `crate::http::body::FerroBody` but `mod body` is private in `http/mod.rs`
- **Fix:** Changed all imports to use the re-export path `crate::http::FerroBody` (already exported via `pub use body::FerroBody` in mod.rs)
- **Files modified:** `server.rs`, `websocket.rs`, `static_files.rs`, `debug/mod.rs`, `middleware/pre_route.rs`

## Known Stubs

None. All SSE types are fully functional at the struct + trait implementation level. The `sse_channel()` / `sse()` factories are wired and return correct headers with `FerroBody::Stream` bodies. The only deferred work is Plan 02 content: the full test suite for `SseEvent` builder ergonomics and the `HttpResponse::sse_channel` handler usage pattern.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced beyond the planned SSE transport primitive.

## Self-Check: PASSED

- `framework/src/http/body.rs` — FOUND
- `framework/src/http/sse.rs` — FOUND
- Task 1 commit 9bf5fe5e — FOUND (`git log --oneline | grep 9bf5fe5e`)
- Task 2 commit 7a630667 — FOUND
- `cargo build -p ferro-rs --all-features` — passes
- `cargo test -p ferro-rs --lib -- http::response::tests` — 15/15 pass
- `cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings` — clean
- Zero `hyper::Response<Full<Bytes>>` in function return positions across 6 modified files
