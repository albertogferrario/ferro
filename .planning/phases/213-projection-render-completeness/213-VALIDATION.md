---
phase: 213
slug: projection-render-completeness
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-12
---

# Phase 213 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. All changes are in
> `ferro-json-ui` (renderer crate); `ferro-projections` types are read-only. Two test layers:
> fast per-gap render unit tests (assert the emitted `Spec` carries the bound component), and
> end-to-end re-verification against the two unmerged gestiscilo probe branches.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` in `ferro-json-ui/src/projection/builder.rs` `mod tests` (+ atoms/data render tests for C/D) |
| **Config file** | none — `cargo test` |
| **Quick run command** | `cargo test -p ferro-json-ui --lib projection::builder` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~20s quick (one crate); full suite minutes |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-json-ui --lib projection::builder` (one CPU op — serialize; do not chain cargo runs)
- **After every wave:** `cargo fmt --all -- --check` then `cargo clippy --all --all-targets -- -D warnings` then `cargo test --all-features` — each a separate serialized invocation
- **Per gap (integration):** rebuild ferro, re-run the matching gestiscilo probe branch against the rebuilt binary via the Phase 209 dev-server + Chrome MCP harness
- **Phase gate:** full suite green + both probe branches re-verified before `/gsd-verify-work`
- **Max feedback latency:** ~20s (builder unit tests)

---

## Per-Task Verification Map

| Task | Gap | Requirement | Test Type | Automated Command | File Exists | Status |
|------|-----|-------------|-----------|-------------------|-------------|--------|
| 213-01 T2 | B | Gap B (actions) | unit | `cargo test -p ferro-json-ui --lib actions_slot_emits_dropdown_from_service_actions` | ❌ W0 | ⬜ |
| 213-01 T2 | B | Gap B (row actions) | unit | `cargo test -p ferro-json-ui --lib datatable_root_has_row_actions_from_service_actions` | ❌ W0 | ⬜ |
| 213-02 T1 | A | Gap A (kanban cols) | unit | `cargo test -p ferro-json-ui --lib kanban_root_derives_columns_from_state_machine` | ❌ W0 | ⬜ |
| 213-02 T1 | A | Gap A (fallback) | unit | `cargo test -p ferro-json-ui --lib kanban_root_fallback_when_no_state_machine` | ❌ W0 | ⬜ |
| 213-03 T2 | C | Gap C (statcard bind) | unit | `cargo test -p ferro-json-ui --lib statcard_root_binds_primary_stat_field` | ❌ W0 | ⬜ |
| 213-03 T2 | C | Gap C (empty) | unit | `cargo test -p ferro-json-ui --lib statcard_root_empty_when_no_stat_field` | ❌ W0 | ⬜ |
| 213-04 T1 | D | Gap D (image col) | unit | `cargo test -p ferro-json-ui --lib datatable_root_includes_image_url_column` | ❌ W0 | ⬜ |
| 213-04 T1 | D | Gap D (image format) | unit | `cargo test -p ferro-json-ui --lib image_column_has_image_format` | ❌ W0 | ⬜ |
| 213-05 T3 | A/B | integration (Orders kanban + actions) | manual | gestiscilo feat/207 re-verify: `/dashboard/cassa/ordini` shows 4 columns + actions | ❌ W0 | ⬜ |
| 213-05 T4 | B/D | integration (Staff actions + avatar) | manual | gestiscilo feat/208 re-verify: `/dashboard/staff` shows row actions + avatar image | ❌ W0 | ⬜ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Exact task IDs assigned at plan time (gaps sequenced B→A→C→D→E).*

---

## Wave 0 Requirements

- [ ] `service_with_state_machine()` fixture in `builder.rs` tests (Gap A).
- [ ] `service_with_actions()` fixture in `builder.rs` tests (Gap B).
- [ ] `service_with_money_field()` fixture (extend existing `sample_service()`) (Gap C).
- [ ] No new test files — all unit tests in the existing `mod tests` blocks of `builder.rs` (+ `atoms.rs`/`data.rs` for the C/D render branches).

---

## Manual-Only Verifications

| Behavior | Gap | Why Manual | Test Instructions |
|----------|-----|------------|-------------------|
| Orders kanban renders real columns + cards | A | Visual + tenant data; against the live migrated handler | feat/207, `/dashboard/cassa/ordini`: 4 status columns (Confermati/In corso/Rientrato/Chiuso); insert orders at different states; cards group correctly |
| Row + page actions render | B | Visual; operator affordances | feat/208, `/dashboard/staff`: row dropdown (View/Edit/Toggle/Delete) + page "Nuovo" CTA |
| StatCard shows real values | C | Visual; depends on handler stat data | gestiscilo Statistics (if migrated): stat cards show non-zero revenue/count |
| Avatar image column renders | D | Visual | feat/208, `/dashboard/staff`: `avatar_url` column renders as image/avatar, not empty |

Harness: `ferro serve --backend-only` (port 8080), magic-link dev auto-login (`jetskiadriatic@gestiscilo.it`, tenant id 3), Chrome DevTools MCP `chrome-devtools-3`. Probe branches are the test bed — NOT to be merged or modified.

---

## Validation Sign-Off

- [ ] Every gap has a render unit test asserting the emitted Spec carries the bound component
- [ ] Sampling continuity: no 3 consecutive tasks without an automated test
- [ ] Wave 0 fixtures added
- [ ] No watch-mode flags; cargo runs serialized (thermal)
- [ ] `Catalog::validate` stays passing; `statcard_metadata_is_orphan_element` + frozen `derive_intents` catalog invariants stay green
- [ ] Both gestiscilo probe branches re-verified end-to-end
- [ ] `nyquist_compliant: true` set after the planner wires the automated commands into tasks

**Approval:** planned 2026-06-12 — automated commands wired into Plans 01-04 task verifies; integration checkpoints in Plan 05; fixtures land in Plan 01 Task 1.
