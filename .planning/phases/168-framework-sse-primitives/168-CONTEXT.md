# Phase 168: Framework SSE Primitives - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; rationale logged per decision)

<domain>
## Phase Boundary

Add Server-Sent Events (SSE) streaming to the `framework` crate so handlers can push
events to the browser incrementally. Deliver four primitives in `framework/src/http/sse.rs`:

1. `SseEvent` — typed event with `data`/`event`/`id`/`retry`, serialized to the SSE wire format.
2. `SseStream` — wraps a `tokio::sync::mpsc` receiver and produces a **streaming HTTP body**;
   emits a `:ping` keep-alive comment every 15s while idle.
3. `HttpResponse::sse(...)` — factory that builds the streaming SSE response with correct headers.
4. A structural guarantee that SSE responses are never whole-body buffered (the property the
   ROADMAP phrased as "excluded from CompressionLayer").

This is a `framework`-crate-only change with no ferro-ai dependency (parallel-capable with
Phases 165–167, which are now complete). It unblocks Phase 169 (`StreamText` JSON-UI component
consuming an SSE URL) and is the transport that will later carry ferro-ai's `TokenStream`
(Phase 165) for streaming LLM output.
</domain>

<scope_premise_correction>
## ⚠ Scope Premise Correction (verified 2026-06-08 — MUST read before planning)

The ROADMAP Success Criteria for this phase were written against an **axum + tower-http** stack.
**That stack does not exist in this framework.** Verified by direct inspection:

- `framework/Cargo.toml` has **no `axum` and no `tower-http` dependency**. The HTTP server is
  built on **raw `hyper` 1** (`hyper::service::service_fn`, custom matchit-based `Router`).
- **No `CompressionLayer` (or any compression) exists anywhere** in the workspace
  (`grep -rniE 'compress|gzip|content-encoding|tower-http'` over `framework/` + `app/` is empty).
- The entire response pipeline is hardcoded to **`hyper::Response<Full<Bytes>>`** — a fully
  *buffered* body. `HttpResponse { status, body: Bytes, headers }` → `HttpResponse::into_hyper()`
  → `Full<Bytes>` (`framework/src/http/response.rs:168`). The serve loop, router, WS path, tenant
  and rate-limit middleware all name `Full<Bytes>` (6 files total).

**Consequences for the Success Criteria — reinterpretations the planner MUST adopt:**

- **SC#2 "implements `IntoResponse` for axum"** → there is no axum `IntoResponse`. The real
  requirement: `SseStream` produces a **streaming `hyper` body** (implements `http_body::Body`
  yielding `Frame<Bytes>`), and `HttpResponse::sse(...)` returns it through the framework's own
  hyper response path. (D-02/D-03)
- **SC#3 "excluded from `CompressionLayer` at the router level"** → there is no CompressionLayer
  to exclude from. The *intent* (SSE must stream, never be whole-body buffered/compressed) is met
  structurally by the streaming body type itself: a streaming body cannot be whole-body buffered.
  The test asserts **incremental frame delivery** (the property the exclusion was meant to
  protect), and — if compression is ever added — it must operate only on the buffered body
  variant and structurally skip the streaming variant. (D-06)

This is surfaced, not worked around (per the audit-and-fix-discrepancies convention). Do not
add axum or tower-http to satisfy the literal wording.
</scope_premise_correction>

<decisions>
## Implementation Decisions

### SseEvent (SC#1)
- **D-01:** `SseEvent` in `framework/src/http/sse.rs` with fields `data: String`, `event: Option<String>`,
  `id: Option<String>`, `retry: Option<u64>` (ms). A `to_wire()`/`Display` serializer emits, in order:
  `event: {e}\n` (if set), `id: {i}\n` (if set), `retry: {r}\n` (if set), then `data: {line}\n` for
  EACH line of `data` (multi-line data → multiple `data:` lines per the SSE spec), terminated by a
  blank line `\n`. Builder ergonomics: `SseEvent::data(s)` + `.event(..)`/`.id(..)`/`.retry(..)`. `[auto]`

