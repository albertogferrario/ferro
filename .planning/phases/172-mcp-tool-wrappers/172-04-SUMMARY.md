---
phase: 172-mcp-tool-wrappers
plan: "04"
subsystem: ferro-cli
tags: [cli-rewiring, thin-wrapper, relevance-deletion, version-bump, docs-parity, full-gate]
dependency_graph:
  requires: [ferro-mcp/src/tools/ai_scaffold.rs, ferro-mcp/src/tools/ai_explain_core.rs, ferro-mcp/src/tools/relevance.rs]
  provides: [thin-wrapper ai_make::run, thin-wrapper ai_explain::run, version 0.2.47, docs/src/features/ai.md MCP section]
  affects: [ferro-cli/src/commands/ai_make.rs, ferro-cli/src/commands/ai_explain.rs, ferro-cli/src/lib.rs, ferro-cli/src/commands/mod.rs, Cargo.toml, docs/src/features/ai.md]
tech_stack:
  added: []
  patterns: [cli-thin-wrapper, tokio-rt-bridge, single-definition-site]
key_files:
  created: []
  modified:
    - ferro-cli/src/commands/ai_make.rs
    - ferro-cli/src/commands/ai_explain.rs
    - ferro-cli/src/lib.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-mcp/src/tools/ai_scaffold.rs
    - Cargo.toml
    - docs/src/features/ai.md
  deleted:
    - ferro-cli/src/relevance.rs
decisions:
  - "ENV_LOCK deleted from ferro-cli commands/mod.rs — all env-var-mutating tests for ai_make/ai_explain relocated to ferro-mcp; no remaining callers in ferro-cli"
  - "scaffold_core::scaffold_core_returns_err_without_ai_config: ENV_LOCK guard scoped to a sync block before the .await to satisfy clippy::await_holding_lock"
  - "ai_explain CLI wrapper calls relocate public resolve_target/build_*_prompt (prose path) not explain_core — prose-only contract preserved per CONTEXT.md Deferred Ideas boundary"
  - "ai_make module doc updated to reflect thin-wrapper reality; complete_with::<ServiceDef> removed from doc comment to satisfy acceptance criterion"
metrics:
  duration: 1100s
  completed: "2026-06-08"
  tasks_completed: 3
  files_changed: 8
---

# Phase 172 Plan 04: CLI Thin Wrappers, Version Bump, Docs Summary

**One-liner:** Rewired ferro-cli AI commands as thin wrappers over the relocated ferro-mcp cores, deleted the duplicated relevance module, bumped workspace to 0.2.47, and documented ai_scaffold/ai_explain in the MCP tools section — SC#3 structural guarantee now enforced by compilation.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewire ai:make + ai:explain CLI wrappers; delete ferro-cli/src/relevance.rs | d978ce64 | ai_make.rs, ai_explain.rs, lib.rs, commands/mod.rs, relevance.rs (deleted) |
| 2 | Bump workspace version 0.2.46→0.2.47; add ai_scaffold + ai_explain docs | fb630cc1 | Cargo.toml, docs/src/features/ai.md |
| 3 | Full release gate (fmt + clippy -D warnings + test --all-features) | 60072ac9 | ai_explain.rs, ai_make.rs, ai_scaffold.rs (gate fixes) |

## Verification

