---
phase: 261-asset-ergonomics
plan: "02"
subsystem: framework
tags: [asset, bundle, re-export, adapter]
dependency_graph:
  requires: [ferro-bundle-leaf]
  provides: [ferro-bundle-reexport, bundle-serve-adapter]
  affects: [framework, ferro-bundle]
tech_stack:
  added: []
  patterns: [free-function framework adapter, pub mod re-export mirroring queue module pattern]
key_files:
  created:
    - framework/src/bundle.rs
    - framework/tests/bundle_serve.rs
  modified:
    - framework/Cargo.toml
    - framework/src/lib.rs
decisions:
  - "Free function ferro::bundle::serve(&Request) -> HttpResponse instead of impl Bundle { serve } — Rust E0116 disallows inherent impl on foreign types; plan explicitly offered (or a free fn)"
  - "pub use ferro_macros::asset deferred to Plan 04 per plan RECOMMENDED path — keeps this plan independently buildable"
metrics:
  duration_seconds: 227
  completed_date: "2026-07-26"
  tasks_completed: 2
  files_modified: 4
requirements: [LIVE-03]
---

# Phase 261 Plan 02: ferro::bundle Re-export + Framework Serve Adapter Summary

Wired the `ferro-bundle` leaf crate into `framework` (`ferro-rs`): added `ferro-bundle` as a dependency (now safe — no cycle), created `framework/src/bundle.rs` re-exporting `Bundle`/`BundleResponse`/`mime_from_ext` from `ferro_bundle` and providing a `pub fn serve(&Request) -> HttpResponse` adapter, and registered `pub mod bundle` in `framework/src/lib.rs`. Four integration tests prove 200/304/301/404 parity.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add ferro-bundle dep + ferro::bundle module + serve adapter | 45c99198 | framework/Cargo.toml, framework/src/bundle.rs, framework/src/lib.rs |
| 2 | Framework serve-adapter parity test | b65611ef | framework/tests/bundle_serve.rs |

## Key API: ferro::bundle (Plan 03/04 reads these)

```rust
// framework/src/bundle.rs

pub use ferro_bundle::{mime_from_ext, Bundle, BundleResponse};

/// Dispatch a request to the bundle registry, returning a framework HttpResponse.
/// Mount on /bundles/{filename} and each registered alias path.
pub fn serve(req: &Request) -> HttpResponse { ... }
```

Usage from a downstream handler:

```rust
use ferro::bundle::{Bundle, serve as bundle_serve};

pub async fn bundle_handler(req: Request) -> Response {
    Ok(bundle_serve(&req))
}
```

`ferro::bundle::Bundle`, `ferro::bundle::mime_from_ext`, and `ferro::bundle::BundleResponse` all resolve. `ferro::asset!` re-export is deferred to Plan 04.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Free function instead of inherent impl for Bundle::serve**
- **Found during:** Task 1 build (E0116)
- **Issue:** Plan proposed `impl Bundle { pub fn serve(...) }` inside `framework/src/bundle.rs`, but Rust's E0116 disallows defining inherent impls for types from other crates (`Bundle` lives in `ferro-bundle`).
- **Fix:** Changed to a public free function `pub fn serve(req: &Request) -> HttpResponse` in the `ferro::bundle` module. Plan explicitly offered "(or a free fn)" as the resolution path.
- **Files modified:** framework/src/bundle.rs
- **Commit:** 45c99198

## Verification

- `cargo build -p ferro-rs`: exit 0 (no cycle, ferro-bundle is a leaf)
- `cargo test -p ferro-rs --test bundle_serve -- --test-threads=1`: 4/4 pass (200/304/301/404)
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy -p ferro-rs --all-targets -- -D warnings`: exit 0
- `grep 'pub use ferro_macros::asset' framework/src/lib.rs`: exits non-zero (correctly deferred)

## Self-Check: PASSED

All key files found on disk. Both task commits verified in git log.
