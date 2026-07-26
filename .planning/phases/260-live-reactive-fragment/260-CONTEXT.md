# Phase 260: Live reactive fragment - Context

**Gathered:** 2026-07-26
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults; review before planning)

<domain>
## Phase Boundary

Add a `LiveFragment` builtin JSON-UI element that binds a child template to a
`ferro-projection` per-key snapshot: it renders the current snapshot to HTML on
first paint, and re-renders in place on each delta — server-authoritatively —
with a small no-WASM client runtime that only opens the existing
`ferro-broadcast` socket and swaps inner HTML. This is the v17.0 killer feature:
it makes the singular projection runtime a first-class JSON-UI rendering target,
composable and introspectable through the same surface as every other component.

Deliverables (from ROADMAP Phase 260 Success Criteria):
1. First-paint: `LiveFragment{projection, key, child}` renders the current
   snapshot to HTML (render test).
2. Live cycle: `event → ProjectionListener → delta` broadcasts the re-rendered
   fragment HTML on `projection.{name}.{key}` (integration test).
3. Client runtime adds **no WASM and no client-side state** — subscribe + swap
   inner HTML only.
4. Exactly **one binding pattern** ships (per-key snapshot); list reconciliation
   is absent by design and documented as a non-goal.

Scope anchors (from ROADMAP v17.0 constraints):
- **No new crates.** Element + client runtime land in `ferro-json-ui`; the
  re-render hook seam lands in `ferro-projection`; transport reuses
  `ferro-broadcast` + the existing `/_ferro/ws` endpoint.
- **Server-authoritative rendering.** The client never re-requests and never
  renders; the server pushes rendered HTML.
- **Single publish at Phase 262.** No publish in Phase 260.

Out of scope (later or deferred):
- Collection/list diffing (keyed reconciliation) — explicit v17.0 non-goal.
- Client-side reactive state / signal system — explicit non-goal.
- Multiple distinct fragment templates over the same projection (see D-01, D-07).
- `generation_context` + ferro-mcp mirror + docs — Phase 262 (see D-06).
- `asset!()` macro — Phase 261.

</domain>

<decisions>
## Implementation Decisions

### Re-render-on-delta hook location (the crate-layering seam) — the substantive decision
- **D-01:** `ferro-projection` exposes a **renderer-agnostic hook seam** registered
  on the runtime; `ferro-json-ui` provides the `LiveFragment` renderer
  implementation; the app wires them at startup (mirroring how
  `ProjectionRuntime::new` already takes the broadcaster + projection). The hook
  is **type-erased at the `serde_json::Value` snapshot boundary**: because
  `P::State: Serialize` (it is already persisted as JSON in
  `projection_snapshots`), the runtime serializes the new state → `Value` and
  hands it to a `dyn`-object hook keyed by projection name. **`ferro-projection`
  gains NO dependency on `ferro-json-ui`.**
  - Rationale: keeps `ferro-projection` renderer-free, consistent with the
    framework's renderer-location rule (renderers live in their output crate, not
    in the projection layer — CLAUDE.md). The hook fires inside/after
    `apply_event` where the full new snapshot is in hand.
  - Rejected: a new bridge crate (violates the no-new-crates v17.0 constraint);
    a feature-flagged `ferro-projection → ferro-json-ui` dependency (dependency
    inversion — the projection runtime must not know about rendering).
  - **One canonical renderer per projection name.** The hook registry keys on
    projection name → one registered `LiveFragment` renderer. Multiple *distinct*
    fragment templates over the same projection is out of scope for v17.0 (see
    D-07 / Deferred). Research must pin exact hook ordering (sync inside
    `apply_event` after the existing delta broadcast, vs. a second registered
    `ferro-events` listener) — see Claude's Discretion.

