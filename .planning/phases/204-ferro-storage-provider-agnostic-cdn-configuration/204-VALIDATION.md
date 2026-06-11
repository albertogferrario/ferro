---
phase: 204
slug: ferro-storage-provider-agnostic-cdn-configuration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-11
---

# Phase 204 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` / `#[tokio::test]` |
| **Config file** | none — cargo test discovery (`ferro-storage/Cargo.toml` dev-deps) |
| **Quick run command** | `cargo test -p ferro-storage -- --test-threads=1` (serial for env-var isolation) |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` |
| **Estimated runtime** | ~30–90 seconds (crate) / longer for `--all-features` gate |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-storage -- --test-threads=1`
- **After every plan wave:** `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite green
- **Max feedback latency:** ~90 seconds (crate-scoped)

---

## Per-Task Verification Map

| SC | Behavior | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|----|----------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC-1 | `cdn::Config::from_env` reads quartet as primary | T-204-TOKEN-REDACT | purge_token `<redacted>` in Debug | unit | `cargo test -p ferro-storage cdn_config_from_env` | ❌ W0 | ⬜ pending |
| SC-2 | Per-var legacy fallback chain + `tracing::warn!` | T-204-DEPRECATION-LEAK | token value never printed in warn | unit | `cargo test -p ferro-storage cdn_fallback_` | ❌ W0 | ⬜ pending |
| SC-3 | `Disk::cdn_url()` byte-identical for `AWS_CDN_URL`-only env | — | parity preserved | unit | `cargo test -p ferro-storage cdn_url_parity` | partial (extends `from_env_cdn_url`) | ⬜ pending |
| SC-4 | `purge()` auths same DO Spaces CDN API with legacy vars | T-204-PURGE-PARITY | same endpoint/auth | integration | `cargo test -p ferro-storage purge_parity_legacy_do` | ❌ W0 | ⬜ pending |
| SC-5a | `CDN_PROVIDER=none` → purge() explicit logged no-op | T-204-SILENT-NOOP | loud no-op, not silent prod fail | unit | `cargo test -p ferro-storage cdn_provider_none` | ❌ W0 | ⬜ pending |
| SC-5b | Invalid `CDN_PROVIDER` → boot error listing valid values | T-204-MISCONFIG | fail loud at boot | unit | `cargo test -p ferro-storage cdn_invalid_provider` | ❌ W0 | ⬜ pending |
| SC-5c | Provider selected but feature off → boot error naming flag | T-204-MISCONFIG | fail loud at boot | unit (cfg) | `cargo test -p ferro-storage cdn_feature_required` | ❌ W0 | ⬜ pending |
| SC-6 | ferro-storage minor bump + CHANGELOG `## [0.2.53]` | — | — | file check | inspect `ferro-storage/Cargo.toml` + `CHANGELOG.md` | ❌ W0 (new file) | ⬜ pending |
| SC-7 | `cargo test --all-features` + `clippy --all -Dwarnings` green | — | — | CI gate | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` | existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-storage/src/cdn/mod.rs` — `Config`, `CdnProvider`, `env_with_fallback`, `build_purge_api` + unit tests
- [ ] `ferro-storage/src/error.rs` — `CdnInvalidProvider`, `CdnFeatureRequired` variants
- [ ] `ferro-storage/CHANGELOG.md` — new file, `## [0.2.53]` entry
- [ ] Test functions SC-1…SC-5c (all new); `serial_test`/`#[serial]` only if env-var collision is real — else construct `Config` directly to bypass env reads

---

## Env-Var Test Isolation Strategy

Existing `from_env_cdn_url` (config.rs) sets/removes env vars in-function with no parallel guard. For new tests:
- Prefer constructing `Config` structs **directly** (bypass env) for all logic tests; only the explicit parity/fallback tests touch real env vars.
- Env-touching tests run serially (`--test-threads=1`) or use `#[serial]` (add `serial_test` dev-dep only if not already present).
- Each env-touching test sets the exact var and removes it in the same function (mirror the existing baseline).

---

## Manual-Only Verifications

| Behavior | SC | Why Manual | Test Instructions |
|----------|----|-----------|--------------------|
| Live DO Spaces CDN purge against a real endpoint | SC-4 | Needs real DO credentials + endpoint; CI uses mocked HTTP (the adapter's `api_base` test override) | Set real `CDN_PROVIDER=digitalocean` + creds, call `purge()`, confirm 204 from DO API |

*All config-surface behaviors have automated verification; only a live-credential purge is manual.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
