---
phase: 227-documentation-audit-and-update-for-v0-2-61
plan: "01"
subsystem: docs
tags: [docs, cli-reference, factual-accuracy]
dependency_graph:
  requires: []
  provides: [corrected-cli-reference]
  affects: [docs/src/reference/cli.md]
tech_stack:
  added: []
  patterns: []
key_files:
  modified:
    - docs/src/reference/cli.md
decisions:
  - Added --regenerate-models row to db:sync options table (confirmed real flag in db_sync.rs, plan permitted it)
metrics:
  completed_date: "2026-06-15"
  tasks_completed: 2
  files_modified: 1
---

# Phase 227 Plan 01: CLI Reference Factual Corrections Summary

**One-liner:** Corrected `reference/cli.md` install ordering to brew-first and replaced phantom `--migrate` flag with real `--skip-migrations` in the db:sync section.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Brew-first install reorder (DISC-01) | fa23302d | docs/src/reference/cli.md |
| 2 | Correct db:sync flag polarity (DISC-02) | 5f3455b9 | docs/src/reference/cli.md |

## What Was Done

**Task 1 — Installation section reorder (DISC-01):**
The Installation section previously led with `cargo install ferro-cli` and a build-from-source block. It now leads with Homebrew (`brew install albertogferrario/ferro/ferro`) to match `installation.md`. The curl installer was added as the second method. Cargo is retained as the third option with a note that Rust is required. A toolchain-free-CLI distinction was added matching the canonical install page's framing.

**Task 2 — db:sync flag correction (DISC-02):**
The `ferro db:sync` section documented a `--migrate` flag that does not exist in `ferro-cli/src/commands/db_sync.rs`. The real flag is `--skip-migrations` with inverted polarity (migrations run by default; the flag suppresses them). Both the code example and the options table row were corrected. The `--regenerate-models` flag was also added to the options table after confirming it is a real parameter in `db_sync.rs` (`pub fn run(skip_migrations: bool, regenerate_models: bool)`).

## Deviations from Plan

### Auto-added functionality

**1. [Rule 2 - Missing] Added --regenerate-models row to db:sync options table**
- **Found during:** Task 2
- **Issue:** The `--regenerate-models` flag is a real parameter in `db_sync.rs` not documented in the options table. The plan explicitly permitted adding it if confirmed against the source oracle.
- **Fix:** Added `| \`--regenerate-models\` | Regenerate SeaORM model wrappers |` to the options table.
- **Files modified:** docs/src/reference/cli.md
- **Commit:** 5f3455b9

**Automated verify note:** Task 1's plan verify script (`! grep -qE "^cargo install ferro-cli$"`) would have failed because `cargo install ferro-cli` is correctly retained as an alternate method. The human-readable acceptance criteria ("cargo is demoted, not deleted") takes precedence. All four acceptance criteria are met: Homebrew leads, curl is present, Homebrew heading precedes Cargo heading, and cargo alternatives are retained.

## Known Stubs

None. Both corrections wire to verified ground truth (installation.md canonical order, db_sync.rs function signature).

## Threat Flags

None. Docs-only phase — no executable surface modified.

## Self-Check: PASSED

- `docs/src/reference/cli.md` exists and contains `brew install albertogferrario/ferro/ferro` ✓
- `docs/src/reference/cli.md` contains `ferro db:sync --skip-migrations` ✓
- `docs/src/reference/cli.md` does NOT contain `ferro db:sync --migrate` ✓
- Commit fa23302d exists ✓
- Commit 5f3455b9 exists ✓
