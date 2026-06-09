---
phase: 173-make-json-view-v2-projection-roundtrip-test
plan: "01"
subsystem: ferro-cli
tags: [json-ui, projections, service-def, cli, ai]
dependency_graph:
  requires: [ferro-mcp (scaffold_core), ferro-json-ui (Spec::from_service_def), ferro-projections (derive_intents)]
  provides: [make:json-view ServiceDef-driven generation path]
  affects: [ferro-cli/src/commands/make_json_view.rs, ferro-cli/src/main.rs]
tech_stack:
  added: []
  patterns: [ServiceDef → derive_intents → Spec::from_service_def deterministic render, cfg(feature = "projections") gating with static template fallback]
key_files:
  created: []
  modified:
    - ferro-cli/src/commands/make_json_view.rs
    - ferro-cli/src/main.rs
decisions:
  - "D-03 applied: generate_with_ai / build_json_view_pass1 / build_json_view_pass2 deleted; ServiceDef is now the single intermediary"
  - "D-04 applied: --from-service-json <path> flag for pre-serialized ServiceDef, no LLM call"
  - "D-02 preserved: Spec::from_json write-gate re-parse inside render_service_def"
  - "cfg(feature = projections) gates both new paths; static template fallback when feature disabled"
metrics:
  duration: "618s"
  completed: "2026-06-09"
  tasks: 3
  files_modified: 2
---

# Phase 173 Plan 01: make:json-view ServiceDef Pipeline Summary

CLI command `ferro make:json-view` rewired from a direct NL→spec two-pass LLM flow to a two-stage ServiceDef projection pipeline: NL via `scaffold_core` or `--from-service-json` → `ServiceDef` → `derive_intents` → `Spec::from_service_def` → catalog write-gate validation.

## What Was Built

### Task 1: --from-service-json arg added to MakeJsonView clap variant
Added `from_service_json: Option<String>` field to the `MakeJsonView` variant in `ferro-cli/src/main.rs` and threaded it through the dispatch arm into `make_json_view::run`.

### Task 2: ServiceDef-driven projection path
Rewrote `ferro-cli/src/commands/make_json_view.rs`:

- **Deleted** `generate_with_ai`, `build_json_view_pass1`, `build_json_view_pass2` and their associated `ferro_ai::client::{Message, Role}` / `CompletionRequest` imports (D-03, feature-branch convention).
- **Added** `render_service_def` helper (`#[cfg(feature = "projections")]`): calls `derive_intents(&service)` → `Spec::from_service_def(&service, &intents, &VisualContext::default())` → `serde_json::to_string_pretty` → `Spec::from_json` write-gate re-parse (D-02). Falls back to static template on any failure with a yellow warning.
- **NL path** (`-d "<text>"` with AI configured): creates `tokio::runtime::Runtime`, calls `ferro_mcp::tools::ai_scaffold::scaffold_core(&desc, &cwd)`, then `render_service_def`. Fallback to static template on scaffold error.
- **File path** (`--from-service-json <path>`): deserializes `ServiceDef` from JSON file via `serde_json::from_str`, then `render_service_def`. Exits with error on parse/IO failure (no network involved).
- **`#[cfg(not(feature = "projections"))]`** fallback for both new paths: clear error message or static template.
- Directory creation, already-exists guard, file-write, and usage-guidance sections preserved unchanged.

### Task 3: Full quality gate
`cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` all green.

## Decisions Made

- `render_service_def` accepts only `&ServiceDef` (no LLM client parameter) — `run()` owns the optional NL scaffold call, keeping the helper purely deterministic.
- `_path` underscore prefix on the `if let` binding; `let path = _path;` inside the `#[cfg(feature = "projections")]` block avoids unused-variable warning under `--no-default-features`.
- `write_content` extracted as a helper to avoid code duplication between the early-return runtime-failure path and the normal file-write path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unused import `ferro_json_ui::global_catalog`**
- **Found during:** Task 2 build
- **Issue:** The old `generate_with_ai` function called `global_catalog()` directly; the new `render_service_def` calls it indirectly via `Spec::from_service_def`. The top-level import became unused.
- **Fix:** Removed the `use ferro_json_ui::global_catalog;` import.
- **Files modified:** `ferro-cli/src/commands/make_json_view.rs`
- **Commit:** 7694932d

**2. [Rule 1 - Bug] Unused variable `path` under `--no-default-features`**
- **Found during:** Task 3 `cargo build -p ferro-cli --no-default-features`
- **Issue:** `let Some(ref path) = from_service_json` bound `path` but the `#[cfg(not(feature = "projections"))]` arm doesn't use it, producing an unused-variable warning that would be a clippy -D warnings failure.
- **Fix:** Changed to `ref _path` at the binding site; aliased with `let path = _path;` inside the projections-enabled cfg block.
- **Files modified:** `ferro-cli/src/commands/make_json_view.rs`
- **Commit:** 7694932d

## Known Stubs

None. The static template fallback is intentional behavior, not a stub — it produces a valid spec that can be manually edited.

## Threat Flags

No new threat surfaces beyond what the plan's threat model describes. `sanitize_description` mitigation (T-173-01) is inherited unchanged from `scaffold_core`.

## Self-Check

```
[ -f "ferro-cli/src/commands/make_json_view.rs" ] → FOUND
[ -f "ferro-cli/src/main.rs" ] → FOUND
git log --oneline | grep "7694932d" → FOUND
```

## Self-Check: PASSED

- `ferro-cli/src/commands/make_json_view.rs` — exists
- `ferro-cli/src/main.rs` — exists
- Commit `7694932d` — verified in git log
- `grep -c "Spec::from_service_def" ferro-cli/src/commands/make_json_view.rs` = 3 (>= 1)
- `grep -c "scaffold_core" ferro-cli/src/commands/make_json_view.rs` = 3 (>= 1)
- `grep -c "generate_with_ai\|build_json_view_pass1\|build_json_view_pass2" ferro-cli/src/commands/make_json_view.rs` = 0
- `grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs` = 0
- `cargo build -p ferro-cli` exits 0
- `cargo build -p ferro-cli --no-default-features` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo clippy --all --all-targets -- -D warnings` exits 0
- `cargo test --all-features` exits 0
