# Phase 260: Live reactive fragment - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-26
**Phase:** 260-live-reactive-fragment
**Mode:** `--auto` (all gray areas auto-selected; recommended default locked per area)
**Areas discussed:** Hook location (layering seam), Delta channel payload, Client runtime + discovery, First-paint empty snapshot, Snapshot→template binding, Catalog membership boundary

---

## Hook location (crate-layering seam)

| Option | Description | Selected |
|--------|-------------|----------|
| Renderer-agnostic hook seam in ferro-projection (type-erased at `Value`); ferro-json-ui implements; app wires | Keeps ferro-projection renderer-free; `P::State: Serialize` is the erasure boundary | ✓ |
| New bridge crate | Violates the no-new-crates v17.0 constraint | |
| Feature-flagged `ferro-projection → ferro-json-ui` dependency | Dependency inversion — projection runtime must not know rendering | |

**Auto-selected:** hook seam (recommended). **Notes:** one canonical renderer per projection name; multi-template-per-projection deferred. Exact hook ordering (sync in `apply_event` vs. second listener) → research/discretion.

---

## Delta channel payload (HTML vs raw delta)

| Option | Description | Selected |
|--------|-------------|----------|
| Additive second `fragment` event carrying `{ html }` on the same channel | Existing `delta` untouched; client swaps on `fragment` only | ✓ |
| Replace the `delta` payload with HTML | Breaks existing raw-delta subscribers | |
| Client receives `delta`, re-requests a render | Spec-rejected: extra round-trip + splits render authority | |

**Auto-selected:** additive `fragment` event (recommended). **Notes:** event name spelling + envelope → discretion.

---

## Client runtime + channel discovery

| Option | Description | Selected |
|--------|-------------|----------|
| New `runtime/live_fragment.rs` `setupLiveFragments`; `[data-live-fragment][data-channel]`; opens `/_ferro/ws`; swaps innerHTML on `fragment` | Reuses endpoint + subscribe protocol + runtime-assembly pattern; no WASM/state | ✓ |

**Auto-selected:** model on the existing `sse.rs` runtime concern (only viable shape; reuses `/_ferro/ws`). **Notes:** one shared socket per page, subscribe per channel; reconnect posture → discretion.

---

## First-paint when snapshot absent (`read` → None)

| Option | Description | Selected |
|--------|-------------|----------|
| Always render the marked container; bind empty/default state; fill on first delta | Container must exist for the client to subscribe | ✓ |
| Omit the container until first delta | Client has nothing to subscribe to | |
| Hard error on absent snapshot | An empty key is a normal first state | |

**Auto-selected:** always render container + empty state (recommended). **Notes:** `{}` vs. declared default → discretion.

---

## Snapshot → child-template binding

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse the existing expression/data-binding engine; snapshot JSON as data scope; same render function for first paint + delta | Conceptual coherence; no duplicate control surface | ✓ |
| Bespoke fragment-only binding syntax | Duplicate control surface, erodes coherence | |

**Auto-selected:** reuse existing engine (recommended).

---

## Catalog membership boundary (260 vs 262)

| Option | Description | Selected |
|--------|-------------|----------|
| Add LiveFragment to `BUILTIN_TYPES`+`BUILTIN_SPECS` and bump canonical count (52→53) in 260; defer ferro-mcp mirror + generation_context + docs to 262 | Render dispatch needs BUILTIN_TYPES membership; keeps the in-crate drift guard green | ✓ |
| Defer all catalog work to 262 | Would break render dispatch / drift guard within 260 | |

**Auto-selected:** canonical bump in 260, mirror+docs in 262 (recommended). **Notes:** research flag — confirm no extra cross-crate count mirror trips in 260.

---

## Claude's Discretion

- Exact hook ordering (sync in `apply_event` vs. second `ferro-events` listener).
- Hook-registry internal types + the type-erased renderer trait/callback signature.
- `fragment` event name + `{ html }` envelope shape.
- Container tag/classes (ferro-base.css regen is a 262 concern if new classes appear).
- WS connection sharing/reconnect in the client runtime.
- `read`-absent binds `{}` vs. a projection-declared default.

## Deferred Ideas

- Keyed live lists / collection reconciliation (second binding pattern).
- Delta-granular fragment patches.
- Multiple distinct fragment templates over the same projection.
- `generation_context` + ferro-mcp mirror count + `docs/src` (Phase 262).
- `asset!()` macro + Iconify/Fontsource fetch (Phase 261).
