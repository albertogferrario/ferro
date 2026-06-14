---
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
verified: 2026-06-14T18:30:00Z
status: human_needed
score: 9/9
overrides_applied: 0
human_verification:
  - test: "cargo install ferro-cli on a clean debian:bookworm-slim container with no libssl-dev/pkg-config installed"
    expected: "install succeeds without any OpenSSL apt packages"
    why_human: "Requires a fresh container environment; cannot be verified by cargo tree or a local build"
  - test: "Download aarch64-unknown-linux-gnu release artifact and run ./ferro --version on a real arm64 host"
    expected: "Binary executes and prints version without error"
    why_human: "CI builds but does not execute the foreign-arch binary; no arm64 hardware available in the automated check path"
---

# Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test — Verification Report

**Phase Goal:** Migrate ferro-cli's TLS-bearing transitive deps (and the workspace sea-orm/lettre backend) from native-tls/OpenSSL to rustls (ring provider) so release artifacts and `cargo install ferro-cli` build with NO system OpenSSL; and add a CI e2e test that runs the actual released `ferro` binary, scaffolds an app, and `cargo build`s it against the published `ferro-rs` library to catch the COMP-04 failure class.

**Verified:** 2026-06-14T18:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

All 9 must-haves are VERIFIED by structural checks and direct file inspection. Two items from the phase's own VALIDATION.md require human testing (clean container install, arm64 execution). The e2e CI jobs being currently-RED against the drifted published scaffold is intentional behavior per D-10 and is NOT a gap.

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | D-01/D-02: ferro-cli tree has no native-tls, no openssl-sys, no aws-lc-rs | VERIFIED | `cargo tree -p ferro-cli --edges no-dev -e features \| grep -E 'native-tls\|openssl-sys'` = 0; `\| grep -E 'aws-lc-sys\|aws-lc-rs'` = 0 |
| 2 | D-02: ring v0.17.14 present in ferro-cli tree (TLS functional) | VERIFIED | `cargo tree -p ferro-cli --edges no-dev \| grep 'ring v'` returns ring v0.17.14 at multiple paths |
| 3 | D-01 workspace swap: no remaining runtime-tokio-native-tls, all reqwest have default-features=false + rustls-tls, lettre uses tokio1-rustls-tls | VERIFIED | `grep -r 'runtime-tokio-native-tls' --include=Cargo.toml` = empty; all 9 reqwest deps confirmed default-features=false; ferro-notifications lettre = tokio1-rustls-tls + default-features=false |
| 4 | D-03: ferro-wallet/Cargo.toml still has openssl = "0.10" (intentionally untouched) | VERIFIED | ferro-wallet/Cargo.toml line 14: `openssl = "0.10"` present |
| 5 | D-04: aarch64 builds natively — no cross:true, both LINKER and CC env vars present | VERIFIED | release.yml: no `cross:` key; `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc` and `CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc` in Build (native) step env |
| 6 | D-06/07/08: e2e-tag (needs:build, push-only) and e2e-drift (schedule/dispatch) present; COMP-04 sequence present; no [patch.crates-io] table | VERIFIED | release.yml: both jobs present; on: includes workflow_dispatch + schedule; COMP-04 6-command sequence present in both; `[patch.crates-io]` exists only as a `# DO NOT add` comment |
| 7 | D-09: ci.yml scaffold-smoke job unchanged by this phase | VERIFIED | ci.yml scaffold-smoke job present, runs `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro`; `git diff --quiet ci.yml` confirmed in 225-03-SUMMARY.md |
| 8 | D-10: both e2e jobs are continue-on-error:true with TODO(D-10); e2e is not a blocking gate | VERIFIED | release.yml lines 150 and 193: `continue-on-error: true   # TODO(D-10): flip to false after...` |
| 9 | WR-01 (code-review follow-up): build, release, update-install-script carry `if: github.event_name == 'push'` | VERIFIED | release.yml lines 17, 95, 122: all three tag-only jobs guarded; commit b1411096 applied the fix post-review |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-cli/Cargo.toml` | reqwest rustls-tls + sea-orm runtime-tokio-rustls | VERIFIED | reqwest: `default-features=false, features=["blocking","json","rustls-tls"]`; sea-orm + sea-orm-migration: `runtime-tokio-rustls` |
| `framework/Cargo.toml` | sea-orm runtime-tokio-rustls | VERIFIED | Confirmed in 225-01-SUMMARY.md file list |
| `ferro-notifications/Cargo.toml` | lettre tokio1-rustls-tls + reqwest rustls-tls | VERIFIED | `lettre = { version = "0.11", default-features = false, features = ["tokio1-rustls-tls", ...] }` |
| `.github/workflows/release.yml` | e2e-tag + e2e-drift jobs; aarch64 native build; push-only guards on tag jobs | VERIFIED | All four structural requirements confirmed by direct file read |
| `.github/workflows/ci.yml` | scaffold-smoke job unchanged | VERIFIED | Job present and unmodified |
| `ferro-wallet/Cargo.toml` | openssl = "0.10" retained (D-03 out-of-scope) | VERIFIED | Line 14 confirmed |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| ferro-cli reqwest | rustls/ring (no native-tls) | `default-features = false, features = [..., "rustls-tls"]` | VERIFIED | Confirmed in ferro-cli/Cargo.toml line 48 |
| sea-orm across 13 workspace crates | ring via sqlx tls-rustls-ring | `runtime-tokio-rustls` feature on all sea-orm deps | VERIFIED | Zero `runtime-tokio-native-tls` remaining in workspace; 225-01-SUMMARY.md lists all 13 crates |
| e2e-tag job | just-built ferro binary | `actions/download-artifact` + `needs: build` + `if: github.event_name == 'push'` | VERIFIED | release.yml lines 145-188 |
| e2e-drift job | crates.io ferro-cli | `cargo install ferro-cli` + schedule/dispatch trigger | VERIFIED | release.yml lines 189-222 |
| build/release/update-install-script | push-only execution | `if: github.event_name == 'push'` guard | VERIFIED | Lines 17, 95, 122 of release.yml |

### Data-Flow Trace (Level 4)

Not applicable — this phase is CI/CD configuration and Cargo manifest changes, not application code with data-binding.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| native-tls/openssl-sys absent from ferro-cli tree | `cargo tree -p ferro-cli --edges no-dev -e features \| grep -E 'native-tls\|openssl-sys' \| wc -l` | 0 | PASS |
| aws-lc-rs absent from ferro-cli tree | `cargo tree -p ferro-cli --edges no-dev \| grep -E 'aws-lc-sys\|aws-lc-rs' \| wc -l` | 0 | PASS |
| ring present in ferro-cli tree | `cargo tree -p ferro-cli --edges no-dev \| grep 'ring v'` | ring v0.17.14 (multiple paths) | PASS |
| No remaining runtime-tokio-native-tls | `grep -r 'runtime-tokio-native-tls' --include=Cargo.toml` | (empty) | PASS |
| All reqwest deps have default-features=false | `grep -r 'reqwest' --include=Cargo.toml \| grep -v default-features` | Comments only, no actual dep lines | PASS |
| cargo build --all-features | (reused from 225-01-SUMMARY.md) | exit 0 (~1m 43s) | PASS |
| cargo clippy --all --all-targets -D warnings | (reused from 225-01-SUMMARY.md) | exit 0 | PASS |
| Clean container cargo install (no libssl-dev) | Needs fresh debian:bookworm-slim container | — | SKIP (human required) |
| aarch64 binary execution on real arm64 | Needs arm64 hardware | — | SKIP (human required) |

### Requirements Coverage

No formal requirement IDs mapped to this phase (ROADMAP requirements = TBD). Coverage contract is CONTEXT.md decisions D-01..D-10 — all verified above.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `.github/workflows/release.yml` | 156, 199 | `dtolnay/rust-toolchain@master` (unpinned action ref) in e2e jobs | Info (IN-02 from code review) | Low — trusted action, unpinned but not a correctness issue |
| `ferro-stripe/Cargo.toml` | (not in scope) | `async-stripe runtime-tokio-hyper` still pulls native-tls into workspace | Info (IN-01 from code review) | Does not affect release binary; affects --all-features workspace builds |
| `ferro-mcp-oauth/Cargo.toml`, `ferro-mcp-server/Cargo.toml` | — | `thiserror = "1"` while workspace uses `thiserror = "2"` | Info (IN-03 from code review) | Pre-existing; doubles compile cost for those two crates only |

All three are info-level items from the code review, none block the phase goal. The code-review warning (WR-01: build/release jobs running on schedule) was fixed in commit b1411096 and is confirmed resolved.

### Human Verification Required

#### 1. Clean Container `cargo install ferro-cli`

**Test:** Build a fresh `debian:bookworm-slim` container. Do NOT install `libssl-dev` or `pkg-config`. Run `cargo install ferro-cli`. Alternatively, modify the COMP-04 Dockerfile at `ferro-cli/tests/fixtures/benchmark/Dockerfile` by removing the `libssl-dev pkg-config` apt line, then run `docker build`.

**Expected:** Install completes successfully with exit 0. No linker errors about `-lssl` or `pkg-config` missing.

**Why human:** Requires a fresh container environment with a clean crates.io registry cache. Cannot be emulated by `cargo tree` alone — the tree check proves no openssl-sys dep is declared, but only an actual cold build proves the end-to-end toolchain resolves cleanly.

#### 2. aarch64 Binary Execution on Real arm64 Hardware

**Test:** Trigger the release workflow on a tag push, download the `ferro-aarch64-unknown-linux-gnu.tar.gz` artifact, extract it, and run `./ferro --version` on an arm64 host (Apple M-series Mac with Rosetta-free terminal, or a Graviton EC2 instance).

**Expected:** Binary prints the version string and exits 0.

**Why human:** The CI workflow builds but does not execute the cross-compiled binary. Correctness of the ring CC env var setup (both `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` and `CC_aarch64_unknown_linux_gnu`) can only be proven by a real tag-push CI run and artifact download.

### Gaps Summary

No gaps. All 9 must-haves pass. The two human verification items are expected operational smoke tests documented in the phase's own VALIDATION.md as "Manual-Only" — they do not represent implementation gaps.

Note on e2e job expected-RED status: both `e2e-tag` and `e2e-drift` jobs are expected to fail against the currently-published `ferro-rs` (0.2.55–0.2.59) due to the pre-existing COMP-04 scaffold-template drift (52 compile errors). This is the intended behavior per D-10, reflected in `continue-on-error: true`. The phase's goal is to add the detecting test, not to fix the drift — the alignment is a separate follow-up phase. This is not a gap.

---

_Verified: 2026-06-14T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
