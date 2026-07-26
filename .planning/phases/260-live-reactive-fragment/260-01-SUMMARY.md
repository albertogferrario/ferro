---
phase: 260-live-reactive-fragment
plan: "01"
subsystem: ferro-projection
tags: [live-fragment, projection-runtime, fragment-hook, broadcast, sc2]
dependency_graph:
  requires: []
  provides: [fragment_hook_seam, with_fragment_renderer_builder]
  affects: [ferro-projection/src/runtime.rs]
tech_stack:
  added: []
  patterns: [type-erased-dyn-Fn-hook, consuming-builder, tokio-spawn-async-broadcast]
key_files:
  modified:
    - ferro-projection/src/runtime.rs
decisions:
  - "FragmentHook type alias introduced to satisfy clippy::type_complexity (fragment_hook field)"
  - "Import from ferro_broadcast root (BroadcastMessage, ServerMessage) not the private message submodule"
  - "Hook comment rephrased to avoid the literal string 'ferro-json-ui' (D-01 grep assertion)"
metrics:
  duration: 384s
  completed: "2026-07-26"
  tasks: 3
  files: 1
requirements: [LIVE-02]
---

# Phase 260 Plan 01: Fragment Hook Seam Summary

Adds the renderer-agnostic re-render hook seam to `ferro-projection`: `fragment_hook` field on `ProjectionRuntime<P>`, consuming `with_fragment_renderer()` builder (default `None`), synchronous hook invocation in `apply_event` step 6.5 after the delta broadcast, and three tests proving SC2 end-to-end at the projection layer with zero dependency on any renderer crate.

## What Was Built

- **`FragmentHook` type alias** — `Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>` — named to pass clippy::type_complexity.
- **`fragment_hook: Option<FragmentHook>`** field on `ProjectionRuntime<P>`, defaulting to `None` in `new()`. Existing callers unchanged.
- **`with_fragment_renderer(hook)`** — consuming builder that stores `Some(Arc::new(hook))`. The hook is type-erased at the `serde_json::Value` snapshot boundary; `ferro-projection` gains no dependency on `ferro-json-ui`.
- **Step 6.5 invocation** in `apply_event` — fires synchronously inside the per-key Mutex, AFTER the step-6 delta broadcast succeeds. `serde_json::to_value` failure degrades to `Value::Null` (never panics, never aborts the apply — the snapshot is already persisted).
- **Three new tests:**
  - `fragment_hook_fires_after_apply_event` — records exactly one call with the correct key and post-apply `total` value.
  - `apply_event_without_hook_still_broadcasts_delta_unchanged` — regression pin (D-02): no-hook path succeeds identically to before.
  - `live_fragment_hook` — SC2 full-chain: real `Arc<Broadcaster>`, subscribed `mpsc` test client on `projection.test.counter.default-key`, production-shaped hook drives a `tokio::spawn`'d second `fragment` broadcast carrying `{ html }`. Bounded poll loop collects both the base `delta` frame and the `fragment` frame and asserts additive delivery (D-02).

## Decisions Made

- **FragmentHook type alias** — clippy::type_complexity fires on the raw `Arc<dyn Fn...>` field type. A named alias is the clean fix vs. a per-field `#[allow]`.
- **Import path fix** — `ferro_broadcast::message` is a private module; `BroadcastMessage` and `ServerMessage` must be imported from the crate root (`use ferro_broadcast::{BroadcastMessage, ServerMessage}`).
- **Comment rephrasing** — the plan's D-01 acceptance criterion greps for the literal string `ferro-json-ui` in `runtime.rs`; the doc comment was rephrased to "renderer-free (no renderer crate dependency)" to keep the grep count at 0 while preserving the intent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro_broadcast::message is a private module**
- **Found during:** Task 3 compile
- **Issue:** The plan's test code used `use ferro_broadcast::message::{BroadcastMessage, ServerMessage}` but `message` is a private module; types are re-exported at the crate root.
- **Fix:** Changed import to `use ferro_broadcast::{BroadcastMessage, ServerMessage}`.
- **Files modified:** `ferro-projection/src/runtime.rs`
- **Commit:** 9629b0bf

**2. [Rule 2 - Clippy] type_complexity on fragment_hook field**
- **Found during:** Task 3 pre-commit clippy gate
- **Issue:** `cargo clippy -D warnings` fired `clippy::type_complexity` on the raw `Option<Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>>` field type.
- **Fix:** Introduced `type FragmentHook = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;` alias and changed the field to `Option<FragmentHook>`.
- **Files modified:** `ferro-projection/src/runtime.rs`
- **Commit:** 9629b0bf

**3. [Rule 2 - Grep assertion] Literal "ferro-json-ui" in doc comment**
- **Found during:** Task 3 acceptance criteria check
- **Issue:** The SC2 test doc comment contained the string "ferro-json-ui" explaining its absence. The D-01 grep assertion counts literal occurrences including comments.
- **Fix:** Rephrased comment to "renderer-free (no renderer crate dependency)".
- **Files modified:** `ferro-projection/src/runtime.rs`
- **Commit:** 9629b0bf

## Known Stubs

None. The fragment hook seam is fully wired. The SC2 test synthesizes HTML with `format!` (intentional — the real renderer lives in `ferro-json-ui`, wired in Plan 02).

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced. The `fragment_hook` field exposes the serialized `P::State` to a user-registered closure — the same state that is already persisted as JSON in `projection_snapshots`. T-260-01 (additive-only tamper guard) is proven by `apply_event_without_hook_still_broadcasts_delta_unchanged` + `live_fragment_hook` (both delta and fragment frames delivered). T-260-02 and T-260-03 are accepted per the plan's threat register.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `ferro-projection/src/runtime.rs` exists | FOUND |
| Commit 2c8a4efa (Task 1: field + builder) | FOUND |
| Commit 95f6ebb8 (Task 2: hook invocation + tests) | FOUND |
| Commit 9629b0bf (Task 3: SC2 live_fragment_hook) | FOUND |
| `grep -c "ferro-json-ui\|ferro_json_ui" ferro-projection/src/runtime.rs` | 0 |
| `cargo test -p ferro-projection --lib` | 28 passed, 0 failed |
