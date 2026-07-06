# Phase 136: Implement Workflow for Executing a Full Roadmap in Auto with GSD - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 136-implement-workflow-for-executing-a-full-roadmap-in-auto-with-gsd
**Areas discussed:** Relationship to /gsd:autonomous, Context window isolation, Failure & recovery strategy, Scope of 'full roadmap', State tracking, GitHub issue on failure, Script location & packaging

---

## Relationship to /gsd:autonomous

| Option | Description | Selected |
|--------|-------------|----------|
| Fix existing autonomous | The workflow exists but has bugs/limitations. Patch and improve it. | |
| Cross-session orchestration | Autonomous dies when context fills up. Need outer loop that survives context resets. | |
| Fully hands-off pipeline | Autonomous still pauses for gray areas and blockers. Want fully automatic mode. | |

**User's choice:** Clarified that the goal is "full autonomous ROADMAP not phase" — the problem is that /gsd:autonomous runs inside one context window and can't drive a full multi-phase roadmap. Need an outer orchestrator.

---

## Context Window Isolation (Invocation Strategy)

| Option | Description | Selected |
|--------|-------------|----------|
| One phase per invocation | Always spawn fresh claude CLI session per phase. Simple, predictable. | ✓ |
| Batch until context fills | Run phases sequentially in one session, detect context exhaustion. | |

**User's choice:** One phase per invocation
**Notes:** Clean isolation, no complexity around context detection.

---

## Failure & Recovery Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Stop and report | Halt roadmap run, write status file. | |
| Skip and continue | Log failure, move to next phase. | |
| Retry once then stop | Give one retry in fresh context. | |

**User's choice:** Stop and open GitHub issue
**Notes:** User specified "stop and open issue" — halt the run and create a GitHub issue with failure details.

---

## Scope of 'Full Roadmap'

| Option | Description | Selected |
|--------|-------------|----------|
| Current milestone only | Run incomplete phases in active milestone. | |
| All milestones sequentially | Auto-advance through milestones. | |
| User-specified range | Accept --from/--to flags for phase subset. | |

**User's choice:** Milestone name as workflow parameter
**Notes:** Script accepts milestone name as param, runs all incomplete phases in that milestone.

---

## State Tracking Between Phases

| Option | Description | Selected |
|--------|-------------|----------|
| Read ROADMAP.md + disk | Use existing gsd-tools roadmap analyze. No new state file. | ✓ |
| Dedicated run log file | Script writes own .planning/RUN-LOG.md. | |
| Both | Roadmap analyze + run log for audit trail. | |

**User's choice:** Read ROADMAP.md + disk
**Notes:** No new state file needed.

---

## GitHub Issue on Failure

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: phase + error | Title + exit code + last lines of output + phase directory link. | ✓ |
| Rich: full diagnostics | Phase goal, failed step, log tail, VERIFICATION.md excerpt. | |
| You decide | Claude's discretion. | |

**User's choice:** Minimal: phase + error

---

## Script Location & Packaging

| Option | Description | Selected |
|--------|-------------|----------|
| gsd bin directory | ~/.claude/get-shit-done/bin/gsd-roadmap-run.sh — standalone. | ✓ |
| GSD skill + shell script | Skill for docs + script for execution. | |
| Pure gsd-tools subcommand | Add roadmap run to gsd-tools.cjs. | |

**User's choice:** Minimal — standalone shell script in bin/
**Notes:** No skill wrapper, no gsd-tools integration.

---

## Claude's Discretion

- Exact claude CLI flags and invocation pattern
- Success/failure detection from exit codes or output parsing
- Whether to use /gsd:autonomous --from N or individual phase commands

## Deferred Ideas

None.
