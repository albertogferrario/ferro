---
phase: 168
slug: framework-sse-primitives
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 168 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust standard `#[test]` + `#[tokio::test]` (already present; no new deps) |
| **Config file** | none — standard cargo test |
| **Quick run command** | `cargo test -p ferro-rs --lib -- http::sse` |
| **Full suite command** | `cargo test --all-features -p ferro-rs` |
| **Estimated runtime** | ~30s (lib tests); time-based keep-alive test uses `tokio::time::pause`/`advance` (no real wall-clock wait) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-rs --lib -- http::sse`
- **After every plan wave:** `cargo test --all-features -p ferro-rs`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` green
- **Max feedback latency:** ~30s

---

## Per-Task Verification Map

| Task ID | Behavior | Req / Decision | Threat Ref | Test Type | Automated Command | File | Status |
|---------|----------|----------------|------------|-----------|-------------------|------|--------|
| T-168-01 | `SseEvent::to_wire()` exact bytes (event/id/retry/data order) | AISSE-01 / D-01 | — | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_event_wire_format` | ❌ W0 | ⬜ |
| T-168-02 | Multi-line `data` → repeated `data:` lines | AISSE-01 / D-01 | T-168-SEC (header/data injection) | unit | `... sse_event_multi_line_data` | ❌ W0 | ⬜ |
| T-168-03 | `SseStream` poll delivers queued event bytes; then Pending | AISSE-01 / D-02 | — | unit | `... sse_stream_poll_delivers_event` | ❌ W0 | ⬜ |
| T-168-04 | Keep-alive `:ping\n\n` on idle interval fire | AISSE-01 / D-05 | — | `#[tokio::test]` (paused time) | `... sse_stream_keep_alive_ping` | ❌ W0 | ⬜ |
| T-168-05 | `FerroBody::Full` poll delegates to `Full<Bytes>` | AISSE-01 / D-03 | — | unit | `... ferro_body_full_variant` | ❌ W0 | ⬜ |
| T-168-06 | `FerroBody::Stream` poll delegates to `SseStream` | AISSE-01 / D-03 | — | unit | `... ferro_body_stream_variant` | ❌ W0 | ⬜ |
| T-168-07 | `HttpResponse::sse()` sets 4 required headers | AISSE-01 / D-04 | — | unit | `... sse_factory_headers` | ❌ W0 | ⬜ |
| T-168-08 | SSE response body is `Stream` variant, NOT `Full` (SC#3 reinterpreted) | AISSE-01 / D-06 | — | unit | `... sse_response_is_stream_variant` | ❌ W0 | ⬜ |
| T-168-09 | Incremental delivery: frame N before event N+1 sent (SC#5) | AISSE-01 / D-07 | — | unit (deterministic poll) | `... sse_stream_incremental_delivery` | ❌ W0 | ⬜ |
| T-168-10 | Buffered-path regression: `into_hyper()` still yields `FerroBody::Full` | AISSE-01 | — | unit | `cargo test -p ferro-rs --lib -- http::response::tests` | ✅ exists (adapt) | ⬜ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `framework/src/http/sse.rs` — `SseEvent`, `SseStream`, `HttpResponse::sse(...)` + inline unit tests (T-168-01..04, 07, 08, 09)
- [ ] `FerroBody` enum + `http_body::Body` impl (in `framework/src/http/body.rs` or `sse.rs` — planner decides) + unit tests (T-168-05, 06)
- [ ] Adapt existing `framework/src/http/response.rs` tests to the `FerroBody` return type (T-168-10) — no new framework, no new deps

*Existing `cargo test` + tokio runtime fully sufficient.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end browser `EventSource` consumption | AISSE-01 | Real browser SSE client behavior (auto-reconnect, event dispatch) is out of unit-test scope; deterministic delivery is covered by T-168-09 | Optional: bind a dev server with an SSE route, open an `EventSource` in a browser, observe incremental events + 15s pings in devtools Network tab |

*The deterministic body-level poll test (T-168-09) is the authoritative SC#5 evidence; the browser check is confirmatory only.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
