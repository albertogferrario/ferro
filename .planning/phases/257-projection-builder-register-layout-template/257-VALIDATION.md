---
phase: 257
slug: projection-builder-register-layout-template
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-06
---

# Phase 257 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-json-ui --features projections` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s · full ~10min |

---

## Sampling Rate

- **After every task commit:** Run the task's targeted `cargo test` filter (below).
- **After every plan wave:** Run `cargo test --all-features`.
- **Before `/gsd-verify-work`:** Full suite green + CI-exact gate (fmt --check, clippy --all-targets --all-features -D warnings, cargo doc).
- **Max feedback latency:** ~120 seconds (serialize CPU-heavy runs — one at a time).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| P01-T1 ElementBuilder.each (D-12) | 257-01 | 1 | POS-10 / SC-4 | T-257-02 | pure builder, no I/O | unit | `cargo test -p ferro-json-ui each_builder_round_trip` | ⬜ Wave 0 | ⬜ pending |
| P01-T2 SpecBuilder.fill_viewport (D-13) | 257-01 | 1 | POS-10 / SC-3 | T-257-02 | pure builder, no I/O | unit | `cargo test -p ferro-json-ui fill_viewport_builder` | ⬜ Wave 0 | ⬜ pending |
| P01-T3 catalog $each guard (D-14) | 257-01 | 1 | POS-10 / SC-4 | T-257-01 | validation skip scoped to each.is_some(); non-template validation intact | unit | `cargo test -p ferro-json-ui catalog_each_template` | ⬜ Wave 0 | ⬜ pending |
| P02-T1 register_template() (D-01/D-02) | 257-02 | 2 | POS-10 / SC-1 | — | — | unit | `cargo test -p ferro-json-ui --features projections register_template` | ⬜ Wave 0 | ⬜ pending |
| P02-T2 emit_register_root + arm (D-04/D-06/D-08/D-09) | 257-02 | 2 | POS-10 / SC-1 | T-257-03, T-257-04 | meaning-only Tile mapping; $data bindings (no raw interpolation) | build+integration | `cargo test -p ferro-json-ui --features projections register_projection` | ⬜ Wave 0 | ⬜ pending |
| P02-T3 register projection tests (SC-1/D-05) | 257-02 | 2 | POS-10 / SC-1 | T-257-03 | catalog-valid + lint-clean harness | integration | `cargo test -p ferro-json-ui --features projections register_projection` | ⬜ Wave 0 | ⬜ pending |
| P03-T1 /cassa flip + deletions (D-15/D-16) | 257-03 | 3 | POS-10 / SC-2 | T-257-06, T-257-07 | no RawHtml/render_file; rimuovi surface removed | build+grep | `cargo build -p app` + RawHtml/render_file/rimuovi grep empty | ⬜ Wave 0 | ⬜ pending |
| P03-T2 cassa render test (SC-2/SC-3) | 257-03 | 3 | POS-10 / SC-2, SC-3 | T-257-07 | 200 + ferro-fill class chain + register markers | integration | `cargo test -p app cassa_render` | ⬜ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — `ferro-json-ui` has established
unit/integration test modules (projection builder tests with the injected-catalog
pattern `from_service_def_with_catalog(.., &Catalog::build_builtins_only())`, spec builder
tests, catalog validate tests, design lint fixtures) and the `app` crate has a `tests/`
module (`app/src/tests/mod.rs`). No new framework install needed. Each plan creates its
own test functions in-place (Wave 0 = the test bodies listed above ship with their tasks).

New test files/functions created by this phase:
- `ferro-json-ui/src/spec.rs` — `each_builder_round_trip`, `fill_viewport_builder` (P01)
- `ferro-json-ui/src/catalog.rs` — `catalog_each_template_null_data`, `catalog_each_template_populated_data` (P01)
- `ferro-json-ui/src/projection/intent_layout.rs` — `register_template_overrides_collect` (P02)
- `ferro-json-ui/src/projection/builder.rs` — `register_projection_is_catalog_valid`, `register_projection_is_lint_clean`, `register_projection_no_actions_errors`, `register_projection_populated_data_validates` (P02)
- `app/src/tests/cassa_render.rs` — `cassa_render_is_projection_derived_fill_viewport` (P03)

All projection tests MUST use `from_service_def_with_catalog` with an injected
`Catalog::build_builtins_only()` (Pitfall 5 — OnceLock isolation).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `/cassa` visual quality on a tablet viewport (fill-viewport panes scroll independently, SelectionPanel updates + running total on tile tap, register feel) | POS-10 | Visual interaction quality is not assertable via HTML string tests | Run the app, open `/cassa` in Chrome DevTools MCP at tablet viewport, tap tiles, verify the panel updates with a running total + no page scroll |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (test bodies ship with their tasks)
- [x] No watch-mode flags
- [x] Feedback latency < 600s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned
