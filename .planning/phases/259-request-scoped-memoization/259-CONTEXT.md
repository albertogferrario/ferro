# Phase 259: Request-scoped memoization - Context

**Gathered:** 2026-07-21
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults; review before planning)

<domain>
## Phase Boundary

Give the render path a request-scoped memo store so an `async fn` or `#[service]`
method marked `#[memoize]` runs its body at most once per `(callsite, arguments)`
per request, coalescing concurrent callers onto one shared computation. Deliver
three things: (1) a `MemoStore`, (2) the `#[memoize]` attribute macro, and (3)
wiring into the `ServiceDef → IntentGraph → JsonUiRenderer` render pass proven by a
render-path integration test showing N intents over one key issue a single fetch.

Scope anchors (from ROADMAP v17.0 constraints):
- **No new crates.** The store lands in `framework`, the macro in `ferro-macros`.
- **Request-scoped only.** Cross-request caching remains `ferro-cache`.
- **Complements, does not replace, `eager_loading`/`BatchLoad`.** Memoization
  deduplicates identical calls; batch loading solves N+1 for related entities.

Out of scope (later v17.0 phases): `LiveFragment` (260), `asset!()` (261),
MCP/catalog/docs/publish (262). Cross-request caching and background/queue-worker
memoization are non-goals for this milestone.

</domain>

<decisions>
## Implementation Decisions

### Store access mechanism (how `#[memoize]` reaches the store)
- **D-01:** The `MemoStore` is held in a `tokio::task_local!` (`MEMO_STORE`),
  scoped per-request in the server middleware chain — mirroring the established
  `TENANT_CONTEXT` pattern (`framework/src/tenant/context.rs`,
  `framework/src/server.rs:280`). `#[memoize]` reads the ambient store; it does
  **not** require a `&Request`/`&Cx` parameter on the memoized function.
  - Rationale: render-path fetches happen deep in the call tree with no `Request`
    handle to thread; the framework already made this ambient-context tradeoff for
    tenant + session, so consistency (conceptual coherence) wins; the attribute
    stays ergonomically transparent (its whole purpose).
  - The spec (§1) leaves "the exact plumbing" open and says it is fixed in Phase
    259 "against the render path's existing context" — this decision fixes it to
    the task-local pattern. The `Request` extensions type-map
    (`framework/src/http/request.rs` `insert`/`get`) remains an available fallback
    but is not the chosen surface, because the render path lacks a `Request`.

### Out-of-scope behavior (no `MEMO_STORE` in the task-local scope)
- **D-02:** **Graceful no-op.** When no store is in scope (background job, test,
  `ferro-queue` worker, any non-request context), the memoized body runs normally
  with no caching and no coalescing — `MEMO_STORE.try_with(...)` returns `Err`,
  and the macro-generated wrapper falls through to a plain call. **No panic.**
  - Rationale: memoization is an optimization, not a correctness boundary; a
    `#[memoize]` function must remain callable outside a request. This mirrors
    `current_tenant()` (context.rs:32-37), which degrades to `None` via `try_with`.

### Argument keying (`MemoKey`)
- **D-03:** `MemoKey = (TypeId of a per-callsite zero-sized marker, u64 hash of the
  non-receiver arguments)`. All value arguments of a memoized function must
  implement `Hash`; the macro emits the `Hash` bound and produces a clear
  compile-time error (or a documented rejection) for non-hashable / interior-mutable
  arguments — never silent mis-keying. For `#[service]` methods, the `&self`
  receiver is **excluded** from the key (service singletons are stateless
  injectables).
  - Rationale: deterministic, collision-resistant within a request; matches the
    spec's honest limitation that non-hashable/interior-mutable args are out of
    scope and "must not be memoized… reject or document rather than silently
    mis-key" (spec Honest limitations).

