---
phase: 255
slug: pos-runtime-modules-double-submit-protection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-05
---

# Phase 255 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detail source: `255-RESEARCH.md` §Validation Architecture (SC-0..SC-4 mapping).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s / full ~10min |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui` (serialize — never parallel with another cargo op)
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full CI-exact gate green (`cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` + docs build)
- **Max feedback latency:** ~600 seconds (full gate)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC-0 rename cascade | TBD | 1 | ROADMAP SC-0 | — | N/A | grep gate | `grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` → zero hits | ✅ | ⬜ pending |
| SC-0 count unchanged | TBD | 1 | ROADMAP SC-0 | — | N/A | unit | `cargo test -p ferro-json-ui builtin_specs_names_match_dispatch` (count 47) | ✅ | ⬜ pending |
| SC-1 bundle presence | TBD | 2 | POS-08 | — | N/A | unit | `cargo test -p ferro-json-ui bundle_contains_all_setup_functions` (setupNumpad, setupFilters, setupTiles) | ✅ | ⬜ pending |
| SC-2 dispatcher | TBD | 2 | POS-08 | — | no-op when elements absent | unit | `cargo test -p ferro-json-ui dispatcher_invokes_every_setup` | ✅ | ⬜ pending |
| SC-3 numpad/filter source | TBD | 2 | POS-08 | — | client-side only, no round-trip | unit (inline-source) | `cargo test -p ferro-json-ui runtime::` contains-assertions (data-numpad-key, data-filter-tokens, data-filter-search, input-event dispatch) | ❌ W0 | ⬜ pending |
| SC-3 tile HTML attrs | TBD | 2 | POS-08 | — | HTML-escaped attribute values | unit (HTML assertion) | `cargo test -p ferro-json-ui render::atoms` (data-filter-text present + escaped) | ✅ extend | ⬜ pending |
| SC-4 disable-on-submit | TBD | 2 | POS-08 | T: double-submit | button disabled after first submit; bfcache re-enable | unit (source + HTML) | `cargo test -p ferro-json-ui` (data-disable-on-submit in bundle + render_button emission tests) | ❌ W0 | ⬜ pending |
| SC-4 idempotency docs | TBD | 2 | POS-08 | T: replayed POST | dedupe on (tenant_id, idempotency_key) documented | docs build | `cargo doc --no-deps` clean + mdBook section present (`grep -n "Double-submit" docs/src/features/write-kernel.md`) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — no new framework or
fixtures needed. New tests are added inside existing test modules
(`runtime/mod.rs` tests, `render/atoms.rs` tests) following the established
contains-assertion / HTML-assertion patterns.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Numpad tactile feel on tablet (press states, no keyboard popup) | POS-08 (adjacent) | Real-device behavior not testable in unit scope | Defer to Phase 256/258 visual UAT on /cassa |
| bfcache re-enable on iPad Safari back-nav | POS-08 | Browser bfcache behavior | Defer to consumer adoption UAT; unit test asserts pageshow handler presence in source |
