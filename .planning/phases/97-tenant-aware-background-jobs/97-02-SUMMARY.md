---
phase: 97-tenant-aware-background-jobs
plan: "02"
subsystem: ferro-queue
tags: [tenant, background-jobs, worker, scope-provider, tdd]
dependency_graph:
  requires: [97-01]
  provides: [Worker::with_tenant_scope, tenant-scope-wrapped job execution, Clone preserves tenant_scope]
  affects: [ferro-queue]
tech_stack:
  added: []
  patterns: [TDD fake Redis TCP server for struct-field tests, match on (tenant_scope, tenant_id) scope dispatch]
key_files:
  created: []
  modified:
    - ferro-queue/src/worker.rs
decisions:
  - "tenant_scope field added to Worker struct as Option<Arc<dyn TenantScopeProvider>>"
  - "with_scope() called inside tokio::spawn — task-locals do not cross spawn boundaries"
  - "match (tenant_scope, tenant_id) — only wraps when both provider and tenant_id are present"
  - "Fake Redis TCP server (responds +OK to all lines) for struct-field tests without live Redis"
  - "tenant_id = ?tenant_id in tracing debug! span — shows None or Some(N) in logs"
metrics:
  duration_seconds: 611
  completed_date: "2026-03-11"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 97 Plan 02: Worker Tenant Scope Injection Summary

**One-liner:** Worker gains `with_tenant_scope()` builder and scope-wrapped job execution via match on `(tenant_scope, tenant_id)` inside `tokio::spawn`.

## What Was Built

Wired `TenantScopeProvider` into `Worker` to complete the ferro-queue side of tenant-aware job execution. When a provider is injected and a job carries a `tenant_id`, the job future is executed inside a tenant context scope.

### Worker (ferro-queue/src/worker.rs)

- `tenant_scope: Option<Arc<dyn TenantScopeProvider>>` field added to `Worker` struct
- `Worker::new()` initializes `tenant_scope: None`
- `Worker::with_tenant_scope(provider: Arc<dyn TenantScopeProvider>) -> Self` builder method
- `process_job()` updated: extracts `tenant_scope` and `tenant_id` before `tokio::spawn`, then matches inside the spawn to call `scope.with_scope(id, job_fut).await` only when both are `Some`
- `tenant_id = ?tenant_id` added to the `debug!` tracing span — outputs `tenant_id=None` or `tenant_id=Some(42)`
- `Clone` impl updated to include `tenant_scope: self.tenant_scope.clone()`

### Backward Compatibility

- No provider set: job runs directly via `handler(payload.data.clone()).await` (unchanged path)
- Provider set but `tenant_id` is `None`: same direct path (no scope wrapping)
- `TenantNotFound` from provider flows through normal job failure path (retry/failed queue)

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add with_tenant_scope() builder and tenant scope wrapping in process_job() | fb73f82 | ferro-queue/src/worker.rs |

## Tests Added

- `worker::tests::test_with_tenant_scope_stores_provider` — field is Some after with_tenant_scope()
- `worker::tests::test_worker_without_scope_has_none_by_default` — field is None by default
- `worker::tests::test_clone_preserves_tenant_scope` — Clone copies Some field
- `worker::tests::test_clone_without_scope_preserves_none` — Clone copies None field
- `worker::tests::test_mock_scope_provider_calls_future` — MockScopeProvider records tenant_id and runs future
- `worker::tests::test_mock_scope_provider_failure_returns_tenant_not_found` — failing variant returns TenantNotFound
- `worker::tests::test_scope_dispatch_tenant_id_some_calls_with_scope` — Some(id) + provider calls with_scope
- `worker::tests::test_scope_dispatch_tenant_id_none_skips_with_scope` — None + provider runs directly
- `worker::tests::test_scope_dispatch_no_provider_runs_job_directly` — Some(id) + no provider runs directly

46 total ferro-queue tests pass (37 existing + 9 new).

## Key Decisions

1. **`tenant_scope: Option<Arc<dyn TenantScopeProvider>>`** — Arc allows Clone without requiring TenantScopeProvider to impl Clone.

2. **`with_scope()` inside `tokio::spawn`** — task-local variables do not cross spawn boundaries. Both `tenant_id` (i64, Copy) and `tenant_scope` (Arc, Clone) cross the boundary safely.

3. **`match (&tenant_scope, tenant_id)`** — only wraps when BOTH provider and tenant_id are Some. Backward compatible: no provider or no tenant_id means direct handler call.

4. **Fake Redis TCP server for tests** — `Worker::new()` requires a `QueueConnection` which requires a live Redis. A fake TCP server that responds `+OK\r\n` to all lines satisfies the `ConnectionManager` initial handshake without a real Redis instance. Struct-field tests never trigger actual Redis I/O.

5. **`tenant_id = ?tenant_id` tracing** — debug format for `Option<i64>` gives `None` or `Some(42)` in logs, making tenant context visible without a custom formatter.

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- FOUND: ferro-queue/src/worker.rs (modified)
- FOUND commit: fb73f82
- 46 ferro-queue tests pass
- cargo fmt passes
- cargo clippy passes
