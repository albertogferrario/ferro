---
phase: 150
plan: 02
subsystem: ferro-json-ui
tags: [assets, sri, quill, cdn, security]
dependency_graph:
  requires: []
  provides: [Quill 2.0.3 CDN asset constants with SHA-384 SRI hashes]
  affects:
    - ferro-json-ui/src/assets/mod.rs
    - ferro-json-ui/src/assets/quill.rs
tech_stack:
  added: []
  patterns: [SRI subresource integrity, pub(crate) const CDN pinning]
key_files:
  created:
    - ferro-json-ui/src/assets/quill.rs
  modified:
    - ferro-json-ui/src/assets/mod.rs
decisions:
  - assets.rs promoted to assets/mod.rs directory module to accommodate quill submodule
  - SHA-384 chosen over SHA-256 per Mozilla MDN SRI 2025 recommendation
  - dead_code allowed on constants pending Plan 03 render.rs wiring
metrics:
  duration: ~12min
  completed: "2026-05-01"
  tasks: 2
  files: 2
---

# Phase 150 Plan 02: Quill 2.0.3 SRI Asset Constants Summary

Pinned Quill 2.0.3 CDN URLs and SHA-384 SRI hashes as four `pub(crate) const` strings in a new `ferro-json-ui/src/assets/quill.rs` submodule, computed from live jsDelivr bytes via `openssl dgst -sha384 -binary | openssl base64 -A`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Compute SHA-384 SRI hashes for Quill 2.0.3 JS and CSS | (no files) | — |
| 2 | Create assets/quill.rs and wire into assets/mod.rs | 1cfc53e4 | ferro-json-ui/src/assets/quill.rs, ferro-json-ui/src/assets/mod.rs |

## Computed SRI Hash Values

| Asset | URL | SRI Hash |
|-------|-----|----------|
| quill.js | https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js | `sha384-utBUCeG4SYaCm4m7GQZYr8Hy8Fpy3V4KGjBZaf4WTKOcwhCYpt/0PfeEe3HNlwx8` |
| quill.snow.css | https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css | `sha384-ecIckRi4QlKYya/FQUbBUjS4qp65jF/J87Guw5uzTbO1C1Jfa/6kYmd6dXUF6D7i` |

Computation timestamp: 2026-05-01T00:59 UTC
OpenSSL version: OpenSSL 3.6.2 7 Apr 2026

Both pipelines were run twice; outputs were byte-identical (jsDelivr immutability confirmed).

## Module Restructuring

The assets module was previously a single file `ferro-json-ui/src/assets.rs`. To accommodate the new `quill` submodule, it was promoted to a directory:

- `ferro-json-ui/src/assets.rs` → `ferro-json-ui/src/assets/mod.rs` (content preserved)
- `ferro-json-ui/src/assets/quill.rs` added alongside

The `include_str!` path was updated from `"../assets/ferro-base.css"` to `"../../assets/ferro-base.css"` to reflect the new depth. `pub(crate) mod quill;` declared in `mod.rs`.

## Deviations from Plan

**1. [Rule 2 - Dead Code] Added `#[allow(dead_code)]` to four pub(crate) constants**

- **Found during:** Task 2 (cargo build)
- **Issue:** All four `pub(crate) const` items emit dead_code warnings because render.rs does not yet use them (Plan 03 wires the asset injection). The crate's clippy config enforces `-D warnings`.
- **Fix:** Added `#[allow(dead_code)]` to each constant. The attributes will be removed when Plan 03 adds the `use crate::assets::quill::...` import.
- **Files modified:** ferro-json-ui/src/assets/quill.rs
- **Commit:** 1cfc53e4 (included in same commit)

## Known Stubs

None — all four constants contain computed, non-placeholder values.

## Threat Flags

None new beyond the threat model already captured in the plan:

| Flag | File | Description |
|------|------|-------------|
| T-150-W2-01 mitigated | ferro-json-ui/src/assets/quill.rs | SHA-384 SRI hashes pinned from live CDN bytes; browsers enforce integrity on load |

## Self-Check: PASSED

- `ferro-json-ui/src/assets/quill.rs` exists: FOUND
- `ferro-json-ui/src/assets/mod.rs` exists: FOUND
- `grep -q 'pub(crate) const QUILL_JS_URL'`: FOUND
- `grep -q 'pub(crate) const QUILL_CSS_URL'`: FOUND
- `grep -q 'pub(crate) const QUILL_JS_SRI'`: FOUND
- `grep -q 'pub(crate) const QUILL_CSS_SRI'`: FOUND
- `grep -q 'pub(crate) mod quill'` in mod.rs: FOUND
- Commit 1cfc53e4 confirmed in git log
- `cargo build -p ferro-json-ui` exits 0
- `cargo fmt --all -- --check` exits 0