### Streaming body (SC#2 — reinterpreted)
- **D-02:** `SseStream` wraps `tokio::sync::mpsc::Receiver<SseEvent>` and implements
  `http_body::Body<Data = Bytes, Error = ...>` (via `http_body_util` helpers), turning each received
  `SseEvent` into a `Frame<Bytes>`. NOT axum's `IntoResponse`. A pairing constructor
  `SseStream::channel(buffer)` → `(mpsc::Sender<SseEvent>, SseStream)` lets the handler hold the
  sender and push events from a spawned task. `[auto]`
- **D-03 (cross-cutting refactor — the phase's main structural work):** Generalize the server
  response body from the concrete `hyper::Response<Full<Bytes>>` to a body type that admits BOTH
  the existing buffered path and the new streaming path. **Recommended: an enum body**
  `FerroBody { Full(Full<Bytes>), Stream(SseStream) }` implementing `http_body::Body` (zero dynamic
  dispatch on the hot buffered path), used as `hyper::Response<FerroBody>`. Update the 6 files that
  name `Full<Bytes>` — `framework/src/{server.rs, http/response.rs, websocket.rs, static_files.rs,
  middleware/pre_route.rs, debug/mod.rs}` — to the generalized body. WebSocket's 101 handshake maps
  to `FerroBody::Full`. `HttpResponse::into_hyper()` returns `FerroBody::Full`. `[auto] recommended:
  enum body over BoxBody to keep the buffered path allocation/dispatch-free.`

### sse() factory + headers (SC#2)
- **D-04:** `HttpResponse::sse(stream: SseStream) -> HttpResponse` (or a dedicated streaming
  response constructor) sets `Content-Type: text/event-stream`, `Cache-Control: no-cache`,
  `Connection: keep-alive`, and `X-Accel-Buffering: no` (defeats nginx/reverse-proxy buffering),
  and carries the streaming body. NOTE the ROADMAP wrote `sse(sender, stream)`; the resolved
  ergonomic surface is `SseStream::channel()` → `(sender, stream)` then `HttpResponse::sse(stream)`
  — the planner may instead make `sse()` itself create the channel and return `(sender, response)`
  if cleaner. Exact arg shape is Claude's discretion within this contract. `[auto]`

### Keep-alive (SC#4)
- **D-05:** `SseStream` holds a `tokio::time::Interval` (15s). When the body is polled and no event
  is ready, once the interval ticks it yields a `:ping\n\n` comment frame. Any real event resets the
  idle window. Implemented in the `poll_frame` body impl (select between receiver and interval). `[auto]`

### Non-buffering guarantee + test (SC#3 — reinterpreted, SC#5)
- **D-06:** The structural guarantee = SSE responses use `FerroBody::Stream`, which is incapable of
  whole-body buffering. A unit test asserts the SSE response's body is the `Stream` variant (not
  `Full`). If compression is added in a future phase, it must match on `FerroBody::Full` only and
  pass `Stream` through untouched — documented as the structural rule. `[auto]`
