# Phase 97: Tenant-Aware Background Jobs - Research

**Researched:** 2026-03-11
**Domain:** ferro-queue context propagation, Rust task-local storage, cross-crate dependency injection
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Context Propagation**
- `tenant_id: Option<i64>` added to `JobPayload` — transparent envelope field like queue/attempts/created_at
- Auto-captured from `current_tenant()` at dispatch time — zero developer effort for tenant jobs
- ID-only in payload — no slug/name/plan stored (avoids stale data, keeps payload small)
- `.for_tenant(tenant_id: i64)` on `PendingDispatch` to override auto-captured value (admin/system dispatching on behalf of a tenant)
- Auto-capture always applies — no `.without_tenant()` opt-out. System jobs dispatched outside tenant scope simply get `None`

**Worker Scope Restoration**
- Worker restores full `TenantContext` via `TenantLookup` before calling `handle()` — wraps execution in `with_tenant_scope()`
- `Worker::with_tenant_lookup()` builder method to inject a `TenantLookup` implementation — follows existing builder patterns
- If tenant_id present but tenant not found (deleted/inactive): job fails with clear error, goes through normal retry/failed flow
- If worker has no `TenantLookup` configured: ignores tenant_id, runs job without tenant context (backward compatible)

**Cross-Tenant Jobs**
- Single `Job` trait with optional tenant — no `TenantJob` sub-trait
- Shared queues — all jobs go to same queue(s), tenant_id is metadata in payload
- No per-tenant queue isolation in v1
- No job query/filtering by tenant_id in v1 — admin tooling is a future concern

**Dispatch Ergonomics**
- Context capture injected from framework side — ferro-queue gains a generic context capture hook, framework provides tenant-specific implementation
- Existing `dispatch(job).await` calls work unchanged — tenant awareness is fully invisible
- Only new API surface: `.for_tenant(id)` on `PendingDispatch`
- No changes needed to Phase 96 make:stripe templates — auto-capture handles it
- Worker adds tenant_id to tracing span when processing tenant jobs — structured log visibility

