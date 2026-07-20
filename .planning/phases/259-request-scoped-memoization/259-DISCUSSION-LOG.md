# Phase 259: Request-scoped memoization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-21
**Phase:** 259-request-scoped-memoization
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Store access mechanism, Out-of-scope behavior, Argument keying, Coalescing/error semantics, Render-path wiring

---

## Milestone-pointer fix (pre-discussion)

`init phase-op 259` initially reported `phase_found: false`: STATE.md `milestone:` was
still `v16.6` (complete) while phase 259 belongs to v17.0, and the ROADMAP overview bullet
for v17.0 carried `📋` (planned) rather than `🚧` (active) — the documented milestone-pointer
drift. Fixed by pointing STATE.md at `v17.0` (`status: in progress`), marking v16.6 `✅` and
v17.0 `🚧` in the ROADMAP overview so exactly one milestone is active. Re-ran init → phase
found (`Request-scoped memoization`, slug `request-scoped-memoization`).

---

## Store access mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Task-local ambient `MemoStore` | `tokio::task_local!` `MEMO_STORE` scoped per request, mirroring `TENANT_CONTEXT`; `#[memoize]` reads ambient store, no `&Request` param | ✓ |
| Explicit `&Request`/`&Cx` parameter | Memoized fn must accept a context handle threaded from the handler | |
| `Request` extensions type-map | Store in `Request::insert`/`get` (spec's literal sketch) | |

**Chosen:** Task-local ambient (D-01).
**Notes:** Render-path fetches are deep in the call tree with no `Request` handle; ferro
already uses task-locals for tenant + session, so this is the conceptually coherent surface.
Explicit-param rejected as contradicting the ergonomic goal and unreachable from the render
pass. Extensions type-map rejected for the same render-path reason (kept as documented fallback).

## Out-of-scope behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful no-op | `try_with` → `Err` → run body un-memoized; no panic | ✓ |
| Panic outside request scope | Fail loudly if no store present | |
| Lazily create a detached per-call store | Each out-of-scope call gets a throwaway store | |

**Chosen:** Graceful no-op (D-02).
**Notes:** Memoize is an optimization, not a correctness boundary; a `#[memoize]` fn must stay
callable in tests/jobs/workers. Mirrors `current_tenant()` degrading to `None` via `try_with`.

## Argument keying (`MemoKey`)

| Option | Description | Selected |
|--------|-------------|----------|
| `(callsite TypeId, hash of non-receiver args)`; `Hash` bound; `&self` excluded | Hash all value args; compile error on non-hashable; exclude receiver | ✓ |
| Author-marked subset of args in the key | Only args the author annotates contribute to the key | |
| User-provided key expression | Author writes an explicit key expression | |

**Chosen:** Hash all non-receiver args, enforce `Hash`, exclude `&self` (D-03).
**Notes:** Deterministic and collision-resistant within a request; matches the spec's honest
limitation (reject/document non-hashable/interior-mutable args, never silently mis-key).
Service singletons are stateless injectables, so `&self` is excluded from the key.

## Coalescing primitive + error semantics

| Option | Description | Selected |
|--------|-------------|----------|
| `futures::Shared` over `Arc<dyn Any>`, cache full return incl. `Err` | Concurrent callers share one future; error is cached for the request | ✓ |
| Re-run on `Err` (cache only `Ok`) | Errors are not cached; next caller recomputes | |

**Chosen:** `futures::Shared`, cache the full resolved value including `Err` (D-04).
**Notes:** Deterministic single-computation semantics per spec §1. Documented caveat: do not
memoize a fetch whose transient error you'd want retried within the same request.

## Render-path wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Ship wiring + render-path integration test; applies to free fns AND service methods | Wire the store into the render pass; prove N intents/one key = one fetch | ✓ |
| Store + attribute only; defer render wiring | Provide the primitive, wire the render path in a later phase | |

**Chosen:** Wire + prove in Phase 259 (D-05).
**Notes:** Success Criterion #3 requires the render-path single-fetch proof; the store alone
isn't the deliverable. The exact fetch/loader seam in `ServiceDef → IntentGraph → JsonUiRenderer`
is a research/planner task against the current render context.

## Claude's Discretion

- Internal `MemoStore` types (Mutex vs RwLock, capacity).
- Per-callsite marker generation inside the macro.
- Debug-mode warning on out-of-scope calls.
- Whether to expose a manual (non-macro) `MemoStore` API.
- Exact `task_local!` scope-entry point in the middleware chain.

## Deferred Ideas

- Cross-request caching (stays `ferro-cache`).
- Background/queue-worker memoization (spec Future direction).
- `LiveFragment` (260), `asset!()` (261), MCP/catalog/docs/publish (262).
