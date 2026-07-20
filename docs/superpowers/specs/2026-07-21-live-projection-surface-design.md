# Live Projection Surface — Design

**Milestone:** v17.0
**Status:** Design — approved, pre-planning
**Date:** 2026-07-21

## Overview

Ferro already carries the two halves of a live-rendering story that have never
been joined. `ferro-projection` (singular) folds domain events into per-key
snapshots and broadcasts deltas on `projection.{name}.{key}` channels over
`ferro-broadcast`; `ferro-json-ui` renders declarative specs to HTML on the
server. There is no authoring primitive that binds a rendered fragment to a
projection key so it updates when the snapshot changes. Today an app that wants a
live view hand-writes WebSocket wiring outside the JSON-UI surface, which an agent
reading the project through `ferro-mcp` cannot discover or compose.

This milestone adds that primitive, plus two supporting ergonomics that the
render path and asset story have been missing: request-scoped memoization (so a
single projection render does not re-fetch the same data once per intent) and a
declarative `asset!()` macro over the existing bundle/pipeline crates.

The defining capability is the live fragment: it makes the singular projection
runtime a first-class JSON-UI rendering target — server-rendered, no WASM,
agent-composable, and introspectable through the same catalog and
`generation_context` surface as every other component.

## Goal

1. A JSON-UI `LiveFragment` element that binds a child template to a projection
   key and re-renders in place on delta, server-authoritatively, with no
   client-side framework or WASM.
2. A `#[memoize]` attribute giving request-scoped, fan-out-deduplicated caching
   for async functions and service methods, wired into the projection render pass.
3. An `asset!()` macro that declares a compile-time asset (embed + content-type
   inference + content-hashed registration) in one line, plus an opt-in
   Iconify/Fontsource fetch helper.
4. Full introspection parity: catalog entry, `generation_context` guidance, and
   `docs/src` coverage for all three, shipped in a single operator-gated publish.

## Non-Goals

- Collection/list diffing inside a live fragment. v17.0 binds one per-key snapshot
  to one fragment. Live lists (keyed reconciliation) are deferred.
- Client-side reactive state or a signal system. Reactivity is server-rendered
  and pushed; the client runtime only opens a socket and swaps HTML.
- A general-purpose caching layer. `#[memoize]` is request-scoped only;
  cross-request caching remains `ferro-cache`.
- Node or native build tooling for assets. The Iconify/Fontsource helper must run
  on the Rust toolchain alone (no node, no nasm — per the workspace toolchain rule).
- Replacing `eager_loading`/`BatchLoad`. Memoization deduplicates identical calls;
  batch loading solves N+1 for related entities. They compose; neither subsumes
  the other.

## Design

### 1. Request-scoped memoization (`framework`, `ferro-macros`)

**Store.** A `MemoStore` lives in the request extensions type-map (`Request::insert`
/ `Request::get`, already present at `framework/src/http/request.rs`). It holds a
map from key to a shared, awaitable slot:

```text
MemoStore {
    entries: Mutex<HashMap<MemoKey, Shared<BoxFuture<Arc<dyn Any + Send + Sync>>>>>
}
```

`MemoKey = (TypeId of a per-callsite marker, u64 hash of the arguments)`. The
shared future coalesces concurrent callers: the first caller inserts the pending
future, later callers within the same request await the same computation (fan-out
dedup), and the resolved value is downcast back to the concrete return type.

**Attribute.** `#[memoize]` in `ferro-macros` (modeled on the existing
`#[handler]`/`#[action]` attribute macros) rewrites an `async fn` or `#[service]`
method so its body runs at most once per `(callsite, args)` per request. The macro
requires access to the current `Request` (or a `&Cx`-equivalent context) to reach
the store; the exact plumbing is fixed during Phase 259 against the render path's
existing context.

**Render-path wiring.** The `ServiceDef → IntentGraph → JsonUiRenderer` pass can
derive multiple intents (e.g. Browse and Summarize) over the same underlying
model set. Fetches on that path route through the memo store so N intents over one
key issue one query. This is the concrete payoff that justifies the primitive.

### 2. Live reactive fragment (`ferro-json-ui`, `ferro-projection`, `ferro-broadcast`)

**Element.** A new builtin JSON-UI element, `LiveFragment`, declares:

- `projection` — the projection name.
- `key` — the per-key channel selector (`projection.{projection}.{key}`).
- a child template — the fragment rendered from the current snapshot.

On first render the server loads the current snapshot for the key and renders the
child template to HTML, wrapping it in a marked container that carries the channel
identifier.