### Claude's Discretion
- Generic context capture hook design (trait vs closure vs OnceLock<fn>)
- `TenantLookup` caching strategy in worker (reuse `DbTenantLookup`'s moka cache vs separate)
- Exact error type for tenant-not-found job failures
- Whether `with_tenant_lookup()` is feature-gated behind "tenant" or always available

### Deferred Ideas (OUT OF SCOPE)
- Per-tenant queue isolation / rate limiting — future phase if noisy-neighbor becomes an issue
- Job query/filtering by tenant_id — admin dashboard tooling
- Tenant-aware job metrics/monitoring
</user_constraints>

---

## Summary

Phase 97 propagates the tenant identity that already exists in HTTP request scope into the background job execution pipeline. The existing infrastructure makes this tractable: `tokio::task_local!` scoping via `with_tenant_scope()` and `TENANT_CONTEXT.scope()` is already used by the HTTP request path, and `TenantLookup` / `DbTenantLookup` already provides a `find_by_id()` method with moka caching that the worker can reuse without any new lookup logic.

The central design challenge is that `ferro-queue` is a leaf crate — it has no dependency on `framework` (ferro-rs). The generic context capture hook must not introduce a circular dependency. The cleanest solution matching existing project patterns (validation bridge, stripe client, notification config) is an `OnceLock<fn() -> Option<i64>>` in `ferro-queue` itself, registered by the framework during bootstrap. The worker side requires `TenantLookup` to be an `Arc<dyn TenantLookup>` stored on the `Worker` struct — this is purely a `ferro-queue` → `ferro-rs` concern resolved via a conditional dependency or a duplicated lightweight trait.

The key insight is that `ferro-queue`'s `Worker::register()` handler closure already receives only the serialized `String` data — tenant context must be captured from the `JobPayload` struct (which lives in `ferro-queue`) and used to call the context hook before the handler executes. This is all within `ferro-queue`; no framework layer is needed at execution time beyond what is injected at startup.

**Primary recommendation:** Use `OnceLock<fn() -> Option<i64>>` for dispatch-time context capture (matches the validation bridge pattern), and `Option<Arc<dyn TenantLookup>>` on `Worker` for execution-time context restoration (declared as a trait in `ferro-queue`, implemented in the framework, injected via builder).

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio::task_local!` | tokio 1.x (workspace) | Per-task context storage | Already used by TENANT_CONTEXT, SESSION_CONTEXT, LANG_CONTEXT |
| `std::sync::OnceLock` | std | Single-init global hook storage | Used by GLOBAL_CONNECTION, STRIPE_CLIENT, VALIDATION_TRANSLATOR |
| `moka` sync cache | 0.12 (workspace) | `TenantLookup` result caching | Already in DbTenantLookup — no new dependency needed |
| `async-trait` | 0.1 (workspace) | Trait object async fn | Already used by TenantLookup, Job |
| `tracing` | 0.1 (workspace) | Structured span with tenant_id | Already in worker.rs tracing calls |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `Arc<RwLock<Option<TenantContext>>>` | std/tokio | Tenant scope cell pattern | Same pattern as existing `tenant_scope()` helper |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `OnceLock<fn() -> Option<i64>>` | Trait object hook | OnceLock fn is simpler, no heap allocation, matches existing bridge pattern |
| `OnceLock<fn() -> Option<i64>>` | Tokio `task_local!` in ferro-queue | Would require ferro-queue to depend on the task-local defined in framework — circular |
| Duplicate `WorkerTenantLookup` trait in ferro-queue | Re-export framework's `TenantLookup` | Trait re-export adds framework as a compile dependency of ferro-queue; duplicating a 2-method trait avoids that |

---

## Architecture Patterns

### Recommended Project Structure

Changes are additive to existing files. No new files unless a `WorkerTenantLookup` trait is extracted:

```
ferro-queue/src/
├── job.rs          # Add: tenant_id: Option<i64> to JobPayload
├── dispatcher.rs   # Add: for_tenant(i64), auto-capture hook call, OnceLock registration fn
├── worker.rs       # Add: tenant_lookup field, with_tenant_lookup(), scope-wrap in process_job()
├── error.rs        # Add: TenantNotFound variant
└── lib.rs          # Add: re-export new hook registration fn, WorkerTenantLookup trait

framework/src/
├── tenant/context.rs  # Visibility bump: pub(crate) → pub for with_tenant_scope / tenant_scope
└── lib.rs             # Re-export: worker hook registration fn; Worker already re-exported
```

### Pattern 1: OnceLock Context Capture Hook

Used by `ferro-queue/src/dispatcher.rs` to read `current_tenant()` at dispatch time without knowing about the framework.

```rust
// ferro-queue/src/dispatcher.rs
use std::sync::OnceLock;

/// Global hook: called at dispatch time to capture current context ID.
/// Returns None when outside any tenant scope (system jobs).
static TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>> = OnceLock::new();

/// Register the tenant capture hook.
/// Called once during application bootstrap by the framework.
/// Silently ignores re-registration (matches VALIDATION_TRANSLATOR pattern).
pub fn register_tenant_capture_hook(f: fn() -> Option<i64>) {
    let _ = TENANT_ID_HOOK.set(f);
}

impl<J> PendingDispatch<J> {
    fn captured_tenant_id(&self) -> Option<i64> {
        // Explicit override wins; then auto-capture; then None
        self.tenant_id.or_else(|| {
            TENANT_ID_HOOK.get().and_then(|f| f())
        })
    }
}
```

**Source:** Matches `VALIDATION_TRANSLATOR` in `framework/src/validation/bridge.rs` (line 20) and `STRIPE_CLIENT` in `ferro-stripe/src/client.rs` (lines 4-5). HIGH confidence.

### Pattern 2: Worker TenantLookup Trait (Duplicated in ferro-queue)

To avoid making `ferro-queue` depend on `framework`, define a minimal trait inside `ferro-queue` that mirrors `TenantLookup`. The framework's concrete `DbTenantLookup` implements it (blanket impl or manual impl declared in framework).

```rust
// ferro-queue/src/worker.rs  (or a new worker_tenant.rs)

/// Minimal trait for tenant lookup — implemented by ferro_rs::DbTenantLookup.
/// Declared here to avoid a circular crate dependency.
#[async_trait]
pub trait WorkerTenantLookup: Send + Sync {
    async fn find_tenant_by_id(&self, id: i64) -> Option<WorkerTenantContext>;
}

/// Minimal tenant data needed by the worker — only id needed to build scope.
pub struct WorkerTenantContext {
    pub id: i64,
}
```

**Alternative simpler approach:** Store `Arc<dyn TenantLookup>` directly on `Worker` by making `ferro-queue` depend on `framework/tenant/lookup.rs` as a sub-crate — but the workspace structure does not have a standalone tenant crate. Trait duplication is therefore the correct approach.

**Simpler yet:** Since `with_tenant_scope()` only needs an `Arc<RwLock<Option<TenantContext>>>` and a future, and the worker can invoke this via an injected `Arc<dyn Fn(i64) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>> + Send + Sync>`, we avoid even needing a trait. This closure pattern matches `JobHandler` already in `worker.rs`.

```rust
// ferro-queue/src/worker.rs
type TenantLookupFn = Arc<
    dyn Fn(i64) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>>
    + Send + Sync
>;
```

But this requires `TenantContext` from framework in ferro-queue — still circular. The cleanest solution that avoids any new type crossing the boundary: store a `Arc<dyn WorkerTenantScope>` where the trait's single method takes `i64` and returns a boxed future that wraps the job execution inside the scope, receiving the job handler closure as a second argument.

**Final recommendation (Claude's discretion):** Use a single-method wrapper trait in ferro-queue:

```rust
// ferro-queue/src/worker.rs
use std::future::Future;
use std::pin::Pin;

/// Injects tenant scope around a job closure.
/// Implemented by the framework — injected at startup via Worker::with_tenant_scope().
#[async_trait]
pub trait TenantScopeProvider: Send + Sync {
    /// Run `f` within a tenant scope for the given id, or run without scope if id not found.
    async fn with_scope(
        &self,
        tenant_id: i64,
        f: Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>,
    ) -> Result<(), crate::Error>;
}
```

The framework provides a concrete implementation that calls `TenantLookup::find_by_id()` then `with_tenant_scope()`. This keeps ALL tenant-type knowledge in the framework crate and exposes only `i64` IDs across the boundary.

### Pattern 3: Worker Builder Method

Follows existing consuming builder pattern in `WorkerConfig` and the precedent of `Worker::new()`:

```rust
// ferro-queue/src/worker.rs
pub struct Worker {
    connection: QueueConnection,
    config: WorkerConfig,
    handlers: HashMap<String, JobHandler>,
    semaphore: Arc<Semaphore>,
    shutdown: Arc<tokio::sync::Notify>,
    // NEW: optional tenant scope provider
    tenant_scope: Option<Arc<dyn TenantScopeProvider>>,
}

impl Worker {
    /// Inject a tenant scope provider.
    /// When set, jobs with a tenant_id in their payload are executed
    /// inside a tenant context scope.
    pub fn with_tenant_lookup(mut self, provider: Arc<dyn TenantScopeProvider>) -> Self {
        self.tenant_scope = Some(provider);
        self
    }
}
```

### Pattern 4: process_job Scope Wrapping

The critical change in `Worker::process_job()` — the closure passed to `tokio::spawn` currently calls `handler(payload.data.clone()).await`. With tenant awareness:

```rust
async fn process_job(&self, payload: JobPayload) -> Result<(), Error> {
    let permit = self.semaphore.clone().acquire_owned().await.unwrap();
    let connection = self.connection.clone();
    let handlers = self.handlers.clone();
    let job_type = payload.job_type.clone();
    let job_id = payload.id;
    let tenant_scope = self.tenant_scope.clone(); // NEW
    let tenant_id = payload.tenant_id;            // NEW

    tokio::spawn(async move {
        let _permit = permit;

        // NEW: add tenant_id to span
        let span = tracing::info_span!(
            "process_job",
            job_id = %job_id,
            job_type = &job_type,
            tenant_id = tenant_id,
        );
        let _enter = span.enter();

        let handler = match handlers.get(&job_type) { /* ... */ };

        let job_fut = handler(payload.data.clone());

        // NEW: wrap in tenant scope if configured and tenant_id is present
        let result = match (tenant_scope.as_ref(), tenant_id) {
            (Some(scope), Some(id)) => scope.with_scope(id, job_fut).await,
            _ => job_fut.await,
        };

        match result { /* existing error handling */ }
    });
    Ok(())
}
```

### Pattern 5: JobPayload tenant_id Field

Adding to the existing struct — backward compatible because `#[serde(default)]` handles old payloads in Redis that lack the field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPayload {
    pub id: Uuid,
    pub job_type: String,
    pub data: String,
    pub queue: String,
    pub attempts: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
    pub available_at: DateTime<Utc>,
    pub reserved_at: Option<DateTime<Utc>>,
    /// Tenant this job belongs to, if any.
    #[serde(default)]
    pub tenant_id: Option<i64>,  // NEW
}
```

### Pattern 6: PendingDispatch for_tenant

```rust
pub struct PendingDispatch<J> {
    job: J,
    queue: Option<&'static str>,
    delay: Option<Duration>,
    tenant_id: Option<i64>,  // NEW: explicit override (None = use auto-capture)
}

