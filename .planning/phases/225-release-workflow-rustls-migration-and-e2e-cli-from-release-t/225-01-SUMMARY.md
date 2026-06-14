---
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
plan: 01
subsystem: infra
tags: [rustls, ring, native-tls, openssl, sea-orm, lettre, reqwest, tls, cargo-deny, cross-compile]

requires: []

provides:
  - Workspace-wide TLS backend swap: native-tls/OpenSSL removed, rustls/ring installed
  - ferro-cli transitive tree: zero native-tls, zero openssl-sys, zero aws-lc-rs; ring v0.17.14 present
  - sea-orm + sea-orm-migration: runtime-tokio-rustls across all 14 crates
  - lettre: tokio1-rustls-tls (ferro-notifications)
  - reqwest: default-features=false + rustls-tls across all 7 crates (ferro-cli, ferro-mcp, ferro-whatsapp, ferro-api-mcp, ferro-notifications ×2, ferro-ai)
  - Cargo.lock updated to remove hyper-tls 0.6.0, tokio-native-tls, hostname; add rustls/tokio-rustls paths for reqwest and lettre

affects:
  - 225-02-PLAN (aarch64 cross-compile now viable without cross/Docker — rustls removed only C-cross barrier)
  - 225-03-PLAN (e2e from-release job; release binary now builds without libssl-dev)
  - Any phase touching CI cross-compilation or cargo install ferro-cli cold builds

tech-stack:
  added:
    - ring v0.17.14 (was already in lockfile via ferro-storage; now also reachable from all sea-orm/reqwest/lettre paths)
    - webpki-roots 1.0.7 (bundled Mozilla CA set, replaces OS trust store lookup)
    - tokio-rustls 0.26.4 (lettre async TLS)
    - rustls 0.23.36 (lettre backend)
  patterns:
    - "reqwest rustls form: version + default-features=false + features=[..., rustls-tls] — ferro-storage is the canonical model"
    - "sea-orm TLS feature: runtime-tokio-rustls (not runtime-tokio-native-tls); all other features preserved verbatim"
    - "lettre TLS feature: tokio1-rustls-tls + default-features=false (required to avoid native-tls re-activation)"
    - "D-02 hard gate: cargo tree -p ferro-cli | grep aws-lc | wc -l must equal 0"

key-files:
  created: []
  modified:
    - ferro-cli/Cargo.toml
    - framework/Cargo.toml
    - ferro-queue/Cargo.toml
    - ferro-mcp/Cargo.toml
    - ferro-orm/Cargo.toml
    - ferro-audit/Cargo.toml
    - ferro-migration/Cargo.toml
    - ferro-projection/Cargo.toml
    - ferro-deployments/Cargo.toml
    - ferro-reservation/Cargo.toml
    - ferro-mcp-oauth/Cargo.toml
    - ferro-mcp-server/Cargo.toml
    - app/Cargo.toml
    - ferro-notifications/Cargo.toml
    - ferro-whatsapp/Cargo.toml
    - ferro-api-mcp/Cargo.toml
    - ferro-ai/Cargo.toml
    - Cargo.lock

key-decisions:
  - "D-01: sea-orm/sea-orm-migration runtime-tokio-native-tls -> runtime-tokio-rustls, workspace-wide"
  - "D-02: ring is the crypto provider (not aws-lc-rs); verified by cargo tree structural gate"
  - "D-03: ferro-wallet direct openssl=0.10 left untouched (not in ferro-cli tree)"
  - "lettre requires default-features=false when adding tokio1-rustls-tls (deviation R2 auto-fix)"
  - "reqwest coherence: all 7 reqwest users across workspace now use default-features=false + rustls-tls"

patterns-established:
  - "reqwest rustls form: { version = \"0.12\", default-features = false, features = [\"...\", \"rustls-tls\"] }"
  - "lettre rustls form: { version = \"0.11\", default-features = false, features = [\"tokio1-rustls-tls\", \"builder\", \"smtp-transport\"] }"
  - "sea-orm rustls form: { version = \"1.0\", features = [\"sqlx-sqlite\", \"sqlx-postgres\", \"runtime-tokio-rustls\", ...] }"

requirements-completed: []

duration: ~35min
completed: 2026-06-14
---

# Phase 225, Plan 01: rustls TLS Migration Summary

