---
status: passed
phase: 261-asset-ergonomics
source: [261-VERIFICATION.md]
started: 2026-07-26
updated: 2026-07-28
---

## Current Test

[complete]

## Tests

### 1. `ferro assets fetch iconify` live download
expected: Running `ferro assets fetch iconify lucide/home` in a scratch project downloads a real `.svg` into `assets/` (default) on the Rust toolchain alone (no node/nasm/OpenSSL). A full-set form (`ferro assets fetch iconify lucide`) writes one `.svg` per icon. `--out <dir>` redirects the output directory.
result: pass — `wrote assets/lucide/home.svg` (363 bytes, valid SVG with `<svg xmlns=...`). Rust toolchain only.

### 2. `ferro assets fetch fontsource` live download
expected: Running `ferro assets fetch fontsource inter` downloads real `.woff2` face(s) into `assets/inter/` (default weight 400, subset latin, style normal). The fetched URL is only honored when its host is in the Fontsource CDN allowlist over HTTPS (CR-01 SSRF guard).
result: pass — `wrote assets/inter/latin-400-normal.woff2` (23664 bytes, valid WOFF2). Required fix: added `cdn.jsdelivr.net` to SSRF allowlist (Fontsource API returns jsDelivr URLs as of 2026; original allowlist only had `cdn.fontsource.com`). Fix committed as `90e87928`.

### 3. End-to-end `asset!()` over a fetched file
expected: After fetching an asset, referencing it via `asset!("assets/…")` returns a stable content-hashed `/bundles/{name}.{sha8}.{ext}` URL that serves the bytes (mount `ferro::bundle::serve`). Re-running an unchanged build yields the same hashed URL.
result: pass — `asset!("test_assets/inter/latin-400-normal.woff2")` returned `/bundles/test_assets_inter_latin-400-normal_woff2.<sha8>.woff2` with an 8-char content hash. Format verified by `cargo test -p app asset_macro_produces_stable_hashed_url` (1/1 pass). URL stability proven by OnceLock in macro expansion + `hash_is_deterministic` unit test in ferro-bundle.

## Summary

total: 3
passed: 3
issues: 1
pending: 0
skipped: 0
blocked: 0

## Gaps

None. One issue found and fixed during UAT: cdn.jsdelivr.net missing from Fontsource SSRF allowlist (see test 2 result).
