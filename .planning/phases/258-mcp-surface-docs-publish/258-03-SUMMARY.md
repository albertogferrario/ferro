---
phase: 258-mcp-surface-docs-publish
plan: 03
subsystem: infra
tags: [crates-io, publish, ferro-rs, ferro-payments, pos-component-suite, v16.6]

requires:
  - phase: 258-02
    provides: "five component docs + register projection surface docs in docs/src"
  - phase: 258-01
    provides: "json_ui_catalog BUILDER_API/RULE_COMPONENTS + generation_context register guidance"
provides:
  - "ferro-rs 0.2.89 on crates.io (v16.6 POS Component Suite milestone exit)"
  - "ferro-payments 0.1.6 on crates.io (return-URL seam rider)"
  - "git tag v0.2.89 on remote"
  - "gestiscilo register-phase handoff brief (version + contract surface)"
affects: [gestiscilo-register-phase, gestiscilo-it]

tech-stack:
  added: []
  patterns:
    - "Operator-gated publish: pre-publish checklist → human approval → ff-only master → HTTPS push → wave verification"
    - "Dual-crate verification via crates.io API (never local refs)"
    - "Stash working-tree state before branch switch to avoid checkout conflicts"

key-files:
  created:
    - ".planning/phases/258-mcp-surface-docs-publish/258-03-SUMMARY.md"
  modified:
    - "Cargo.toml (workspace version 0.2.88 → 0.2.89)"
    - "Cargo.lock (updated to 0.2.89)"

key-decisions:
  - "Pre-bump 0.2.88 → 0.2.89 manually so CI publishes directly (no double-bump); should_publish=yes path taken"
  - "Stash .planning/config.json (_auto_chain_active flag) before checkout master; avoids ff-only block"
  - "GitHub Release with binaries requires manual trigger (release.yml auto-dispatch failed — known CI limitation); git tag v0.2.89 created, crates published"
  - "Gestiscilo handoff is brief-only; no consumer-tree edits (D-17)"

patterns-established:
  - "config.json working-tree change pattern: stash → checkout master → ff-only → push → (restore stash if needed)"

requirements-completed: [POS-13]

duration: ~22min (CI: 13min test + 8min publish waves)
completed: 2026-07-06
---

# Phase 258 Plan 03: Publish + Gestiscilo Handoff Summary

**ferro-rs 0.2.89 + ferro-payments 0.1.6 published to crates.io via operator-approved ff-only push; all five publish waves green; git tag v0.2.89 created; gestiscilo register phase can now pin 0.2.89**

## Performance

- **Duration:** ~22 min total (13 min CI tests + 8 min publish waves)
- **Started:** 2026-07-06T17:01:51Z
- **Completed:** 2026-07-06T17:23:xx Z
- **Tasks:** 3 (Tasks 1-2 in prior agent, Task 3 this agent)
- **Files modified:** 2 (Cargo.toml, Cargo.lock)

## Accomplishments

- Fast-forwarded master from `feat/billable-return-url-seam` (ff-only, HEAD=master asserted from main repo root)
- Pushed master to remote via gh HTTPS credential helper; origin/master local ref corrected to HEAD
- CI Publish run 28808914072: Test green (13m1s), Bump Version skipped (pre-bump), all publish waves green (1a→1b→1c→2→3)
- ferro-rs 0.2.89 and ferro-payments 0.1.6 confirmed live on crates.io via API
- Git tag v0.2.89 pushed to remote (verified via `gh api repos/.../git/refs/tags/v0.2.89`)
- Gestiscilo register-phase handoff brief written (see below)

## Task Commits

1. **Task 1: /cassa flip verification + CI gate + 0.2.89 bump** - `34279ca7` (docs: version bump)
2. **Task 2: Operator publish checkpoint** - approved, no commit
3. **Task 3: Push + verify + handoff** - (this SUMMARY commit)

## Publish Verification

| Check | Expected | Actual | Result |
|-------|----------|--------|--------|
| `curl ferro-rs | jq .crate.max_version` | 0.2.89 | 0.2.89 | PASS |
| `curl ferro-payments | jq .crate.max_version` | 0.1.6 | 0.1.6 | PASS |
| git tag v0.2.89 on remote | exists | `refs/tags/v0.2.89` | PASS |
| GitHub Release v0.2.89 | v0.2.89 | v0.2.88 (latest) | SEE NOTE |
| HEAD=master | master | master | PASS |
| origin/master == HEAD | equal | equal (34279ca7) | PASS |

**GitHub Release note:** The release.yml auto-dispatch failed (annotation: "Could not auto-dispatch release.yml — run it manually for v0.2.89"). This is a pre-existing CI limitation — the binary build + brew tap update requires manual workflow dispatch. The git tag v0.2.89 and crates.io publication are complete; the binaries are the only missing artifact.

## Files Created/Modified

