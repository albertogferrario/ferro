---
phase: 226-homebrew-tap-distribution-for-ferro-cli
plan: "02"
subsystem: ci/distribution
tags: [homebrew, release-pipeline, tap-ci, github-actions]
requirements: [D-03, D-04, D-05]

dependency_graph:
  requires: [226-01]
  provides: [bump-homebrew-formula-job, tap-ci-workflow]
  affects: [.github/workflows/release.yml]

tech_stack:
  added: []
  patterns: [post-release-sibling-job, push-event-guard, non-prerelease-gate, env-secret-injection]

key_files:
  created:
    - homebrew/tap-ci/tests.yml
    - homebrew/tap-ci/README.md
  modified:
    - .github/workflows/release.yml

decisions:
  - "bump job uses if: push && !contains(ref_name, '-') — belt-and-suspenders non-prerelease gate on top of the existing push-event guard"
  - "no permissions: contents: write on bump job — pushes only to the external tap via PAT, never to this repo"
  - "tap CI --online audit PR-only: avoids URL-not-yet-propagated race on direct bump pushes to the tap"
  - "Homebrew/actions/setup-homebrew@main kept as written — this is the upstream-documented pin for that action"

metrics:
  duration: "99s"
  completed: "2026-06-14"
  tasks: 2
  files_modified: 1
  files_created: 2
---

# Phase 226 Plan 02: Release pipeline wiring + tap CI staging Summary

**One-liner:** `bump-homebrew-formula` CI job wired into `release.yml` (post-release, push+non-prerelease gated, PAT via env) plus tap CI workflow staged for operator placement in `homebrew-ferro`.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add bump-homebrew-formula job to release.yml | 3c2a8d4b | `.github/workflows/release.yml` |
| 2 | Stage tap CI workflow and operator instructions | 95d07b90 | `homebrew/tap-ci/tests.yml`, `homebrew/tap-ci/README.md` |

## What Was Built

### Task 1: bump-homebrew-formula job

Added a new job to `.github/workflows/release.yml` placed between `release` and `update-install-script` (sibling `needs: release` jobs). The job:

- `if: github.event_name == 'push' && !contains(github.ref_name, '-')` — the established `push`-event guard plus a non-prerelease gate rejecting `v1.0.0-alpha`/`-rc`-style tags (D-03).
- `needs: release` — waits for tarballs to be attached to the GitHub Release before computing SHA256s (D-04).
- `runs-on: ubuntu-latest`, `actions/checkout@v4` to get the bump script and formula template from this repo.
- `HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}` injected via `env:` only — never echoed or hardcoded.
- `run: bash scripts/bump-homebrew-formula.sh "${{ github.ref_name }}"` — delegates all SHA256 computation, template rendering, and tap push logic to the Plan 01 script.
- No `permissions: contents: write` — the script pushes to the external tap via the PAT, not to this repo via GITHUB_TOKEN (least-privilege).

### Task 2: Tap CI workflow and operator instructions

Created `homebrew/tap-ci/tests.yml` for the operator to place at `albertogferrario/homebrew-ferro/.github/workflows/tests.yml`. It runs:

- `ruby -c Formula/ferro.rb` — fastest syntax check, no Homebrew binary required.
- `brew audit --strict Formula/ferro.rb` on push to `main`.
- `brew audit --strict --online Formula/ferro.rb` on `pull_request` only — the `--online` check verifies the release tarball URL is reachable; skipped on push to avoid racing a just-pushed bump commit.
- `Homebrew/actions/setup-homebrew@main` + `actions/cache@v4` for the Homebrew environment setup.

Created `homebrew/tap-ci/README.md` with operator instructions for seeding the tap and placing the CI file.

## Verification

```
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"  # exit 0
python3 -c "import yaml; yaml.safe_load(open('homebrew/tap-ci/tests.yml'))"     # exit 0
```

Both files parse as valid YAML. All acceptance criteria met.

## Deviations from Plan

None — plan executed exactly as written. The task YAML for the bump job was taken verbatim from `226-02-PLAN.md` Task 1 action block; the tap CI YAML was taken verbatim from `226-RESEARCH.md` lines 529-568.

## Operator Note

Pushing `.github/workflows/release.yml` to trigger the bump job requires the `workflow` GitHub token scope (`gh auth refresh -s workflow` or SSH/PAT with workflow scope). This is the same constraint noted in Phase 225.

The bump job will silently fail if either:
1. `albertogferrario/homebrew-ferro` repo does not exist, or
2. `HOMEBREW_TAP_TOKEN` secret is not set in the `ferro` repo.

Both are operator-created prerequisites documented in `226-04-PLAN.md`.

## Known Stubs

None. This plan wires existing infrastructure (the Plan 01 script) into the release pipeline; no placeholder data flows to UI.

## Threat Flags

No new threat surface beyond what the plan's threat model covers (T-226-05 through T-226-08). The bump job's token is confined to `env:` scope on a single step; no new network endpoints or trust boundaries introduced in this repo.

## Self-Check: PASSED

- `.github/workflows/release.yml` exists and parses: FOUND
- `homebrew/tap-ci/tests.yml` exists and parses: FOUND
- `homebrew/tap-ci/README.md` exists: FOUND
- Commit 3c2a8d4b exists: FOUND
- Commit 95d07b90 exists: FOUND
