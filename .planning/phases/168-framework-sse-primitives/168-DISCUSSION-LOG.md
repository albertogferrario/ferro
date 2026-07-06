# Phase 168: Framework SSE Primitives - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 168-framework-sse-primitives
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per question)
**Areas discussed:** scope premise (stack reality), streaming body type, sse() factory shape, keep-alive, non-buffering guarantee + test strategy

---

## Scope Premise — stack reality (decisive finding)

| Option | Description | Selected |
|--------|-------------|----------|
| Assume axum + tower-http (ROADMAP wording) | Implement `IntoResponse`/`CompressionLayer` exclusion literally | |
| Verify, then build to the real hyper stack | Inspect deps; reinterpret SC#2/#3 against raw hyper | ✓ |

**Auto choice:** Verify-then-correct. `framework/Cargo.toml` has no axum/tower-http; server is raw hyper 1; no CompressionLayer exists; response pipeline is `hyper::Response<Full<Bytes>>` (buffered). SC#2/#3 reinterpreted in CONTEXT.
**Notes:** Per the "verify scope premises" + "audit/fix discrepancies" conventions — surfaced, not worked around. Do NOT add axum/tower-http.

## Streaming body type

| Option | Description | Selected |
|--------|-------------|----------|
| Enum `FerroBody { Full, Stream }` | Zero dynamic dispatch on buffered path; one type across serve loop | ✓ |
| `UnsyncBoxBody<Bytes, E>` | Boxed body, simpler signatures, heap + dynamic dispatch on every response | |
| Separate response path (like WS hijack) | SSE bypasses normal pipeline entirely | |

**Auto choice:** Enum body. Keeps the 99% buffered path allocation/dispatch-free; generalizes the 6 `Full<Bytes>` sites once.

## `HttpResponse::sse` factory shape

| Option | Description | Selected |
|--------|-------------|----------|
| `SseStream::channel()` → `(sender, stream)`, then `sse(stream)` | Caller holds sender, pushes from spawned task | ✓ |
| `sse()` creates channel, returns `(sender, response)` | One call | (acceptable alt) |

**Auto choice:** channel() pairing + `sse(stream)`; planner may pick the one-call variant within the header contract (text/event-stream, no-cache, keep-alive, X-Accel-Buffering: no).

## Keep-alive (SC#4)

| Option | Description | Selected |
|--------|-------------|----------|
| 15s `tokio::Interval` merged in `poll_frame`, `:ping\n\n` on idle | Resets on real events | ✓ |
| Separate ping task writing to the channel | Extra task, race on close | |

**Auto choice:** Interval inside the body poll. Matches SC#4 exactly.

## Non-buffering guarantee + test (SC#3 reinterpreted, SC#5)

| Option | Description | Selected |
|--------|-------------|----------|
| Structural: SSE uses `FerroBody::Stream` (non-bufferable) + body-level poll test + e2e server test | Deterministic; asserts incremental delivery | ✓ |
| Axum TestServer end-to-end only | Wrong stack; not available | |

**Auto choice:** Structural guarantee via the stream variant; body-level `poll_frame` ordering test (deterministic) plus one ephemeral-port hyper e2e test.

## Claude's Discretion

- `FerroBody` enum vs BoxBody; exact `sse()` signature; mpsc buffer size (≈16); body `Error` associated type.

## Deferred Ideas

- Real CompressionLayer (doesn't exist; must skip `Stream` if added later).
- axum migration (explicitly not this phase).
- `ferro_ai::TokenStream` → `SseStream` wiring (Phase 169+).
- `StreamText` component (Phase 169).
- Last-Event-ID reconnection / replay buffer.
