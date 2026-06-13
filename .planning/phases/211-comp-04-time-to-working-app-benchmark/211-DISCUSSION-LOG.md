# Phase 211: COMP-04 — Time-to-Working-App Benchmark - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves alternatives considered.

**Date:** 2026-06-13
**Phase:** 211-comp-04-time-to-working-app-benchmark
**Mode:** --auto (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Execution model, Cold-cache run, Result document, Agent-assistance level, CI threshold

---

## Execution model (warm-local vs cold-Docker split)

| Option | Description | Selected |
|--------|-------------|----------|
| Warm criterion benchmark for per-step + Docker for cold number | Hybrid; mirrors COMP-03 gate | ✓ |
| Single cold-only measurement | No per-step granularity, slower iteration | |
| Warm-only | Violates SC#3 (cold-cache required) | |

**Auto-selected:** Hybrid (D-01). Warm `iter_custom` (FERRO_BENCH=1) gives per-step breakdown; Docker gives the headline cold number.

## Cold-cache run (who executes)

| Option | Description | Selected |
|--------|-------------|----------|
| Human-action (autonomous: false) | No Docker in autonomous env; heavy disk/time | ✓ |
| Autonomous in-CI | Infeasible — no Docker daemon, disk budget | |

**Auto-selected:** Human-action (D-02). Commit Dockerfile + run command; developer runs and commits the number.

## Result document (location/format)

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-cli/tests/fixtures/benchmark/RESULTS.md` with env-spec table | Alongside the benchmark | ✓ |
| Phase-dir-only doc | Detached from the apparatus | |

**Auto-selected:** Fixtures-adjacent RESULTS.md with the SC#4 env-spec table (D-05).

## Agent-assistance level (SC#4 field)

| Option | Description | Selected |
|--------|-------------|----------|
| Manual commands | Deterministic, reproducible baseline | ✓ |
| Agent-driven | A future variant | |

**Auto-selected:** Manual commands (D-06).

## CI wall-clock threshold

| Option | Description | Selected |
|--------|-------------|----------|
| Defer (assert exit-code 0 only) | Per ROADMAP calibration note | ✓ |
| Assert a threshold now | Premature — no baseline yet | |

**Auto-selected:** Deferred (D-07).

## Claude's Discretion

criterion `iter_custom` structure; entity/job names; Dockerfile base tag; RESULTS.md layout; per-step timing capture mechanism.

## Deferred Ideas

CI threshold (after first cold run); agent-driven variant; asserting the gate in CI (disk permitting).
