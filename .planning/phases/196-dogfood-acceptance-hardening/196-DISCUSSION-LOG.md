# Phase 196: Dogfood Acceptance + Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 196-dogfood-acceptance-hardening
**Mode:** --auto (recommended defaults auto-selected)
**Areas discussed:** Poisoned fixture form, Live consumer selection, Acceptance evidence/gate, Zero-finding seam demotion

---

## Poisoned Synthetic Fixture

| Option | Description | Selected |
|--------|-------------|----------|
| Committed acceptance test fixture (existing tempdir helpers) | Reproducible, exact-assertable; reuses `project_with_projection`/`add_model` | ✓ |
| Poison an `app/` sample projection | Real-app feel, but couples acceptance to a mutable sample app | |
| Standalone synthetic catalog directory | New infra; heavier than needed for one planted defect | |

**User's choice:** [auto] Committed acceptance test fixture (recommended default)
**Notes:** SC-1 requires naming exactly the planted dangling field and no other; tempdir fixture gives precise control over column set vs field set.

---

## Live Consumer Selection

| Option | Description | Selected |
|--------|-------------|----------|
| In-repo `app/` sample application | 9 projections vs 3 models — real mismatch; reproducible from the gate | ✓ |
| gestiscilo (external friction-loop consumer) | Strongest evidence, but not checked out as a sibling — unreachable from this repo | |

**User's choice:** [auto] `app/` sample application (recommended default)
**Notes:** gestiscilo not at any sibling path. RESEARCH FLAG recorded: run checkpoint against `app/` early; zero findings ⇒ acceptance NOT satisfied.

---

## Acceptance Evidence / Go-No-Go Gate

| Option | Description | Selected |
|--------|-------------|----------|
| Automated tests + committed acceptance report | Tests lock SC-1/SC-3; report records live run + GO/NO-GO + seam tally | ✓ |
| Automated tests only | No durable record of the live-consumer go/no-go decision | |
| Report doc only | No regression protection for poisoned fixture / cap | |

**User's choice:** [auto] Both — tests + `196-ACCEPTANCE.md` report (recommended default)
**Notes:** The report's finding tally feeds the zero-finding seam demotion (D-04).

---

## Zero-Finding Seam Demotion

| Option | Description | Selected |
|--------|-------------|----------|
| Code default → `not_checked` + documentation | Data-driven; reuses existing `not_checked` variant; satisfies SC-4 | ✓ |
| Documentation-only (leave code emitting pass) | Risks emitting vacuous `pass` for unproven seams | |
| Config flag to toggle seams | Adds a control surface; over-engineered for an acceptance hardening pass | |

**User's choice:** [auto] Code default → `not_checked` + documentation (recommended default)
**Notes:** Evidence-driven — only seams with zero findings across all dogfood inputs are demoted; seam 2 exempt (poisoned fixture proves it).

---

## Claude's Discretion

- Test function names; `196-ACCEPTANCE.md` structure beyond required GO/NO-GO + tally; `not_checked` reason wording; cap as named `const` vs inline literal.

## Deferred Ideas

- External-consumer (gestiscilo) dogfood once reachable from a shared checkout.
- Phase 194 IN-02 `DataType` subject polish — only if this phase touches that path.
