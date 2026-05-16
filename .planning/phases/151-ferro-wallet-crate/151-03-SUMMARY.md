---
phase: 151-ferro-wallet-crate
plan: 03
subsystem: infra
tags: [ferro-wallet, config, env-loading, apple-wallet, google-wallet]

requires:
  - phase: 151-01-scaffold
    provides: ferro-wallet crate scaffold + WalletError enum + module placeholders
  - phase: 151-02-subject-trait
    provides: WalletSubject trait + Branding/Field/RgbColor value types
provides:
  - WalletConfig struct (app_name + app_url + Option<AppleConfig> + Option<GoogleConfig>)
  - AppleConfig struct (5 required PEM/identifier fields + optional key_password)
  - GoogleConfig struct (3 required service-account fields)
  - WalletConfig::from_env() with D-02 permissive semantics (never errors on missing wallet vars)
  - AppleConfig::from_env_optional() / GoogleConfig::from_env_optional() (Ok(None) on partial cluster)
  - Restored pub use config::{AppleConfig, GoogleConfig, WalletConfig} re-export in lib.rs
affects:
  - 151-05-apple-builder (ApplePassBuilder::new consumes AppleConfig)
  - 151-07-google-builder (GoogleWalletBuilder::new consumes GoogleConfig)
  - 151-06-apple-integration-test (builds AppleConfig with self-signed cert)
  - 151-08-google-jwt-test (builds GoogleConfig with runtime-minted RSA keypair)
  - downstream gestiscilo-it wallet integration (gates feature on apple.is_some / google.is_some)

tech-stack:
  added: []
  patterns:
    - "Permissive cluster-optional env loading (D-02): partial cluster collapses to Option::None, never errors"
    - "EnvGuard RAII pattern: process-global env vars saved on capture, restored on drop — survives assertion panics unlike ferro-stripe's manual save/restore"
    - "ENV_LOCK Mutex serializes env-touching tests within a single mod tests block (ferro-stripe sidesteps this by having only one such test)"

key-files:
  created: []
  modified:
    - "ferro-wallet/src/config.rs (placeholder -> 400 LoC: 3 structs + permissive from_env + 7 tests)"
    - "ferro-wallet/src/lib.rs (restored pub use config::{...} re-export per D-11)"

key-decisions:
  - "[151-03] APP_NAME / APP_URL fallbacks hardcode the exact strings from framework::config::providers::app.rs (\"Ferro Application\" / \"http://localhost:8080\") rather than depending on framework — keeps ferro-wallet a true leaf crate with zero internal ferro deps per spec §5"
  - "[151-03] AppleConfig and GoogleConfig clusters fail to None on ANY missing required var (not just all missing) — a partial cluster is a misconfiguration, surfacing it as None forces the caller to fix env vars rather than silently issue half-formed passes"
  - "[151-03] from_env signature returns Result<Self, WalletError> not Self — current impl never returns Err but the Result allows forward-compatible non-wallet validation (e.g. malformed URL parsing) without a breaking API change"
  - "[151-03] EnvGuard RAII + ENV_LOCK Mutex chosen over external serial_test dependency to keep ferro-wallet's dep set minimal (matches RESEARCH.md ferro-wallet leaf-crate posture)"

patterns-established:
  - "Permissive cluster-optional config: any-missing-required-var ⇒ Ok(None) for the cluster, full presence ⇒ Ok(Some(populated))"
  - "Panic-safe env restoration in tests: RAII Drop guard rather than ferro-stripe's manual restore-after-assert (better for modules with multiple env-touching tests)"
  - "Test-level serialization for process-global mutation: static Mutex<()> + lock_env() helper that recovers from poisoned mutex"

requirements-completed: [ACC-1a, ACC-1b, ACC-1c]

duration: 4m 15s
completed: 2026-05-11
---

# Phase 151 Plan 03: WalletConfig — env-driven permissive configuration

**Permissive `WalletConfig::from_env` with optional Apple / Google clusters and `framework::config::AppConfig`-matching APP_NAME / APP_URL defaults, gated behind a panic-safe RAII test pattern.**

## Performance

- **Duration:** 4m 15s
- **Started:** 2026-05-11T03:44:43Z
- **Completed:** 2026-05-11T03:48:58Z
- **Tasks:** 2 (both `auto`, Task 1 carried `tdd="true"`)
- **Files modified:** 2 (`ferro-wallet/src/config.rs`, `ferro-wallet/src/lib.rs`)

## Accomplishments

- Implemented `WalletConfig`, `AppleConfig`, `GoogleConfig` structs matching spec §3.2 exactly (field names, types, derives).
- Wired permissive D-02 semantics: any missing var in the Apple cluster (5 required) or Google cluster (3 required) collapses the cluster to `None`; `from_env` never returns `Err` for absent wallet env vars.
- Locked APP_NAME / APP_URL fallbacks to the exact strings consumed by `framework::config::providers::app.rs` (`"Ferro Application"` / `"http://localhost:8080"`), preserving framework parity without taking a framework dep.
- Shipped 7 unit tests covering ACC-1a, ACC-1b, ACC-1c plus partial-cluster, fully-populated cluster, and the load-bearing permissive invariant — 21/21 ferro-wallet tests green.
- Restored `pub use config::{AppleConfig, GoogleConfig, WalletConfig}` in `lib.rs` per D-11.

