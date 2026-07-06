# Phase 185: ferro::queue — DB-Backed Job Queue - Context

**Gathered:** 2026-06-07 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the Redis-only ferro-queue backend with a DB-backed job queue: consumers implement `Job`, the `WorkerLoop` runs in-process inside the app server (work-stealing across identical instances per gestiscilo D-01), and the claim path is atomic on both production Postgres (`FOR UPDATE SKIP LOCKED`) and dev SQLite (`BEGIN IMMEDIATE` + `UPDATE … RETURNING`). Includes retry with exponential backoff + jitter, stuck-job reaper, poison-job isolation, graceful shutdown, idempotency-key hook, and a portable `jobs` table migration helper.

**Killer feature:** zero-infrastructure background jobs — the atomic dual-backend claim makes work-stealing "just work" on the app's own database in both dev (SQLite) and prod (Postgres), letting consumers drop Redis entirely. Everything else in the phase supports this.

Requirements: QUEUE-F-01, QUEUE-F-02, QUEUE-F-03, QUEUE-F-04. Consumer: gestiscilo Phase 188 (migrates 4 job types).

</domain>

<decisions>
## Implementation Decisions

### Crate strategy
- **D-01:** Refactor `ferro-queue` crate in place. The DB backend **replaces** the Redis backend entirely — delete Redis code, drop the `redis` dependency (gestiscilo STACK doc: "replace the queue backend, not extend it"; project not in production, breaking changes permitted).
- **D-02:** The framework exposes a canonical `ferro::queue` module path (requirement QUEUE-F-01 names `ferro::queue::Job`). Replace the current flat root re-exports in `framework/src/lib.rs:194-199` with a namespaced `pub mod queue` re-export — one control surface, no duplicate paths.
- **D-03:** `ferro-queue` takes a `sea_orm::DatabaseConnection` (it may depend on `sea-orm` directly; it must NOT depend on `framework`). The framework wires the app's connection in at bootstrap.

### Jobs table schema
- **D-04:** Single `jobs` table. Explicit `status` column (`pending` / `claimed` / `failed`) plus `claimed_at` timestamp. Successful jobs are **deleted** on completion (keeps the table small); failed jobs are **parked in the same table** with `status = 'failed'` and the error message recorded — satisfies poison-job isolation without a separate `failed_jobs` table.
- **D-05:** Columns (planner refines exact types): `id` (i64 autoincrement — portable + natural FIFO ordering), `job_type`, `payload` (TEXT JSON), `queue` (lane name), `attempts`, `max_retries`, `idempotency_key` (nullable), `tenant_id` (nullable — Phase 97 continuity), `available_at`, `claimed_at`, `claimed_by` (worker instance id), `error` (nullable), `created_at`.
- **D-06:** Migration helper ships in `ferro-queue` following `ferro-migration` portability conventions (SchemaManager-based, no backend-specific SQL in any migration file). Success criterion: "no raw `FOR UPDATE SKIP LOCKED` SQL in any migration file".

### Claim mechanics
- **D-07:** Claim SQL branches at runtime on the live `DatabaseBackend`: Postgres → `SELECT … FOR UPDATE SKIP LOCKED` claim; SQLite → `BEGIN IMMEDIATE` + `UPDATE jobs SET status='claimed', claimed_at=… WHERE status='pending' AND available_at <= now ORDER BY id LIMIT 1 … RETURNING`. Both paths claim exactly one job per iteration. The SQLite fallback is a hard requirement, not best-effort (gestiscilo PITFALLS A-02: dev-mode worker doing nothing masks integration bugs).
- **D-08:** Worker concurrency: the loop claims jobs one at a time but executes up to a configurable number of in-flight jobs concurrently (reuse existing `WorkerConfig::max_jobs` semantics). Idle polling backs off (configurable sleep, existing `sleep_duration`).

### WorkerLoop integration
- **D-09:** `WorkerLoop` auto-starts inside the app server path of `Application::run` when at least one job type is registered — no separate process, no separate CLI command required for normal operation (D-01 work-stealing single binary). Job registration stays typed-registry-based like current `Worker::register::<J>()`.
- **D-10:** Graceful shutdown: SIGTERM handler sets a shutdown flag; the loop stops claiming, drains in-flight jobs to completion (or their `failed()` hook), and re-queues claimed-but-not-started jobs (reset to `pending`). Per PITFALLS A-03.
- **D-11:** Panic isolation at the worker-loop level — a panicking `handle()` must never kill the loop; the panic counts as a failed attempt. Per PITFALLS A-01 ("never trust `handle()` to be panic-free").
- **D-12:** CPU-heavy job bodies are documented (docs/src/) to use `tokio::task::spawn_blocking` — guidance, not enforcement.

### Retry, reaper, idempotency
- **D-13:** Default retry delay: exponential backoff with full jitter — base 5s, factor 2^attempt, cap 15min. The existing `Job::retry_delay(attempt)` hook remains the override point; only its default changes.
- **D-14:** Stuck-job reaper fires before each claim cycle: re-queues rows where `status='claimed'` and `claimed_at` is older than the visibility timeout, incrementing `attempts`. Visibility timeout configurable, default 5 minutes (PITFALLS A-01). Reaped jobs that exceed `max_retries` park as `failed`.
- **D-15:** Idempotency hook: `fn idempotency_key(&self) -> Option<String>` on `Job`, default `None`. When `Some`, enqueue skips insertion if a `pending`/`claimed` row with the same `(job_type, idempotency_key)` already exists.

