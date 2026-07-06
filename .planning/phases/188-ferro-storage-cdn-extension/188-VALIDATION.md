---
phase: 188
slug: ferro-storage-cdn-extension
status: ready
nyquist_compliant: true
wave_0_complete: true
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
| 188-01-T1 | 01 | 1 | STOR-F-01 | T-188-03 | Error::cdn carries no secret | unit/compile | `cargo build -p ferro-storage --features cdn-bunny,cdn-cloudflare` | ✅ | ✅ green |
| 188-01-T2 | 01 | 1 | STOR-F-01 | T-188-01 | double-slash-safe URL composition | unit | `cargo test -p ferro-storage cdn_url` (+ `from_env_cdn_url`) | ✅ | ✅ green |
| 188-02 | 02 | 2 | STOR-F-02 | T-188-04..09 | DO adapter: batching/throttle/no-op; token redaction | integration (wiremock) | `cargo test -p ferro-storage do_adapter` (+ `purge_empty_noop`, `debug_does_not_contain_token`) | ✅ | ✅ green |
| 188-03 | 03 | 3 | STOR-F-02 (crit 4) | T-188-10..13 | Bunny/CF redacted Debug; chunking; default-graph absence | compile + full gate | `cargo test -p ferro-storage --all-features` (bunny/cf tests) + `cargo tree` absence proof | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Validation Audit 2026-06-08

| Metric | Count |
|--------|-------|
| Requirements / criteria | 2 reqs (STOR-F-01/02), 4 success criteria |
| COVERED (automated) | 4/4 criteria — all green |
| PARTIAL | 0 |
| MISSING | 0 |
| Manual-only | 1 (live DO endpoint smoke — needs real credentials) |

**Evidence:** `cargo test -p ferro-storage --all-features` green at HEAD (lib unit + integration + doc-tests, 0 failures). 26 CDN/cdn_url-related test functions cover every criterion:
- SC-1 → `cdn_url_returns_cdn_when_configured`, `cdn_url_falls_back_to_origin`, `cdn_url_no_double_slash`, `cdn_url_via_storage_facade`, `from_env_cdn_url`
- SC-2 → `do_adapter_request_shape`, `do_adapter_batches_over_50`, `do_adapter_wildcard_slot`, `do_adapter_throttle_serializes`, `do_adapter_error_on_non_204`, `do_adapter_missing_token_errors`, `purge_empty_noop`, `debug_does_not_contain_token`
- SC-3 → `do_adapter_noop_missing_id` (wiremock `.expect(0)`)
- SC-4 → `bunny_adapter_*`, `cf_adapter_*`, `cf_batch_size_chunks_correctly` (compiled+run under `--all-features`); default-graph absence proven by identical `cargo tree`
- Code-review-fix coverage: `cf_batch_size_chunks_correctly` (WR-03), `*_missing_*_errors` (WR-04), `test_register_disk_with_cdn_url`/`_none_falls_back` (IN-01)

No gaps to fill — auditor not spawned. `nyquist_compliant: true` confirmed.

---

## Success-Criterion → Validation Map (Nyquist)

| Criterion | What must be observable | Test |
|-----------|-------------------------|------|
| 1 — cdn_url + fallback | `cdn_url(path)` returns `{cdn_base}/{path}` when configured, origin `url()` when unset; `Storage::cdn_url` facade delegates to default disk | unit (configured + unset disk + Storage-facade delegation) |
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

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (wiremock, reqwest, features)
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ready