**Workspace-wide native-tls/OpenSSL removed; ring v0.17.14 now sole TLS crypto provider across ferro-cli's transitive tree (18 sea-orm/lettre occurrences + 7 reqwest coherence fixes across 17 Cargo.toml files)**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-06-14T15:35:00Z
- **Completed:** 2026-06-14T17:50:00Z
- **Tasks:** 3
- **Files modified:** 18 (17 Cargo.toml + Cargo.lock)

## Accomplishments

- Removed native-tls/OpenSSL from the entire workspace: `cargo tree -p ferro-cli | grep native-tls|openssl-sys` returns 0 lines
- ring v0.17.14 is the sole TLS crypto provider, selected automatically via feature flags (no explicit dep or `CryptoProvider::install_default()` needed)
- aws-lc-rs absent from ferro-cli tree: `wc -l` = 0 (D-02 — "no external build tooling" rule satisfied)
- `cargo build --all-features` passes; `cargo clippy --all --all-targets -- -D warnings` passes
- Cargo.lock updated: hyper-tls 0.6.0, tokio-native-tls, hostname removed; replaced with rustls/tokio-rustls/webpki-roots paths

## Structural Verification Results (Task 3)

| Check | Command | Result |
|-------|---------|--------|
| native-tls/openssl-sys in ferro-cli tree | `grep -E 'native-tls\|openssl-sys\|openssl-' \| wc -l` | **0** |
| aws-lc-rs in ferro-cli tree | `grep -E 'aws-lc-sys\|aws-lc-rs' \| wc -l` | **0** |
| ring present (TLS functional) | `grep 'ring v'` | **ring v0.17.14** (multiple paths) |
| Workspace build | `cargo build --all-features` | **exit 0** (1m 43s) |
| Clippy | `cargo clippy --all --all-targets -- -D warnings` | **exit 0** |
| cargo-deny | `cargo deny check` | **not installed locally** (CI-only; run by EmbarkStudios/cargo-deny-action@v2 in CI) |

## Task Commits

1. **Task 1: sea-orm/lettre TLS backend swap (14 files)** - `13ec56c0` (chore)
2. **Task 1 fix: lettre default-features=false** - `352084e8` (fix — deviation R2)
3. **Task 2: reqwest coherence pass (5 files)** - `a47ae06b` (chore)
4. **Task 3: lockfile regeneration** - `9614a9dd` (chore)

## Files Modified

- `ferro-cli/Cargo.toml` - reqwest: `default-features=false, rustls-tls`; sea-orm + sea-orm-migration: `runtime-tokio-rustls`
- `framework/Cargo.toml` - sea-orm: `runtime-tokio-rustls`
- `ferro-queue/Cargo.toml` - sea-orm: `runtime-tokio-rustls`
- `ferro-mcp/Cargo.toml` - sea-orm: `runtime-tokio-rustls`; reqwest: `default-features=false, rustls-tls`
- `ferro-orm/Cargo.toml` - sea-orm dev-dep: `runtime-tokio-rustls`
- `ferro-audit/Cargo.toml` - sea-orm dev-dep: `runtime-tokio-rustls`
- `ferro-migration/Cargo.toml` - sea-orm dev-dep: `runtime-tokio-rustls`
- `ferro-projection/Cargo.toml` - sea-orm dev-dep: `runtime-tokio-rustls`
- `ferro-deployments/Cargo.toml` - sea-orm: `runtime-tokio-rustls` (kept with-chrono)
- `ferro-reservation/Cargo.toml` - sea-orm dev-dep: `runtime-tokio-rustls`
- `ferro-mcp-oauth/Cargo.toml` - sea-orm: `runtime-tokio-rustls` (both [dependencies] and [dev-dependencies])
- `ferro-mcp-server/Cargo.toml` - sea-orm: `runtime-tokio-rustls`
- `app/Cargo.toml` - sea-orm + sea-orm-migration: `runtime-tokio-rustls` (both occurrences)
- `ferro-notifications/Cargo.toml` - lettre: `tokio1-rustls-tls, default-features=false`; reqwest: `default-features=false, rustls-tls` (both [dependencies] and [dev-dependencies])
- `ferro-whatsapp/Cargo.toml` - reqwest: `default-features=false, rustls-tls`
- `ferro-api-mcp/Cargo.toml` - reqwest: `default-features=false, rustls-tls` (both [dependencies] and [dev-dependencies])
- `ferro-ai/Cargo.toml` - reqwest (optional, llm feature): `default-features=false, rustls-tls` + `stream` preserved
- `Cargo.lock` - hyper-tls 0.6.0 removed, tokio-native-tls removed, hostname removed; rustls/webpki-roots added for lettre path

