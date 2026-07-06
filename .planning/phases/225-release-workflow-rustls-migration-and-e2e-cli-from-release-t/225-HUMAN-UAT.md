---
status: partial
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
source: [225-VERIFICATION.md]
started: 2026-06-14T00:00:00Z
updated: 2026-06-14T00:00:00Z
---

## Current Test

[awaiting human testing — both items require environments not available in the authoring session]

## Tests

### 1. Clean-container `cargo install ferro-cli` with no OpenSSL dev packages
expected: Inside a fresh `debian:bookworm-slim` with NO `libssl-dev`/`pkg-config` installed, `cargo install ferro-cli` exits 0 with no `-lssl`/openssl linker errors. Fastest path: remove the `libssl-dev pkg-config` apt line from `ferro-cli/tests/fixtures/benchmark/Dockerfile` and `docker build`. This proves the rustls migration removed the system-OpenSSL build dependency (the COMP-04 cold-Debian friction).
result: [pending]

### 2. aarch64-unknown-linux-gnu release artifact runs on real arm64 hardware
expected: After a tag-push release run, download `ferro-aarch64-unknown-linux-gnu.tar.gz` and run `./ferro --version` on an arm64 host — prints version, exits 0. Proves the native (no-`cross`) aarch64 build with the gcc cross-linker + ring CC env produces a working binary. CI builds it but does not execute the foreign-arch artifact.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
