---
phase: 176
slug: json-ui-v2-runtime-patches-booking-staff-field-test
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-20
---

# Phase 176 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner via `cargo test` |
| **Config file** | none (workspace `Cargo.toml`) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60s quick / ~3–4 min full |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui` (quick)
- **After every plan wave:** Run the full suite (`fmt --check && clippy -D warnings && test --all-features`)
- **Before `/gsd-verify-work`:** Full suite must be green AND the consumer booking↔staff binding UAT must be re-run end-to-end against the patched runtime (chrome-mcp snapshot)
- **Max feedback latency:** 60 seconds for the quick run

---

## Per-Task Verification Map

> Task IDs reflect the research-recommended plan shape (176-01 = F7+F8 combined; 176-02 = F9 reproduction + audit). Planner backfills exact IDs after PLAN.md creation.

| Task ID | Plan | Wave | Finding | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|---------|-----------------|-----------|-------------------|-------------|--------|
| 176-01-01 | 01 | 1 | F7 schema | `CardProps` schema regenerates with `badge: Option<String>` property | unit | `cargo test -p ferro-json-ui card_props_schema_includes_badge` | ❌ W0 | ⬜ pending |
| 176-01-02 | 01 | 1 | F7 round-trip | `Card { badge: Some("X") }` round-trips through serde | unit | `cargo test -p ferro-json-ui card_props_round_trips_badge` | ❌ W0 | ⬜ pending |
| 176-01-03 | 01 | 1 | F7 omit-empty | `Card { badge: None }` serializes without the key | unit | `cargo test -p ferro-json-ui card_props_omits_empty_badge_in_json` | ❌ W0 | ⬜ pending |
| 176-01-04 | 01 | 1 | F7 render | `render_card` with `badge="B"` emits a Badge-styled span containing "B" | unit | `cargo test -p ferro-json-ui render_card_emits_badge_when_present` | ❌ W0 | ⬜ pending |
| 176-01-05 | 01 | 1 | F8 schema | `CardProps` schema regenerates with `subtitle: Option<String>` property | unit | `cargo test -p ferro-json-ui card_props_schema_includes_subtitle` | ❌ W0 | ⬜ pending |
| 176-01-06 | 01 | 1 | F8 round-trip | `Card { subtitle: Some("S") }` round-trips through serde | unit | `cargo test -p ferro-json-ui card_props_round_trips_subtitle` | ❌ W0 | ⬜ pending |
| 176-01-07 | 01 | 1 | F8 omit-empty | `Card { subtitle: None }` serializes without the key | unit | `cargo test -p ferro-json-ui card_props_omits_empty_subtitle_in_json` | ❌ W0 | ⬜ pending |
| 176-01-08 | 01 | 1 | F8 render | `render_card` with `subtitle="S"` emits muted-text element containing "S" beneath title | unit | `cargo test -p ferro-json-ui render_card_emits_subtitle_when_present` | ❌ W0 | ⬜ pending |
| 176-01-09 | 01 | 1 | F7+F8 ordering | `render_card` emits subtitle BEFORE description and AFTER title; badge in title row | unit | `cargo test -p ferro-json-ui render_card_emits_title_subtitle_description_badge_together` | ❌ W0 | ⬜ pending |
| 176-01-10 | 01 | 1 | F7+F8 regression | All existing `render_card_*` tests still pass unchanged | suite | `cargo test -p ferro-json-ui render::containers::card` | ✅ | ⬜ pending |
| 176-01-11 | 01 | 1 | F7+F8 docs | `docs/src/json-ui/components.md` Card section mentions `badge` and `subtitle` slots | grep | `grep -q 'badge' docs/src/json-ui/components.md && grep -q 'subtitle' docs/src/json-ui/components.md` | ❌ W0 | ⬜ pending |
| 176-02-01 | 02 | 1 | F9 reproduction | Construct minimal spec mirroring consumer chip-strip Grid with `visible: {path:"/has_staff", operator:"eq", value:true}` and `data.has_staff=true`; assert render output contains the Grid + children | unit | `cargo test -p ferro-json-ui grid_renders_when_visible_true` | ❌ W0 | ⬜ pending |
| 176-02-02 | 02 | 1 | F9 negative | Same spec with `data.has_staff=false`; assert render output omits the Grid entirely | unit | `cargo test -p ferro-json-ui grid_hidden_when_visible_false` | ❌ W0 | ⬜ pending |
| 176-02-03 | 02 | 1 | F9 audit — visibility-supporting components | Document the union of v2 components whose `visible` is honored (covered by `render_element` at `mod.rs:155-160` — applies to every Element); regression-confirm at least Card, Badge, Button, Grid, Wave | grep | `grep -nC2 'evaluate_visibility\|el.visible' ferro-json-ui/src/render/mod.rs` AND existing `visibility.rs` tests pass | ✅ | ⬜ pending |
| 176-02-04 | 02 | 1 | F9 docs | `docs/src/json-ui/components.md` Grid section clarifies `visible` semantics and explicitly states `visible` is element-level (applies to all v2 components) | grep | `grep -q 'visible' docs/src/json-ui/components.md` AND `grep -q 'element-level\|every component' docs/src/json-ui/components.md` | ❌ W0 | ⬜ pending |
| 176-02-05 | 02 | 1 | F9 root-cause note | If repro fails (visibility evaluator is correct), `176-02-PLAN.md` records the no-repro finding + consumer-side investigation pointer; if repro succeeds, plan adds the fix and additional assertions | manual | Read `176-02-PLAN.md` `<root_cause>` section after plan execution | ❌ W0 | ⬜ pending |
| 176-PHASE-01 | — | final | Phase gate | Full workspace suite passes with zero warnings | suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*File Exists column: ✅ infrastructure or test already present; ❌ W0 = needs to be added in Wave 0 of the owning plan*

---

## Wave 0 Requirements

Wave 0 of each plan creates the test scaffolding the implementation tasks will satisfy.

- [ ] `ferro-json-ui/src/component.rs` (tests module) — `card_props_schema_includes_badge`, `card_props_schema_includes_subtitle`, `card_props_round_trips_badge`, `card_props_round_trips_subtitle`, `card_props_omits_empty_badge_in_json`, `card_props_omits_empty_subtitle_in_json`
- [ ] `ferro-json-ui/src/render/containers.rs` (tests module) — `render_card_emits_badge_when_present`, `render_card_omits_badge_when_absent`, `render_card_emits_subtitle_when_present`, `render_card_omits_subtitle_when_absent`, `render_card_emits_title_subtitle_description_badge_together`, `grid_renders_when_visible_true`, `grid_hidden_when_visible_false`, `grid_visible_consumer_reproduction`
- [ ] `docs/src/json-ui/components.md` — Card section gains `badge` + `subtitle` documentation; Grid section gains `visible` clarification (element-level, applies to every v2 component)

Framework install: not required — Rust's built-in test runner is the only dependency.

---

## Manual-Only Verifications

| Behavior | Finding | Why Manual | Test Instructions |
|----------|---------|------------|-------------------|
| Consumer kanban card shows countdown badge text ("Scade tra Nm") | F7 | Requires running the gestiscilo-it consumer β UAT in a browser via chrome-mcp; the DOM assertion in Rust unit tests covers structure but not the consumer's specific styling context | Re-run `gestiscilo-it/.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` Bug R2 verification against this branch as a local-path dependency |
| Consumer kanban card shows staff-name subtitle ("Marco Rossi") beneath customer name | F8 | Same — consumer-specific layout context | Re-run Bug R3 verification |
| Consumer per-staff chip strip Grid appears when `has_staff: true`, hidden when false | F9 | Visual / consumer-context confirmation that the chip strip Grid renders correctly in the kanban dashboard | Re-run Bug R4 verification; toggle `has_staff` between true and false in test data and confirm strip appears/disappears |

---

## Validation Sign-Off

- [ ] All implementation tasks have an `<automated>` verify command OR a Wave 0 test dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all ❌ W0 references listed above
- [ ] No watch-mode flags in test commands
- [ ] Feedback latency < 60s for the quick run
- [ ] `nyquist_compliant: true` set in frontmatter before phase verification

**Approval:** pending
