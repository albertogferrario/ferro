---
phase: 215-non-visual-rendering-context-basecontext-intent-extensions
plan: "02"
subsystem: ferro-json-ui, ferro-mcp
tags: [rendering, context, intent, label, visual-context, base-context]
requirements: [CHAN-01, CHAN-02]

dependency_graph:
  requires:
    - BaseContext.evaluated_guards (Plan 01 — ferro-projections)
    - BaseContext.verbosity (Plan 01 — ferro-projections)
    - Intent::label() (Plan 01 — ferro-projections)
  provides:
    - VisualContext { base: BaseContext, mode, templates } (single source of truth)
    - ferro-mcp IntentInfo.intent lowercase via Intent::label()
  affects:
    - ferro-json-ui (VisualContext shape change — all consumers updated)
    - ferro-mcp (label value change: PascalCase → lowercase for intent strings)
    - ferro-mcp-server (no source edits — recompiles cleanly)

tech_stack:
  added: []
  patterns:
    - "Embedding BaseContext in VisualContext (composition over duplication)"
    - "Hand-written Default impl retained when one field has no Default derive (RenderMode)"
    - "BaseContext import scoped to test module in builder.rs (avoids unused-import warning on non-test path)"
    - "Intent::label().to_string() for user-facing intent label strings"

key_files:
  created: []
  modified:
    - ferro-json-ui/src/projection/mod.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-mcp/src/tools/render_projection.rs
    - ferro-mcp/src/tools/generate_projection.rs
    - ferro-mcp/src/tools/projection_coverage.rs
    - ferro-ai/tests/projection_roundtrip.rs
    - ferro-mcp/tests/agent_harness.rs

decisions:
  - "Used EMBED approach (D-02, preferred): VisualContext.base: BaseContext collapses parallel sources of truth"
  - "BaseContext import moved to test module in builder.rs — unused in production path, avoiding unused-import warning"
  - "full cargo test --all-features skipped (disk ~8.8 GB free, ENOSPC-prone); ran per-crate gate: ferro-json-ui --all-features + ferro-mcp (308 + 307 tests all green)"

metrics:
  duration: "480s"
  completed: "2026-06-13"
  tasks_completed: 2
  files_modified: 7
---

# Phase 215 Plan 02: VisualContext BaseContext Embedding + MCP Label Migration Summary

Adopts the Plan 01 `ferro-projections` surface across downstream consumers, completing CHAN-01 (VisualContext single source of truth) and CHAN-02 (label migration) at the consumer level. The embed approach was used throughout (D-02, no fallback needed).

## What Was Built

### Task 1 — Embed `base: BaseContext` in `VisualContext`; migrate all struct-literal sites

**File:** `ferro-json-ui/src/projection/mod.rs`

- `BaseContext` added to import: `use ferro_projections::render::{BaseContext, Renderer}`
- `VisualContext` struct: flat fields `intent_index: usize` and `current_state: Option<String>` replaced by `pub base: BaseContext`
- Hand-written `impl Default for VisualContext` retained (RenderMode has no `Default` derive); produces: `base: BaseContext::default(), mode: RenderMode::Display, templates: None`
- Test `visual_context_default_has_sensible_values`: two assertions updated to `ctx.base.intent_index` / `ctx.base.current_state`
- Test `render_out_of_bounds_intent_returns_render_error`: struct literal migrated to embedded shape

**File:** `ferro-json-ui/src/projection/builder.rs`

Production access sites migrated (3):

| Line | Before | After |
|------|--------|-------|
| ~67 | `ctx.intent_index` | `ctx.base.intent_index` |
| ~94 | `ctx.intent_index` | `ctx.base.intent_index` |
| ~485 | `ctx.current_state.clone()` | `ctx.base.current_state.clone()` |