impl<J> PendingDispatch<J> {
    /// Override the auto-captured tenant ID.
    /// Use when dispatching jobs on behalf of a tenant from a non-tenant-scoped context
    /// (e.g., admin actions, CLI commands, system webhooks).
    pub fn for_tenant(mut self, tenant_id: i64) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}
```

### Anti-Patterns to Avoid

- **Storing full TenantContext in payload:** Slug/name/plan go stale. Only `i64` ID in payload, always re-fetch at execution time.
- **Adding framework as a dependency of ferro-queue:** Creates a circular dependency. Use the `TenantScopeProvider` trait boundary.
- **Skipping `#[serde(default)]` on `tenant_id`:** Existing jobs in Redis have no `tenant_id` field. Without `default`, deserialization will fail.
- **Panicking when tenant not found:** Worker must fail the job cleanly (through normal retry/failed flow) rather than panicking. `TenantScopeProvider::with_scope()` returns `Result<(), Error>` so a `TenantNotFound` error flows through the existing retry machinery.
- **`with_tenant_scope()` and `tenant_scope()` are `pub(crate)` today:** The worker's `TenantScopeProvider` implementation lives in the framework, which is in the same crate — no visibility change needed for the worker approach. The framework's impl of `TenantScopeProvider` uses these internal functions directly.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tenant lookup caching | Custom HashMap + Mutex | `DbTenantLookup` (moka, 5-min TTL) | Already implemented with invalidation support |
| Task-local context storage | Custom thread-local | `tokio::task_local!` + `TENANT_CONTEXT.scope()` | Tokio-aware, works across await points, already proven in HTTP path |
| Global hook registration | Custom init ordering | `OnceLock::set()` with silent ignore | Pattern already in `VALIDATION_TRANSLATOR`, `STRIPE_CLIENT` |
| Job failure routing | Custom error tracking | Existing `connection.fail()` / `connection.release()` | Already handles retry backoff and failed queue |

