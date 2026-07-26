---
phase: 261-asset-ergonomics
plan: "01"
subsystem: ferro-bundle
tags: [asset, bundle, decouple, mime, leaf-crate]
dependency_graph:
  requires: []
  provides: [ferro-bundle-leaf, mime_from_ext, BundleResponse, serve_path]
  affects: [framework, ferro-macros, publish.yml]
tech_stack:
  added: []
  patterns: [BundleResponse framework-agnostic response type, serve_path public dispatcher]
key_files:
  created: []
  modified:
    - ferro-bundle/src/lib.rs
    - ferro-bundle/Cargo.toml
    - ferro-bundle/tests/serve_cold.rs
    - ferro-bundle/tests/serve_304.rs
    - ferro-bundle/tests/alias_redirect.rs
    - .github/workflows/publish.yml
decisions:
  - "BundleResponse exposes status_code/headers/body_bytes — same accessor names as HttpResponse, minimising test churn"
  - "serve_path is a free function (not Bundle::serve method) since it no longer needs a framework Request"
  - "ferro-bundle moved to Wave 1a; only ferro-cli remains in Wave 3"
metrics:
  duration_seconds: 370
  completed_date: "2026-07-26"
  tasks_completed: 2
  files_modified: 6
requirements: [LIVE-03]
---

# Phase 261 Plan 01: ferro-bundle Decouple + mime_from_ext Summary

Broke the `ferro-bundle → ferro-rs` circular dependency by replacing the `ferro_rs::HttpResponse`-typed serve surface with a framework-agnostic `BundleResponse` struct, exposing the dispatcher as a public `serve_path` free function. Added `mime_from_ext` as the single ext→MIME source of truth the `asset!()` macro will call. Moved `ferro-bundle` from Wave 3 to Wave 1a in `publish.yml`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add mime_from_ext, BundleResponse, serve_path; remove ferro_rs from lib.rs | cca079bc | ferro-bundle/src/lib.rs |
| 2 | Remove ferro-rs dep, migrate tests to serve_path, move to Wave 1a | d0abca4a | ferro-bundle/Cargo.toml, tests/*.rs, publish.yml |

## Key API: serve_path + BundleResponse (Plan 02 reads these)

```rust
// ferro-bundle/src/lib.rs

/// Framework-agnostic result of dispatching a bundle request.
pub struct BundleResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Bytes,
}

impl BundleResponse {
    pub fn status_code(&self) -> u16 { ... }
    pub fn headers(&self) -> &[(String, String)] { ... }
    pub fn body_bytes(&self) -> &Bytes { ... }
}

/// Dispatch a request to the bundle registry by path + optional If-None-Match.
/// Returns a framework-agnostic BundleResponse.
pub fn serve_path(path: &str, if_none_match: Option<&str>) -> BundleResponse { ... }

/// Map a file extension to its MIME type string.
/// Unknown extensions return "application/octet-stream".
pub fn mime_from_ext(ext: &str) -> &'static str { ... }
```

Plan 02 (framework adapter) must:
1. Add `ferro-bundle` to `framework/Cargo.toml` dependencies (now safe — no cycle)
2. Expose `pub mod bundle { pub use ferro_bundle::Bundle; pub use ferro_bundle::mime_from_ext; }` from `framework/src/lib.rs`
3. Add a `Bundle::serve(req: Request) -> HttpResponse` adapter in framework that calls `ferro_bundle::serve_path(req.path(), req.header("if-none-match"))` and converts `BundleResponse` → `HttpResponse`

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo build -p ferro-bundle`: exit 0 (leaf crate, no internal deps)
- `cargo test -p ferro-bundle -- --test-threads=1`: 10 tests pass (7 unit + 3 integration)
- `cargo tree -p ferro-bundle -e normal`: no `ferro-rs`
- `cargo clippy -p ferro-bundle --all-targets -- -D warnings`: exit 0
- `cargo fmt --all -- --check`: exit 0
- `grep ferro-rs ferro-bundle/Cargo.toml`: exits non-zero (dep removed)
- `grep -q 'ferro-assets ferro-bundle' .github/workflows/publish.yml`: exits 0 (Wave 1a)
- `grep 'WAVE3_CRATES="ferro-cli"' .github/workflows/publish.yml`: exits 0 (Wave 3 clean)

## Self-Check: PASSED

All key files found on disk. Both task commits verified in git log.