### API continuity
- **D-16:** Preserve the existing public surface where possible: `Job` trait methods (`handle`, `name`, `max_retries`, `retry_delay`, `failed`, `timeout`), `Queueable` blanket trait (`dispatch`, `delay`, `on_queue`), `dispatch`/`dispatch_later`/`dispatch_to` free functions. Any unavoidable break documented with a migration table (success criterion 5; consumer gestiscilo Phase 188 migrates `RenderDocumentPdfJob`, `SendBookingReminderJob`, `DeliverNotificationJob`, `screenshot_worker`).
- **D-17:** Tenant scoping carries over unchanged: `tenant_id` column in the jobs table, `TenantScopeProvider` + `register_tenant_capture_hook` mechanisms preserved (Phase 97 behavior).
- **D-18:** Queue introspection reimplemented over the DB: `QueueStats`/`JobInfo`/`FailedJobInfo` types, `/_ferro/queue/jobs` + `/_ferro/queue/stats` debug endpoints, and ferro-mcp tools (`queue_status`, `list_jobs`, `job_history`) must keep working against the `jobs` table. ferro-mcp update is in scope per CLAUDE.md.

### Claude's Discretion
- Exact claim SQL formulation per backend (as long as the race test passes on both)
- Worker instance id format (`claimed_by`)
- Whether `JobPayload` stays as the serialization envelope or is absorbed into the jobs-table row mapping
- Reaper interval default (within "fires before each claim cycle" constraint)
- How the typed job registry is threaded from `Application` bootstrap to the loop

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Locked design (gestiscilo v7.1 — source of this milestone)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-ARCHITECTURE.md` — D-01 (work-stealing across identical instances, `FOR UPDATE SKIP LOCKED` semantics, `spawn_blocking` for CPU jobs), D-06 (single-binary topology), component table (`ferro::queue` deliverable definition)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-PITFALLS.md` §A (A-01 poison job + reaper + panic isolation; A-02 SQLite claim fallback is a hard prerequisite; A-03 SIGTERM drain semantics; A-04 behavior-drift checklist for the 4 consumer job types)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-STACK.md` §D-01 — "replace the queue backend, not extend it"; no new external crates needed for the queue

### Ferro repo
- `.planning/ROADMAP.md` §"v12.3 Deployment Platform Primitives" — requirements QUEUE-F-01..04, phase success criteria, consumer pairing (gestiscilo Phase 188)
- `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md` — workspace conventions for the refactor

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-queue/src/job.rs` — `Job` trait already has `max_retries`, `retry_delay(attempt)`, `failed()`, `timeout()`; `JobPayload` already carries `tenant_id`, `attempts`, `available_at`. The trait surface mostly survives; only the storage layer changes.
- `ferro-queue/src/worker.rs` — `WorkerConfig` (queues, max_jobs, sleep_duration), `TenantScopeProvider`, typed `register::<J>()` pattern, shutdown `Notify` — all reusable shapes for the new `WorkerLoop`.
- `ferro-queue/src/dispatcher.rs` — `dispatch`/`dispatch_later`/`dispatch_to`, `PendingDispatch`, `register_tenant_capture_hook` — keep the public API, swap the sink from Redis to the `jobs` table.
- `ferro-migration` crate — portability conventions for the migration helper (SchemaManager-based helpers, SQLite + Postgres).
- `framework/src/database/` — SeaORM connection management; backend detectable at runtime for claim-SQL branching.

### Established Patterns
- Framework re-exports queue API at `framework/src/lib.rs:194-199` (flat) — to be replaced by namespaced `ferro::queue` module (D-02).
- `framework/src/app.rs` `Application::run` already dispatches CLI subcommands (`schedule:work` etc.) and owns the server path — natural integration point for WorkerLoop startup (D-09).
- `framework/src/server.rs` handles `/_ferro/queue/jobs` and `/_ferro/queue/stats` debug endpoints — must keep working over the DB backend (D-18).
- Error types: thiserror, one Error enum per crate. Builder pattern: consuming `with_*` methods.

### Integration Points
- `framework/Cargo.toml` depends on `ferro-queue` path+version — version bump on publish (GH Actions publish workflow, ferro-queue already in it).
- ferro-mcp tools `queue_status`, `list_jobs`, `job_history` read queue state — update for DB backend (D-18).
- Phase 97 tenant-aware job execution (`TenantScopeProvider`, tenant capture hook) — must survive unchanged (D-17).
- `docs/src/` queue documentation — must be rewritten for the DB backend (no Redis setup section; spawn_blocking guidance per D-12).

</code_context>

<specifics>
## Specific Ideas

- The race test (two concurrent WorkerLoops, each job claimed exactly once) is the phase's proof artifact — SQLite always-on, Postgres behind a cfg-gated test (success criterion 1).
- gestiscilo PITFALLS A-04 lists the 4 consumer job types and their semantics (`failed()` side effects, retry counts, lane assignments) — the migration table in this phase's docs should map old→new for exactly these patterns.
- `ferro serve` (CLI) is a dev process manager that spawns the app binary — "WorkerLoop in ferro serve" means the app server runtime (`Application::run`), not `ferro-cli/src/commands/serve.rs`.

</specifics>

<deferred>
## Deferred Ideas

- Operator alerting on stuck-job accumulation (PITFALLS A-01 mentions paging at ≥10 permanently-claimed rows) — observability concern, belongs with a later monitoring phase; the reaper itself is in scope here.
- Job chaining (mentioned in old ferro-queue docs) — not in QUEUE-F requirements; do not rebuild unless it falls out for free.

</deferred>

---

*Phase: 185-ferro-queue-db-backed-job-queue*
*Context gathered: 2026-06-07 (auto mode)*
