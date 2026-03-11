---
phase: 97-tenant-aware-background-jobs
verified: 2026-03-11T14:45:00Z
status: passed
score: 13/13 must-haves verified
re_verification: false
gaps: []
---

# Phase 97: Tenant-Aware Background Jobs Verification Report

**Phase Goal:** Propagate tenant context into ferro-queue jobs so `current_tenant()` and `TenantScope` work inside job handlers. Jobs dispatched from tenant-scoped request handlers automatically carry `tenant_id` through Redis and restore full `TenantContext` in the worker before executing.
**Verified:** 2026-03-11T14:45:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | JobPayload carries tenant_id as an optional envelope field | VERIFIED | `ferro-queue/src/job.rs:99` — `#[serde(default)] pub tenant_id: Option<i64>` |
| 2  | Old payloads without tenant_id deserialize to None (backward compat) | VERIFIED | `job.rs:246-249` — test_old_payload_without_tenant_id_deserializes_to_none passes |
| 3  | PendingDispatch.for_tenant(id) stores explicit tenant override | VERIFIED | `dispatcher.rs:65-68` — `for_tenant()` sets `self.tenant_id = Some(tenant_id)` |
| 4  | Auto-capture hook reads current tenant ID at dispatch time without framework dependency | VERIFIED | `dispatcher.rs:11` — `static TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>>`, consumed in `captured_tenant_id()` at line 73-76 |
| 5  | TenantScopeProvider trait defines the contract for worker-side scope injection | VERIFIED | `worker.rs:20-27` — `#[async_trait] pub trait TenantScopeProvider: Send + Sync` with `with_scope()` method |
| 6  | TenantNotFound error variant exists for job failures when tenant lookup fails | VERIFIED | `error.rs:65-69` — `TenantNotFound { tenant_id: i64 }` with message format |
| 7  | Worker::with_tenant_scope() stores a TenantScopeProvider on the worker | VERIFIED | `worker.rs:108-111` — builder method sets `self.tenant_scope = Some(provider)` |
| 8  | process_job() wraps job execution in tenant scope when tenant_id is present and provider is configured | VERIFIED | `worker.rs:232-238` — `match (&tenant_scope, tenant_id) { (Some(scope), Some(id)) => scope.with_scope(id, job_fut).await, _ => handler(...).await }` |
| 9  | process_job() runs jobs without scope when no TenantScopeProvider is configured (backward compat) | VERIFIED | Same match arms — `_` path calls handler directly |
| 10 | Worker::clone() preserves the tenant_scope field | VERIFIED | `worker.rs:280` — `tenant_scope: self.tenant_scope.clone()` in Clone impl |
| 11 | Framework provides a concrete TenantScopeProvider implementation using TenantLookup + with_tenant_scope() | VERIFIED | `framework/src/tenant/worker.rs:16-47` — `FrameworkTenantScopeProvider` calls `find_by_id()` then `with_tenant_scope()` |
| 12 | register_tenant_capture_hook is called with a closure that reads current_tenant().map(t => t.id) | VERIFIED | `docs/src/features/multi-tenancy.md:250` — documented; `framework/src/lib.rs:162` re-exports the function |
| 13 | Documentation explains tenant-aware job setup | VERIFIED | `docs/src/features/multi-tenancy.md:233-303` — Background Jobs section present with setup, usage, for_tenant(), error behavior |