Test struct-literal sites migrated (8): lines 846, 870, 891, 944, 983, 1011, 1093, 1133 — all converted from `{ intent_index: <expr>, mode: ..., ..Default::default() }` to `{ base: BaseContext { intent_index: <expr>, ..Default::default() }, mode: ..., ... }`.

`BaseContext` import scoped to `mod tests` block (not top-level) to avoid unused-import warning on the non-test build path.

Two sites (template_override + statcard_metadata_is_orphan_element) had `..Default::default()` removed after clippy caught `needless_update` — all three VisualContext fields were already specified.

**External sites migrated (3):**

- `ferro-ai/tests/projection_roundtrip.rs:33` — `VisualContext { intent_index: browse_idx, ..VisualContext::default() }` → embedded shape; `use ferro_projections::render::BaseContext` added
- `ferro-mcp/tests/agent_harness.rs:275` — full literal with `intent_index`, `current_state`, `mode`, `templates` migrated to `base: BaseContext { intent_index, current_state, ..Default::default() }, mode, templates`; `use ferro_projections::render::BaseContext` added
- `ferro-mcp/src/tools/render_projection.rs:72` — production `VisualContext` literal migrated to embedded shape; `use ferro_projections::render::BaseContext` added

**Total struct-literal sites migrated: 12** (8 builder tests + 1 mod.rs test + 3 external/production).

**Commit:** `dbbb6730`

### Task 2 — Migrate the four ferro-mcp `{:?}` label sites to `Intent::label()`

**File:** `ferro-mcp/src/tools/render_projection.rs`

- Line ~94 (`all_intents` map): `format!("{:?}", is.intent)` → `is.intent.label().to_string()`
- Line ~102 (`RenderResult.intent`): `format!("{:?}", selected.intent)` → `selected.intent.label().to_string()`
- Test at ~line 488: `intent: "Browse".to_string()` → `intent: "browse".to_string()` (×2: struct field + all_intents entry); `json_str.contains("Browse")` → `json_str.contains("browse")`

**File:** `ferro-mcp/src/tools/generate_projection.rs`

- Line ~89 (`intent_infos` map): `format!("{:?}", score.intent)` → `score.intent.label().to_string()`

**File:** `ferro-mcp/src/tools/projection_coverage.rs`

- Line ~173 (`derive_primary_intent`): `Some(format!("{:?}", primary.intent))` → `Some(primary.intent.label().to_string())`

**Commit:** `e6e7aedf`

### Clippy fix — `needless_update` on fully-specified VisualContext literals

Two test sites in `builder.rs` (template_override + statcard_metadata_is_orphan_element) had `..Default::default()` that clippy flagged as needless (all three VisualContext fields were already explicitly set). Removed.

**Commit:** `30e3478b`

## Verification Results

- `cargo test -p ferro-json-ui --all-features`: **608 tests, all green**
- `cargo test -p ferro-mcp`: **307 tests (302 + 5), all green**
- `cargo build -p ferro-mcp-server`: **clean, zero source edits to ferro-mcp-server**
- `cargo build -p ferro-ai --tests`: **clean**
- `cargo build -p ferro-mcp --tests`: **clean**
- `cargo fmt --all -- --check`: **clean**
- `cargo clippy --all --all-targets -- -D warnings`: **clean** (after needless_update fix)
- `cargo test --all-features`: skipped — disk at 8.8 GB free (ENOSPC-prone environment); per-crate gate covers all changed code paths