**Key insight:** Every primitive needed (task-local, OnceLock, moka cache, TenantLookup) is already in the codebase. This phase is pure wiring.

---

## Common Pitfalls

### Pitfall 1: Circular Crate Dependency
**What goes wrong:** `ferro-queue` imports `framework` types (`TenantContext`, `TenantLookup`) to restore scope. Since `framework` already depends on `ferro-queue`, this creates a cycle that `cargo` will reject at compile time.
**Why it happens:** It's tempting to directly use `framework::tenant::TenantLookup` in worker.rs.
**How to avoid:** Define a `TenantScopeProvider` trait in `ferro-queue` that takes only primitive types (`i64`) across the boundary. The framework provides the concrete impl.
**Warning signs:** Compiler error "cycle detected when computing the crate graph".

### Pitfall 2: Deserialization Failure for Existing Jobs
**What goes wrong:** Adding `tenant_id` to `JobPayload` without `#[serde(default)]` causes `serde_json::from_str()` to fail on any job payload already sitting in Redis (no `tenant_id` key in the JSON).
**Why it happens:** `serde` requires all fields by default.
**How to avoid:** Always annotate new nullable/optional fields in serialized structs with `#[serde(default)]`.
**Warning signs:** `DeserializationFailed` errors on existing jobs immediately after deploy.

