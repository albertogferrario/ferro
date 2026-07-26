---
status: partial
phase: 261-asset-ergonomics
source: [261-VERIFICATION.md]
started: 2026-07-26
updated: 2026-07-26
---

## Current Test

[awaiting human testing]

## Tests

### 1. `ferro assets fetch iconify` live download
expected: Running `ferro assets fetch iconify lucide/home` in a scratch project downloads a real `.svg` into `assets/` (default) on the Rust toolchain alone (no node/nasm/OpenSSL). A full-set form (`ferro assets fetch iconify lucide`) writes one `.svg` per icon. `--out <dir>` redirects the output directory.
result: [pending]

### 2. `ferro assets fetch fontsource` live download
expected: Running `ferro assets fetch fontsource inter` downloads real `.woff2` face(s) into `assets/inter/` (default weight 400, subset latin, style normal). The fetched URL is only honored when its host is in the Fontsource CDN allowlist over HTTPS (CR-01 SSRF guard).
result: [pending]

### 3. End-to-end `asset!()` over a fetched file
expected: After fetching an asset, referencing it via `asset!("assets/…")` returns a stable content-hashed `/bundles/{name}.{sha8}.{ext}` URL that serves the bytes (mount `ferro::bundle::serve`). Re-running an unchanged build yields the same hashed URL.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
