---
phase: 260-live-reactive-fragment
verified: 2026-07-26T18:45:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "End-to-end live-browser DOM swap over /_ferro/ws"
    expected: "With a running app that has a registered projection + a LiveFragment page, dispatching a domain event causes the fragment's innerHTML to update in the browser without a page reload"
    why_human: "Requires a running server, active WebSocket connection, and browser — automated integration test proves the HTML reaches the broadcast channel (SC2), but the final DOM swap step needs a real browser. The VALIDATION.md acknowledges this as the only manual-only verification."
---

# Phase 260: Live reactive fragment — Verification Report

**Phase Goal:** Add a `LiveFragment` JSON-UI element that binds a child template to a
`ferro-projection` per-key snapshot, renders the current snapshot to HTML on first paint, and
re-renders in place on each delta — server-authoritatively, with a client runtime that only
opens the `ferro-broadcast` socket and swaps HTML.

**Verified:** 2026-07-26T18:45:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A `LiveFragment` with `projection`/`key`/child renders the current snapshot to HTML on first paint (render test) | VERIFIED | `render_live_fragment` in `ferro-json-ui/src/render/containers.rs:1639` decodes `LiveFragmentProps`, renders child via `render_spec_to_html(&child_spec, data)`, emits `<div data-live-fragment data-channel="projection.{name}.{key}">`. Test `render_live_fragment_renders_container_with_channel` (containers.rs:3511) and `live_fragment_end_to_end_first_paint_and_delta_use_one_render_path` (render/mod.rs:706) both pass. SC1 proven via the public `render_spec_to_html` API. |
| 2 | `event → ProjectionListener → delta` broadcasts the re-rendered fragment HTML on `projection.{name}.{key}` (integration test) | VERIFIED | `with_fragment_renderer()` builder (runtime.rs:76) registers the hook; step-6.5 (runtime.rs:177) fires the hook AFTER the delta broadcast and INDEPENDENT of delta broadcast success (WR-02 fix). Hook receives the already-serialized `state_json` (WR-03 fix). Test `live_fragment_hook` (runtime.rs:662) uses a real `Arc<Broadcaster>` + subscribed `mpsc` client, dispatches an event, and asserts BOTH a `delta` frame AND a `fragment` frame carrying `{ html }` arrive on `projection.test.counter.default-key`. |
| 3 | The client runtime adds NO WASM and NO client-side state (subscribe + swap only) | VERIFIED | `ferro-json-ui/src/runtime/live_fragment.rs` — `setupLiveFragments` uses a pre-built `channelMap` (WR-01 fix: no CSS selector from server data), opens one shared `/_ferro/ws` socket, sends Subscribe per channel, swaps `innerHTML` on `msg.event === 'fragment'`. Test `live_fragment_runtime_no_wasm_no_state` asserts absence of `WebAssembly`, `useState`, and `eval(` in the SOURCE, plus presence of `channelMap` and absence of selector concatenation. |
| 4 | Exactly one binding pattern ships (per-key snapshot); list reconciliation absent and documented as non-goal | VERIFIED | `live_fragment_ships_one_binding_pattern_no_list_reconciliation` (render/mod.rs:757) greps `containers.rs` for `"reconcile"` and `"keyed_diff"` — both absent. D-05 single-render-path proven by `delta_html_a == delta_html_b` assertion in the end-to-end test. Context deferred section explicitly names keyed live lists as out of scope for v17.0. |

