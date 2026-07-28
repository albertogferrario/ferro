---
status: partial
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
source: [225-VERIFICATION.md]
started: 2026-06-14T00:00:00Z
updated: 2026-07-28T00:00:00Z
---

## Current Test

[item 1 passed 2026-07-28; item 2 blocked on arm64 hardware]

## Tests

### 1. Clean-container `cargo install ferro-cli` with no OpenSSL dev packages
expected: Inside a fresh `debian:bookworm-slim` with NO `libssl-dev`/`pkg-config` installed, `cargo install ferro-cli` exits 0 with no `-lssl`/openssl linker errors. Fastest path: remove the `libssl-dev pkg-config` apt line from `ferro-cli/tests/fixtures/benchmark/Dockerfile` and `docker build`. This proves the rustls migration removed the system-OpenSSL build dependency (the COMP-04 cold-Debian friction).
result: PASS — 2026-07-28. `cargo install ferro-cli --version 0.2.90 --locked` on `debian:bookworm-slim` with only `curl ca-certificates build-essential git` (no `libssl-dev`, no `pkg-config`) exits 0, `ferro --version` prints `ferro 0.2.90`. Benchmark Dockerfile updated to remove the now-unnecessary packages.

### 2. aarch64-unknown-linux-gnu release artifact runs on real arm64 hardware
expected: After a tag-push release run, download `ferro-aarch64-unknown-linux-gnu.tar.gz` and run `./ferro --version` on an arm64 host — prints version, exits 0. Proves the native (no-`cross`) aarch64 build with the gcc cross-linker + ring CC env produces a working binary. CI builds it but does not execute the foreign-arch artifact.
result: [pending — blocked on arm64 hardware]

## Summary

total: 2
passed: 1
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