- **D-07:** Integration test (SC#5) verifies token-by-token delivery against the framework's OWN
  hyper stack — NOT an axum test server. Recommended: drive `SseStream`'s `poll_frame` directly
  (deterministic — assert event N's frame is produced before event N+1 is sent through the channel),
  PLUS an optional end-to-end test that binds a `Server` on an ephemeral port, hits an SSE route
  with a `hyper` client, and reads frames incrementally. `[auto] recommended: body-level poll test
  for determinism + one e2e server test.`

### Module + exports
- **D-08:** Code in `framework/src/http/sse.rs`; re-exported via `framework/src/http/mod.rs` and the
  public surface in `framework/src/lib.rs` (`pub use http::{... SseEvent, SseStream}`), alongside the
  existing `HttpResponse` re-export. `[auto]`

### Claude's Discretion
- Exact `FerroBody` representation (enum vs `UnsyncBoxBody`) — D-03 recommends the enum; research/
  planner may pick BoxBody if the enum's `http_body::Body` impl proves unwieldy across all 6 files.
- Exact `HttpResponse::sse` signature (channel-returning vs stream-taking) within D-04's header contract.
- mpsc channel buffer size default (recommend a small bounded buffer, e.g. 16).
- Error type for the body `Error` associated type (recommend `std::io::Error` or `Infallible`-ish;
  the existing pipeline is infallible-bodied).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 168: Framework SSE Primitives" — goal + 5 Success Criteria (READ ALONGSIDE the Scope Premise Correction above; SC#2/#3 wording is stack-corrected here)
- `.planning/REQUIREMENTS.md` AISSE-01 — the SSE streaming requirement

### Existing framework stack (the REAL stack — extend, do not assume axum)
- `framework/src/http/response.rs` — `HttpResponse { status, body: Bytes, headers }` + `into_hyper() -> hyper::Response<Full<Bytes>>` (the seam D-03 generalizes; line ~168)
- `framework/src/server.rs` — hyper `service_fn` serve loop; `handle_request -> hyper::Response<Full<Bytes>>` (the body type to generalize; lines ~25, 94, 161, 188, 284)
- `framework/src/websocket.rs` — `handle_ws_upgrade`: precedent for a non-standard response path; its 101 handshake returns `Full<Bytes>` → maps to `FerroBody::Full`
- `framework/src/{static_files.rs, middleware/pre_route.rs, debug/mod.rs}` — the remaining `Full<Bytes>` sites to update under D-03
- `framework/src/http/mod.rs`, `framework/src/lib.rs` — public re-export points for `SseEvent`/`SseStream` (D-08)
- `framework/Cargo.toml` — already has `hyper 1` (full), `http-body-util`, `bytes`, `tokio` (full); SSE needs NO new deps (confirm `http-body-util` exposes `StreamBody`/`Frame`/`combinators`)

### Library docs (fetch live during research — do not rely on training cutoff)
- `hyper` 1.x `Response` + `http-body` `Body`/`Frame` traits — the streaming-body contract
- `http-body-util` 0.1 — `StreamBody`, `BodyExt`, `combinators::{BoxBody, UnsyncBoxBody}`, `Full`
- SSE wire-format spec (WHATWG `text/event-stream`) — `data:`/`event:`/`id:`/`retry:`/`:comment` framing, multi-line data rule

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HttpResponse` builder + `into_hyper()` — the conversion seam; gains a streaming sibling (D-03/D-04)
- `handle_ws_upgrade` (`websocket.rs`) — proves the serve path can carry a non-`Full<Bytes>` response; reference for how to thread a special-cased response through hyper
- `hyper 1` + `http-body-util` already present — streaming body needs no new crate

### Established Patterns
- Response = `hyper::Response<Full<Bytes>>` everywhere (6 files) — the single thing this phase generalizes; do it once, coherently (continuous-coherence principle)
- Builder pattern (`with_*`/consuming `self`) used across the framework — apply to `SseEvent` builders
- One Error enum per crate (thiserror) — reuse framework's existing error types; avoid a new public error if the body can be infallible

### Integration Points
- `framework/src/lib.rs` public surface — new `SseEvent`/`SseStream` exports
- The generalized `FerroBody` flows through `server.rs` serve loop → all middleware/tenant `service_fn`s that currently return `Full<Bytes>`
- Phase 169 `StreamText` will consume an SSE *route URL*; Phase 165 `ferro_ai::TokenStream` will later feed an `SseStream` at the application layer
</code_context>

<specifics>
## Specific Ideas

- The streaming-body generalization is the load-bearing work — get `FerroBody` right and the SSE
  surface is small on top of it. Keep the buffered path zero-cost (enum, not boxed) so the 99%
  non-streaming case is unaffected.
- SSE is the AI streaming story's transport: the same `SseStream` later carries token deltas from
  `ferro_ai`. Design `SseEvent`/`SseStream` so a `Stream<Item = String>` (token chunks) maps onto
  it trivially in a later phase.
</specifics>

<deferred>
## Deferred Ideas

- A real `CompressionLayer` (gzip/brotli) — does not exist today; out of scope. If added later it
  must structurally skip `FerroBody::Stream` (noted in D-06).
- Migrating the framework to axum — explicitly NOT this phase; the framework is intentionally raw-hyper.
- Wiring `ferro_ai::TokenStream` → `SseStream` (LLM token streaming) — application/Phase-169+ layer.
- `StreamText` JSON-UI component — Phase 169.
- Last-Event-ID reconnection / event replay buffer — future enhancement, not in AISSE-01.

None of the above is required by SC#1–#5.
</deferred>

---

*Phase: 168-framework-sse-primitives*
*Context gathered: 2026-06-08*
