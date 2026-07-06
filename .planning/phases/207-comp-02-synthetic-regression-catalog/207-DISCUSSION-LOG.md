# Phase 207: COMP-02 — Synthetic Regression Catalog - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 207-comp-02-synthetic-regression-catalog
**Mode:** auto (all gray areas auto-selected, recommended option taken per area)
**Areas discussed:** Render-assertion strategy, Snapshot tooling scope, Proptest invariants, Adversarial fixtures breadth, Confidence-threshold style, Catalog organization & CI

**Pre-discussion note:** init `phase-op 207` returned `phase_found: false` because
`extractCurrentMilestone` matched the first `### ...v13.0...` heading in ROADMAP.md
(`### 🔭 v13.0 Future UI Spec Evaluation (Phase 174)`, a stale future-spec label) instead of
`## 🚧 v13.0 Compressive Validation (Phases 207–211)`. Fixed by dropping the stale `v13.0` label
from the Phase 174 exploration heading so the milestone resolver lands on the real milestone.

---

## Render-assertion strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Assert on `ServiceDef` + `derive_intents()` structure in-crate (no renderer dep) | Express SC#2 "table shape" as a structural invariant on the fixture; no cross-crate dependency | ✓ |
| Add `ferro-json-ui` dev-dep, render real specs | Reverse dependency (`ferro-json-ui` already depends on `ferro-projections`) — cycle through dev edge; violates CLAUDE.md:9 | |
| Wait for Phase 208 sketch renderers | Out of phase scope; Phase 207 has no in-crate renderer | |

**Choice:** In-crate structural assertion (recommended). **Notes:** D-01/D-02. The binding
constraint is `CLAUDE.md:9` ("do not add dependencies to ferro-projections") plus the actual
dependency direction in `ferro-json-ui/Cargo.toml:24`.

---

## Snapshot tooling scope (insta)

| Option | Description | Selected |
|--------|-------------|----------|
| Add `insta` dev-dep, snapshot only the 7 named canonical shapes | Snapshot ranked `(intent, signals)`, confidences redacted; structural asserts dominate | ✓ |
| No insta — inline structural asserts only | Drops the named-canonical-shape artifact the roadmap deliverable calls for | |

**Choice:** Add insta, minimal use (recommended). **Notes:** D-03/D-04. SC#2 requires structural
asserts outnumber insta snapshots; snapshotting names+ranking (not floats) avoids fragility.

---

## Proptest invariants

| Option | Description | Selected |
|--------|-------------|----------|
| Engine-robustness invariants over generated `ServiceDef`s | Never panics, non-empty, confidence ∈ [0,1], sorted desc, no duplicate intents | ✓ |
| Generate per-intent fixtures and assert the intent | Too coupled to scoring; flaky | |
| Skip proptest | Roadmap deliverable lists proptest invariants explicitly | |

**Choice:** Engine-robustness invariants (recommended). **Notes:** D-05. `proptest = "1"` matches
sibling crates.

---

## Adversarial fixtures breadth

| Option | Description | Selected |
|--------|-------------|----------|
| One competing-signal fixture per confusable pair (Browse↔Summarize, Process↔Track, Analyze↔Summarize, Collect↔Focus) | ≥4 documented adversaries targeting the realistic confusions | ✓ |
| Exactly one adversarial fixture total | SC#3 floor only; weaker honesty coverage | |
| One adversarial fixture per intent (7) | Deliverable-line literal; some intents lack a meaningful adversary | |

**Choice:** Per-confusable-pair (recommended). **Notes:** D-06. Satisfies SC#3 floor and the
deliverable spirit while serving the milestone honesty requirement.

---

## Confidence-threshold style

| Option | Description | Selected |
|--------|-------------|----------|
| Hard intent identity + runner-up margin + conservative per-intent floor calibrated post-run | Resilient to benign derive.rs re-tuning, still catches regression | ✓ |
| Fixed absolute floor (e.g. ≥0.5) for all | Fragile / arbitrary across intents | |
| Intent identity only, no confidence assertion | Misses silent confidence erosion (SC#1 wants a threshold) | |

**Choice:** Identity + margin + calibrated floor (recommended). **Notes:** D-07. Numbers set after
a first real run, not at plan time.

---

## Catalog organization & CI

| Option | Description | Selected |
|--------|-------------|----------|
| Single `tests/catalog.rs`, no `#[ignore]`, default CI gate | Roadmap-named path; legible CI failure on regression | ✓ |
| Split across multiple files | Unnecessary; roadmap names one file | |

**Choice:** Single file in default gate (recommended). **Notes:** D-08. SC#4.

---

## Claude's Discretion

- Builder-function names, fixture field sets, bounded proptest `Strategy` shape.
- Assertion helper structure for margin/floor.
- insta snapshot file naming.

## Deferred Ideas

- Real JSON-UI render-tree assertions importing the catalog fixtures — belongs in `ferro-json-ui`.
- Expanding adversarial fixtures to all 7 intents if the honesty review finds gaps.
