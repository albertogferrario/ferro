---
phase: 259-request-scoped-memoization
plan: "03"
subsystem: framework + ferro-macros
tags: [memoization, render-path, sc-3, rustdoc, ci-gate, projections]
dependency_graph:
  requires:
    - "259-01: MemoStore, MEMO_STORE task-local, current_memo_store()"
    - "259-02: #[memoize] proc-macro, macro_tests module"
  provides:
    - "SC-3 render-path proof: N intents over one key issue one fetch (render_path_tests module)"
    - "CI-exact gate green: fmt + clippy --all-features + test --all-features"
    - "Rustdoc audit complete: all exported memo symbols have neutral /// docs"
  affects:
    - framework/src/memo/mod.rs (render_path_tests mod registration added)
    - framework/src/memo/render_path_tests.rs (new — SC-3 in-crate test module)
    - ferro-macros/src/lib.rs (#[memoize] doc expanded: D-02/D-04/constraints)
tech_stack:
  added: []
  patterns:
    - "In-crate #[cfg(all(test, feature = projections))] mod — projections-gated SC-3 proof reachable via pub(crate) MEMO_STORE"
    - "static AtomicUsize FETCH_COUNTER + #[memoize] loader — honest single-fetch counter pattern"
    - "derive_intents + Spec::from_service_def over Browse + Summarize intents — real render pass, not a stub"
key_files:
  created:
    - framework/src/memo/render_path_tests.rs
  modified:
    - framework/src/memo/mod.rs
    - ferro-macros/src/lib.rs
decisions:
  - "render_path_tests.rs as a sibling file (in-crate) rather than inline in mod.rs — keeps mod.rs readable; both are in-crate so MEMO_STORE remains reachable"
  - "product ServiceDef fixture (id/name/price/stock) chosen because Money+Quantity signals derive Browse (baseline) + Summarize — confirmed by derive.rs:164-168 and existing tests"
  - "Rustdoc audit found all exported symbols already documented from Plans 01-02; only the #[memoize] macro registration doc needed expansion (D-04 Err-cached semantics + constraints list)"
metrics:
  duration: "~15 minutes"
  completed: "2026-07-21"
  tasks: 2
  files: 3
requirements: [LIVE-01]
---

# Phase 259 Plan 03: SC-3 Render-Path Proof + CI Gate Summary

SC-3 proved honestly: N intents over one key issue exactly one underlying fetch through the memo store, via a genuine multi-intent render (`derive_intents` + `Spec::from_service_def`) with a constructed `#[memoize]`d loader asserting `FETCH_COUNTER == 1` across Browse + Summarize renders. CI-exact gate green.

## What Was Built

**Task 1 — SC-3 render-path integration harness** (`1539a53a`)

- `framework/src/memo/render_path_tests.rs` (new, 152 lines): in-crate test module registered via `#[cfg(all(test, feature = "projections"))] mod render_path_tests;` in `framework/src/memo/mod.rs`.
  - `static FETCH_COUNTER: AtomicUsize` — counts underlying loader invocations within the scope.
  - `#[ferro::memoize] async fn load_model_set(key: u32) -> Vec<String>` — the memoized loader; body increments `FETCH_COUNTER` and returns a representative payload.
  - `fn product_service() -> ServiceDef` — fixture with `id`/`name`/`price`/`stock` fields; Money+Quantity signals derive both Browse (baseline) and Summarize (Money signal, derive.rs:164-168).
  - `#[tokio::test] async fn render_path_single_fetch()`:
    1. Derives intents from the schema via `derive_intents(&service)` (schema-only, no I/O).
    2. Enters `with_memo_scope(memo_scope(), ...)`.
    3. Calls `load_model_set(1)` → Browse render via `Spec::from_service_def` (intent_index = browse_idx).
    4. Calls `load_model_set(1)` again → Summarize render via `Spec::from_service_def` (intent_index = summarize_idx).
    5. Asserts both `Spec` results have `schema == "ferro-json-ui/v2"` and contain a root element (real render path ran).
    6. Asserts `FETCH_COUNTER.load(SeqCst) == 1` — SC-3 proven.
- `framework/src/memo/mod.rs`: added `#[cfg(all(test, feature = "projections"))] mod render_path_tests;` after the `mod macro_tests;` block.

D-05 honesty preserved: the render pass is schema-only (`Spec::from_service_def` takes `&ServiceDef` + `&[IntentScore]`, no I/O). The memoized loader is the data-loader a real multi-intent handler calls once and reuses — not a fabricated fetch inside the renderer.

**Task 2 — Rustdoc audit + CI-exact gate** (`dcdb3009`)

- `ferro-macros/src/lib.rs`: `#[memoize]` registration doc expanded to explicitly document:
  - D-04: "The full return value — including `Result::Err` — is cached for the duration of the request."
  - D-02: "Outside a request context the body runs normally with no caching (graceful no-op, D-02)."
  - All five constraints in a `# Constraints` section (Hash args, Clone+Send+Sync+'static return, `&self` exclusion, async-only, Pat::Ident-only).

All other exported memo symbols (`MemoSlot`, `MemoKey`, `MemoKey::new`, `MemoStore`, `MemoStore::new`, `MemoStore::get_or_insert`, `current_memo_store`) already had complete `///` docs from Plans 01-02. Audit confirmed — no gaps found.

## CI-Exact Gate Evidence

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | Exit 0 — formatting clean |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | Exit 0 — 0 warnings |
| `cargo test --all-features` | Exit 0 — all test results `ok`, no FAILEDs |
| `cargo doc -p ferro-rs --no-deps` | Exit 0 — 0 warnings |
| `cargo doc -p ferro-macros --no-deps` | Exit 0 — 0 warnings |

SC-3 test result from `cargo test -p ferro-rs render_path_single_fetch --all-features`:
```
test memo::render_path_tests::render_path_single_fetch ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 633 filtered out
```

Disk at gate time: 29 GiB free — no ENOSPC risk.

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Documentation

The plan required `render_path_tests` to be either inline in `mod.rs` or as a sibling file `render_path_tests.rs`. The sibling-file approach was chosen for readability; both are in-crate and satisfy the pub(crate) reachability requirement.

## Known Stubs

None. The SC-3 test drives the real `derive_intents` + `Spec::from_service_def` pipeline. No stub renderer or fabricated fetch was introduced.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. T-259-07 mitigation confirmed: `render_path_single_fetch` enters a fresh `MEMO_STORE.scope` per test invocation; the single-fetch assertion transitively confirms no cross-scope reuse leaks a prior fetch.

## Self-Check: PASSED

- `framework/src/memo/render_path_tests.rs` exists: FOUND
- `framework/src/memo/mod.rs` contains `mod render_path_tests`: FOUND
- `framework/src/memo/render_path_tests.rs` contains `derive_intents`: FOUND
- `framework/src/memo/render_path_tests.rs` contains `from_service_def`: FOUND
- `framework/src/memo/render_path_tests.rs` contains `FETCH_COUNTER`: FOUND
- `framework/src/memo/render_path_tests.rs` contains `== 1`: FOUND
- `framework/tests/memoize_render_path.rs` does NOT exist: CONFIRMED
- Commit `1539a53a` exists: FOUND
- Commit `dcdb3009` exists: FOUND
- SC-3 test passes: VERIFIED (`cargo test -p ferro-rs render_path_single_fetch --all-features` → ok)
- CI-exact gate: fmt exit 0, clippy exit 0, test --all-features exit 0