### Pitfall 3: tenant_scope / with_tenant_scope Visibility
**What goes wrong:** `tenant_scope()` and `with_tenant_scope()` are currently `pub(crate)` in `framework/src/tenant/context.rs`. The `TenantScopeProvider` impl (also in `framework`) uses them — so no visibility change is needed. But if someone tries to implement the trait outside the framework crate, they cannot.
**Why it happens:** The worker injects a framework-provided impl, not a user-provided one in external code.
**How to avoid:** Keep these functions `pub(crate)`. The `TenantScopeProvider` impl is inside `framework`, so access is fine.
**Warning signs:** Attempting to implement `TenantScopeProvider` outside `ferro-rs` and hitting "function is private" errors.

### Pitfall 4: tokio::spawn Loses Task-Local Context
**What goes wrong:** `process_job()` spawns a new tokio task via `tokio::spawn`. Task-locals are NOT inherited by spawned tasks. If the scope wrapping is done in the calling task before the spawn, the spawned task has no context.
**Why it happens:** `tokio::task_local!` scopes are per-task and do not cross spawn boundaries.
**How to avoid:** The `TenantScopeProvider::with_scope()` call must happen INSIDE the `tokio::spawn` closure, not outside it. The `tenant_id` (a plain `i64`) can cross the spawn boundary freely.
**Warning signs:** `current_tenant()` returns `None` inside job handlers even when tenant_id is correctly read from `JobPayload`.