- `test ! -f ferro-cli/src/relevance.rs`: PASS
- `! grep -q "mod relevance" ferro-cli/src/lib.rs`: PASS
- `grep -q "ai_scaffold::scaffold_core" ferro-cli/src/commands/ai_make.rs`: PASS
- `! grep -q "complete_with::<ServiceDef>" ferro-cli/src/commands/ai_make.rs`: PASS
- `grep -q "render_output" ferro-cli/src/commands/ai_make.rs`: PASS
- `grep -q "emit_service_def_source" ferro-cli/src/commands/ai_make.rs`: PASS
- `grep -q "ai_explain_core::" ferro-cli/src/commands/ai_explain.rs`: PASS
- `! grep -q "fn resolve_target" ferro-cli/src/commands/ai_explain.rs`: PASS
- `! grep -q "fn build_route_prompt" ferro-cli/src/commands/ai_explain.rs`: PASS
- `grep -q "schema: None" ferro-cli/src/commands/ai_explain.rs`: PASS
- `grep -q 'version = "0.2.47"' Cargo.toml`: PASS
- `grep -q "### \`ai_scaffold\`" docs/src/features/ai.md`: PASS
- `grep -q "### \`ai_explain\`" docs/src/features/ai.md`: PASS
- `grep -q "Does NOT write" docs/src/features/ai.md`: PASS
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --all --all-targets -- -D warnings`: PASS
- `cargo test --all-features`: PASS (0 failures across all crates)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Stale doc comment in ai_make.rs module header**
- **Found during:** Task 1 acceptance criterion check (`complete_with::<ServiceDef>` grep)
- **Issue:** The module-level doc comment still referenced `complete_with::<ServiceDef>()` from the old implementation; the grep caught it as a false positive for "pipeline still in CLI".
- **Fix:** Updated module doc to describe the thin-wrapper role accurately.
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Commit:** d978ce64

**2. [Rule 1 - Bug] Deleted ENV_LOCK from ferro-cli commands/mod.rs (dead code)**
- **Found during:** Task 1 — cargo test warned `static ENV_LOCK is never used`
- **Issue:** ENV_LOCK had no remaining callers after the env-var-mutating tests for ai_make/ai_explain were removed (those tests relocated to ferro-mcp in Plan 02). Dead code would become a clippy `-D warnings` failure.
- **Fix:** Deleted the static and its doc comment from commands/mod.rs.
- **Files modified:** ferro-cli/src/commands/mod.rs
- **Commit:** d978ce64

**3. [Rule 1 - Bug] rustfmt line-length violation in ai_explain.rs import**
- **Found during:** Task 3 fmt check
- **Issue:** The `use ferro_mcp::tools::ai_explain_core::{...}` import line exceeded the 100-char limit because all six items were on one continuation line.
- **Fix:** Split `resolve_max_tokens_with_default` onto a separate continuation line.
- **Files modified:** ferro-cli/src/commands/ai_explain.rs
- **Commit:** 60072ac9

**4. [Rule 1 - Bug] Trailing blank line before closing brace in ai_make.rs test module**
- **Found during:** Task 3 fmt check
- **Issue:** rustfmt reported a diff — blank line between the last test's closing `}` and the module-closing `}`.
- **Fix:** Removed the blank line.
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Commit:** 60072ac9

**5. [Rule 1 - Bug] clippy::await_holding_lock in ferro-mcp ai_scaffold.rs test**
- **Found during:** Task 3 clippy run
- **Issue:** `scaffold_core_returns_err_without_ai_config` held `ENV_LOCK` across the `.await` on `scaffold_core(...)`. Clippy `-D warnings` rejects this as a potential deadlock.
- **Fix:** Scoped the lock acquisition and env-var clearing into a sync block that ends before the `.await` call. The env vars are cleared before the lock is released, so the test isolation guarantee is preserved.
- **Files modified:** ferro-mcp/src/tools/ai_scaffold.rs
- **Commit:** 60072ac9

## Known Stubs

None. All CLI wrappers are fully wired:
- `ai_make::run()` calls `scaffold_core` then `render_output` — both implemented.
- `ai_explain::run()` calls `resolve_target` + `build_*_prompt` + `client.complete()` — all implemented.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. All plan threat mitigations verified:

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-172-PI | STRUCTURAL — CLI now routes through `scaffold_core`, which applies `sanitize_description`. Deleting the CLI's own copy makes it impossible for a drifted second implementation to exist. |
| T-172-DUP | ELIMINATED — `ferro-cli/src/relevance.rs` deleted; CLI imports `ferro_mcp::tools::relevance::*`. One definition site by construction. |
| T-172-REGRESS | PRESERVED — `ai_explain` CLI wrapper calls `resolve_target`/`build_*_prompt`/prose path; `schema: None` confirmed; existing CLI tests pass unchanged. |

## Self-Check: PASSED

- `ferro-cli/src/relevance.rs` does not exist ✓
- `ferro-cli/src/lib.rs` has no `mod relevance` ✓
- `ferro-cli/src/commands/ai_make.rs` calls `scaffold_core` ✓
- `ferro-cli/src/commands/ai_explain.rs` imports from `ai_explain_core` ✓
- `Cargo.toml` version = "0.2.47" ✓
- `docs/src/features/ai.md` contains `ai_scaffold` and `ai_explain` sections ✓
- Commits d978ce64, fb630cc1, 60072ac9 exist ✓
- Full gate (fmt + clippy + test) green ✓