**Score:** 13/13 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-queue/src/job.rs` | JobPayload with tenant_id: Option<i64> | VERIFIED | Field present at line 99 with `#[serde(default)]`, `with_tenant_id()` builder at line 134, 6 tests covering all serde scenarios |
| `ferro-queue/src/error.rs` | Error::TenantNotFound variant | VERIFIED | Variant at line 66, convenience constructor `tenant_not_found()` at line 94, 2 tests pass |
| `ferro-queue/src/dispatcher.rs` | OnceLock capture hook, for_tenant builder, auto-capture in dispatch | VERIFIED | `TENANT_ID_HOOK` at line 11, `register_tenant_capture_hook()` at line 18, `for_tenant()` at line 65, `captured_tenant_id()` at line 73, `dispatch_to_queue()` applies hook at line 134+141 |
| `ferro-queue/src/lib.rs` | Re-exports of new public API including TenantScopeProvider | VERIFIED | Line 54 exports `register_tenant_capture_hook`, line 61 exports `TenantScopeProvider, Worker, WorkerConfig` |
| `ferro-queue/src/worker.rs` | Worker with tenant scope support including tenant_scope field | VERIFIED | Field at line 86, `with_tenant_scope()` at line 108, `process_job()` scope dispatch at line 232, Clone at line 280, 9 new tests pass |
| `framework/src/tenant/worker.rs` | FrameworkTenantScopeProvider implementation | VERIFIED | Created — full implementation at lines 16-47, 3 tests including `current_tenant_accessible_inside_scope` |
| `framework/src/tenant/mod.rs` | pub mod worker + re-exports | VERIFIED | `pub mod worker` at line 21, `pub use worker::FrameworkTenantScopeProvider` at line 32 |
| `framework/src/lib.rs` | Re-exports of register_tenant_capture_hook, TenantScopeProvider, FrameworkTenantScopeProvider | VERIFIED | Lines 110, 162, 164 — all three present |
| `docs/src/features/multi-tenancy.md` | Tenant-aware background jobs documentation | VERIFIED | Lines 233-303 — Background Jobs section with complete setup guide |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `dispatcher.rs` | `TENANT_ID_HOOK OnceLock` | `captured_tenant_id()` calls hook then stores in JobPayload | WIRED | `captured_tenant_id()` at line 73 calls `TENANT_ID_HOOK.get().and_then(|f| f())`; result applied via `payload.with_tenant_id(tenant_id)` at line 141 |
| `dispatcher.rs` | `job.rs` JobPayload | `JobPayload::new/with_delay` receive tenant_id from PendingDispatch | WIRED | `dispatch_to_queue()` creates payload then calls `.with_tenant_id(tenant_id)` before push |
| `worker.rs (process_job)` | `TenantScopeProvider::with_scope` | match on `(tenant_scope, tenant_id)` | WIRED | Lines 232-238 — match arm `(Some(scope), Some(id))` calls `scope.with_scope(id, job_fut).await` inside `tokio::spawn` |
| `worker.rs (Clone impl)` | `tenant_scope field` | `self.tenant_scope.clone()` | WIRED | Line 280 — `tenant_scope: self.tenant_scope.clone()` |
| `framework/src/tenant/worker.rs` | `framework/src/tenant/context.rs` | Uses `tenant_scope()` + `with_tenant_scope()` | WIRED | Lines 3, 40, 45 — imports and calls both `pub(crate)` functions |
| `framework/src/tenant/worker.rs` | `framework/src/tenant/lookup.rs` | Uses `TenantLookup::find_by_id()` | WIRED | Line 36 — `self.lookup.find_by_id(tenant_id).await` |
| `framework/src/lib.rs` | `ferro-queue/src/dispatcher.rs` | Re-exports `register_tenant_capture_hook` | WIRED | Line 162 — included in ferro-queue re-export block |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TBJ-01 | 97-01 | tenant_id field in JobPayload | SATISFIED | `job.rs:99` — `#[serde(default)] pub tenant_id: Option<i64>` |
| TBJ-02 | 97-01 | Backward-compatible deserialization (old payloads) | SATISFIED | `job.rs:245-249` — test passes for old JSON without tenant_id key |
| TBJ-03 | 97-01 | PendingDispatch::for_tenant() explicit override | SATISFIED | `dispatcher.rs:65` — `for_tenant()` builder method |
| TBJ-04 | 97-01 | OnceLock auto-capture hook, register_tenant_capture_hook() | SATISFIED | `dispatcher.rs:11-20` — TENANT_ID_HOOK + register function |
| TBJ-05 | 97-02 | Worker::with_tenant_scope() stores TenantScopeProvider | SATISFIED | `worker.rs:108` — builder method confirmed working |
| TBJ-06 | 97-02 | process_job() wraps execution in scope when both provider and tenant_id present | SATISFIED | `worker.rs:232-238` — match arm logic verified |
| TBJ-07 | 97-02 | Backward compat: no provider or no tenant_id means direct execution | SATISFIED | `worker.rs:237` — `_ => handler(payload.data.clone()).await` wildcard path |
| TBJ-08 | 97-02 | Worker::clone() preserves tenant_scope | SATISFIED | `worker.rs:280` — Clone impl updated |
| TBJ-09 | 97-03 | FrameworkTenantScopeProvider bridges ferro-queue to framework tenant infrastructure | SATISFIED | `framework/src/tenant/worker.rs:16-47` — full implementation |
| TBJ-10 | 97-03 | Re-exports (register_tenant_capture_hook, TenantScopeProvider, FrameworkTenantScopeProvider) from framework lib.rs | SATISFIED | `framework/src/lib.rs:110,162,164` — all present |
| TBJ-11 | 97-03 | Documentation for tenant-aware background jobs | SATISFIED | `docs/src/features/multi-tenancy.md:233-303` — complete section |

