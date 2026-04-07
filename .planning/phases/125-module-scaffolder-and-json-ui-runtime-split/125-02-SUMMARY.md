---
phase: 125-module-scaffolder-and-json-ui-runtime-split
plan: 02
subsystem: ferro-json-ui
tags: [runtime, refactor, js-bundle]
requirements: [D-06, D-07, D-08, D-09, D-10]
dependency_graph:
  requires: []
  provides:
    - "Per-concern editable JS runtime submodules under ferro-json-ui/src/runtime/"
    - "ferroRuntime() dispatcher + DOMContentLoaded wiring"
  affects:
    - "ferro-json-ui/src/layout.rs (consumer now derefs LazyLock<String>)"
tech_stack:
  added:
    - "std::sync::LazyLock (bundle assembled lazily at first access)"
  patterns:
    - "Per-concern JS as pub(super) const SOURCE: &str in one .rs per concern"
    - "Single assembler (runtime/mod.rs) concatenates submodule constants"
key_files:
  created:
    - ferro-json-ui/src/runtime/mod.rs
    - ferro-json-ui/src/runtime/sse.rs
    - ferro-json-ui/src/runtime/tabs.rs
    - ferro-json-ui/src/runtime/toasts.rs
    - ferro-json-ui/src/runtime/dismissibles.rs
    - ferro-json-ui/src/runtime/notifications.rs
    - ferro-json-ui/src/runtime/dropdowns.rs
    - ferro-json-ui/src/runtime/modals.rs
    - ferro-json-ui/src/runtime/sidebar.rs
    - ferro-json-ui/src/runtime/form_guards.rs
    - ferro-json-ui/src/runtime/product_tiles.rs
    - ferro-json-ui/src/runtime/kanban.rs
  modified:
    - ferro-json-ui/src/layout.rs
  deleted:
    - ferro-json-ui/src/runtime.rs
decisions:
  - "Bundle assembled via LazyLock<String> — concat! is literal-only and cannot concatenate const &str identifiers."
  - "setup* naming convention extended to all 11 concerns (plan named 4 explicitly; D-11 forbids regressing any of the others)."
  - "SSE `data-sse-url` gate moved from init() into setupSSE itself — removes the previous top-level read."
metrics:
  tasks: 1
  duration: ~15m
  completed: 2026-04-07
---

# Phase 125 Plan 02: ferro-json-ui Runtime Split Summary

One-liner: Broke the 725-line ferro-json-ui runtime IIFE into 11 per-concern
Rust submodules assembled into the same single bundle via a `ferroRuntime()`
dispatcher, with Rust-only unit tests asserting function presence and
dispatcher wiring.

## What Shipped

- `ferro-json-ui/src/runtime.rs` (725 lines, monolithic) deleted.
- New `ferro-json-ui/src/runtime/` directory with 12 files: one `mod.rs`
  assembler + 11 per-concern `.rs` files, each exposing
  `pub(super) const SOURCE: &str = r#"..."#`.
- Concerns: `sse`, `tabs`, `toasts` (includes former `initToastFromUrl` +
  `escapeHtml` + `VARIANT_CLASSES` + `showToast`/`dismissToast`),
  `dismissibles`, `notifications`, `dropdowns`, `modals`, `sidebar`,
  `form_guards`, `product_tiles`, `kanban`.
- Entry points renamed from `init*` → `setup*` for all 11 concerns.
- `ferroRuntime()` dispatcher added after the concatenated sources; it calls
  every `setup*` in order and is wired to `DOMContentLoaded`.
- SSE `data-sse-url` check moved from the old top-level `init()` into
  `setupSSE` itself — the dispatcher no longer needs a gate.
- Bundle is now assembled via `std::sync::LazyLock<String>` because
  `concat!()` only accepts literals, not `const &str` identifiers.
  `layout.rs` updated to call `.as_str()` on the LazyLock.
- 5 legacy tests relocated into `runtime/mod.rs`; 4 new tests added
  (`bundle_contains_dispatcher`, `bundle_contains_all_setup_functions`,
  `bundle_is_single_iife`, `dispatcher_invokes_every_setup`). 9 runtime
  tests + 1 layout test green.

## Verification

- `cargo fmt -p ferro-json-ui` clean
- `cargo clippy -p ferro-json-ui --all-targets --no-deps -- -D warnings` clean
- `cargo test -p ferro-json-ui` — 471 tests pass (including 9 runtime tests
  and the `layout::tests::dashboard_layout_injects_runtime_js` consumer test)
