---
phase: 129-publish-workflow-refinement
plan: "03"
subsystem: docs
tags: [publishing, documentation, version-model, publish-gating, PUBLISHING.md]

dependency_graph:
  requires:
    - phase: 129-01
      provides: Library-change gate in publish.yml with should_publish=none
    - phase: 129-02
      provides: ferro_versions schema reservation in FerroDeployMetadata
  provides:
    - PUBLISHING.md::Version Model section (lockstep model + ferro_versions reservation)
    - PUBLISHING.md::Publish Gating section (excluded paths + scenario table)
  affects:
    - Any contributor reading PUBLISHING.md for release semantics

tech_stack:
  added: []
  patterns:
    - Neutral architectural voice for repository docs (CLAUDE.md repo-document rules)
    - Exclusion list in doc mirrors workflow case statement verbatim (single source of truth)

key_files:
  created: []
  modified:
    - PUBLISHING.md

key_decisions:
  - "## Publish Gating inserted before ## Publishing Order (not at end) — groups operational context near the publishing steps it governs"
  - "## Version Coordination replaced entirely by ## Version Model — avoids duplicating content; old section was a four-line checklist now absorbed into the release checklist inside Version Model"
  - "Exclusion list written as bullet list (prose) to match PUBLISHING.md doc style; scenario table added for scan-readability"

metrics:
  duration: "~2 min"
  completed: "2026-04-09"
  tasks: 2
  files: 1
---

# Phase 129 Plan 03: PUBLISHING.md Version Model and Publish Gating Documentation Summary

Document the lockstep version model and the library-change publish gate in PUBLISHING.md — both sections written in neutral architectural voice with the exclusion list matching the workflow case statement verbatim.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Version Model and Publish Gating sections to PUBLISHING.md | 6659c03d | PUBLISHING.md |
| 2 | Cross-check exclusion list against workflow | — (verification only, no changes) | — |

## What Was Built

Added two sections to `PUBLISHING.md`:

**`## Publish Gating`** (inserted before `## Publishing Order`):
- Documents the library-change gate from Plan 01
- Lists all excluded paths verbatim, matching the `case` statement in `.github/workflows/publish.yml`
- Explains first-run edge case (no `v*` tag → publish proceeds)
- Scenario table covering `should_publish=none` and `should_publish=yes` outcomes

**`## Version Model`** (replaced `## Version Coordination`):
- States the lockstep release model: every library crate at the same version
- Documents the single `ferro_version` field in `[package.metadata.ferro.deploy]`
- Describes `docker:init` rewrite behavior
- Includes condensed release checklist (bump → commit → push → workflow handles rest)
- Sub-section `### Per-crate override reservation` documents the `ferro_versions` schema hook with TOML example and explanation that it is parsed/round-tripped but not yet consulted by rewrite logic

## Verification

All acceptance criteria met:

- `grep -q '^## Version Model$' PUBLISHING.md` — PASS
- `grep -q '^## Publish Gating$' PUBLISHING.md` — PASS
- `grep -q 'ferro_versions' PUBLISHING.md` — PASS
- `grep -q '^## Version Coordination$' PUBLISHING.md` returns NO match — PASS (old section replaced)
- `grep -q 'should_publish=none' PUBLISHING.md` — PASS
- `grep -q 'should_publish=yes' PUBLISHING.md` — PASS
- All 16 exclusion tokens present in both files (`ferro-cli/`, `app/`, `docs/`, `.github/`, `.planning/`, `scripts/`, `LICENSE`, `Cargo.lock`, `.gitignore`, `.editorconfig`, `rustfmt.toml`, `bacon.toml`, `deny.toml`, `dev.sh`, `llms.txt`, `rust-toolchain.toml`) — PASS
- No CLAUDE.md trigger phrases (`killer feature`, `load-bearing`, `forcing function`, `the bet`, `no stop-loss`) — PASS
- `cargo fmt --all -- --check` — exit 0
- `cargo clippy --all --all-targets -- -D warnings` — exit 0
- `cargo test --all-features` — all tests PASS

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. PUBLISHING.md sections are complete and accurate. The `ferro_versions` reservation is documented as intentionally incomplete (schema-only, not wired); the doc reflects this correctly.

## Self-Check: PASSED