### Coalescing primitive + error semantics
- **D-04:** Concurrent callers of the same `(callsite, args)` within one request
  coalesce onto a single shared future (`futures::future::Shared`) whose output is
  `Arc<dyn Any + Send + Sync>`, downcast back to the concrete return type. The
  **full return value is cached for the request, including `Err`** for
  `Result`-returning functions — every caller within the request observes the same
  resolved value.
  - Rationale: deterministic single-computation semantics (spec §1 "the shared
    future coalesces concurrent callers"). Document that a memoized fetch whose
    transient error the author would want retried within the same request should
    not be memoized.

### Render-path wiring + proof (the deliverable, not just the store)
- **D-05:** Phase 259 wires the memo store into the
  `ServiceDef → IntentGraph → JsonUiRenderer` fetch path and ships a render-path
  **integration test** proving that a projection deriving multiple intents (e.g.
  Browse + Summarize) over one key issues exactly **one** underlying fetch
  (Success Criterion #3). `#[memoize]` applies to **both** free `async fn` and
  `#[service]` methods (LIVE-01 wording). Unit table tests cover hit/miss, distinct
  args recompute, and concurrent-call coalescing (Criteria #1, #2); the store is
  dropped with the request.
  - Rationale: the store alone is not the payoff — "N intents over one key issue one
    fetch" is the concrete justification for the primitive (spec §1 render-path
    wiring). The precise wiring point (which loader/fetch the render invokes) is a
    research/planner task against the current render context — see Canonical
    References and Claude's Discretion.

### Claude's Discretion
- Internal `MemoStore` types (e.g. `Mutex<HashMap<…>>` vs `RwLock`, initial
  capacity) — spec sketches `Mutex<HashMap<MemoKey, Shared<…>>>` but the exact
  shape is an implementation detail.
- The per-callsite zero-sized marker generation inside the macro (how the callsite
  TypeId is minted).
- Whether to emit a debug-mode warning when a `#[memoize]` call runs out of scope.
- Whether to expose a manual `MemoStore` API for non-macro use, or keep the store
  crate-internal behind the attribute.
- Exact tokio `task_local!` scope-entry point in the middleware chain (alongside
  vs nested within the tenant scope at `server.rs:280`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (authoritative)
- `docs/superpowers/specs/2026-07-21-live-projection-surface-design.md` §1
  "Request-scoped memoization" — `MemoStore` shape, `MemoKey`, `#[memoize]`
  attribute, render-path wiring; §"Non-Goals" (request-scoped only, not
  cross-request; complements not replaces `eager_loading`/`BatchLoad`);
  §"Honest limitations" (non-hashable/interior-mutable args rejected/documented).

### Roadmap (goal, depends-on, success criteria)
- `.planning/ROADMAP.md` §"Phase 259: Request-scoped memoization" (~L4132-4146) —
  goal, `Depends on`, three Success Criteria.
- `.planning/ROADMAP.md` §"v17.0 … Architectural constraints" (~L4096-4108) —
  no-new-crates, request-scoped, complements-not-replaces, single-publish-at-262.

### Requirement
- Requirement **LIVE-01** (request-scoped `#[memoize]` + render-path fetch dedup).
  Note: v17.0 requirements are defined inline in `.planning/ROADMAP.md`
  (Requirement → Phase Mapping, ~L4200) — they are **not** in
  `.planning/REQUIREMENTS.md`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets / patterns to mirror
- `framework/src/tenant/context.rs` — the `tokio::task_local!` `TENANT_CONTEXT`
  ambient-context pattern to model `MEMO_STORE` on: `try_with(...)` reader that
  degrades to `None`/no-op outside scope (the exact idiom for D-02), plus
  `with_tenant_scope(ctx, f) → TENANT_CONTEXT.scope(ctx, f).await`.
- `framework/src/server.rs:280` — `TENANT_CONTEXT.scope(request_host, chain.execute(...))`
  wraps the handler chain per request; the memo-store scope enters here too.
- `ferro-macros/src/{service.rs, handler.rs, action.rs}` + `ferro-macros/src/lib.rs`
  (proc-macro registration) — the attribute-macro machinery to model `#[memoize]`
  on. `#[handler]`/`#[action]`/`#[service]` are the closest analogs.

### Established patterns it constrains / complements
- `framework/src/database/eager_loading.rs` (`BatchLoad`, `BatchLoadMany`,
  `batch_load_by_id`, `batch_load_has_many`) + `framework/src/database/query_builder.rs`
  — the N+1 path memoization **complements, not replaces** (spec Non-Goals).
- `framework/src/http/request.rs` — extensions type-map (`insert`/`get`/`get_mut`,
  L84-100); available fallback store home, but not chosen (render path has no
  `Request`).

### Integration points (render pass to wire into)
- `ferro-projections/src/derive.rs` `derive_intents()` and
  `ferro-projections/src/render/mod.rs` `render()` — the `ServiceDef → IntentGraph`
  pass; the `JsonUiRenderer` in `ferro-json-ui`. The single-fetch-across-multi-intent
  proof (D-05) targets whatever fetch/loader this pass invokes — research must pin
  the exact seam.

</code_context>

<specifics>
## Specific Ideas

- The spec sketch: `MemoStore { entries: Mutex<HashMap<MemoKey, Shared<BoxFuture<Arc<dyn Any + Send + Sync>>>>> }`
  and `MemoKey = (TypeId of a per-callsite marker, u64 hash of the arguments)`.
  Treat as the intended shape; internal type choices are Claude's discretion (D-05).
- `#[memoize]` modeled on the existing `#[handler]`/`#[action]` attribute macros
  (spec §1) — same crate, same registration style.

</specifics>

<deferred>
## Deferred Ideas

- **Cross-request / general-purpose caching** — stays `ferro-cache`; explicit
  non-goal for `#[memoize]`.
- **Memoization in background / queue-worker contexts** — spec Future direction
  ("scope extension to background/offloaded work contexts if a need surfaces");
  D-02 keeps those call sites working (un-memoized) for now.
- **`LiveFragment` element + client runtime** — Phase 260.
- **`asset!()` macro + Iconify/Fontsource fetch** — Phase 261.
- **MCP catalog / `generation_context` / docs / publish** — Phase 262.

None of the above are scope for Phase 259.

</deferred>

---

*Phase: 259-request-scoped-memoization*
*Context gathered: 2026-07-21*
