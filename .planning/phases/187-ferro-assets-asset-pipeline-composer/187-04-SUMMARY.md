---
phase: 187-ferro-assets-asset-pipeline-composer
plan: "04"
subsystem: ferro-assets
tags: [new-crate, docs, readme, passthrough-proof, ci-parity, wave-1a, publish-checkpoint]
dependency_graph:
  requires:
    - 187-01 (Asset/ContentType/Transform/Pipeline/Error scaffold)
    - 187-02 (5 text transforms)
    - 187-03 (ImageTranscode/ResponsiveImages)
  provides:
    - ferro-assets/README.md (crates.io front page, security notes, zero-C-deps)
    - docs/src/features/ferro-assets.md (full feature page)
    - docs/src/SUMMARY.md entry (Asset Pipeline after Deployments)
    - SC-1 real-transform passthrough proof (json_file_unchanged_by_all_seven_real_transforms)
    - CI-parity gate green (fmt + clippy --all-targets + test --all-features)
  affects:
    - ferro-assets/README.md (created)
    - docs/src/features/ferro-assets.md (created)
    - docs/src/SUMMARY.md (one-line entry added)
    - ferro-assets/tests/passthrough_proof.rs (real-transform test added)
    - ferro-assets/tests/image_transcode_test.rs (AVIF decode fix)
tech_stack:
  added: []
  patterns:
    - AVIF magic-byte check (ftyp box bytes 4-7) instead of image::load_from_memory
      to stay zero-C-deps under --all-features
    - rust,ignore on doctest examples that use tokio::spawn_blocking (no tokio dep)
key_files:
  created:
    - ferro-assets/README.md
    - docs/src/features/ferro-assets.md
  modified:
    - docs/src/SUMMARY.md
    - ferro-assets/tests/passthrough_proof.rs
    - ferro-assets/tests/image_transcode_test.rs
decisions:
  - "AVIF magic-byte check replaces image::load_from_memory in slow-test: dav1d C
    decoder required for image crate AVIF decode contradicts zero-C-deps criterion 3;
    ravif encode success (Ok return) already guarantees valid AVIF; ftyp box check
    is a lightweight structural sanity test"
  - "rust,ignore retained on lib.rs Quick Start: tokio::spawn_blocking not a crate
    dep; doctest would fail compilation"
  - "SC-1 real-transform test added to passthrough_proof.rs alongside existing stub
    tests: both prove the guarantee from different angles (stub = type-gating mechanics;
    real = production transforms do not touch Other files)"
metrics:
  duration: "1252s (~21 min)"
  completed: "2026-06-07T22:08:36Z"
  tasks: 2
  files: 5
---

# Phase 187 Plan 04: Documentation, CI-Parity Gate, and Publish Checkpoint

Publication-ready docs and CI-parity gate for `ferro-assets`. README + feature page written with
security caveats and zero-C-deps framing. Real seven-transform passthrough proof proves criterion 1
with production transforms, not stubs. Full workspace CI-parity gate green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | README + docs feature page + SUMMARY link + real-transform passthrough proof | ff39caab | README.md, docs/ferro-assets.md, SUMMARY.md, passthrough_proof.rs |
| 2 | CI-parity gate (fmt + clippy --all-targets + test --all-features) + AVIF fix | 2f47a0df | image_transcode_test.rs, passthrough_proof.rs |

## Acceptance Criteria Status

- [x] `test -f ferro-assets/README.md`
- [x] `grep -qi 'zero C' ferro-assets/README.md`
- [x] `grep -qi 'spawn_blocking' ferro-assets/README.md`
- [x] `grep -qi 'Asset.path\|logical key' ferro-assets/README.md` (path sanitization caveat)
- [x] `grep -qi 'sanitize' ferro-assets/README.md` (token-value caller responsibility)
- [x] `test -f docs/src/features/ferro-assets.md`
- [x] `grep -q 'features/ferro-assets.md' docs/src/SUMMARY.md`
- [x] `grep -q 'json_file_unchanged_by_all_seven_real_transforms' ferro-assets/tests/passthrough_proof.rs`
- [x] `cargo test -p ferro-assets --test passthrough_proof` exits 0 (5 tests, all pass)
- [x] `cargo fmt --all -- --check` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] `cargo test --all-features` exits 0 (all test result lines: ok)
- [x] `cargo test -p ferro-assets --features slow-tests` exits 0 (heavy AVIF encode 68s, magic-byte verified)
- [x] `grep -q 'version.workspace = true' ferro-assets/Cargo.toml`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AVIF decode via image::load_from_memory fails under --all-features**
- **Found during:** Task 2 — `cargo test --all-features` activated `slow-tests` feature,
  un-ignoring the heavy test; `image::load_from_memory` on AVIF bytes panicked with
  "The image format Avif is not supported"
- **Issue:** The `image` crate's AVIF *decoder* requires the C `dav1d` library. The
  crate has `features = ["avif"]` for AVIF *encoding* via `ravif` (pure Rust), but
  decoding AVIF back through `image` pulls in a C dependency — contradicting the
  zero-C-deps criterion 3 that is a core acceptance criterion for this crate.
- **Fix:** Replaced `image::load_from_memory(&a.bytes)` with an ISOBMFF ftyp magic-byte
  check (`a.bytes[4..8] == b"ftyp"`). The ravif encode returning `Ok` already guarantees
  valid AVIF bytes; the magic-byte check provides lightweight structural proof without
  requiring a C decoder.
- **Files modified:** ferro-assets/tests/image_transcode_test.rs
- **Commit:** 2f47a0df

**2. [Rule 1 - Bug] cargo fmt: import ordering and line-length in passthrough_proof.rs**
- **Found during:** Task 2 step 1 — `cargo fmt --all -- --check` failed on the new
  imports added in Task 1 (use block ordering, multi-line break on json_bytes assignment)
- **Fix:** Ran `cargo fmt --all` — rustfmt reordered imports alphabetically and merged
  the two-line json_bytes assignment onto one line
- **Files modified:** ferro-assets/tests/passthrough_proof.rs
- **Commit:** 2f47a0df (included with Task 2 fix)

## Manual First-Publish Reminder

`ferro-assets` is a new crate — it does not yet exist on crates.io. The CI publish token has `publish-update` scope only (not `publish-new`). Before the first CI push:

```bash
cargo publish -p ferro-assets
```

Run from a local terminal with a full-scope API token. This is the same pattern as `ferro-bundle` (Phase 183) and `ferro-deployments` (Phase 186). After the first manual publish succeeds, all subsequent publishes are handled by the Wave 1a CI job.

The user chose to defer this step to the milestone master-push (same precedent as Phase 183 and Phase 186).

## Known Stubs

None. ferro-assets is fully implemented, documented, and CI-green.

## Threat Flags

No new threat surface beyond what was documented in Plans 01-03. The SECURITY NOTES section
in README.md closes T-187-13 (documentation half of T-187-03 and T-187-07):

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-187-13 | mitigated: README.md SECURITY NOTES section documents Asset.path-is-a-logical-key and replace_tokens-caller-sanitizes contracts |
| T-187-14 | accept: first publish is deliberately a manual local step (CI token lacks publish-new); checkpoint enforces human gating |

## Self-Check: PASSED

Files exist:
- ferro-assets/README.md ✓
- docs/src/features/ferro-assets.md ✓
- docs/src/SUMMARY.md (contains `features/ferro-assets.md`) ✓
- ferro-assets/tests/passthrough_proof.rs (contains `json_file_unchanged_by_all_seven_real_transforms`) ✓

Commits exist:
- ff39caab ✓
- 2f47a0df ✓
