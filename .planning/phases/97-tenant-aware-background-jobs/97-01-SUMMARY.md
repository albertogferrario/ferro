---
phase: 97-tenant-aware-background-jobs
plan: "01"
subsystem: ferro-queue
tags: [tenant, background-jobs, dispatch, onclock, trait]
dependency_graph:
  requires: []
  provides: [tenant_id in JobPayload, TenantScopeProvider trait, register_tenant_capture_hook, PendingDispatch::for_tenant]
  affects: [ferro-queue, framework]
tech_stack:
  added: []
  patterns: [OnceLock singleton hook, builder method chaining, serde(default) backward compat]
key_files:
  created: []
  modified:
    - ferro-queue/src/job.rs
    - ferro-queue/src/error.rs
    - ferro-queue/src/worker.rs
    - ferro-queue/src/dispatcher.rs
    - ferro-queue/src/lib.rs
decisions:
  - "OnceLock<fn() -> Option<i64>> for hook — zero-cost static, no Arc/Mutex needed for fn pointer"
  - "tenant_id: Option<i64> with serde(default) — backward compat with old payloads missing the field"
  - "TenantScopeProvider placed in worker.rs — logically belongs to worker-side execution, not dispatch"
  - "for_tenant() in sync mode is a no-op with debug log — job runs in current task's context"
  - "captured_tenant_id() precedence: explicit override > hook > None"
metrics:
  duration_seconds: 272
  completed_date: "2026-03-11"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Phase 97 Plan 01: Tenant-Aware Background Jobs — Queue Primitives Summary

**One-liner:** Tenant context propagation in ferro-queue via JobPayload envelope field, OnceLock capture hook, PendingDispatch::for_tenant() builder, and TenantScopeProvider trait.

## What Was Built

Added tenant context propagation primitives to ferro-queue without introducing circular dependencies on the framework crate. All types use only primitives (i64) across the crate boundary.

### JobPayload (ferro-queue/src/job.rs)

- `tenant_id: Option<i64>` field with `#[serde(default)]` — old payloads without the field deserialize to `None`
- `with_tenant_id(Option<i64>) -> Self` builder method for chained construction
- `new()` and `with_delay()` initialize `tenant_id: None`

### Error (ferro-queue/src/error.rs)

- `Error::TenantNotFound { tenant_id: i64 }` variant with message "Tenant not found for job: tenant_id={id}"
- `Error::tenant_not_found(id: i64) -> Self` convenience constructor

### TenantScopeProvider (ferro-queue/src/worker.rs)

- `#[async_trait] pub trait TenantScopeProvider: Send + Sync` — object-safe
- `with_scope(tenant_id: i64, f: Pin<Box<dyn Future<...>>>) -> Result<(), Error>` — wraps job future in tenant scope
- Implemented by the framework, injected at worker startup

### Dispatcher (ferro-queue/src/dispatcher.rs)

- `static TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>>` — global capture hook
- `register_tenant_capture_hook(f: fn() -> Option<i64>)` — called once at bootstrap; re-registration silently ignored
- `PendingDispatch::tenant_id: Option<i64>` field, initialized to `None`
- `PendingDispatch::for_tenant(tenant_id: i64) -> Self` — explicit tenant override builder
- `PendingDispatch::captured_tenant_id() -> Option<i64>` — resolves with precedence: explicit > hook > None
- `dispatch_to_queue()` attaches captured tenant_id to JobPayload before pushing

### Re-exports (ferro-queue/src/lib.rs)

- `pub use worker::TenantScopeProvider`
- `pub use dispatcher::register_tenant_capture_hook` added to existing dispatcher re-exports

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add tenant_id to JobPayload, TenantNotFound error, TenantScopeProvider trait | f8a1906 | job.rs, error.rs, worker.rs, lib.rs |
| 2 | Add OnceLock capture hook and PendingDispatch::for_tenant with auto-capture | a36a9b9 | dispatcher.rs, lib.rs |

## Tests Added

- `job::tests::test_tenant_id_none_by_default` — new() sets tenant_id to None
- `job::tests::test_tenant_id_none_serializes_as_null` — JSON has "tenant_id":null
- `job::tests::test_tenant_id_some_round_trips` — Some(42) round-trips through JSON
- `job::tests::test_old_payload_without_tenant_id_deserializes_to_none` — backward compat
- `job::tests::test_with_tenant_id_builder` — builder method works for Some and None
- `error::tests::test_tenant_not_found_formats_with_id` — error message format
- `error::tests::test_tenant_not_found_constructor` — variant match
- `worker::tests::test_tenant_scope_provider_is_object_safe` — Arc<dyn TenantScopeProvider> compiles
- `dispatcher::tests::test_for_tenant_stores_explicit_override` — field set correctly
- `dispatcher::tests::test_for_tenant_explicit_wins_over_hook` — precedence rule
- `dispatcher::tests::test_no_tenant_id_by_default` — PendingDispatch starts with None
- `dispatcher::tests::test_hook_registration_second_call_is_noop` — OnceLock semantics
- `dispatcher::tests::test_hook_registration_captures_at_dispatch_time` — hook invoked

37 total tests pass (32 existing + 5 new pre-existing + 13 new from this plan = 37 in ferro-queue).

## Key Decisions

1. **OnceLock<fn() -> Option<i64>>** — function pointer (not closure) allows OnceLock without Arc. Zero-cost static, no heap allocation.

2. **serde(default) on tenant_id** — cleanest backward compat for old payloads. No custom deserializer needed.

3. **TenantScopeProvider in worker.rs** — logically belongs with worker-side execution infrastructure, not the dispatch path.

4. **for_tenant() in sync mode is a no-op** — sync mode runs the job in the current tokio task, which already inherits the caller's tenant context. A debug log documents this behavior.

5. **captured_tenant_id() precedence** — explicit override (for_tenant) > auto-capture hook > None. Allows admin/CLI overrides to work correctly regardless of ambient context.

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- FOUND: ferro-queue/src/job.rs
- FOUND: ferro-queue/src/error.rs
- FOUND: ferro-queue/src/worker.rs
- FOUND: ferro-queue/src/dispatcher.rs
- FOUND: ferro-queue/src/lib.rs
- FOUND commit: f8a1906 (Task 1)
- FOUND commit: a36a9b9 (Task 2)