**Score: 4/4 truths verified**

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projection/src/runtime.rs` | fragment_hook field + with_fragment_renderer() builder + step-6.5 hook invocation | VERIFIED | `FragmentHook` type alias at line ~45; `fragment_hook: Option<FragmentHook>` field; `with_fragment_renderer()` consuming builder; step-6.5 block at line 177, fires before `send_result` error check (WR-02). |
| `ferro-json-ui/src/component.rs` | `LiveFragmentProps` struct (Serialize/Deserialize/JsonSchema, no Eq) | VERIFIED | `pub struct LiveFragmentProps` at line 753; fields `projection: String`, `key: String`, `template: serde_json::Value`; derives `Debug/Clone/Serialize/Deserialize/JsonSchema`; no `Eq`. |
| `ferro-json-ui/src/render/containers.rs` | `render_live_fragment` fn + first-paint + absent-snapshot unit tests | VERIFIED | `pub(crate) fn render_live_fragment` at line 1639; calls `html_escape` on projection and key; uses `super::render_spec_to_html(&child_spec, data)`; two tests at lines 3511 and 3543 pass. `#[allow(dead_code)]` removed by Plan 04 once dispatch arm was wired. |
| `ferro-json-ui/src/runtime/live_fragment.rs` | `setupLiveFragments` SOURCE + SC3 no-WASM test | VERIFIED | `pub(super) const SOURCE` with `setupLiveFragments`; channelMap-based DOM lookup (WR-01 fix applied in `f8ded454`); SC3 test at line 68 with assertions for absence of `WebAssembly`/`useState`/`eval(` and presence of `channelMap`. |
| `ferro-json-ui/src/runtime/mod.rs` | mod decl + push_str + dispatcher entry + enumerating tests updated | VERIFIED | `mod live_fragment;` at line 14; `s.push_str(live_fragment::SOURCE)` at line 50; `setupLiveFragments` in dispatcher at line 73; appears 3 times total (`grep -c` returns 3) — dispatcher array + both enumerating test arrays. |
| `ferro-json-ui/src/render/mod.rs` | `"LiveFragment"` in BUILTIN_TYPES + dispatch arm + integration tests | VERIFIED | `"LiveFragment"` at line 90 in BUILTIN_TYPES; dispatch arm `"LiveFragment" => containers::render_live_fragment(el, spec, data, depth)` at line 229; `live_fragment_end_to_end_first_paint_and_delta_use_one_render_path` at line 706; `live_fragment_ships_one_binding_pattern_no_list_reconciliation` at line 757. |
| `ferro-json-ui/src/catalog.rs` | BUILTIN_SPECS entry + LiveFragmentProps import + count 53 | VERIFIED | `LiveFragmentProps` in import at line 34; BUILTIN_SPECS tuple at line 382 with `schema_for!(LiveFragmentProps)`; count guard at line 1303 asserts `BUILTIN_TYPES.len() == 53`; history comment updated `→ 53 (LiveFragment)`. 13 KB prompt budget not exceeded — no bump needed. |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | mirror count 53 + LiveFragment in expected names | VERIFIED | Count assertion at line 419 is `53`; `"LiveFragment"` in expected names at line 478. Fixed in commit `10b92b6c` — D-06 originally said "defer to 262" but since the catalog derives from `global_catalog()`, the count test broke immediately and the fix was correctly pulled into Phase 260. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ProjectionRuntime::apply_event` | `self.fragment_hook` | sync call after step-6 delta broadcast, inside per-key Mutex | WIRED | Step-6.5 block at runtime.rs:177; fires BEFORE `send_result` error check — hook is additive and independent (WR-02 fix verified) |
| `fragment_hook closure` | subscriber on `projection.{name}.{key}` | `tokio::spawn`'d second broadcast with event `"fragment"` carrying `{ html }` | WIRED | Proven by `live_fragment_hook` integration test; subscriber receives both `delta` and `fragment` frames |
| `render_element` dispatch | `containers::render_live_fragment` | match arm on `"LiveFragment"` | WIRED | render/mod.rs line 229 |
| `BUILTIN_SPECS` | `schema_for!(LiveFragmentProps)` | catalog tuple entry | WIRED | catalog.rs line 385 |
| `setupLiveFragments` | `/_ferro/ws` WebSocket | `channelMap` lookup; Subscribe per channel on open; `innerHTML` swap on `fragment` event | WIRED | live_fragment.rs SOURCE; uses pre-built channelMap, not CSS selector from server data (WR-01 fix) |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `render_live_fragment` | `data` (snapshot `Value`) | Passed in by caller from `ProjectionRuntime::read()` result (or `{}` for absent — D-04) | Yes — the caller resolves the live snapshot; the renderer is a pure function | FLOWING |
| `live_fragment_hook` in `apply_event` | `state_json` | `serde_json::to_value(&state)` computed at step-5 — reused directly (WR-03 fix) | Yes — the persisted post-apply state | FLOWING |

---

### Behavioral Spot-Checks

Step 7b is SKIPPED for the server-side render path (no runnable entry points without a full app boot). The projection layer integration test `live_fragment_hook` serves as the behavioral proof at the correct abstraction level — it uses a real in-process broadcaster and asserts actual frame delivery, which is stronger than a CLI smoke check.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| LIVE-02 | 260-01, 260-02, 260-03, 260-04 | `LiveFragment` element + projection render hook + client runtime | SATISFIED | All four plans complete; 4/4 success criteria verified against actual codebase; 12+ new tests passing |

Note: LIVE-02 is defined inline in `.planning/ROADMAP.md` (Requirement → Phase Mapping), not in `.planning/REQUIREMENTS.md` — by design per CONTEXT.md canonical refs.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-projection/src/runtime.rs` | ~190 (original) | WR-02: hook skipped when delta broadcast fails | RESOLVED | Fixed in `f8ded454` — hook now fires before `send_result` error check |
| `ferro-projection/src/runtime.rs` | ~190 (original) | WR-03: silent `Value::Null` on serialization failure | RESOLVED | Fixed in `f8ded454` — hook receives `state_json` already computed at step-5 |
| `ferro-json-ui/src/runtime/live_fragment.rs` | 37-40 (original) | WR-01: CSS selector injection via server-pushed `msg.channel` | RESOLVED | Fixed in `f8ded454` — replaced querySelector with pre-built `channelMap`; SC3 test asserts absence of the unsafe selector pattern |
| `ferro-json-ui/src/render/containers.rs` | ~1675 (original) | IN-01: `inner_html` in format string without explicit pre-escaped annotation | RESOLVED | Fixed in `f8ded454` — comment added noting `inner_html` is output of `render_spec_to_html` (already-escaped trusted HTML) |

