---
phase: 151
slug: ferro-wallet-crate
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-11
---

# Phase 151 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Source of truth: `151-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` + `#[test]` attribute (no external runner) |
| **Config file** | None — workspace `Cargo.toml` `[workspace]` section drives discovery |
| **Quick run command** | `cargo test -p ferro-wallet --lib` (unit tests only, ~5s expected) |
| **Per-test-file command** | `cargo test -p ferro-wallet --test apple_integration` / `--test google_jwt` |
| **Full suite command** | `cargo test --all-features` (workspace-wide, matches CI) |
| **Estimated runtime** | ~5s unit; ~10–15s with integration tests; ~30–60s full workspace suite |

---

## Sampling Rate

- **After every task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-wallet` (per CLAUDE.md gate; ferro-wallet-scoped for speed)
- **After every plan wave:** `cargo test --all-features` (full workspace suite)
- **Before `/gsd-verify-work`:** Full suite must be green AND `cargo doc --no-deps -p ferro-wallet` clean
- **Max feedback latency:** ~5s per-task, ~60s per-wave

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 151-03-* | 03 | 1 | ACC-1a | — | Missing Apple env vars do not panic or error in `from_env` | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_apple_missing_is_none` | ❌ W0 | ⬜ pending |
| 151-03-* | 03 | 1 | ACC-1b | — | Missing Google env vars do not panic or error in `from_env` | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_google_missing_is_none` | ❌ W0 | ⬜ pending |
| 151-03-* | 03 | 1 | ACC-1c | — | APP_NAME / APP_URL defaults match `framework::config::AppConfig` | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_defaults_match_appconfig` | ❌ W0 | ⬜ pending |
| 151-02-* | 02 | 1 | ACC-1e | — | `RgbColor::from_hex` parses `#RRGGBB` deterministically | unit | `cargo test -p ferro-wallet --lib subject::tests::rgb_from_hex` | ❌ W0 | ⬜ pending |
| 151-02-* | 02 | 1 | ACC-1f | T-151-Apple-COLOR | BT.601 luminance threshold derives white for dark backgrounds | unit | `cargo test -p ferro-wallet --lib subject::tests::auto_foreground_dark_bg_is_white` | ❌ W0 | ⬜ pending |
| 151-04-* | 04 | 1 | ACC-1g | — | `fit_to` produces exact target dimensions with transparent padding | unit | `cargo test -p ferro-wallet --lib images::tests::fit_to_exact_dims_transparent` | ❌ W0 | ⬜ pending |
| 151-04-* | 04 | 1 | ACC-1h | — | `qr::png` returns valid PNG bytes (magic-byte check) | unit | `cargo test -p ferro-wallet --lib qr::tests::png_starts_with_png_magic` | ❌ W0 | ⬜ pending |
| 151-05-* | 05 | 2 | ACC-1d | T-151-Apple-MANIFEST | `build_manifest` produces lowercase hex SHA1 per file | unit | `cargo test -p ferro-wallet --lib apple::manifest::tests::manifest_sha1_lowercase_hex` | ❌ W0 | ⬜ pending |
| 151-06-* | 06 | 3 | ACC-1j | T-151-Apple-SIGN | `.pkpass` ZIP contains 9 expected files; `pass.json` carries correct identifiers, barcode message, primary field value (self-signed cert roundtrip) | integration | `cargo test -p ferro-wallet --test apple_integration` | ❌ W0 | ⬜ pending |
| 151-07-* | 07 | 2 | ACC-1i | — | `save_url(jwt)` returns `https://pay.google.com/gp/v/save/{jwt}` | unit | `cargo test -p ferro-wallet --lib google::jwt::tests::save_url_format` | ❌ W0 | ⬜ pending |
| 151-08-* | 08 | 3 | ACC-1k | T-151-Google-JWT | RS256 JWT decodes with public key; claims `iss/aud=google/typ=savetowallet` match; payload contains exactly one `eventTicketObjects` entry with expected `id` and `barcode.value` | integration | `cargo test -p ferro-wallet --test google_jwt` | ❌ W0 | ⬜ pending |
| 151-*-* | all | all | ACC-2 | — | `cargo build --workspace` green with new crate registered | build | `cargo build --workspace` | ✅ | ⬜ pending |
| 151-*-* | all | all | ACC-3 | — | `cargo doc --no-deps -p ferro-wallet` clean (no warnings) | build | `cargo doc --no-deps -p ferro-wallet` | ✅ | ⬜ pending |
| 151-09-* | 09 | 4 | ACC-4 | — | `ferro-wallet` published to crates.io after workspace version bump (first-publish requires manual bootstrap per memory) | release | GH Actions `publish.yml` Wave 1a → `cargo publish -p ferro-wallet` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

All test files are net-new (greenfield crate) — Wave 0 work is "create the test file alongside its production code in the same task":

- [ ] `ferro-wallet/src/config.rs` `#[cfg(test)] mod tests {}` block covering ACC-1a / ACC-1b / ACC-1c (created in PLAN-03)
- [ ] `ferro-wallet/src/subject.rs` `#[cfg(test)] mod tests {}` block covering ACC-1e / ACC-1f (created in PLAN-02)
- [ ] `ferro-wallet/src/apple/manifest.rs` `#[cfg(test)] mod tests {}` block covering ACC-1d (created in PLAN-05)
- [ ] `ferro-wallet/src/images.rs` `#[cfg(test)] mod tests {}` block covering ACC-1g (created in PLAN-04)
- [ ] `ferro-wallet/src/qr.rs` `#[cfg(test)] mod tests {}` block covering ACC-1h (created in PLAN-04)
- [ ] `ferro-wallet/src/google/jwt.rs` `#[cfg(test)] mod tests {}` block covering ACC-1i (created in PLAN-07)
- [ ] `ferro-wallet/tests/apple_integration.rs` end-to-end (mints self-signed X.509 at runtime via openssl) covering ACC-1j (created in PLAN-06)
- [ ] `ferro-wallet/tests/google_jwt.rs` RS256 roundtrip (mints RSA keypair at runtime via jsonwebtoken) covering ACC-1k (created in PLAN-08)

No external framework install needed — Rust's built-in test runner ships with the toolchain. No additional `[dev-dependencies]` beyond what the implementation already pulls in (`openssl` for the Apple integration cert mint, `jsonwebtoken` for the Google JWT decode).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-publish to crates.io | ACC-4 (bootstrap) | CI publish token has `publish-update` only, not `publish-new` (per global memory `project_ferro_publish_token_scoping.md`). The very first `cargo publish -p ferro-wallet` must be run from a local terminal with a personal token. Subsequent versions auto-publish via CI. | After version bump merges to master: `cargo publish -p ferro-wallet --token <PERSONAL_PUBLISH_TOKEN>`. Verify it lands on crates.io. After that, future workspace version bumps publish via GH Actions. |
| Actual `.pkpass` validates on iPhone | (out of scope) | Self-signed cert in `tests/apple_integration.rs` is well-formed for openssl but ill-formed for Apple Wallet (no real WWDR chain). Real-device validation requires real Apple Developer cert + WWDR intermediate. | Out of scope for v1 — gated by downstream gestiscilo-it integration. |

---

## Validation Sign-Off

- [ ] All tasks have automated `cargo test` verify or are flagged Wave 0
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (every plan creates at least one test alongside production code)
- [ ] Wave 0 covers all MISSING references (all 8 test sites listed above)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (per-task < 5s; per-wave < 60s)
- [ ] `nyquist_compliant: true` set in frontmatter after phase verification

**Approval:** pending
