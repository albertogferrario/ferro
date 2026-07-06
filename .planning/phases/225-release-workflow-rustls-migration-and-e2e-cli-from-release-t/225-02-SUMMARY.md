---
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
plan: "02"
subsystem: ci-release
tags: [ci, release, aarch64, cross-compile, rustls, ring]
dependency_graph:
  requires: ["225-01"]
  provides: ["native-aarch64-release-build"]
  affects: [".github/workflows/release.yml"]
tech_stack:
  added: []
  patterns: ["native cross-compile via gcc-aarch64-linux-gnu + CC env var for ring build script"]
key_files:
  modified: [".github/workflows/release.yml"]
decisions:
  - "D-04: drop cross/Docker for aarch64-unknown-linux-gnu; build natively with rustup target add + gcc-aarch64-linux-gnu apt + CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER + CC_aarch64_unknown_linux_gnu"
metrics:
  duration: "~3 minutes"
  completed: "2026-06-14"
  tasks_completed: 1
  files_modified: 1
---

# Phase 225 Plan 02: Drop cross for aarch64 — Native Build with Ring CC Env

**One-liner:** `release.yml` aarch64-unknown-linux-gnu now builds via `cargo build` with `gcc-aarch64-linux-gnu` apt cross-linker and both `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` + `CC_aarch64_unknown_linux_gnu` env vars — no `cross` tool, no Docker.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Drop cross for aarch64; build natively with cross-linker + ring CC env | 1e6e18cc | `.github/workflows/release.yml` |

## What Was Built

Edited `.github/workflows/release.yml` to implement decision D-04:

1. **Removed** `cross: true` from the `aarch64-unknown-linux-gnu` matrix entry — the matrix variable no longer exists.

2. **Deleted** two `cross`-guarded steps entirely:
   - `Install cross` (`cargo install cross --git https://github.com/cross-rs/cross`)
   - `Build (cross)` (`cross build --release --target ...`)

3. **Updated** "Add Rust target" — removed `if: '!matrix.cross'` guard; now runs unconditionally for all targets including aarch64.

4. **Added** "Install cross-compilation toolchain (aarch64)" step (gated `if: matrix.target == 'aarch64-unknown-linux-gnu'`):
   ```yaml
   sudo apt-get update -q
   sudo apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu
   ```

5. **Updated** "Build (native)" — removed `if: '!matrix.cross'` guard; added `env` block with both env vars:
   ```yaml
   CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc
   CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc
   ```
   The env vars are present on all matrix rows but only matter on the aarch64 row — harmless on others.

## Verification Passed

All automated checks from the plan passed:
- `! grep -qE 'matrix\.cross|cargo install cross|cross build|cross: true'` — PASS
- `grep -q 'CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc'` — PASS
- `grep -q 'CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc'` — PASS
- `grep -q 'gcc-aarch64-linux-gnu'` — PASS
- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"` — PASS (valid YAML)

## Decisions Made

- **D-04 implemented:** Workflow env var approach chosen over `.cargo/config.toml` (env vars cover both linker and CC in one place; self-contained in the workflow without touching committed `.cargo/` config).
- **CC_aarch64_unknown_linux_gnu required:** Ring's build script uses the `cc` crate and probes for a C compiler independently from Cargo's linker. Without this var, ring may fall back to the wrong toolchain (`arm-linux-gnueabihf-gcc`, 32-bit ARM) on ubuntu-latest, producing "cc1" / "ARM_ARCH" errors (ring issue #2131). Both vars are mandatory.

## Operator Note

Pushing `.github/workflows/*` changes requires the `workflow` gh-token scope. The current token lacks this scope (`project_ferro_ci_disk_and_push`). To push:
```
gh auth refresh -s workflow
git push
```
Or use an SSH key / PAT with `workflow` scope. This does not block local execution — the change is committed and ready to push when the operator has the appropriate token.

## Final Proof

The aarch64 matrix job exit-0 can only be confirmed on the next tag-triggered CI run (`git push origin v*`). The local verification (no cross references, valid YAML, correct env vars, correct step guards) is complete.

## Deviations from Plan

None — plan executed exactly as written. All four D-1..D-4 edits applied as specified.

## Known Stubs

None.

## Threat Flags

None. T-225-04 and T-225-05 from the plan's threat model are addressed: the apt cross-linker comes from standard ubuntu-latest repos (same trust level as existing toolchain), and both required CC env vars are present to pin the correct compiler for ring.

## Self-Check: PASSED

- `.github/workflows/release.yml` exists and was modified: FOUND
- Commit `1e6e18cc` exists: FOUND (`git log --oneline | head -1` confirms `1e6e18cc ci(225-02): drop cross for aarch64...`)
- No `cross`/`matrix.cross` references remain: VERIFIED
- Both env vars present: VERIFIED
- YAML parses: VERIFIED
