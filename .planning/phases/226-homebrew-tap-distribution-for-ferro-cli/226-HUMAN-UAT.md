---
status: partial
phase: 226-homebrew-tap-distribution-for-ferro-cli
source: [226-04-SUMMARY.md]
started: 2026-06-14T00:00:00Z
updated: 2026-06-14T00:00:00Z
---

## Current Test

[awaiting ferro's first published release, then a live install]

## Tests

### 1. Tap auto-bumps on the first ferro release
expected: After ferro publishes its first release (push + tag `vX.Y.Z` → `release.yml` builds the
4 tarballs), the tap's `update-formula` workflow (6-hourly, or run manually via "Run workflow" in
`albertogferrario/homebrew-ferro`) commits a `Formula/ferro.rb` whose 4 `sha256` lines are real
(not the 64-zero placeholders) and whose `version` matches the release. Tap `tests.yml` (audit) green.
result: [pending — no ferro release exists yet]

### 2. End-to-end `brew install` on a clean machine
expected: On a Mac with no Rust installed: `brew install albertogferrario/ferro/ferro && ferro --version`
prints `ferro X.Y.Z` and `ferro new myapp --no-interaction --no-git` creates a scaffold. (Optionally
also on Linux via Homebrew-on-Linux.)
result: [pending — depends on test 1]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