## Env Var Reference

| Cluster | Env Var | Required? | Falls back to |
|---------|---------|-----------|---------------|
| App | `APP_NAME` | optional | `"Ferro Application"` (matches `AppConfig::from_env`) |
| App | `APP_URL` | optional | `"http://localhost:8080"` (matches `AppConfig::from_env`) |
| Apple | `APPLE_WALLET_PASS_TYPE_ID` | required (cluster) | cluster ⇒ `None` |
| Apple | `APPLE_WALLET_TEAM_ID` | required (cluster) | cluster ⇒ `None` |
| Apple | `APPLE_WALLET_CERT_PEM` | required (cluster) | cluster ⇒ `None` |
| Apple | `APPLE_WALLET_KEY_PEM` | required (cluster) | cluster ⇒ `None` |
| Apple | `APPLE_WALLET_WWDR_PEM` | required (cluster) | cluster ⇒ `None` |
| Apple | `APPLE_WALLET_KEY_PASSWORD` | optional | `AppleConfig.key_password = None` |
| Google | `GOOGLE_WALLET_ISSUER_ID` | required (cluster) | cluster ⇒ `None` |
| Google | `GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL` | required (cluster) | cluster ⇒ `None` |
| Google | `GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM` | required (cluster) | cluster ⇒ `None` |

## Acceptance Criteria → Test Mapping

| Criterion | Test Name | Behaviour |
|-----------|-----------|-----------|
| ACC-1a | `config::tests::from_env_apple_missing_is_none` | Missing Apple env vars ⇒ `WalletConfig.apple == None`, no error |
| ACC-1b | `config::tests::from_env_google_missing_is_none` | Missing Google env vars ⇒ `WalletConfig.google == None`, no error |
| ACC-1c | `config::tests::from_env_defaults_match_appconfig` | Unset `APP_NAME` / `APP_URL` ⇒ `"Ferro Application"` / `"http://localhost:8080"` |

Supporting tests in the same module (not directly mapped to ACC-IDs but locked in):
- `from_env_apple_partial_returns_none` — 4 of 5 Apple vars set, one unset ⇒ `apple: None`
- `from_env_apple_all_set_returns_some` — all 5 + optional password ⇒ populated cluster, fields match env values
- `from_env_google_all_set_returns_some` — all 3 ⇒ populated cluster, fields match env values
- `from_env_never_errors_on_missing_wallet_vars` — D-02 invariant: any env state ⇒ `Ok(_)`

## Task Commits

1. **Task 1: Implement WalletConfig + AppleConfig + GoogleConfig + permissive from_env + 7 tests** — `765d17ac` (feat)
2. **Task 2: Restore config re-exports in lib.rs** — `1ff86bb4` (feat)

## Files Created/Modified

- `ferro-wallet/src/config.rs` — Replaced `// placeholder` with three config structs, `WalletConfig::from_env` (permissive), and a `#[cfg(test)] mod tests` block containing the `EnvGuard` RAII helper, `ENV_LOCK` mutex, and 7 acceptance tests.
- `ferro-wallet/src/lib.rs` — Uncommented `pub use config::{AppleConfig, GoogleConfig, WalletConfig};` (D-11 staged-export reveal).

## Decisions Made

- **APP_NAME / APP_URL fallback strings hardcoded** rather than imported from `framework::config::providers::app.rs`. ferro-wallet is a leaf crate per spec §5; pulling in `framework` would invert the dependency graph. The fallback strings are part of ferro-wallet's public contract via the doc-comment + ACC-1c test, so any future change to framework's defaults must be mirrored here (not coupled).
- **Cluster-optional fail-to-None on partial config** rather than fail-loud on partial. A partial Apple cluster (4 of 5 vars set) is unambiguously a deployment misconfiguration; returning `None` forces the caller to discover the issue at startup-feature-gate time rather than at first pass-issuance time. The `from_env_apple_partial_returns_none` test locks this in.
- **`from_env` keeps `Result<Self, WalletError>` signature** even though the current implementation never returns `Err`. Forward-compatible: future non-wallet validation (e.g. malformed `APP_URL` parsing) can flip the error path on without an SemVer break. Spec §3.2 documents the signature with `Result`, and downstream PLAN-05 / PLAN-07 already use `?` to chain the call.
- **EnvGuard RAII over `serial_test` crate** — adding a dev-dep would gain panic-safety + serialization but at the cost of pulling in a procedural-macro dependency for what amounts to ~30 lines of in-module code. The internal `Mutex<()>` + `Drop` guard pattern is self-contained.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced `std::panic::catch_unwind` test pattern with RAII guards**

