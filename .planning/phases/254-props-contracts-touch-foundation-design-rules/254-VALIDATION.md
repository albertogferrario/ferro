---
phase: 254
slug: props-contracts-touch-foundation-design-rules
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-05
audited: 2026-07-05
---

# Phase 254 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none — workspace Cargo.toml |
| **Quick run command** | `cargo test -p ferro-json-ui render::classes design::` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~30s; full suite several minutes (check disk space first — recurrent ENOSPC on full gate) |

---

## Sampling Rate

- **After every task commit:** Run the targeted quick command for the touched module (`cargo test -p ferro-json-ui <module path>`)
- **After every plan wave:** Run `cargo test -p ferro-json-ui && cargo test -p ferro-mcp`
- **Before `/gsd-verify-work`:** Full suite (`cargo test --all-features`) must be green
- **Max feedback latency:** ~120 seconds (targeted package tests)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 254-01-T1/T2 | 01 | 1 | POS-02 props compile + schema | T-254-01 | serde strict decode | unit | `cargo test -p ferro-json-ui component::schema_smoke_tests` (5 new POS smoke tests, component.rs:1973-1993) | ✅ | ✅ green |
| 254-01-T1 | 01 | 1 | POS-02 legacy round-trip | — | N/A | unit | `product_tile_legacy_json_round_trips_unchanged` (component.rs:2413) + `grid_props_row_weights_round_trips` (:2469) | ✅ | ✅ green |
| 254-02-T2 | 02 | 2 | POS-02 byte-identical legacy render + attribute contract | T-254-03 | html_escape on attribute emission | unit | `product_tile_legacy_render_is_byte_identical` (atoms.rs:2564) + `product_tile_escapes_categories` (:2611) + `product_tile_normalizes_spaces_in_category_names` (:2599) | ✅ | ✅ green |
| 254-02-T1 | 02 | 2 | POS-07 constants composition | T-254-04 | full-literal classes | unit | `pos_constants_are_full_literals_and_token_compliant` (classes.rs:111) | ✅ | ✅ green |
| 254-02-T2 | 02 | 2 | POS-07 drift guard (no inline literals) | T-254-04 | scanner-visible literals | unit | `pos_render_functions_use_constants_not_literals` (classes.rs:83, read_dir scan — auto-covers Phase 256 files) | ✅ | ✅ green |
| 254-03-T2 | 03 | 1 | POS-11 4×3 rule fixtures | T-254-06 | lint diagnostics-only | unit | 13 fixture tests (design/rules.rs:1341-1608; 12 planned + `pos_grid_fill_data_bound_fill_no_misfire` from WR-02) | ✅ | ✅ green |
| 254-03-T1 | 03 | 1 | POS-11 RULE_COMPONENTS exhaustive (SC-4) | — | N/A | unit | `design_system_component_guidance_drift_guarded` (ferro-mcp json_ui_catalog.rs:729) — re-run green post-WR-02 fix | ✅ | ✅ green |
| 254-03-T1 | 03 | 1 | POS-11 patterns.md ↔ registry sync | — | N/A | unit | `patterns_md_matches_rule_registry` (design/mod.rs:325) | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Backward-compat round-trip test for `ProductTileProps` (`product_tile_legacy_json_round_trips_unchanged`)
- [x] Byte-identical render output test for legacy ProductTile spec (`product_tile_legacy_render_is_byte_identical`)
- [x] `POS_*` constants composition tests in `classes.rs` tests module
- [x] `pos_render_functions_use_constants_not_literals` drift-guard test in `classes.rs` tests module
- [x] 12 new fixtures in `design/rules.rs` tests module (13 shipped: 4 rules × 3 fixtures + WR-02 regression fixture)
- [x] 4 new rule sections in `docs/src/design-system/patterns.md` (drift test enforces)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions | Result |
|----------|-------------|------------|-------------------|--------|
| ferro-base.css regen diff sanity | POS-07 (D-08) | Generated-artifact diff review | Run `scripts/gen-ferro-base-css.sh`, inspect diff: only additive utilities from new constants | ✅ verified 2026-07-05 — regen run in 254-02 T3 (commit 08166f54); code review independently confirmed all nine new utilities present exactly once, no anomalies |

---

## Validation Audit 2026-07-05

| Metric | Count |
|--------|-------|
| Requirements audited | 8 |
| COVERED | 8 |
| PARTIAL | 0 |
| MISSING (gaps found) | 0 |
| Resolved | 0 (none needed) |
| Escalated | 0 |

Evidence: `cargo test -p ferro-json-ui` 746 passed / 0 failed on post-fix HEAD; `design_system_component_guidance_drift_guarded` re-run green post-WR-02; full `cargo test --all-features` green in plan 254-02 Task 3 CI-exact gate. Every mapped test verified present by exact name and line number.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-05 (post-execution audit: 8/8 COVERED, zero gaps)
