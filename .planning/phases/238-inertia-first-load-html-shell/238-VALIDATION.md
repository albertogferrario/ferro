---
phase: 238
slug: inertia-first-load-html-shell
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-21
---

# Phase 238 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (+ `tokio::test` only if async needed) |
| **Config file** | none — Cargo workspace test harness; `tempfile` already in `ferro-inertia` dev-deps |
| **Quick run command** | `cargo test -p ferro-inertia` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30–60 seconds (quick `-p ferro-inertia`); full suite minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-inertia`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| from_env constructor | config | 1 | D-03 | — | env-only, no hardcoded tenant identity | unit | `cargo test -p ferro-inertia -- from_env_reads` | ❌ W0 | ⬜ pending |
| InertiaConfig new fields (title/head_extras/mount_id) | config | 1 | D-05 | — | head_extras is raw HTML — document trust boundary | unit | `cargo test -p ferro-inertia -- mount_id_applied title_override` | ❌ W0 | ⬜ pending |
| to_html_response template extension | response | 2 | D-05/D-06/D-07 | T-238-01 | data-page JSON stays HTML-attr-escaped | unit | `cargo test -p ferro-inertia -- head_extras_in_html mount_id_applied` | ❌ W0 | ⬜ pending |
| content negotiation HTML vs JSON | response | 2 | D-01 | — | same handler, header-driven branch | unit | `cargo test -p ferro-inertia -- non_inertia_request_returns_html_document inertia_request_returns_json_contract` | ❌ W0 | ⬜ pending |
| data-page == JSON page object equality | response | 2 | D-12 (SC-1) | — | identical page object both paths | unit | `cargo test -p ferro-inertia -- html_data_page_equals_json_contract` | ❌ W0 | ⬜ pending |
| dev-mode vite client tags | response | 2 | D-01 (SC-3) | — | dev URL from config | unit | `cargo test -p ferro-inertia -- dev_mode_emits_vite_client_script` | ❌ W0 | ⬜ pending |
| prod-mode manifest resolution | manifest | 2 | D-08 (SC-3) | — | hashed paths from manifest.json | unit | `cargo test -p ferro-inertia -- parse_manifest_and_resolve_entry` | ✅ exists | ⬜ pending |
| App::set_inertia_config + global + call sites | global | 3 | D-02/D-04 | — | async-safe OnceLock, default fallback | unit | `cargo test -p framework --features inertia -- inertia_config` | ❌ W0 | ⬜ pending |
| docs same-origin + proxy + drift fix | docs | 3 | D-10/D-11 (SC-5) | — | n/a | manual | `cargo doc --no-deps` (no broken intra-doc links) | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-inertia/src/response.rs` — add `#[cfg(test)] mod content_negotiation_tests` (no existing test covers `to_html_response()`); use `development: true` for HTML-structure assertions to avoid the manifest `OnceLock` global.
- [ ] `ferro-inertia/src/config.rs` — add `#[cfg(test)]` tests for `InertiaConfig::from_env()`.
- [ ] `framework/` — add a test for `App::set_inertia_config` / `get_inertia_config` default-fallback behavior, matching the existing framework global-config test style.
- [ ] Manifest tests reuse the existing `ViteManifest::resolve()` direct pattern (`manifest.rs:82`) — do NOT call `resolve_assets()` in tests (global cache bleeds across tests).

*Framework already present (Rust `#[test]` + `tempfile`); no installer task needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Same-origin / Vite `server.proxy` recipe forwards session cookie | D-10 (SC-5) | Requires a live split-port dev setup (Vite + Ferro) and a browser; not unit-testable in-crate | Follow the new docs recipe in `docs/src/features/inertia.md`; load the app cold, confirm the session cookie flows across the proxy and the page hydrates from `data-page`. |
| Docs render without broken links | D-11 | mdBook render is a build-time/manual check | `cargo doc --no-deps` clean; mdBook build of `docs/` succeeds |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
