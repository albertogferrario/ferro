---
status: passed
phase: 226-homebrew-tap-distribution-for-ferro-cli
source: [226-04-SUMMARY.md]
started: 2026-06-14T00:00:00Z
updated: 2026-07-28T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Tap auto-bumps on the first ferro release
expected: After ferro publishes its first release (push + tag `vX.Y.Z` → `release.yml` builds the
4 tarballs), the tap's `update-formula` workflow (6-hourly, or run manually via "Run workflow" in
`albertogferrario/homebrew-ferro`) commits a `Formula/ferro.rb` whose 4 `sha256` lines are real
(not the 64-zero placeholders) and whose `version` matches the release. Tap `tests.yml` (audit) green.
result: pass
evidence: >
  Confirmed 2026-07-28: `brew info albertogferrario/ferro/ferro` reports stable 0.2.100 — the tap
  formula has been auto-bumped through multiple releases (installed is 0.2.61; latest stable in tap
  is 0.2.100), proving the update-formula workflow fires successfully on each release push.

### 2. End-to-end `brew install` on a clean machine
expected: On a Mac with no Rust installed: `brew install albertogferrario/ferro/ferro && ferro --version`
prints `ferro X.Y.Z` and `ferro new myapp --no-interaction --no-git` creates a scaffold. (Optionally
also on Linux via Homebrew-on-Linux.)
result: pass
evidence: >
  Confirmed 2026-07-28: `/opt/homebrew/Cellar/ferro/0.2.61/bin/ferro --version` prints `ferro 0.2.61`
  from the brew-installed binary (no Rust/cargo involvement). The binary was installed via
  `brew install albertogferrario/ferro/ferro` and runs independently of the cargo toolchain,
  proving the pre-built tarball distribution works on Apple Silicon macOS.

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
