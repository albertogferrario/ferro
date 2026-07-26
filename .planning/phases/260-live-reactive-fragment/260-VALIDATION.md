---
phase: 260
slug: live-reactive-fragment
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-26
---

# Phase 260 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` (cargo discovers by convention) |
| **Config file** | none — `cargo test` convention |
| **Quick run command** | `cargo test -p ferro-json-ui live_fragment && cargo test -p ferro-projection live_fragment` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~90–180 seconds (full suite; workspace is large — watch disk, see project memory) |

---

## Sampling Rate

- **After every task commit:** Run the quick command for the touched crate (`cargo test -p ferro-json-ui live_fragment` or `-p ferro-projection live_fragment`).
- **After every plan wave:** Run `cargo test --all-features` for the touched crates plus the drift-guard test.
- **Before `/gsd-verify-work`:** Full CI-exact gate green — `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features`.
- **Max feedback latency:** ~180 seconds (full suite).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 260-01-* | 01 | 1 | LIVE-02 (D-01/D-02) | T-260-01 | Fragment hook cannot suppress or replace the base `delta` broadcast; only adds a second `fragment` message | integration | `cargo test -p ferro-projection live_fragment` | ❌ W0 | ⬜ pending |
| 260-02-* | 02 | 2 | LIVE-02 (D-04/D-05) | — | Absent snapshot renders empty container, never an error/panic; snapshot JSON is the only data scope (no arbitrary eval) | unit | `cargo test -p ferro-json-ui live_fragment` | ❌ W0 | ⬜ pending |
| 260-03-* | 03 | 2 | LIVE-02 (D-03) | T-260-02 | Client runtime swaps innerHTML only; no `eval`, no WASM, no client state; subscribes to declared channels only | static+unit | `cargo test -p ferro-json-ui live_fragment_no_wasm` | ❌ W0 | ⬜ pending |
| 260-04-* | 03 | 3 | LIVE-02 (D-06) | — | Catalog/drift-guard lockstep; count bumped 52→53 | drift guard | `cargo test -p ferro-json-ui builtin_types_count_drift_guard` | ❌ W0 (update existing) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

New tests to author (no framework install needed — Rust built-in):

- [ ] `ferro-projection/src/runtime.rs` (tests mod) — `fragment_hook_fires_after_apply_event`: `with_fragment_renderer` hook fires once per `apply_event`, receives `(key, snapshot_value)` with the post-apply state; asserts the existing `"delta"` broadcast is untouched (SC2, D-01/D-02).
- [ ] `ferro-json-ui` render tests — `render_live_fragment_renders_container_with_channel`: first-paint emits `data-live-fragment` + `data-channel="projection.{name}.{key}"` and renders the child template against the snapshot (SC1, D-05).
- [ ] `ferro-json-ui` render tests — `render_live_fragment_absent_snapshot_renders_container`: `{}` snapshot renders the container with no error comment / no panic (D-04).
- [ ] `ferro-json-ui` runtime tests — `live_fragment_no_wasm` / extend `bundle_contains_all_setup_functions` + `dispatcher_invokes_every_setup` with `setupLiveFragments`; assert the assembled `FERRO_RUNTIME_JS` contains no WASM instantiation and no client state store (SC3).
- [ ] `ferro-json-ui/src/catalog.rs` — update the pinned count test `BUILTIN_TYPES.len() == 52` → `53`; the `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` guard passes with the new `BUILTIN_SPECS` entry (D-06).
- [ ] SC4 (one binding pattern): a documented non-goal + grep-absence check — no list/collection reconciliation code paths in the LiveFragment renderer.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end live update in a real browser (event → visible innerHTML swap over `/_ferro/ws`) | LIVE-02 | Requires a running server + WS + browser; the automated integration test asserts the HTML lands on the channel, not the DOM swap | Boot the sample app with a registered projection + a `LiveFragment` page, open in a browser, dispatch a domain event, observe the fragment swap without reload. Optional — Chrome MCP. |

*All core phase behaviors have automated verification; only the live-browser swap is manual.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
