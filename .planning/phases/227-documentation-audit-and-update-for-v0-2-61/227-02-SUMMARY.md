---
phase: 227-documentation-audit-and-update-for-v0-2-61
plan: "02"
subsystem: docs
tags: [docs, factual-accuracy, cli, mcp]
dependency_graph:
  requires: []
  provides:
    - corrected-agent-scaffolding-example
    - version-neutral-docker-init-example
    - valid-docker-init-cross-link
    - correct-ferro-mcp-invocation
    - current-status-string
    - version-neutral-tool-count
  affects:
    - docs/src/getting-started/working-with-agents.md
    - docs/src/cli/frontend-types.md
    - docs/src/upgrading/migration-guide.md
    - docs/src/introduction.md
tech_stack:
  added: []
  patterns:
    - version-neutral phrasing for tool counts and version pins
key_files:
  created: []
  modified:
    - docs/src/getting-started/working-with-agents.md
    - docs/src/cli/frontend-types.md
    - docs/src/upgrading/migration-guide.md
    - docs/src/introduction.md
decisions:
  - "Cross-link for ferro docker:init updated to ../reference/cli.md#ferro-dockerinit (heading confirmed present at line 1128)"
  - "Tool counts in both introduction.md and working-with-agents.md made version-neutral with identical phrasing to avoid future contradictions"
  - "migration-guide.md After-block key renamed from ferro-mcp to ferro to match canonical working-with-agents.md form"
metrics:
  duration: ~2 minutes
  completed_date: "2026-06-14"
  tasks_completed: 4
  files_modified: 4
---

# Phase 227 Plan 02: Doc Factual Corrections (DISC-03 through DISC-07) Summary

One-liner: Fixed four single-file factual discrepancies — phantom CLI command, stale version pin, broken cross-link, wrong MCP binary name, stale milestone string, and contradictory tool counts — across four doc pages.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Replace phantom make:model + stale tool count in working-with-agents.md | 5f5513b8 | docs/src/getting-started/working-with-agents.md |
| 2 | Version-neutral pin + broken cross-link in frontend-types.md | b6503c90 | docs/src/cli/frontend-types.md |
| 3 | Fix stale MCP binary name in migration-guide.md | 4d68fd78 | docs/src/upgrading/migration-guide.md |
| 4 | Update stale milestone string + tool count in introduction.md | facf7383 | docs/src/introduction.md |

## Changes Made

### Task 1 — working-with-agents.md (DISC-04, DISC-07)

- Replaced `exposes 57 introspection tools` with `exposes a full suite of introspection tools` (version-neutral).
- Replaced 3 occurrences of phantom `ferro make:model` with real `ferro make:scaffold` (verified against `ferro-cli/src/commands/make_scaffold.rs`; no `make_model.rs` exists).
- Fixed generated path: `app/models/post.rs` → `src/models/post.rs` (real scaffold writes to `src/models/`).

### Task 2 — frontend-types.md (DISC-03)

- Replaced hard-pinned `0.2.33` with `<pinned>` placeholder in the `--ferro-version` override example; the Dockerfile snippet at line 79 already used `<pinned>` and was left untouched.
- Fixed broken cross-link: `(do-init.md)` (the DigitalOcean page) → `(../reference/cli.md#ferro-dockerinit)` (confirmed heading `### \`ferro docker:init\`` exists at line 1128 of `docs/src/reference/cli.md`).

### Task 3 — migration-guide.md (DISC-05)

- Updated the "After" MCP config block: replaced phantom `ferro-mcp` binary + `serve` arg with the real `ferro mcp` subcommand form (`/absolute/path/to/target/debug/ferro` + `args: ["mcp"]`), matching the canonical `working-with-agents.md` form.
- The "Before" block (`cancer-mcp`) was left unchanged as the illustrative old value.

### Task 4 — introduction.md (DISC-06, DISC-07)

- Dropped stale milestone sentence: `Current milestone work targets v12.0 spec-driven rendering.` (project is well past v12.0; current milestone is v15.0).
- Replaced `80+ tools` with `a full suite of tools` — version-neutral, now consistent with the identical phrasing in `working-with-agents.md`.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — docs-only edits, no executable surface modified.

## Self-Check: PASSED

- `docs/src/getting-started/working-with-agents.md` exists and contains `ferro make:scaffold Post` ✓
- `docs/src/cli/frontend-types.md` exists and contains `ferro-version <pinned>` and `../reference/cli.md#ferro-dockerinit` ✓
- `docs/src/upgrading/migration-guide.md` exists and contains `"args": ["mcp"]` ✓
- `docs/src/introduction.md` exists and contains `pre-1.0` without `v12.0 spec-driven rendering` or `80+ tools` ✓
- All 4 commits present: 5f5513b8, b6503c90, 4d68fd78, facf7383 ✓
