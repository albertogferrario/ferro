---
phase: 226-homebrew-tap-distribution-for-ferro-cli
plan: "01"
subsystem: distribution
tags: [homebrew, formula, shell-script, release-automation]
dependency_graph:
  requires: []
  provides: [homebrew-formula-template, bump-script]
  affects: [release.yml, homebrew-ferro-tap]
tech_stack:
  added: [Homebrew Formula Ruby DSL]
  patterns: [on_macos/on_linux nested DSL blocks, sed-based template rendering, idempotent git commit]
key_files:
  created:
    - homebrew/Formula/ferro.rb.tpl
    - homebrew/Formula/ferro.rb
    - scripts/bump-homebrew-formula.sh
  modified: []
decisions:
  - "on_macos/on_linux + on_arm/on_intel nested blocks chosen over deprecated case/when Hardware::CPU style"
  - "Script-based bump (not mislav action) because mislav cannot handle multi-sha256 formulae"
  - "64-zero placeholder sha256s in seed formula: structurally valid, non-installable, fails closed on sha mismatch"
  - "VERSION_PLACEHOLDER used for bare version field; vVERSION_PLACEHOLDER used in URLs (with literal v prefix)"
metrics:
  duration_minutes: 2
  completed_date: "2026-06-14T16:56:01Z"
  tasks_completed: 2
  files_created: 3
---

# Phase 226 Plan 01: Homebrew Formula Template and Bump Script Summary

Multi-arch binary Homebrew formula template + seed + in-repo bump script for zero-prerequisite `brew install albertogferrario/ferro/ferro` distribution.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Multi-arch binary formula template + rendered seed | 3993b967 | homebrew/Formula/ferro.rb.tpl, homebrew/Formula/ferro.rb |
| 2 | Bump script: render template, compute SHA256s, push to tap | 12e93654 | scripts/bump-homebrew-formula.sh |

## Verification Results

All plan-level gates passed:

- `ruby -c homebrew/Formula/ferro.rb.tpl` → Syntax OK
- `ruby -c homebrew/Formula/ferro.rb` → Syntax OK
- `bash -n scripts/bump-homebrew-formula.sh` → exits 0
- Dry render (sed substitution at VER=9.9.9 with 4 distinct 64-hex placeholders): `ruby -c` clean, 4 sha256 lines, 4 distinct checksums
- All four target triples present in both template and script: aarch64-apple-darwin, x86_64-apple-darwin, aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu
- No `set -x` in bump script; `HOMEBREW_TAP_TOKEN` never echoed

## Formula Design

The template uses `on_macos`/`on_linux` + `on_arm`/`on_intel` nested blocks (current Homebrew-blessed DSL, not deprecated `case/when Hardware::CPU` style). One `url` + `sha256` per leaf block covering 4 Unix targets. The `livecheck` block enables `brew livecheck` to track new releases via `:github_latest`.

The `test do` block asserts `assert_match version.to_s, shell_output("#{bin}/ferro --version")` (A3 resolved: clap emits bare `ferro 0.2.59`, matched by `version.to_s = "0.2.59"`) and runs an offline `ferro new smoke-app --no-interaction --no-git` smoke (A2 resolved: no network calls, sandbox-safe via `chdir: testpath`).

The seed formula (`ferro.rb`) uses 64-zero placeholder sha256s. These are structurally valid 64-hex strings that pass `ruby -c`; they fail on actual install (sha mismatch) — failing closed is the correct behavior. The first real release overwrites them via the bump script.

## Bump Script Design

Key behaviors:
- `VER="${TAG#v}"` strips the v-prefix from the tag for the `version` field; `$TAG` (full vX.Y.Z) is used in URL paths
- `compute_sha256()` pipes each tarball URL through `shasum -a 256 | awk '{print $1}'` — works on both macOS and Linux CI runners
- `git diff --staged --quiet` guard makes the commit idempotent (no-op if formula already at this version)
- `HOMEBREW_TAP_TOKEN` appears only in the clone URL; never in an `echo` or `print` call; no `set -x`
- Clones to `_tap_clone/` (not `tap-repo/` as in the RESEARCH skeleton — RESEARCH.md lines 459-510 use `_tap_clone` as the canonical name)

## Deviations from Plan

None — plan executed exactly as written. Both files match RESEARCH.md lines 410-457 (template) and 459-510 (script) verbatim.

## Known Stubs

None. This plan produces the two source artifacts (template + bump script); the plan does not wire release.yml (that is Plan 02). The seed formula's placeholder sha256s are documented placeholders, not stubs — the design intent is that the first real release overwrites them.

## Threat Surface Scan

No new network endpoints or auth paths introduced in this repo. The bump script introduces a pattern where `HOMEBREW_TAP_TOKEN` is interpolated into a git clone URL at runtime in CI — this is the intended design (T-226-01 in the plan's threat register). The token is never written to disk or logged. T-226-02 (sha256 integrity) is addressed by computing checksums from the official GitHub Release asset URLs after `needs: release` ensures they are attached.

## Self-Check: PASSED

- `homebrew/Formula/ferro.rb.tpl` exists and passes `ruby -c`
- `homebrew/Formula/ferro.rb` exists and passes `ruby -c`
- `scripts/bump-homebrew-formula.sh` exists and passes `bash -n`
- Commit 3993b967: `git log --oneline` confirms Task 1 commit
- Commit 12e93654: `git log --oneline` confirms Task 2 commit