### Delta channel payload for a live fragment (HTML vs raw delta)
- **D-02:** **Additive second broadcast.** The existing `delta` event
  (`broadcast_event_name()`, default `"delta"`, data = the consumer's `Delta`) is
  left untouched. The hook broadcasts a **second** `BroadcastMessage` with a
  **distinct event name** (working name `"fragment"`) carrying `{ html }` on the
  **same** `projection.{name}.{key}` channel. The client runtime swaps
  `innerHTML` only on the `fragment` event and ignores `delta`.
  - Rationale: additive and backward-compatible — existing raw-`delta`
    subscribers are unaffected; the fragment view is a parallel projection of the
    same channel.
  - Rejected: replacing the `delta` payload with HTML (breaks raw-delta
    subscribers); client receives `delta` then re-requests a render
    (spec-rejected: extra round-trip + splits render authority).

### Client runtime shape + channel discovery
- **D-03:** New `ferro-json-ui/src/runtime/live_fragment.rs` contributing a
  `setupLiveFragments` function, registered in the `FERRO_RUNTIME_JS` assembly and
  the `ferroRuntime()` dispatcher (exact prior-art pattern of `sse.rs` +
  `data-sse-*`). The server wraps the rendered child in a **marked container**
  carrying `data-live-fragment` and `data-channel="projection.{name}.{key}"`. On
  `DOMContentLoaded` the runtime scans `[data-live-fragment]`, opens the existing
  `/_ferro/ws` WebSocket (`framework/src/websocket.rs`), sends
  `ClientMessage::Subscribe { channel }` for each, and replaces the container's
  inner HTML on each matching `fragment` message. **No WASM, no client-side state,
  no framework.**
  - Rationale: reuses the whole existing transport (endpoint + subscribe protocol
    + runtime-assembly pattern); the only new client code is one `setup*` concern.
  - One shared socket for all fragments on the page (subscribe per channel), not
    a socket per fragment — Claude's Discretion on connection sharing details.

### First-paint when the snapshot is absent (`read` → `None`)
- **D-04:** **Always render the marked container** so the client can subscribe;
  when `ProjectionRuntime::read(key)` returns `Ok(None)`, bind the child template
  against an **empty/default snapshot** (empty object). The fragment fills in on
  the first delta. **Never omit the container; never hard-error on first paint.**
  - Rationale: a live view over a not-yet-populated key is a legitimate first
    state; the container must exist in the DOM for the client to subscribe at all.
  - Rejected: omit the container until first delta (client has nothing to
    subscribe to); hard error (an empty key is normal, not a fault).

### Snapshot → child-template data binding
- **D-05:** **Reuse the existing `ferro-json-ui` expression / data-binding
  engine** (`expression.rs` + `data.rs`). The snapshot JSON becomes the data scope
  for the child template — the same binding vocabulary the rest of JSON-UI uses.
  The **same render function** serves both first paint and delta re-render (only
  the input snapshot differs), preserving the server-authoritative single render
  path.
  - Rationale: conceptual coherence — no parallel fragment-only binding syntax
    (no duplicate control surface).
  - Rejected: a bespoke fragment binding syntax.

### Catalog membership boundary (what Phase 260 touches vs Phase 262)
- **D-06:** `LiveFragment` is a **builtin** (the spec explicitly rejects a plugin
  because the primitive must appear in `json_ui_catalog`). Render dispatch keys on
  `BUILTIN_TYPES` membership, and the in-crate drift guard asserts
  `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` with a pinned count
  (`BUILTIN_TYPES.len() == 52`). Therefore Phase 260 **adds `LiveFragment` to
  `BUILTIN_TYPES` + `BUILTIN_SPECS` and bumps the canonical count (52 → 53)** so
  the phase builds green — this is a render-dispatch necessity, not scope creep.
  Phase 262 owns only the **ferro-mcp mirror count**, `generation_context`
  guidance, and `docs/src`.
  - Rationale: keeps each phase's build green; the canonical catalog entry cannot
    be separated from render dispatch. ROADMAP 262 SC#1 ("both count assertions …
    agree at the bumped count") reconciles the mirror against this canonical bump.
  - Research flag: confirm no additional cross-crate count mirror (beyond
    ferro-mcp) trips in Phase 260; if one does, it moves into 260 with the bump.

### Claude's Discretion
- Exact hook ordering: re-render synchronously inside `apply_event` (after the
  existing delta broadcast) vs. a second registered `ferro-events` listener.
- Internal hook-registry types (map shape, keying), and the exact trait/callback
  signature for the type-erased renderer seam.
- The `fragment` event name spelling and the `{ html }` payload envelope shape.
- Container element/tag and any wrapper classes (subject to `ferro-base.css`
  regeneration only if new utility classes are introduced — a Phase 262 concern).
- WebSocket connection sharing/reconnect details in the client runtime (model on
  `sse.rs` reconnect posture).
- Whether `read`-absent binds `{}` vs. a projection-declared default state.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (authoritative)
- `docs/superpowers/specs/2026-07-21-live-projection-surface-design.md` —
  §"Overview" + §Goal(1) (the live-fragment thesis); §"Non-Goals" (no list
  diffing, no client reactive state, server-push-HTML); **§Design.2 "Live
  reactive fragment"** (Element / Transport / Client runtime / Scope guard — the
  core contract); §"Alternatives considered" (why server-push HTML, why builtin
  not plugin); §"Testing" (first-paint render test + event→delta→HTML integration
  test); §"Honest limitations" (whole-fragment re-render per delta); §"Future
  direction" (keyed live lists, delta-granular patches — deferred).

