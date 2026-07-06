---
phase: 153
plan: 02
subsystem: workspace-registration
tags: [rust, workspace, publish-ci, documentation, version-bump]
dependency_graph:
  requires: [153-01]
  provides: [workspace-version-0.2.31, publish-yml-wave1a-ferro-audit, claude-md-table-ferro-audit, readme-ferro-audit]
  affects: [Cargo.toml, .github/workflows/publish.yml, CLAUDE.md, README.md]
tech_stack:
  added: []
  patterns: [workspace-version-bump, wave1a-publish-registration]
key_files:
  created: []
  modified:
    - Cargo.toml
    - .github/workflows/publish.yml
    - CLAUDE.md
    - README.md
decisions:
  - "Version bumped 0.2.30 → 0.2.31 (RESEARCH F-07 correction; D-38's stale 0.2.25→0.2.26 was ignored)"
  - "ferro-audit appended at end of WAVE1A_CRATES (after ferro-orm); Wave 1a order is irrelevant for the publish loop"
  - "README bullet added at end of What's included list — no ferro-orm bullet pre-existed to anchor position"
metrics:
  duration: "~5 minutes"
  completed: "2026-05-13"
  tasks: 4
  files: 4
requirements_addressed: [D-04, D-38, D-39]
---

# Phase 153 Plan 02: Register ferro-audit in Workspace Surfaces — Summary

Registered `ferro-audit` across all workspace-root surfaces: version bump (`0.2.30 → 0.2.31`), CI publish slot (Wave 1a), developer documentation (`CLAUDE.md` Workspace Structure table, `README.md` What's included list).

## What Was Built

### Files Modified (4)

| File | Change |
|------|--------|
| `Cargo.toml` | `[workspace.package] version` bumped from `"0.2.30"` to `"0.2.31"` |
| `.github/workflows/publish.yml` | `ferro-audit` appended to `WAVE1A_CRATES` string (line 201) |
| `CLAUDE.md` | Row added to Workspace Structure table between `ferro-orm` and `app` |
| `README.md` | Bullet added to "What's included" list at end of list |

### Version Bump Applied

`0.2.30 → 0.2.31` — verified by reading `Cargo.toml` at execution time; the current value was `0.2.30` (matching RESEARCH F-07; D-38's stale `0.2.25 → 0.2.26` was not used).

### Exact Insertions

**Cargo.toml** — `[workspace.members]` already contained `"ferro-audit"` (added by plan 153-01, planned deviation). Only the version field was edited:
```toml
version = "0.2.31"
```

**publish.yml** — `WAVE1A_CRATES` line after edit:
```
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit"
```
`ferro-audit` appended at the end (after `ferro-orm`). `WAVE2_CRATES` unchanged.

**CLAUDE.md** — New row positioned between `ferro-orm` and `app`:
```
| `ferro-audit` | Append-only structured before/after audit log with replay | `src/lib.rs` |
```

**README.md** — New bullet appended at end of "What's included" list:
```
- **Structured audit log** — append-only before/after history with replay (`ferro-audit`)
```

## Verification Results

| Check | Result |
|-------|--------|
| `grep -q '"ferro-audit"' Cargo.toml` | pass |
| `grep -E '^version = "0\.2\.31"' Cargo.toml` | pass |
| `cargo metadata --no-deps` | exit 0 |
| `grep -q 'WAVE1A_CRATES=".*ferro-audit"' publish.yml` | pass |
| `! grep -q 'WAVE2_CRATES=".*ferro-audit' publish.yml` | pass |
| YAML validation (`python3 yaml.safe_load`) | pass |
| `grep -q '| \`ferro-audit\` |' CLAUDE.md` | pass |
| `grep -q 'ferro-audit' README.md` | pass |
| No forbidden phrases in README | pass |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all --all-targets -- -D warnings` | exit 0, no warnings |

## Deviations from Plan

### README.md insertion position

**Issue:** The plan's Task 4 action says to insert the `ferro-audit` bullet "IMMEDIATELY AFTER the `ferro-orm` bullet". No `ferro-orm` bullet existed in README.md — the "What's included" list uses capability descriptions (not crate names), and Phase 152 did not add a `ferro-orm` entry there.

**Fix (Rule 1 — match actual format):** Added the `ferro-audit` bullet at the end of the list (before the blank line that ends the list), following the exact same format as the other bullets. Position is consistent with the list's ordering (newer/more-specific capabilities appear later). The plan's constraint — "new entry in the same list as `ferro-orm`, with parallel phrasing, referencing the crate as `(ferro-audit)`" — is satisfied except for adjacency to a non-existent `ferro-orm` entry.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. Changes are pure documentation and CI configuration. The `publish.yml` edit extends the Wave 1a publish loop to include `ferro-audit`; the first publish is bootstrapped manually in plan 153-06 (the CI token cannot create new crates, only publish updates to existing ones).

## Self-Check: PASSED

- [x] `Cargo.toml` — version is `0.2.31`; `"ferro-audit"` present in `[workspace.members]`
- [x] `.github/workflows/publish.yml` — `WAVE1A_CRATES` ends with `ferro-audit`
- [x] `CLAUDE.md` — row `| \`ferro-audit\` | Append-only structured before/after audit log with replay | \`src/lib.rs\` |` present
- [x] `README.md` — `ferro-audit` present in "What's included" list
- [x] Commit `fe29e670` — version bump
- [x] Commit `cc8f9454` — publish.yml Wave 1a
- [x] Commit `b02c5f85` — CLAUDE.md table row
- [x] Commit `1010c543` — README.md list entry
- [x] `cargo clippy --all --all-targets -- -D warnings` — exit 0
- [x] `cargo fmt --all -- --check` — exit 0
