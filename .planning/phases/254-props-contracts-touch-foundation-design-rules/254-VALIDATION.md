---
phase: 254
slug: props-contracts-touch-foundation-design-rules
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-05
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
| TBD | TBD | TBD | POS-02 props compile + schema | — | N/A | unit | `cargo test -p ferro-json-ui component::schema_smoke_tests` | ✅ module exists | ⬜ pending |
| TBD | TBD | TBD | POS-02 legacy round-trip | — | N/A | unit | `cargo test -p ferro-json-ui` (new test in component.rs) | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | POS-02 byte-identical legacy render | — | N/A | unit | `cargo test -p ferro-json-ui render` (new test) | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | POS-07 constants composition | — | N/A | unit | `cargo test -p ferro-json-ui render::classes` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | POS-07 drift guard (no inline literals) | — | N/A | unit | `cargo test -p ferro-json-ui render::classes::tests::pos_render_functions_use_constants_not_literals` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | POS-11 4×3 rule fixtures | — | lint is diagnostics-only; never executes spec content | unit | `cargo test -p ferro-json-ui design::rules::tests` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | POS-11 RULE_COMPONENTS exhaustive (SC-4) | — | N/A | unit | `cargo test -p ferro-mcp` | ✅ guard exists; entries needed | ⬜ pending |
| TBD | TBD | TBD | POS-11 patterns.md ↔ registry sync | — | N/A | unit | `cargo test -p ferro-json-ui design::` (docs drift test) | ✅ test exists; sections needed | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*(Task IDs filled by planner; the requirement→test mapping above is the contract.)*

---

## Wave 0 Requirements

- [ ] Backward-compat round-trip test for `ProductTileProps` (new test in `component.rs`)
- [ ] Byte-identical render output test for legacy ProductTile spec (new test in render tests)
- [ ] `POS_*` constants composition tests in `classes.rs` tests module
- [ ] `pos_render_functions_use_constants_not_literals` drift-guard test in `classes.rs` tests module
- [ ] 12 new fixtures in `design/rules.rs` tests module (4 rules × 3 fixtures each)
- [ ] 4 new rule sections in `docs/src/design-system/patterns.md` (existing drift test enforces)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ferro-base.css regen diff sanity | POS-07 (D-08) | Generated-artifact diff review | Run `scripts/gen-ferro-base-css.sh`, inspect diff: only additive utilities from new constants |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-05 (plan-checker Dimension 8 PASS; all Wave 0 gaps mapped to tasks)
