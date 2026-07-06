---
phase: 226-homebrew-tap-distribution-for-ferro-cli
plan: "03"
subsystem: docs
tags: [homebrew, distribution, install, docs, readme]
requirements: [D-06]

dependency_graph:
  requires: []
  provides: [brew-install-docs]
  affects: [docs/src/getting-started/installation.md, README.md]

tech_stack:
  added: []
  patterns: [three-method install section, brew-first ordering]

key_files:
  modified:
    - docs/src/getting-started/installation.md
    - README.md

decisions:
  - "Homebrew listed as recommended path (no Rust required); curl and cargo retained alongside it"
  - "curl URL in docs matches scripts/install.sh line 3 verbatim"
  - "README uses inline comment style (# or, with Rust: cargo install) to avoid a second code block"

metrics:
  duration_seconds: 180
  completed_date: "2026-06-14T16:59:01Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 226 Plan 03: Surface Homebrew Install in Docs and README Summary

Adds `brew install albertogferrario/ferro/ferro` as the recommended new-user install path in both
`docs/src/getting-started/installation.md` and `README.md`, alongside (not replacing) the existing
`cargo install ferro-cli` and `curl | sh` instructions (D-06).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Expand installation.md to lead with Homebrew | d0c01b05 | docs/src/getting-started/installation.md |
| 2 | Surface brew install in README Quick Start | dea7e537 | README.md |

## What Was Built

**installation.md:** Replaced the single `cargo install ferro-cli` block under `## Installing the CLI`
with a three-method structure: `### Homebrew (macOS and Linux — recommended)` first, then
`### curl installer (macOS and Linux)`, then `### Cargo (requires Rust)`. The `Or build from source`
git-clone block that follows is unchanged.

**README.md:** Prepended `brew install albertogferrario/ferro/ferro` as the first install line in
the Quick Start code block, with `cargo install ferro-cli` retained as an inline comment alternative.
The `ferro new myapp / cd myapp / ferro serve` flow and the prose paragraph that follows are unchanged.

## Deviations from Plan

None — plan executed exactly as written.

## Verification

```
grep -q 'brew install albertogferrario/ferro/ferro' docs/src/getting-started/installation.md README.md
```
PASS — brew install present in both files.

- cargo and curl paths retained in both files.
- Homebrew section precedes cargo in both files (line order verified).
- curl URL in installation.md matches `scripts/install.sh` line 3 exactly.
- No marketing language; copy is factual and minimal.

## Known Stubs

None. The brew install command is the stable, correct form regardless of when the operator seeds
the tap (Plan 04). No placeholder text introduced.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. Both files are public documentation.
T-226-09 (install command tampering) mitigated: commands point only at `albertogferrario/ferro`
(canonical tap/repo) and the project's own `main` branch raw URL; no third-party or shortener URLs.

## Self-Check: PASSED

- `docs/src/getting-started/installation.md` — exists, contains brew/curl/cargo.
- `README.md` — exists, contains brew before cargo, flow preserved.
- Commit d0c01b05 — exists.
- Commit dea7e537 — exists.