- **Found during:** Task 1 (first `cargo test` run)
- **Issue:** The plan's task body proposed wrapping assertions in `std::panic::catch_unwind` so env restore would happen even if an assertion panicked. The build failed because `Result<WalletConfig, WalletError>` is not `UnwindSafe` (transitively through `std::io::Error::Custom`'s `Box<dyn std::error::Error>` field), so the closure passed to `catch_unwind` could not capture the result.
- **Fix:** Swapped to an `EnvGuard` struct that snapshots the env in `capture_all()` and restores in `Drop`. The guard provides the same panic-safety guarantee (drop runs on panic, before the stack unwinds out of the test) without requiring `UnwindSafe`. Net code is shorter and the pattern is reusable for the upcoming PLAN-06 / PLAN-08 integration tests that will also touch env vars.
- **Files modified:** `ferro-wallet/src/config.rs` (test module only)
- **Verification:** `cargo test -p ferro-wallet --lib config::tests` runs 7/7 green; deliberately panicking inside the assert block in local experimentation still leaves env clean afterwards.
- **Committed in:** `765d17ac` (Task 1 commit, integrated before commit)

**2. [Rule 2 - Missing Critical] Added `ENV_LOCK: Mutex<()>` to serialize env-touching tests**

- **Found during:** Task 1 design (before writing tests)
- **Issue:** ferro-stripe's config module has ONE env-touching test, so it does not race against itself under `cargo test`'s parallel execution. ferro-wallet has SEVEN env-touching tests in the same module; without serialization, e.g. `from_env_apple_all_set_returns_some` could `set_var` `APPLE_WALLET_PASS_TYPE_ID` while `from_env_apple_missing_is_none` is mid-assert on the same var, producing flaky behaviour. The plan said "match ferro-stripe's pattern" but did not address this multi-test case.
- **Fix:** Added a static `Mutex<()>` inside the test module; every env-touching test acquires `_lock = lock_env()` (which recovers from a poisoned mutex if a prior test panicked) before manipulating env vars.
- **Files modified:** `ferro-wallet/src/config.rs` (test module only)
- **Verification:** Ran `cargo test -p ferro-wallet --lib config::tests` repeatedly; all 7 tests pass on every run, no order-dependent failures.
- **Committed in:** `765d17ac` (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 Rule 1 build bug, 1 Rule 2 test-correctness gap)
**Impact on plan:** Both deviations were strictly internal to the test module — production API surface, public types, and the env-var contract match the plan exactly. No scope creep.

## Issues Encountered

- `std::io::Error` is not `UnwindSafe`, which made the plan's `catch_unwind` test pattern uncompilable (see Deviation 1).

## Threat Flags

None. The threat-register dispositions from the plan's `<threat_model>` (T-151-Apple-SIGN `accept (partial)` documenting `Debug`-includes-PEM exposure; T-151-DEFAULT-CRED `accept` documenting that the default strings are framework-agnostic placeholders, not credentials) are honoured in code via the module doc-comment warning. No new surface introduced.

## Verification Gates

- [x] `cargo test -p ferro-wallet --lib config::tests::from_env_apple_missing_is_none` — exits 0
- [x] `cargo test -p ferro-wallet --lib config::tests::from_env_google_missing_is_none` — exits 0
- [x] `cargo test -p ferro-wallet --lib config::tests::from_env_defaults_match_appconfig` — exits 0
- [x] `cargo test -p ferro-wallet --lib config::tests` — 7/7 pass
- [x] `cargo test -p ferro-wallet` — 21/21 pass (7 new + 14 from PLAN-01 / PLAN-02)
- [x] `cargo build --workspace` — exits 0
- [x] `cargo clippy -p ferro-wallet --all-targets -- -D warnings` — exits 0
- [x] `cargo fmt --all -- --check` — exits 0
- [x] `grep -F 'pub use config::{AppleConfig, GoogleConfig, WalletConfig};' ferro-wallet/src/lib.rs` — one match

## Next Phase Readiness

- PLAN-04 (images + qr) and PLAN-05 (apple builder) are now unblocked — both depend on either PLAN-03's `AppleConfig` (PLAN-05) or have no PLAN-03 dependency at all (PLAN-04, sibling of PLAN-03 in the wave decomposition).
- PLAN-07 (google builder) is unblocked — consumes `GoogleConfig` shipped here.
- The `EnvGuard` RAII test pattern is now available as the canonical reference for any future env-mutating test in `ferro-wallet/tests/*` (PLAN-06 integration test, PLAN-08 integration test).

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*

## Self-Check: PASSED

- `ferro-wallet/src/config.rs` — FOUND
- `ferro-wallet/src/lib.rs` — FOUND
- `.planning/phases/151-ferro-wallet-crate/151-03-SUMMARY.md` — FOUND
- Commit `765d17ac` (Task 1 — config.rs) — FOUND
- Commit `1ff86bb4` (Task 2 — lib.rs) — FOUND