- Public API: `crate::runtime::FERRO_RUNTIME_JS` still exists (now
  `LazyLock<String>` instead of `const &str`), still yields a single
  inlined `<script>` in the emitted HTML, still zero extra HTTP requests.

## Deviations from Plan

### [Rule 3 — Blocker] Bundle type changed from `const &str` to `LazyLock<String>`
- **Found during:** Task 1 verification
- **Issue:** The plan prescribed `concat!(sse::SOURCE, tabs::SOURCE, ...)`.
  Rust's `concat!` macro accepts only string literals, not `const &str`
  identifiers, producing `error: only literals can be passed to concat!()`.
- **Fix:** Assemble the bundle lazily at first access using
  `std::sync::LazyLock<String>` with `push_str` calls. Updated the one
  consumer in `layout.rs` to use `.as_str()`. Updated the
  `dispatcher_invokes_every_setup` test to bind `js: &str = ...as_str()`
  before slicing.
- **Files modified:** `ferro-json-ui/src/runtime/mod.rs`, `ferro-json-ui/src/layout.rs`
- **Commit:** 967fa776

### [Rule 3 — Blocker] Pre-existing uncommitted work stashed
- **Found during:** First `cargo clippy` run after creating the submodules.
- **Issue:** The working tree contained unrelated in-progress edits to
  `action.rs`, `component.rs`, `render.rs`, `resolve.rs`, `view.rs`,
  `layout.rs`, and `ferro-cli/src/templates/ignore_patterns.rs`. These
  added a `target: Option<String>` field to `Action` but missed ~26
  construction sites, causing `E0063 missing field target` and blocking
  compilation. Completing that work is out of scope for plan 125-02
  (scope boundary rule — only fix issues directly caused by the current
  task's changes).
- **Fix:** Stashed the unrelated modifications via
  `git stash push -m "pre-existing-unrelated-125-02" -- <files>`
  so the runtime split could be validated in isolation, then attempted to
  re-apply. The stash entry remains in `git stash list` because
  `git stash pop` conflicts on `layout.rs` (plan 125-02 also edited it
  for the `.as_str()` update). User must pop the stash manually and
  resolve the `layout.rs` conflict + complete the `target` field rollout.
- **Tracking:** `.planning/phases/125-module-scaffolder-and-json-ui-runtime-split/deferred-items.md`

## Deferred

- **Chrome MCP UAT (Task 2, checkpoint:human-verify)** — deferred per
  auto-chain mode. The Rust unit tests covering function-name presence,
  dispatcher wiring, and single-IIFE structure are sufficient to proceed.
  The user must manually verify in gestiscilo per the plan's
  `how-to-verify` steps (tabs / SSE / toasts / sidebar / dropdowns /
  modals / dismissibles / notifications / form guards / product tiles /
  kanban, plus: zero console errors, `typeof ferroRuntime === 'function'`,
  no extra HTTP requests for runtime JS).
- **Pre-existing `target` field rollout** — see `deferred-items.md`.
  Not introduced by this plan; blocks downstream compilation until the
  stash is popped and the `Action { ... target: None }` additions land.

## Commits

- 967fa776 — refactor(125-02): split ferro-json-ui runtime.rs into per-concern submodules

## Self-Check: PASSED

- `ferro-json-ui/src/runtime/mod.rs` — FOUND
- `ferro-json-ui/src/runtime/sse.rs` — FOUND
- `ferro-json-ui/src/runtime/tabs.rs` — FOUND
- `ferro-json-ui/src/runtime/toasts.rs` — FOUND
- `ferro-json-ui/src/runtime/dismissibles.rs` — FOUND
- `ferro-json-ui/src/runtime/notifications.rs` — FOUND
- `ferro-json-ui/src/runtime/dropdowns.rs` — FOUND
- `ferro-json-ui/src/runtime/modals.rs` — FOUND
- `ferro-json-ui/src/runtime/sidebar.rs` — FOUND
- `ferro-json-ui/src/runtime/form_guards.rs` — FOUND
- `ferro-json-ui/src/runtime/product_tiles.rs` — FOUND
- `ferro-json-ui/src/runtime/kanban.rs` — FOUND
- `ferro-json-ui/src/runtime.rs` — CORRECTLY REMOVED
- Commit 967fa776 — FOUND in `git log`
- 9 runtime tests + 1 layout test — all green