### Pitfall 5: Clone of Worker Loses Tenant Scope
**What goes wrong:** `Worker::clone()` explicitly creates a new instance with `handlers: HashMap::new()` (handlers can't be cloned). Currently the clone impl also resets handlers — if `tenant_scope` is added to `Worker` and not handled in the `Clone` impl, the clone loses tenant lookup.
**Why it happens:** The existing `Clone` impl is manual and was written before `tenant_scope` existed.
**How to avoid:** Update the `Clone` impl to also clone `self.tenant_scope.clone()` (since `Arc` is `Clone`).
**Warning signs:** Worker clones process jobs without any tenant scope restoration.

### Pitfall 6: Sync Mode in dispatcher skips payload construction
**What goes wrong:** `dispatch_immediately()` (sync mode) calls `self.job.handle().await` directly without creating a `JobPayload`. The tenant_id capture and `for_tenant()` override have nowhere to go. Sync mode runs the job directly in the current task, which already has the task-local tenant context — so `current_tenant()` works without scope restoration. But if `.for_tenant(id)` is called in sync mode, the override is silently ignored.
**Why it happens:** Sync mode bypasses the queue entirely.
**How to avoid:** In sync mode, if an explicit `tenant_id` override via `for_tenant()` is set and differs from `current_tenant()`, wrap the `handle()` call in a new tenant scope using the hook. If no explicit override, current task context is correct already.
**Warning signs:** `.for_tenant(id)` in sync mode doesn't change which tenant the job sees — causes test confusion.

---

## Code Examples

Verified patterns from existing codebase:

### Existing OnceLock Hook Pattern (VALIDATION_TRANSLATOR)
```rust
// Source: framework/src/validation/bridge.rs:20
pub(crate) static VALIDATION_TRANSLATOR: OnceLock<TranslatorFn> = OnceLock::new();

// Registration (called once at bootstrap):
pub fn register_validation_translator(f: TranslatorFn) {
    let _ = VALIDATION_TRANSLATOR.set(f);  // Silently ignores re-registration
}
```

Apply the same pattern in `ferro-queue/src/dispatcher.rs` for `TENANT_ID_HOOK`.

### Existing Task-Local Scope Pattern
```rust
// Source: framework/src/tenant/context.rs:55
pub(crate) async fn with_tenant_scope<F, R>(ctx: Arc<RwLock<Option<TenantContext>>>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    TENANT_CONTEXT.scope(ctx, f).await
}

// Usage in TenantMiddleware:
let scope = tenant_scope();
{ let mut guard = scope.write().await; *guard = Some(ctx); }
with_tenant_scope(scope, next(request)).await
```

The `TenantScopeProvider` framework impl replicates this pattern with a `find_by_id()` call first.

### Existing JobHandler Closure Pattern
```rust
// Source: ferro-queue/src/worker.rs:53-54
type JobHandler =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>;
```

The `TenantScopeProvider` trait method signature mirrors this functional style.

### Existing Builder Pattern on Worker
```rust
// Source: ferro-queue/src/worker.rs:46-50 (WorkerConfig)
pub fn max_jobs(mut self, max: usize) -> Self {
    self.max_jobs = max;
    self
}
```

`with_tenant_lookup(mut self, ...) -> Self` follows the same consuming builder pattern.

### Framework Bootstrap Registration Point
```rust
// Source: framework/src/app.rs — bootstrap_fn is async
// Registration happens inside the user-provided bootstrap closure:
Application::new()
    .bootstrap(|_| async move {
        let lookup = Arc::new(DbTenantLookup::new(...));
        let worker = Worker::new(Queue::connection(), WorkerConfig::default())
            .with_tenant_lookup(Arc::new(FrameworkTenantScopeProvider::new(lookup)));
        register_tenant_capture_hook(|| {
            current_tenant().map(|t| t.id)
        });
    })
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Job handlers with no context | Task-local tenant context propagation | Phase 97 | Jobs can call `current_tenant()`, `TenantScope` |
| Manual tenant_id params on every job struct | Invisible envelope field in `JobPayload` | Phase 97 | Zero job-specific boilerplate |

**No deprecated items introduced:** This phase only adds fields and new types, does not remove or replace existing API.

---

## Open Questions

1. **`TenantScopeProvider` or inline closure stored as `Arc<dyn Fn>`?**
   - What we know: Both approaches avoid circular deps. `Arc<dyn Fn(i64, ...) -> ...>` matches `JobHandler` pattern. A named trait is more readable in docs.
   - What's unclear: Which the project owner prefers stylistically.
   - Recommendation: Use a named trait (`TenantScopeProvider`) for discoverability and testability. Easy to mock in tests.

2. **Feature-gating `with_tenant_lookup()` behind `"tenant"` feature?**
   - What we know: `ferro-queue` currently has no features. The method and its type bound would always compile, just never called if not configured.
   - What's unclear: Whether the project wants `ferro-queue` to remain feature-free.
   - Recommendation: Always compile (no feature gate). The method is a no-op if not called, and the trait compiles to zero overhead when unused. Simpler for CI.

3. **Sync mode + `for_tenant()` override**
   - What we know: Sync mode calls `handle()` directly without payload; the task already has the calling task's tenant context.
   - Recommendation: In sync mode, if `self.tenant_id` (explicit override) is `Some`, wrap `handle()` in a new scope using the capture hook registration mechanism reversed (or just document that `.for_tenant()` in sync mode is a no-op and the current task context applies). Keep it simple — sync mode is dev/test only.

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` |
| Config file | none — `cargo test --all-features` |
| Quick run command | `cargo test -p ferro-queue --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| ID | Behavior | Test Type | Automated Command | File Exists? |
|----|----------|-----------|-------------------|-------------|
| — | `JobPayload` serializes with `tenant_id: null` when absent | unit | `cargo test -p ferro-queue job::tests` | Wave 0 (new test in job.rs) |
| — | `JobPayload` serializes and deserializes `tenant_id: Some(42)` | unit | `cargo test -p ferro-queue job::tests` | Wave 0 |
| — | Old payload JSON (no `tenant_id` field) deserializes to `tenant_id: None` | unit | `cargo test -p ferro-queue job::tests` | Wave 0 |
| — | `PendingDispatch::for_tenant(42)` stores tenant_id override | unit | `cargo test -p ferro-queue dispatcher::tests` | Wave 0 |
| — | Auto-capture hook returns captured id at dispatch time | unit | `cargo test -p ferro-queue dispatcher::tests` | Wave 0 |
| — | `Worker::with_tenant_lookup()` stores provider on builder | unit | `cargo test -p ferro-queue worker::tests` | Wave 0 |
| — | `process_job()` with `tenant_id = Some(1)` calls `with_scope(1, ...)` | unit | `cargo test -p ferro-queue worker::tests` | Wave 0 |
| — | `process_job()` with no `TenantScopeProvider` runs job without scope | unit | `cargo test -p ferro-queue worker::tests` | Wave 0 |
| — | `TenantScopeProvider` returns `TenantNotFound` error when id not found → job fails | unit | `cargo test -p ferro-queue worker::tests` | Wave 0 |
| — | `Worker::clone()` preserves `tenant_scope` field | unit | `cargo test -p ferro-queue worker::tests` | Wave 0 |
| — | Framework `register_tenant_capture_hook` called once; double-registration is silent no-op | unit | `cargo test -p ferro-queue dispatcher::tests` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-queue --all-features`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`

### Wave 0 Gaps
- [ ] New test cases in `ferro-queue/src/job.rs` — covers tenant_id serialization / backward compat
- [ ] New test cases in `ferro-queue/src/dispatcher.rs` — covers hook registration + for_tenant
- [ ] New test cases in `ferro-queue/src/worker.rs` — covers TenantScopeProvider injection + scope wrapping

---

## Sources

### Primary (HIGH confidence)
- Direct code read: `ferro-queue/src/job.rs` — `JobPayload` struct and `new()` constructor
- Direct code read: `ferro-queue/src/dispatcher.rs` — `PendingDispatch`, dispatch flow, sync mode
- Direct code read: `ferro-queue/src/worker.rs` — `Worker`, `process_job()`, `Clone` impl
- Direct code read: `ferro-queue/src/error.rs` — `Error` enum, existing variants
- Direct code read: `framework/src/tenant/context.rs` — `TENANT_CONTEXT`, `tenant_scope()`, `with_tenant_scope()`, visibility
- Direct code read: `framework/src/tenant/lookup.rs` — `TenantLookup` trait, `DbTenantLookup`, `find_by_id()`
- Direct code read: `framework/src/tenant/middleware.rs` — how `with_tenant_scope()` is used
- Direct code read: `framework/src/validation/bridge.rs` — `OnceLock` hook registration pattern
- Direct code read: `ferro-queue/src/queue.rs:450` — `GLOBAL_CONNECTION: OnceLock` pattern
- Direct code read: `framework/Cargo.toml` — feature flags, no circular dep today

### Secondary (MEDIUM confidence)
- Tokio documentation (training data, August 2025): `tokio::task_local!` task-local values are NOT inherited by `tokio::spawn`. This is a fundamental design property of task-local storage.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries are already in the workspace, versions confirmed from Cargo.toml files
- Architecture: HIGH — all patterns are verified from existing codebase implementations (bridge.rs, context.rs, worker.rs)
- Pitfalls: HIGH — circular dep, serde default, and task-local inheritance are well-understood Rust/tokio constraints confirmed by code inspection
- Discretion recommendations: MEDIUM — the `TenantScopeProvider` trait vs closure trade-off is a style choice; both work correctly

**Research date:** 2026-03-11
**Valid until:** 2026-04-11 (stable crates, internal codebase)