- `Cargo.toml` — workspace version bumped 0.2.88 → 0.2.89 (the CI publish trigger)
- `Cargo.lock` — updated to reflect the new workspace version

## Decisions Made

- Stashed `.planning/config.json` (`_auto_chain_active` flag) before `git checkout master` to avoid the ff-only block; the committed config on master is unaffected
- CI path: `should_publish=yes` (pre-bumped, version not yet tagged) → Test → skip BumpVersion → Publish waves → Tag
- GitHub Release auto-dispatch failure is a known limitation; documented but not blocking (crates.io publish is the primary deliverable)

## Deviations from Plan

None - plan executed exactly as written. The stash of `.planning/config.json` was an adaptation to a working-tree state that wasn't anticipated in the plan spec but follows standard git hygiene.

## Gestiscilo Register-Phase Handoff Brief

**For:** gestiscilo register-phase implementation (the consumer-side adoption of v16.6)

**Pin version:** `ferro-rs = "0.2.89"` and `ferro-payments = "0.1.6"` (if using the return-URL seam)

### Public contracts now pinnable at 0.2.89

**Five new builtin components (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad)**
- All five are in the builtin catalog; discoverable via `json_ui_catalog` MCP tool
- Docs: `docs/src/json-ui/components.md` — dedicated section per component with props table and usage examples

**Register layout template**
- `register_template()` helper in `ferro-json-ui::projection::intent_layout` — one-call Collect→Register projection
- Produces a two-pane Register layout: tile grid (left) + selection panel (right) + form + filter strip + search
- Working sample: `app/src/controllers/cassa.rs`

**Builder API additions**
- `SpecBuilder::fill_viewport(bool)` — fill the viewport for kiosk/tablet layouts (required for register layouts)
- `ElementBuilder::each(path, as_)` — iterate collections in JSON spec

**Four design-lint rules for the register pattern** (rule ids corrected 2026-07-07 at milestone audit — the original brief listed three invented ids)
- `register-fill-viewport` — a TileGrid/SelectionPanel/Numpad outside a `fill_viewport` spec causes silent whole-page scroll
- `register-grid-fill` — the register-root Grid must set `fill: true` under `fill_viewport` or panes lose internal scroll
- `register-selection-present` — a TileGrid with no SelectionPanel anywhere is an incomplete register
- `fill-viewport-layout-unknown` — `fill_viewport` only supports the `app`/`dashboard` layouts
- Run via: `ferro design:lint` or the `design_lint` MCP tool

**MCP generation_context register guidance**
- The `generation_context` MCP tool now includes a `register_composition` section covering:
  - When to use `register_template()` vs. a form-only Collect spec
  - Hidden-input quantity accumulation contract (`data-qty-input`)
  - Filter/numpad data attributes (`data-filter-tokens`, `data-filter-text`)
  - `fill_viewport` dependency
  - The four `register-*` lint rule ids

**Key interaction model (from docs)**
- One tap on a tile adds one unit; all quantity editing happens in SelectionPanel
- `disable_on_submit` double-submit guard on the Form component
- SelectionPanel is a live client-side view of form state — not a second source of truth
- Numpad is NOT part of the v1 register template; compose manually if needed

**Documentation**
- `docs/src/json-ui/components.md` — five new component sections
- `docs/src/json-ui/layouts.md` — Register layout template + fill_viewport chain
- `docs/src/json-ui/spec-construction.md` — fill_viewport and each() builder additions

## Issues Encountered

**GitHub Release auto-dispatch:** CI annotation "Could not auto-dispatch release.yml — run it manually for v0.2.89." The binary build + brew tap update for v0.2.89 requires a manual `gh workflow run release.yml` dispatch against the v0.2.89 tag. This is a pre-existing limitation of the CI setup (documented in project memory). The crates.io publish, which is the milestone deliverable, is complete.

## Next Phase Readiness

- v16.6 POS Component Suite milestone is published and closed from the ferro side
- gestiscilo register phase can pin `ferro-rs = "0.2.89"` immediately
- `/gsd-complete-milestone v16.6` archival is deferred (CONTEXT deferred list — not needed for gestiscilo to proceed)
- GitHub Release v0.2.89 binaries: run `gh workflow run release.yml` against tag v0.2.89 if CLI distribution is needed

---
*Phase: 258-mcp-surface-docs-publish*
*Completed: 2026-07-06*

## Self-Check: PASSED

- SUMMARY.md: FOUND at `.planning/phases/258-mcp-surface-docs-publish/258-03-SUMMARY.md`
- Task commit 34279ca7: FOUND in git log
- crates.io ferro-rs: 0.2.89 (API verified)
- crates.io ferro-payments: 0.1.6 (API verified)
- git tag v0.2.89: FOUND at `refs/tags/v0.2.89`
- HEAD=master: CONFIRMED
- origin/master == HEAD (34279ca7): CONFIRMED
- Gestiscilo handoff brief: embedded in SUMMARY.md