### Roadmap (goal, depends-on, success criteria)
- `.planning/ROADMAP.md` §"Phase 260: Live reactive fragment" (~L4152-4173) —
  goal, `Depends on`, four Success Criteria.
- `.planning/ROADMAP.md` §"v17.0 … Architectural constraints" (~L4096-4108) —
  no-new-crates, server-authoritative, single-publish-at-262.
- `.planning/ROADMAP.md` §"Requirement → Phase Mapping (v17.0)" (~L4202-4209) —
  LIVE-02 → Phase 260.

### Requirement
- Requirement **LIVE-02** (`LiveFragment` element + projection render hook +
  client runtime). v17.0 requirements are defined inline in `.planning/ROADMAP.md`
  (Requirement → Phase Mapping) — they are **not** in `.planning/REQUIREMENTS.md`.

### Prior phase this builds on
- `.planning/phases/259-request-scoped-memoization/259-CONTEXT.md` — `#[memoize]`
  + render-path fetch dedup (shipped in-tree). A LiveFragment re-render is a
  render-path render; memoization dedups repeated fetches within each re-render.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets / patterns to model on
- **Client-runtime assembly (model exactly):** `ferro-json-ui/src/runtime/mod.rs`
  — `FERRO_RUNTIME_JS` (`LazyLock<String>`) concatenates per-concern `SOURCE`
  string constants, each a `setup*` function invoked in the `ferroRuntime()`
  dispatcher on `DOMContentLoaded`, each isolated so one throw does not abort the
  rest. Add `mod live_fragment;` + `s.push_str(live_fragment::SOURCE)` +
  `setupLiveFragments` in the dispatcher list.
- **Prior-art server-push swap:** `ferro-json-ui/src/runtime/sse.rs` — already
  does data-attribute-driven server-push updates (`data-sse-url`,
  `data-sse-target`, `updateLiveValues`, app handler registration via
  `window.__ferroSSEHandlers`). LiveFragment mirrors this shape over the
  `ferro-broadcast` WebSocket instead of `EventSource`.
- **Snapshot read for first paint:** `ferro-projection/src/runtime.rs`
  `ProjectionRuntime::read(&self, key) -> Ok(Option<P::State>)` (composite-PK
  `Entity::find_by_id`); `read_required` errors when absent. `P::State:
  Serialize` (persisted as JSON) is the type-erasure boundary for the hook (D-01).