**Grep verifications:**
- `grep -n "pub base: BaseContext" ferro-json-ui/src/projection/mod.rs` → line 47: FOUND
- `! grep -n "pub intent_index: usize" ferro-json-ui/src/projection/mod.rs` → PASS (flat field gone)
- `grep -c "ctx.base.intent_index" ferro-json-ui/src/projection/builder.rs` → 4 (≥2 required)
- `grep -n "ctx.base.current_state" ferro-json-ui/src/projection/builder.rs` → FOUND
- `grep -n "impl Default for VisualContext" ferro-json-ui/src/projection/mod.rs` → FOUND (hand-written retained)
- `grep -n "base: BaseContext" ferro-mcp/src/tools/render_projection.rs` → FOUND (line-72 production literal)
- `grep -c ".label().to_string()" ferro-mcp/src/tools/render_projection.rs` → 2
- `grep -n ".label().to_string()" ferro-mcp/src/tools/generate_projection.rs` → FOUND
- `grep -n ".label().to_string()" ferro-mcp/src/tools/projection_coverage.rs` → FOUND
- `! grep -rn 'format!("{:?}", [a-z_]*\.intent)' ferro-mcp/src/tools` → PASS (no label-deriving {:?} remains)
- `grep -n 'intent: "browse"' ferro-mcp/src/tools/render_projection.rs` → lines 488, 493 FOUND
- `! grep -n 'intent: "Browse"' ferro-mcp/src/tools/render_projection.rs` → PASS

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy `needless_update` on two fully-specified VisualContext test literals**
- **Found during:** Wave-close `cargo clippy --all --all-targets -- -D warnings`
- **Issue:** Two test sites (template_override + statcard_metadata_is_orphan_element) specified all three VisualContext fields (`base`, `mode`, `templates`) but retained `..Default::default()` from the original migration pattern
- **Fix:** Removed `..Default::default()` from those two literals; all other sites correctly needed it
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** `30e3478b`

**2. [Rule 2 - Missing functionality] BaseContext import scope**
- **Found during:** First build after embedding
- **Issue:** `BaseContext` was initially added to the top-level import in `builder.rs` but is only used in test code; the non-test build path triggered an `unused_import` warning (would fail `-D warnings`)
- **Fix:** Moved `use ferro_projections::render::BaseContext` from top-level to inside `mod tests`
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** Part of `dbbb6730`

**3. [Rule 2 - Missing functionality] Struct literal in mod.rs test**
- **Found during:** Initial read — the plan listed 8 builder.rs test sites and 3 external sites, but `mod.rs` also contained a `VisualContext { intent_index: ... }` literal in `render_out_of_bounds_intent_returns_render_error`
- **Fix:** Migrated to embedded shape; added `use ferro_projections::render::BaseContext` to mod.rs test module
- **Files modified:** `ferro-json-ui/src/projection/mod.rs`
- **Commit:** Part of `dbbb6730`

## ferro-mcp-server Source Edits

Zero. `ferro-mcp-server` had no source edits — it recompiled cleanly against the updated `ferro-json-ui` and `ferro-mcp` dependencies. Confirmed with `cargo build -p ferro-mcp-server` (clean).

## Embed vs Flat Fallback

The **embed approach** (D-02, preferred) was used throughout. The flat fallback was not needed. The cascade was contained: 12 struct-literal sites total (plan anticipated 11), handled in a single task pass using the compiler's "no field `intent_index` on type `VisualContext`" errors as the migration checklist.

## Known Stubs

None — no stubs introduced. All changes are structural refactors with full compiler and test validation.

## Threat Flags

None — no new trust boundaries introduced. The label value change (PascalCase → lowercase) is the intended behavioral change; T-215-03 accepted per threat model in the plan.

## Self-Check: PASSED

- `ferro-json-ui/src/projection/mod.rs` — FOUND
- `ferro-json-ui/src/projection/builder.rs` — FOUND
- `ferro-mcp/src/tools/render_projection.rs` — FOUND
- `ferro-mcp/src/tools/generate_projection.rs` — FOUND
- `ferro-mcp/src/tools/projection_coverage.rs` — FOUND
- `ferro-ai/tests/projection_roundtrip.rs` — FOUND
- `ferro-mcp/tests/agent_harness.rs` — FOUND
- Commit `dbbb6730` (Task 1) — FOUND
- Commit `e6e7aedf` (Task 2) — FOUND
- Commit `30e3478b` (clippy fix) — FOUND
