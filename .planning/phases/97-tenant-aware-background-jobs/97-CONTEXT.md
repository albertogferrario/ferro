# Phase 97: Tenant-Aware Background Jobs - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Propagate tenant context into ferro-queue jobs so `current_tenant()` and `TenantScope` work inside job handlers with the same API surface as HTTP handlers. Jobs dispatched from tenant-scoped request handlers automatically carry tenant_id through Redis and restore full TenantContext in the worker before executing. No new job traits — tenant awareness is a framework-level concern layered onto the existing Job/Worker infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Context Propagation
- `tenant_id: Option<i64>` added to `JobPayload` — transparent envelope field like queue/attempts/created_at
- Auto-captured from `current_tenant()` at dispatch time — zero developer effort for tenant jobs
- ID-only in payload — no slug/name/plan stored (avoids stale data, keeps payload small)
- `.for_tenant(tenant_id: i64)` on PendingDispatch to override auto-captured value (admin/system dispatching on behalf of a tenant)
- Auto-capture always applies — no `.without_tenant()` opt-out. System jobs dispatched outside tenant scope simply get None

### Worker Scope Restoration
- Worker restores full TenantContext via `TenantLookup` before calling `handle()` — wraps execution in `with_tenant_scope()`
- `Worker::with_tenant_lookup()` builder method to inject a `TenantLookup` implementation — follows existing builder patterns
- If tenant_id present but tenant not found (deleted/inactive): job fails with clear error, goes through normal retry/failed flow
- If worker has no TenantLookup configured: ignores tenant_id, runs job without tenant context (backward compatible)

### Cross-Tenant Jobs
- Single `Job` trait with optional tenant — no TenantJob sub-trait
- Shared queues — all jobs go to same queue(s), tenant_id is metadata in payload
- No per-tenant queue isolation in v1
- No job query/filtering by tenant_id in v1 — admin tooling is a future concern

### Dispatch Ergonomics
- Context capture injected from framework side — ferro-queue gains a generic context capture hook, framework provides tenant-specific implementation
- Existing `dispatch(job).await` calls work unchanged — tenant awareness is fully invisible
- Only new API surface: `.for_tenant(id)` on PendingDispatch
- No changes needed to Phase 96 make:stripe templates — auto-capture handles it
- Worker adds tenant_id to tracing span when processing tenant jobs — structured log visibility

### Claude's Discretion
- Generic context capture hook design (trait vs closure vs OnceLock<fn>)
- TenantLookup caching strategy in worker (reuse DbTenantLookup's moka cache vs separate)
- Exact error type for tenant-not-found job failures
- Whether with_tenant_lookup() is feature-gated behind "tenant" or always available

</decisions>

<specifics>
## Specific Ideas

- Pattern follows the same invisible-propagation approach as distributed tracing context (capture at boundary, restore at processing)
- Stripe webhook handlers (Phase 96) are a primary consumer — they dispatch jobs from tenant-scoped routes and need tenant context restored in workers

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `framework/src/tenant/context.rs`: `tenant_scope()`, `with_tenant_scope()`, `current_tenant()` — worker wraps `handle()` in `with_tenant_scope()`
- `framework/src/tenant/lookup.rs`: `TenantLookup` trait + `DbTenantLookup` with moka cache — worker uses this to restore full TenantContext from ID
- `ferro-queue/src/job.rs`: `JobPayload` — gains `tenant_id: Option<i64>` field
- `ferro-queue/src/dispatcher.rs`: `PendingDispatch` — gains `.for_tenant(id)` and auto-capture hook
- `ferro-queue/src/worker.rs`: `Worker` — gains `with_tenant_lookup()` and scope-wrapping in `process_job()`

### Established Patterns
- Task-local context: `tokio::task_local!` with `Arc<RwLock<Option<T>>>` scoped via `CONTEXT.scope()` — used by session, lang, tenant
- Builder pattern: consuming `mut self -> Self` — Worker already uses this for config
- Feature-gated modules: `#[cfg(feature = "stripe")]` in tenant/mod.rs — tenant-aware worker may follow similar gating
- OnceLock global config: `Queue::connection()`, `Stripe::client()` — context capture hook could use similar pattern

### Integration Points
- `ferro-queue/src/job.rs`: JobPayload struct needs tenant_id field + serialization
- `ferro-queue/src/dispatcher.rs`: PendingDispatch needs context capture hook + for_tenant()
- `ferro-queue/src/worker.rs`: Worker needs with_tenant_lookup() + scope wrapping in process_job()
- `framework/src/lib.rs`: Register tenant context capture hook during Application::run() or similar bootstrap
- `framework/src/tenant/context.rs`: tenant_scope() and with_tenant_scope() may need pub visibility bump for worker usage

</code_context>

<deferred>
## Deferred Ideas

- Per-tenant queue isolation / rate limiting — future phase if noisy-neighbor becomes an issue
- Job query/filtering by tenant_id — admin dashboard tooling
- Tenant-aware job metrics/monitoring

</deferred>

---

*Phase: 97-tenant-aware-background-jobs*
*Context gathered: 2026-03-11*
