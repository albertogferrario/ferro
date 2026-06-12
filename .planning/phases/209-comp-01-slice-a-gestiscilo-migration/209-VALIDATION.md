---
phase: 209
slug: comp-01-slice-a-gestiscilo-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-12
---

# Phase 209 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Cross-repo phase: ferro-side automated tests (intent derivation) live in this repo;
> gestiscilo-side equivalence evidence (screenshots + functional checklists) lives in
> this ferro phase directory per CONTEXT D-08. The executable migration code lives in
> gestiscilo history.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` (ferro intent assertions) + manual checklist markdown (gestiscilo equivalence) |
| **Config file** | none — ferro tests in `ferro-projections/tests/` (Phase 207 catalog baseline) |
| **Quick run command** | `cargo test -p ferro-projections gestiscilo_slice_a` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30 seconds (ferro intent tests); screenshot capture is manual per gestiscilo merge |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections gestiscilo_slice_a` (per-entity intent assertion)
- **After every plan wave:** Run `cargo test --all-features`
- **Per gestiscilo entity merge:** Chrome DevTools MCP before/after capture + functional checklist review
- **Before `/gsd-verify-work`:** Full suite green; all three equivalence records filed; weakness note non-empty
- **Max feedback latency:** ~30 seconds (ferro tests)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 209-01-xx | 01 | 1 | COMP-01 | — | N/A (read-only validation, no auth surface) | unit | `cargo test -p ferro-projections staff_browse_intent` | ❌ W0 | ⬜ pending |
| 209-02-xx | 02 | 2 | COMP-01 | — | N/A | unit | `cargo test -p ferro-projections orders_process_intent` | ❌ W0 | ⬜ pending |
| 209-03-xx | 03 | 3 | COMP-01 | — | N/A | unit | `cargo test -p ferro-projections stats_summarize_intent` | ❌ W0 | ⬜ pending |
| 209-xx-xx | — | final | COMP-01 | — | N/A | manual | Human review: weakness note names ≥1 abstraction gap | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Plan/task IDs are placeholders pending the planner's wave assignment. Each of the three
entity migrations is its own sequential gestiscilo merge (D-03); waves here reflect that
strict ordering, not parallelism.*

---

## Per-Entity Functional Checklist (equivalence record contract)

Each migration's equivalence record (`EQUIV-{entity}.md` in this phase directory) MUST assert:

1. **Data fields shown** — every data column in the before screenshot appears in the after screenshot (field names may differ; data values must match).
2. **Actions available** — every row/page action reachable before (View, Edit, Delete, status transitions) is reachable after.
3. **Primary-use-case flow** — the most common operator action works in the migrated view.
4. **Intent confirmation** — `derive_intents(&service)[0].intent == Expected` passes in a `#[test]`.
5. **Intentional visual deltas documented** — any layout/markup difference is listed explicitly; unlisted differences block the merge (D-02: functional parity, not pixel-identity).

---

## Wave 0 Requirements

- [ ] `ferro-projections/tests/` — three canonical gestiscilo ServiceDef fixtures (staff/order/stats) with `derive_intents()` assertions. Live in `catalog.rs` as a `gestiscilo_slice_a` sub-module OR a new `tests/gestiscilo.rs` (planner decides — open question 3).
- [ ] gestiscilo `Cargo.toml` — enable `projections` feature: `ferro = { ..., features = ["json-ui", "theme", "projections"] }` (one-line; activates already-published 0.2.54 code, no ferro bump).
- [ ] Three equivalence record stubs in this phase directory: `EQUIV-staff-browse.md`, `EQUIV-orders-process.md`, `EQUIV-stats-summarize.md`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Before/after render equivalence (per entity) | COMP-01 / SC#2 | Visual parity is a human judgement against the live before view; projection markup differs by design | Chrome DevTools MCP: capture before (current `render_file` view) and after (projection-driven view) for the same record; fill the functional checklist in `EQUIV-{entity}.md` |
| "What the migration revealed" weakness note non-empty | COMP-01 / SC#5 | Naming a real abstraction gap requires human judgement; an empty note fails the phase | Human review confirms ≥1 named gap with evidence (e.g. Gap 1 SVG chart, Gap 2 signed URL, Gap 3 kanban columns) |
| One-per-merge branch discipline | COMP-01 / SC#1,#3 | Git history / merge cadence, not a code property | Confirm each entity merged to gestiscilo master before the next branch opened; no gestiscilo branch alive > 2 weeks; no ferro master API change while a migration branch open |

---

## Validation Sign-Off

- [ ] All ferro tasks have automated `derive_intents()` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (manual gestiscilo equivalence steps are paired with the per-entity Rust intent test)
- [ ] Wave 0 covers all MISSING references (intent fixtures, Cargo feature flag, equivalence stubs)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (ferro intent tests)
- [ ] `nyquist_compliant: true` set in frontmatter after planner wires automated verifies into tasks

**Approval:** pending