All 11 requirements (TBJ-01 through TBJ-11) are satisfied. No orphaned requirements detected. Requirements from ROADMAP.md match exactly the union of plan frontmatter requirements: [TBJ-01..04] + [TBJ-05..08] + [TBJ-09..11].

---

### Anti-Patterns Found

No anti-patterns detected. Scan covered:
- `ferro-queue/src/job.rs` — clean, no TODOs or empty implementations
- `ferro-queue/src/error.rs` — clean
- `ferro-queue/src/dispatcher.rs` — clean; sync-mode no-op is intentional and documented
- `ferro-queue/src/worker.rs` — clean; `handlers: HashMap::new()` in Clone is existing intentional behavior with comment
- `framework/src/tenant/worker.rs` — clean
- `framework/src/lib.rs` — clean
- `docs/src/features/multi-tenancy.md` — complete documentation

---

### Human Verification Required

None. All truths are verifiable programmatically via code inspection and test execution.

The one behavior that would require a running system — that `current_tenant()` actually returns the correct tenant inside a dispatched background job — is covered by `current_tenant_accessible_inside_scope` in `framework/src/tenant/worker.rs` tests (tokio async test, not requiring Redis).

---

### Test Coverage Summary

| Crate / Module | Tests | Status |
|---------------|-------|--------|
| ferro-queue (total) | 46 | All pass |
| job::tests (tenant-related) | 5 | All pass |
| error::tests | 2 | All pass |
| dispatcher::tests (tenant-related) | 5 | All pass |
| worker::tests (tenant-related) | 9 | All pass |
| framework tenant::worker::tests | 3 | All pass |

---

### Commit Verification

All 6 phase commits exist and are reachable:

| Commit | Plan | Description |
|--------|------|-------------|
| `f8a1906` | 97-01 Task 1 | Add tenant_id to JobPayload, TenantNotFound error, TenantScopeProvider trait |
| `a36a9b9` | 97-01 Task 2 | Add OnceLock capture hook and PendingDispatch::for_tenant |
| `fb73f82` | 97-02 Task 1 | Wire TenantScopeProvider into Worker with scope-wrapped job execution |
| `3682917` | 97-03 Task 1 | Implement FrameworkTenantScopeProvider and wire re-exports |
| `cd70d79` | 97-03 Task 2 | Add tenant-aware background jobs section to multi-tenancy docs |
| `5f7c9c8` | 97-03 | Phase summary docs |

---

### Summary

Phase 97 goal is fully achieved. The end-to-end tenant-aware background job pipeline is complete:

1. **Dispatch-time capture** (`ferro-queue`): `TENANT_ID_HOOK` OnceLock captures the current tenant ID from task-local context at the moment a job is dispatched. `PendingDispatch::for_tenant()` provides an explicit override for admin/system contexts. The captured `tenant_id` is stored in `JobPayload` with `#[serde(default)]` for backward compatibility with existing payloads.

2. **Worker-side scope restoration** (`ferro-queue`): `Worker::with_tenant_scope()` accepts a `Arc<dyn TenantScopeProvider>`. In `process_job()`, when both a provider and a `tenant_id` are present, the job future is wrapped via `scope.with_scope(id, job_fut).await` inside `tokio::spawn` — correctly respecting task-local boundary semantics. The `Clone` impl preserves the provider.

3. **Framework bridge** (`framework`): `FrameworkTenantScopeProvider` implements the `TenantScopeProvider` trait using `TenantLookup::find_by_id()` to restore a full `TenantContext`, then calls `with_tenant_scope()` so `current_tenant()` is available inside job handlers. Returns `TenantNotFound` when lookup fails, flowing through the normal retry/failed-queue machinery.

4. **Public API**: `register_tenant_capture_hook`, `TenantScopeProvider`, and `FrameworkTenantScopeProvider` are all re-exported from `framework/src/lib.rs`. Bootstrap setup is documented in `docs/src/features/multi-tenancy.md`.

---

_Verified: 2026-03-11T14:45:00Z_
_Verifier: Claude (gsd-verifier)_
