---
phase: 188
slug: ferro-storage-cdn-extension
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 188 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (built-in) + `wiremock` for HTTP doubles |
| **Config file** | `[dev-dependencies]` in `ferro-storage/Cargo.toml` (add `wiremock`) |
| **Quick run command** | `cargo test -p ferro-storage` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~10-30s for the crate (wiremock spins a local server; no network) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-storage` (add `--all-features` when touching Bunny/CF adapters)
- **After every plan wave:** `cargo clippy -p ferro-storage --all-targets -- -D warnings && cargo test -p ferro-storage`
- **Before `/gsd-verify-work`:** full CI-parity `--all-features` suite green (compiles Bunny/CF feature code)
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

> Filled by the planner. Each task maps to a `cargo test` target or grep-verifiable acceptance criterion.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 188-01-01 | 01 | 1 | STOR-F-01 | — | N/A | unit | `cargo test -p ferro-storage cdn_url` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Success-Criterion → Validation Map (Nyquist)

| Criterion | What must be observable | Test |
|-----------|-------------------------|------|
| 1 — cdn_url + fallback | `cdn_url(path)` returns `{cdn_base}/{path}` when configured, origin `url()` when unset | unit (configured + unset disk) |
| 2 — PurgeApi + DO adapter | DO adapter sends `DELETE /v2/cdn/endpoints/{id}/cache` body `{"files":[...]}` Bearer auth; batches >50 into N requests; wildcard = 1 slot; throttle serializes ≥6 rapid calls under 5/10s | wiremock: assert request shape/count/batching; timing assertion for throttle |
| 3 — missing-id no-op | DO adapter with no `DO_SPACES_CDN_ID` → `purge()` logs + returns `Ok(())`, makes NO HTTP call | unit (wiremock receives zero requests) |
| 4 — Bunny/CF feature-gated | `cdn-bunny`/`cdn-cloudflare` adapters compile under `--all-features`; absent from default `cargo tree` | `cargo build --features cdn-bunny,cdn-cloudflare` + `cargo tree` default has no bunny/cf module symbols |

---

## Wave 0 Requirements

- [ ] `ferro-storage/Cargo.toml` `[dev-dependencies]` add `wiremock`; deps add `reqwest` (lean rustls) + `[features] cdn-bunny`, `cdn-cloudflare`; tokio `"time"` feature
- [ ] Confirm `cargo tree -p ferro-storage` introduces no NEW `*-sys` C-binding crate (reqwest rustls reuses existing `ring`)

*Existing ferro-storage test infrastructure (tokio test-util, tempfile) covers the rest.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real DO Spaces CDN purge against a live endpoint | STOR-F-02 | Needs live DO credentials + a real CDN endpoint id | Operator runs against staging with `DO_SPACES_CDN_ID` + `DIGITALOCEAN_ACCESS_TOKEN` set; out of automated scope |

*All in-crate behaviors have automated `wiremock`-backed coverage; only live-endpoint smoke is manual.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (wiremock, reqwest, features)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