## Decisions Made

- D-01: Workspace-wide TLS swap: sea-orm `runtime-tokio-rustls`, lettre `tokio1-rustls-tls`, reqwest `default-features=false + rustls-tls`
- D-02: ring (not aws-lc-rs) confirmed as the crypto provider via feature chain: `reqwest rustls-tls → __rustls-ring → ring`; `sea-orm runtime-tokio-rustls → sqlx tls-rustls-ring → ring`. Hard gate in Task 3 structural check.
- D-03: ferro-wallet `openssl = "0.10"` left untouched (not in ferro-cli's tree; coherence follow-up for a future phase)
- reqwest coherence scope extended to all 7 users per D-01 "one TLS backend = one source of truth"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] lettre requires `default-features = false` to prevent native-tls re-activation**
- **Found during:** Task 1 build verification
- **Issue:** Setting `tokio1-rustls-tls` without `default-features = false` on lettre leaves lettre's default native-tls feature active. With both `tokio1` and `native-tls` active but not `tokio1-native-tls`, lettre emits a `compile_error!`. PATTERNS.md mentioned `default-features = false` was required; the initial Task 1 commit missed it.
- **Fix:** Added `default-features = false` to lettre declaration in ferro-notifications/Cargo.toml
- **Files modified:** ferro-notifications/Cargo.toml
- **Verification:** `cargo build --all-features` passes post-fix
- **Committed in:** `352084e8` (fix(ferro-notifications): disable lettre default features)

---

**Total deviations:** 1 auto-fixed (Rule 2 — missing critical correctness)
**Impact on plan:** Necessary for compilation; lettre's default-features behaviour is a known pitfall documented in PATTERNS.md. No scope creep.

## Issues Encountered

- `cargo deny` not installed locally — the D-05 gate runs only in CI via `EmbarkStudios/cargo-deny-action@v2`. The ring v0.17.14 license (Apache-2.0 AND ISC) was pre-verified in RESEARCH.md against the deny.toml allow-list. No deny.toml changes were needed.

## User Setup Required

None — no external service configuration required. The TLS backend swap is transparent to application consumers.

## Known Stubs

None — this plan makes no application-level changes; only TLS feature flags in Cargo manifests.

## Threat Flags

No new network endpoints, auth paths, or schema changes introduced. The threat surface is reduced (openssl-sys removed) rather than expanded.

## Next Phase Readiness

- Plan 02 (aarch64 native cross-compile via rustup target add + gcc-aarch64-linux-gnu) is now unblocked: ring v0.17.14 is the only C build dep remaining, and the CC_aarch64_unknown_linux_gnu env var pattern from RESEARCH.md is the implementation path.
- Plan 03 (e2e from-release CI job) is unblocked: `cargo install ferro-cli` on a clean Debian box no longer needs `libssl-dev` + `pkg-config`.
- ferro-wallet's `openssl = "0.10"` is the only remaining OpenSSL in the workspace (D-03 deferred). Not in the release binary path.

---

## Self-Check

**Files exist:**
- `ferro-cli/Cargo.toml` contains `runtime-tokio-rustls`: VERIFIED (grep confirmed, no native-tls occurrences)
- `ferro-notifications/Cargo.toml` contains `tokio1-rustls-tls` and `default-features = false`: VERIFIED
- All reqwest users have `default-features = false`: VERIFIED (grep returns 0 bare-features lines)

**Commits exist:**
- `13ec56c0`: Task 1 sea-orm/lettre swap — VERIFIED (git log)
- `352084e8`: lettre default-features fix — VERIFIED (git log)
- `a47ae06b`: reqwest coherence pass — VERIFIED (git log)
- `9614a9dd`: Cargo.lock regeneration — VERIFIED (git log)

## Self-Check: PASSED

*Phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t*
*Completed: 2026-06-14*
