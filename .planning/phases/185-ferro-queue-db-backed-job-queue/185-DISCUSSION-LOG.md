# Phase 185: ferro::queue — DB-Backed Job Queue - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 185-ferro-queue-db-backed-job-queue
**Mode:** auto (`--auto`) — recommended options selected without interactive prompts
**Areas discussed:** Crate strategy, Jobs table schema, Claim mechanics, WorkerLoop integration, Retry/reaper/idempotency, API continuity

---

## Crate strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Refactor ferro-queue in place, delete Redis | Replace the backend inside the existing crate; framework adds `ferro::queue` namespace | ✓ |
| New `framework::queue` module, delete crate | Move queue code into framework; loses independent crate versioning/publish wave | |
| Add DB backend alongside Redis | Two backends behind a config switch — duplicate control surface, contradicts STACK doc | |

**Rationale:** gestiscilo v7.1-STACK.md explicitly says "replace the queue backend, not extend it"; project is not in production; delete-old-code principle; crate already wired into the publish workflow and framework dependency graph.

---

## Jobs table schema

| Option | Description | Selected |
|--------|-------------|----------|
| Single table, status column, delete-on-success | `pending`/`claimed`/`failed`; failures parked in-table with error | ✓ |
| jobs + failed_jobs split (Laravel-style) | Two tables; more moving parts for the migration helper | |
| Keep completed rows (status=completed) | Audit-friendly but unbounded growth; audit belongs to ferro-audit | |

**Rationale:** simplest schema satisfying poison-job isolation and "parked as failed with error recorded"; bounded table size.

---

## Claim mechanics

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime branch on live backend | Postgres `FOR UPDATE SKIP LOCKED`; SQLite `BEGIN IMMEDIATE` + `UPDATE…RETURNING` | ✓ |
| Portable optimistic claim everywhere | Single SQL path; loses the canonical Postgres work-stealing primitive | |

**Rationale:** mandated by QUEUE-F-02 and success criterion 1; PITFALLS A-02 makes the SQLite fallback a hard prerequisite.

---

## WorkerLoop integration

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-start in `Application::run` when jobs registered | Single binary, zero extra config (D-01/D-06) | ✓ |
| Explicit opt-in builder (`app.with_worker()`) | More explicit but adds a knob the design says shouldn't exist | |
| Separate `queue:work` CLI subcommand | Role-split topology — explicitly rejected by D-01 | |

**Rationale:** D-01 work-stealing single binary; success criterion 4 ("no separate process").

---

## Retry, reaper, idempotency

| Option | Description | Selected |
|--------|-------------|----------|
| Backoff base 5s ×2^attempt, full jitter, cap 15min; reaper before each claim cycle, visibility timeout 5min; `idempotency_key()` dedupe at enqueue | Defaults per PITFALLS A-01 guidance | ✓ |
| Fixed retry delay (current default) | Fails success criterion 3 | |

---

## API continuity

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve Job/Queueable/dispatch surface + tenant scoping; document unavoidable breaks | Success criterion 5; gestiscilo Phase 188 migrates 4 job types against it | ✓ |
| Clean-slate API redesign | Breaking changes allowed, but criterion 5 asks for preservation where possible | |

---

## Claude's Discretion

- Exact claim SQL per backend, worker instance id format, JobPayload envelope fate, reaper interval default, registry threading from Application bootstrap

## Deferred Ideas

- Stuck-job accumulation alerting/paging (observability phase)
- Job chaining (not in QUEUE-F requirements)