No open anti-patterns. All four code-review findings (WR-01/02/03, IN-01) were resolved in `f8ded454` before the review was marked resolved in `b0e473bc`.

---

### Locked Decisions Verified

| Decision | Requirement | Status | Evidence |
|----------|-------------|--------|----------|
| D-01: NO `ferro-projection` → `ferro-json-ui` dependency | Zero `ferro-json-ui`/`ferro_json_ui` references in `ferro-projection/src/` | VERIFIED | `grep -rn "ferro-json-ui\|ferro_json_ui" ferro-projection/src/` returns 0 results |
| D-06: BUILTIN_TYPES + dispatch + BUILTIN_SPECS + count all at 53 | Canonical catalog count 53; ferro-mcp mirror ALSO 53 | VERIFIED | catalog.rs count guard asserts 53; ferro-mcp json_ui_catalog.rs count assertion is 53; `"LiveFragment"` present in both |
| ferro-base.css NOT regenerated | No ferro-base.css changes in any Phase 260 commit | VERIFIED | `git log 402c8123..b0e473bc --name-only | grep ferro-base.css` returns empty; `LiveFragment` uses only `data-*` attributes, no new Tailwind utility classes |

---

### Human Verification Required

#### 1. Live-browser end-to-end DOM swap

**Test:** Boot the sample `app/` (or a ferro application) with a `ProjectionRuntime` registered and a `LiveFragment` element on a page (projecting the same `projection`/`key`). Dispatch a domain event that changes the projection's state. Observe the browser tab.

**Expected:** The `LiveFragment` container's `innerHTML` updates in place (reflecting the new snapshot HTML pushed from the server) without any page reload or client-side re-request. The update should be visible within 1-2 seconds of the event dispatch.

**Why human:** The automated integration test `live_fragment_hook` proves the HTML reaches the broadcast channel (`projection.{name}.{key}`) and the correct `fragment` event is delivered to a subscribed client — the projection+broadcast layer is fully proven in-process. The final browser DOM swap requires a running HTTP server, an active `/_ferro/ws` WebSocket handshake, and a real browser to observe. Chrome MCP can be used for this if the app is booted.

---

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `generation_context` documentation for `LiveFragment` usage pattern | Phase 262 | Phase 262 SC#2: "`generation_context` documents when to use `LiveFragment` (live projection binding)…" |
| 2 | `docs/src` coverage with usage examples | Phase 262 | Phase 262 SC#3: "`docs/src` covers all three capabilities with at least one usage example each" |
| 3 | Reconnect strategy in `setupLiveFragments` client runtime | Future phase (explicitly deferred in source comment) | Source comment in `live_fragment.rs`: "Reconnect strategy deferred to a future phase." D-03 documents this as Claude's Discretion deferred. |

---

### Gaps Summary

No gaps. All four ROADMAP success criteria are verified against the actual codebase with passing tests and correct wiring. All code-review findings (WR-01/02/03 + IN-01) are resolved. The only item requiring action is the live-browser DOM swap, which is a human verification step by design — the automated proof covers the broadcast-delivery layer completely; the DOM swap is a browser concern.

---

_Verified: 2026-07-26T18:45:00Z_
_Verifier: Claude (gsd-verifier)_