- **Broadcast message shape:** `ferro-broadcast/src/message.rs`
  `BroadcastMessage { event, channel, data: Value }` + `ClientMessage::Subscribe {
  channel, .. }` + `ServerMessage`; `ferro-broadcast/src/broadcast.rs`
  `Broadcast::new(broadcaster).channel(..).send(..)`.

### Integration points
- **Projection runtime (hook seam lands here):** `ferro-projection/src/runtime.rs`
  `apply_event` (persist → broadcast on `projection.{name}.{key}`) and `rebuild`;
  `ferro-projection/src/projection.rs` `Projection` trait (`State`, `Delta`,
  `NAME`, `apply`, `broadcast_event_name()` default `"delta"`);
  `ferro-projection/src/listener.rs` `ProjectionListener<P>` (a
  `ferro_events::Listener`).
- **Browser WS endpoint (reuse — do NOT add one):** `framework/src/websocket.rs`
  — `/_ferro/ws` upgrade + connection loop handling `ClientMessage::Subscribe` /
  `ServerMessage::Subscribed`. The client runtime opens exactly this.
- **Element + render dispatch:** `ferro-json-ui/src/spec.rs`
  (`Spec`, `Element { type_name, props, children }`, builders — child-template
  shape); `ferro-json-ui/src/render/mod.rs` (`BUILTIN_TYPES`, `is_builtin`
  dispatch — LiveFragment registration + render entry).
- **Catalog + drift guard (D-06):** `ferro-json-ui/src/catalog.rs` —
  `BUILTIN_SPECS`, `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` guard, and the
  pinned `BUILTIN_TYPES.len() == 52` count test to bump.
- **Data binding (D-05):** `ferro-json-ui/src/expression.rs` +
  `ferro-json-ui/src/data.rs` — snapshot JSON as the child-template data scope.

### Naming caution
- `ferro-projection` (singular, live read-model runtime — this phase's substrate)
  vs. `ferro-projections` (plural, the `ServiceDef → IntentGraph` abstraction).
  The `ferro-json-ui/src/projection/` module is the **plural** schema-driven
  pipeline (`Spec::from_service_def`) — unrelated to LiveFragment. Do not conflate.

</code_context>

<specifics>
## Specific Ideas

- Spec §Design.2 is the contract sketch: `LiveFragment` declares `projection`,
  `key`, and a child template; first render loads the snapshot and wraps the
  child in a marked container carrying the channel id; on delta a render hook
  re-renders against the new snapshot and broadcasts the HTML on the existing
  channel; a small no-build no-WASM client swaps inner HTML.
- Server-push-HTML (not client-re-request) is a deliberate choice in the spec
  (§Alternatives) — keeps rendering in one place, one round-trip.
- Builtin (not plugin) is deliberate (§Alternatives) — it must appear in
  `json_ui_catalog` / `generation_context` so agents can compose it.

</specifics>

<deferred>
## Deferred Ideas

- **Keyed live lists / collection reconciliation** — spec Future direction; second
  binding pattern, out of scope for v17.0.
- **Delta-granular fragment patches** (patch instead of full re-render) — spec
  Future direction; whole-fragment re-render is the accepted v17.0 cost.
- **Multiple distinct fragment templates over the same projection** — v17.0 is one
  canonical renderer per projection name (D-01); multi-template dispatch is a
  later concern.
- **`generation_context` guidance + ferro-mcp mirror count + `docs/src`
  coverage** — Phase 262 (D-06 keeps only the canonical catalog bump in 260).
- **`asset!()` macro + Iconify/Fontsource fetch** — Phase 261.

None of the above are scope for Phase 260.

</deferred>

---

*Phase: 260-live-reactive-fragment*
*Context gathered: 2026-07-26*