**Transport (server-pushes-HTML).** When `ferro-projection` applies an event and
produces a new snapshot, a render hook re-renders the fragment's child template
against the new snapshot and broadcasts the resulting HTML on the existing
`projection.{name}.{key}` channel. The client runtime swaps the container's inner
HTML. Rendering stays entirely server-authoritative; the client never re-requests
and never renders. This choice keeps the no-WASM thesis intact and minimizes
round-trips.

**Client runtime.** A small script (served through the asset pipeline, no build
step, no WASM) opens the `ferro-broadcast` WebSocket, subscribes to the channels
named by `LiveFragment` containers present in the DOM, and replaces inner HTML on
each message. It is the only client-side code introduced and does no state
management.

**Scope guard.** Exactly one binding pattern: a single per-key snapshot to a
single fragment. No list reconciliation, no nested live fragments sharing a key's
sub-paths — deferred to a later milestone.

### 3. Asset declaration ergonomics (`ferro-macros`, `ferro-assets`, `ferro-bundle`, `ferro-cli`)

**Macro.** `asset!("relative/path.js")` expands at compile time to
`include_bytes!` of the path, content-type inference from the extension, and
registration of a `ferro-bundle` `Bundle`, returning the content-hashed URL. It
collapses the current boot-time builder chain
(`Bundle::new(name, bytes).content_type(ct).with_alias(path)`) to one call at the
use site while reusing the same immutable-cache machinery underneath.

**Icon/font fetch.** An opt-in `ferro-cli` subcommand (e.g. `ferro assets fetch`)
downloads Iconify sets and Fontsource families into the project's asset directory,
after which they flow through the existing `ferro-assets` transform pipeline and
`asset!()`. The fetch runs on the Rust toolchain alone. It is opt-in and additive;
no default feature pulls a network fetch into a normal build.

## Alternatives considered

- **Client re-requests render on delta** (broadcast state, client fetches HTML).
  Rejected: extra round-trip per update and splits render authority. Server-push
  of rendered HTML is simpler and keeps rendering in one place.
- **`#[memoize]` as a general cross-request cache.** Rejected: that is
  `ferro-cache`'s job and introduces invalidation concerns the render path does
  not need. Request scope is the whole point — deterministic lifetime, no eviction.
- **Live fragment as a plugin rather than a builtin.** Rejected: the primitive
  must appear in `json_ui_catalog` and `generation_context` so agents can compose
  it; a plugin sits outside that introspection contract.

## Phase decomposition

- **Phase 259 — Request-scoped memoization.** `MemoStore` + `#[memoize]` macro +
  render-path wiring. Table tests for hit/miss and concurrent-call coalescing; a
  projection-render test proving N intents over one key issue one fetch.
- **Phase 260 — Live reactive fragment (killer feature).** `LiveFragment` element,
  `ferro-projection` render hook, server-push transport, client runtime. One
  binding pattern. Integration test: event → delta → re-rendered HTML on channel.
- **Phase 261 — `asset!()` ergonomics.** Macro over bundle/pipeline; opt-in
  Iconify/Fontsource fetch subcommand. Tests: hashed-URL stability, content-type
  inference, passthrough for unknown types.
- **Phase 262 — MCP + catalog + docs + publish.** Catalog entry for `LiveFragment`
  through the full drift-guard checklist (BUILTIN_TYPES + dispatch + catalog spec +
  count assertion + `ferro-mcp` mirror count); `generation_context` guidance for
  all three capabilities; `docs/src` sections; regenerate `ferro-base.css` if the
  client runtime adds classes; single operator-gated publish. No mid-milestone
  publishes.

## Testing

- Memoization: unit table tests (hit, miss, distinct args, concurrent coalescing),
  plus a render-path integration test asserting a single query for a multi-intent
  projection over one key.
- Live fragment: an integration test driving `event → ProjectionListener → delta`
  and asserting the re-rendered fragment HTML lands on the expected channel; a
  render test for first-paint snapshot HTML.
- Assets: hashed-URL determinism across builds, content-type inference table,
  byte-identical passthrough for unrecognized extensions.
- Full CI-exact gate green (`cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features`)
  before the Phase 262 publish.

## Honest limitations

- Server-push re-renders the whole fragment per delta; large fragments updating at
  high frequency pay full-render cost each time. Acceptable for v17.0's per-key
  snapshot scope; frequency/size tuning is future work.
- `#[memoize]` keys on an argument hash; non-hashable or interior-mutable arguments
  are out of scope and must not be memoized. The macro should reject or document
  this rather than silently mis-key.
- The Iconify/Fontsource fetch reaches the network at author time; it is opt-in and
  never runs in a normal `cargo build`.

## Future direction

- Keyed live lists (collection reconciliation) as a second binding pattern.
- Optional delta-granular fragment updates (patch rather than full re-render) once
  the per-key path is proven.
- Memoization scope extension to background/offloaded work contexts if a need
  surfaces from `ferro-projection` consumers.
