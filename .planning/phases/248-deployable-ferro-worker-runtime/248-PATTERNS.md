# Phase 248: Deployable `ferro worker` Runtime — Pattern Map

**Mapped:** 2026-08-14
**Files analyzed:** 7 (2 new, 5 modified)
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-queue/tests/worker_runtime.rs` | test | event-driven (worker drain) | `ferro-queue/tests/race_claim_sqlite.rs` | exact |
| `framework/tests/worker_boot.rs` | test | request-response (boot path) | `framework/tests/offload_delta_broadcast.rs` | role-match |
| `framework/src/app.rs` | boot orchestration | request-response | self (lines 399–470, `run_server_internal`) | exact (self-refactor) |
| `app/src/main.rs` | CLI wiring | request-response | self (lines 70–147, `Commands` enum + `main`) | exact (self-extend) |
| `ferro-macros/src/offload.rs` + `service.rs` | proc-macro | transform | self (lines 183–200 of `service.rs`, lines 222–381 of `offload.rs`) | exact (self-extend) |
| `ferro-queue/src/db.rs` | registry API | CRUD | self (lines 75–109, `has_registered_jobs` + `JobRegistrarEntry`) | exact (self-extend) |
| `app/src/bootstrap.rs` (line 186) | boot wiring | request-response | `framework/tests/offload_delta_broadcast.rs` lines 371–375 | role-match |

---

## Pattern Assignments

### `ferro-queue/tests/worker_runtime.rs` (test, event-driven)

**Analog:** `/Users/alberto/repositories/albertogferrario/ferro/ferro-queue/tests/race_claim_sqlite.rs`

**Test structure pattern** (lines 1–99 of analog):

```rust
// OnceLock guard: all scenarios in ONE #[tokio::test] (not separate fns).
// Pitfall 3: multiple #[tokio::test] fns collapse into one suite fn; a
// name filter silently matches 0 tests and exits 0.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn worker_runtime_suite() {
    // One outer test; named inner async fns per scenario.
    worker_consumes_only_selected_queue().await;
    two_worker_loops_split_work_without_duplicates().await;
    queue_fault_isolation().await;
}
```

**SQLite fixture pattern** (lines 36–43 of analog):

```rust
// CRITICAL: NamedTempFile, NOT sqlite::memory:
// Per-connection in-memory DBs see different tables; cross-connection
// concurrency tests vacuously pass (both workers claim 0 jobs).
let db_file = tempfile::NamedTempFile::new().unwrap();
let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
let conn1 = Database::connect(&db_url).await.unwrap();
let conn2 = Database::connect(&db_url).await.unwrap();
TestMigrator::up(&conn1, None).await.unwrap();
```

**Inline migrator pattern** (lines 19–26 of analog):

```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(CreateJobsTable)]
    }
}
```

**Enqueue + concurrent drain pattern** (lines 47–99 of analog):

```rust
const N: usize = 20;
let now = chrono::Utc::now();
for _ in 0..N {
    enqueue(&conn1, "default", "TestJob", "{}", 3, None, None, None, now)
        .await
        .expect("enqueue failed");
}

async fn drain(conn: sea_orm::DatabaseConnection, worker_id: &'static str,
               out: Arc<Mutex<Vec<i64>>>) {
    loop {
        match claim(&conn, "default", worker_id).await {
            Ok(Some(row)) => { out.lock().unwrap().push(row.id); let _ = delete_job(&conn, row.id).await; }
            Ok(None) => break,
            Err(e) => panic!("claim error: {e:?}"),
        }
    }
}

let (h1, h2) = (
    tokio::spawn(drain(conn1, "w1", c1.clone())),
    tokio::spawn(drain(conn2, "w2", c2.clone())),
);
let _ = tokio::join!(h1, h2);

let unique: HashSet<i64> = all.iter().cloned().collect();
assert_eq!(unique.len(), all.len(), "a job was claimed more than once");
assert_eq!(unique.len(), N, "not all jobs claimed exactly once");
```

**SC#1 adaptation note:** SC#1 (worker consumes only selected queue) requires enqueuing
on two distinct queues (`"reports"`, `"default"`) and asserting only one queue's jobs
were claimed. Use `WorkerConfig::new(vec!["reports".to_string()])` — the exact
`WorkerConfig::new` signature is at `ferro-queue/src/worker.rs:80`.

**SC#3 synchronization note:** Use `tokio::sync::Barrier` or a channel rather than
`tokio::time::sleep` to coordinate slow vs. fast queue workers. A `Barrier::new(2)`
lets both workers start simultaneously without a time dependency.

**Imports block to copy:**

```rust
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use ferro_queue::{claim, delete_job, enqueue, CreateJobsTable, WorkerConfig, WorkerLoop};
```

**Error type note:** `ferro-queue` uses its own `Error` type from `ferro-queue/src/error.rs`.
Tests `expect("…")` / `unwrap()` on `Ok` paths; `panic!` on `Err` paths. No `?` in test fns.

---

### `framework/tests/worker_boot.rs` (test, request-response)

**Analog:** `/Users/alberto/repositories/albertogferrario/ferro/framework/tests/offload_delta_broadcast.rs`

**Top-level test structure pattern** (lines 364–385 of analog):

```rust
extern crate ferro_rs as ferro;  // standard framework test crate alias

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial]  // protects OnceLocks shared across tests in the crate
async fn worker_boot_suite() {
    // Scenario 1: WR-01 — transport URL + feature on → with_transport called
    transport_url_attaches_redis_transport().await;
    // Scenario 2: D-07 — feature off + URL set → warn! fires, no panic
    transport_url_no_feature_warns().await;
}
```

**Feature-gated test block pattern** (lines 391–473 of analog):

```rust
#[cfg(feature = "redis-transport")]
mod redis_tests {
    use super::*;

    fn redis_url() -> Option<String> {
        std::env::var("REDIS_URL").ok().filter(|s| !s.is_empty())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[serial_test::serial]
    async fn redis_cross_replica() {
        let Some(url) = redis_url() else {
            eprintln!("REDIS_URL not set — skipping redis_cross_replica");
            return;
        };
        // ... test body
    }
}
```

**Broadcaster construction pattern for WR-01 test** (lines 370–378 of analog):

```rust
use ferro_broadcast::{transport::memory::InMemoryTransport, Broadcaster};
use std::sync::Arc;

let bus = Arc::new(InMemoryTransport::new(64));
let broadcaster_a = Arc::new(Broadcaster::new().with_transport(bus.clone()));
let broadcaster_b = Arc::new(Broadcaster::new().with_transport(bus));
```

**D-07 warning test note:** The `tracing` subscriber must be initialized to capture
`warn!` output. Use `tracing_subscriber::fmt().with_test_writer().init()` inside the
test, wrapped in a `std::panic::catch_unwind` or a `once_cell::sync::Lazy` to avoid
double-init panics across scenarios.

**Imports block to copy:**

```rust
extern crate ferro_rs as ferro;

use ferro_broadcast::{BroadcastConfig, Broadcaster};
use ferro_queue::{Queue, WorkerConfig, WorkerLoop};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
```

---

### `framework/src/app.rs` — boot path refactor (boot orchestration, request-response)

**Analog:** self — `/Users/alberto/repositories/albertogferrario/ferro/framework/src/app.rs` lines 399–470

**Current `run_server_internal` to be factored** (lines 399–470):

```rust
async fn run_server_internal(
    bootstrap_fn: Option<BootstrapFn>,
    routes_fn: Option<Box<dyn FnOnce() -> Router + Send>>,
) {
    if let Some(bootstrap_fn) = bootstrap_fn { bootstrap_fn().await; }          // line 404

    if !ferro_queue::Queue::is_initialized() {                                    // line 411
        let conn = Self::get_database_connection().await;
        let _ = ferro_queue::Queue::init(conn).await;
    }

    match crate::App::get::<ferro_broadcast::Broadcaster>() {                    // line 426
        Some(broadcaster) => {
            crate::offload::register_offload_hooks_with_broadcaster(
                std::sync::Arc::new(broadcaster));
        }
        None => crate::offload::register_offload_hooks(),
    }

    if ferro_queue::Queue::has_registered_jobs() {                               // line 434
        if ferro_queue::QueueConfig::is_sync_mode() { eprintln!("WARNING…"); }
        let config = ferro_queue::WorkerConfig::default();
        let worker = ferro_queue::WorkerLoop::from_registry(config);
        tokio::spawn(async move {
            if let Err(e) = worker.run().await { eprintln!("WorkerLoop exited: {e}"); }
        });
    }

    let router = if let Some(routes_fn) = routes_fn { routes_fn() } else { Router::new() };
    if let Err(e) = Server::from_config(router).run().await {                   // line 466
        eprintln!("Failed to start server: {e}");
        std::process::exit(1);
    }
}
```

**Seam to extract:** everything before `Server::from_config(router).run()` becomes
`run_common_boot(bootstrap_fn, no_worker: bool)`. The WR-01 transport-attach block goes
inside this shared boot fn, AFTER `bootstrap_fn().await` and BEFORE the
`register_offload_hooks*` call.

**WR-01 feature-flag pattern** (derive from `offload_delta_broadcast.rs` lines 391–432):

```rust
// Inside run_common_boot, after bootstrap_fn().await, before hook registration:
#[cfg(feature = "redis-transport")]
{
    if let Some(bc) = crate::App::get::<ferro_broadcast::Broadcaster>() {
        if let Some(ref url) = bc.config().transport_redis_url {
            match ferro_broadcast::transport::redis::RedisTransport::connect(url).await {
                Ok(t) => {
                    let bc2 = bc.with_transport(std::sync::Arc::new(t));
                    crate::App::singleton(bc2.clone());
                    crate::offload::register_offload_hooks_with_broadcaster(
                        std::sync::Arc::new(bc2));
                    return; // hook registered; skip fallback below
                }
                Err(e) => {
                    tracing::warn!(error = %e,
                        "BROADCAST_REDIS_URL set but Redis connect failed — in-process hub only");
                    crate::offload::register_offload_hooks_with_broadcaster(
                        std::sync::Arc::new(bc));
                    return;
                }
            }
        }
    }
}
#[cfg(not(feature = "redis-transport"))]
if let Some(bc) = crate::App::get::<ferro_broadcast::Broadcaster>() {
    if bc.config().transport_redis_url.is_some() {
        tracing::warn!(
            "BROADCAST_REDIS_URL is set but the `redis-transport` feature is disabled \
             — falling back to in-process hub");
    }
    crate::offload::register_offload_hooks_with_broadcaster(std::sync::Arc::new(bc));
} else {
    crate::offload::register_offload_hooks();
}
```

**In-process worker spawn pattern** (lines 434–455 of `run_server_internal`):

```rust
// D-05: serve spawns a worker for ALL registered queues (not just "default").
if !no_worker && ferro_queue::Queue::has_registered_jobs() {
    if ferro_queue::QueueConfig::is_sync_mode() {
        eprintln!("WARNING: queue jobs registered but QUEUE_CONNECTION is sync …");
    }
    let all_queues = ferro_queue::Queue::registered_queue_names(); // new API
    let config = ferro_queue::WorkerConfig::new(all_queues);
    let worker = ferro_queue::WorkerLoop::from_registry(config);
    tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            eprintln!("WorkerLoop exited with error: {e}");
        }
    });
}
```

**New `run_worker` public entry point pattern:**

```rust
pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>) {
    Self::run_common_boot(bootstrap_fn, /*no_worker=*/true).await;
    let effective_queues = if queues.is_empty() {
        ferro_queue::Queue::registered_queue_names()  // D-03: all registered
    } else {
        queues
    };
    let config = ferro_queue::WorkerConfig::new(effective_queues);
    let worker = ferro_queue::WorkerLoop::from_registry(config);
    if let Err(e) = worker.run().await {
        eprintln!("Worker exited with error: {e}");
        std::process::exit(1);
    }
}
```

**Convention notes:**
- `App::singleton(x)` re-registration semantics must be verified by reading
  `framework/src/container.rs` before implementing the WR-01 broadcast swap.
- Keep the `None`-broadcaster fallback path (`register_offload_hooks()`) intact —
  it is valid for headless worker-only deployments with no broadcaster.
- The `run_worker` public API should be exported at the `framework/src/lib.rs`
  re-export level alongside `Server`, `Application`, etc.

---

### `app/src/main.rs` — CLI wiring (CLI, request-response)

**Analog:** self — `/Users/alberto/repositories/albertogferrario/ferro/app/src/main.rs` lines 70–166

**Existing `Commands` enum to extend** (lines 70–103):

```rust
#[derive(Subcommand)]
enum Commands {
    /// Run the web server (default command)
    Serve {
        /// Skip running migrations on startup
        #[arg(long)]
        no_migrate: bool,
        // ADD: --no-worker flag (D-05)
        // #[arg(long)]
        // no_worker: bool,
    },
    // ... db:migrate, db:status, db:rollback, db:fresh, schedule:work, ...
    // ADD: Worker variant (D-01, D-02)
    // /// Run background job consumer
    // Worker {
    //     /// Queues to consume (repeatable; omit for all registered queues)
    //     #[arg(long, action = clap::ArgAction::Append)]
    //     queue: Vec<String>,
    // },
}
```

**Existing `match` arm pattern to extend** (lines 115–146):

```rust
match cli.command {
    None | Some(Commands::Serve { no_migrate: false }) => {
        run_migrations_silent().await;
        run_server().await;
    }
    Some(Commands::Serve { no_migrate: true }) => {
        run_server().await;
    }
    // ADD: Worker arm
    // Some(Commands::Worker { queue }) => {
    //     run_migrations_silent().await;
    //     framework::run_worker(Some(bootstrap::register), queue).await;
    // }
    // ...
}
```

**Existing `run_server` function pattern** (lines 149–166):

```rust
async fn run_server() {
    bootstrap::register().await;
    let router = routes::register();
    Server::from_config(router).run().await.unwrap_or_else(|e| {
        fail_with("Server failed to start", e, &[
            "Check SERVER_HOST and SERVER_PORT in .env",
            "Ensure the port is not already in use",
        ])
    });
}
```

**Clap `ArgAction::Append` note (Assumption A1):** The correct form for accumulating
multiple `--queue` occurrences is `#[arg(long, action = clap::ArgAction::Append)]`.
Verify this compiles against the clap version pinned in `app/Cargo.toml` before
committing — `clap 4.x` supports `ArgAction::Append` on `Vec<String>`.

**Convention notes:**
- `fail_with` (lines 28–40) is the established fatal-error pattern: print context,
  cause, and fix steps; then `std::process::exit(1)`. Use it for the worker arm too.
- `no_worker` changes the `Serve` variant; all existing `Serve { no_migrate }` match
  arms must become `Serve { no_migrate, no_worker }` — no wildcard suppression.

---

### `ferro-macros/src/offload.rs` + `service.rs` — attribute arg parsing (proc-macro, transform)

**Analog:** self — `ferro-macros/src/service.rs` lines 183–200 for the strip loop;
`ferro-macros/src/offload.rs` lines 222–381 for `emit_job_items`.

**Current attribute-strip loop in `service.rs`** (lines 183–200):

```rust
let mut offload_infos: Vec<crate::offload::OffloadMethodInfo> = Vec::new();
for item in &mut item_trait.items {
    if let syn::TraitItem::Fn(method) = item {
        if let Some(pos) = method
            .attrs
            .iter()
            .position(|a| a.path().is_ident("offload"))
        {
            method.attrs.remove(pos);
            match crate::offload::collect_info(&trait_ident, method) {
                Ok(info) => offload_infos.push(info),
                Err(e) => return e.to_compile_error().into(),
            }
        }
    }
}
```

**Required change — parse the queue arg before stripping:** The `attr` at `pos` must
be read BEFORE `method.attrs.remove(pos)`. Extract queue name from it:

```rust
// syn 2.x API (confirmed: ferro-macros/Cargo.toml uses syn = { version = "2" })
// attr.meta.require_path_only() returns Err when there are arguments.
let attr = &method.attrs[pos];
let mut declared_queue: Option<String> = None;
if attr.meta.require_path_only().is_err() {
    // Has arguments: parse key = value pairs.
    if let Err(e) = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("queue") {
            let value: syn::LitStr = meta.value()?.parse()?;
            declared_queue = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("unknown #[offload] argument; expected `queue = \"name\"`"))
        }
    }) {
        return e.to_compile_error().into();
    }
}
method.attrs.remove(pos);
// Then call collect_info — but collect_info signature must also accept Option<String>.
```

**`OffloadMethodInfo` struct change** (lines 48–66 of `offload.rs`):

```rust
pub(crate) struct OffloadMethodInfo {
    pub job_ident: proc_macro2::Ident,
    pub method_ident: proc_macro2::Ident,
    pub field_names: Vec<proc_macro2::Ident>,
    pub field_types: Vec<TokenStream2>,
    field_forwards: Vec<FieldForward>,
    pub is_async: bool,
    pub returns_result: bool,
    pub output_type: TokenStream2,
    // ADD:
    /// Declared queue name. None = "default".
    pub declared_queue: Option<String>,
}
```

**`collect_info` signature change** (line 132):

```rust
// Before: collect_info(trait_ident, method)
// After:
pub(crate) fn collect_info(
    trait_ident: &proc_macro2::Ident,
    method: &TraitItemFn,
    declared_queue: Option<String>,   // ADD
) -> syn::Result<OffloadMethodInfo> {
    // ...
    Ok(OffloadMethodInfo { /* existing fields */, declared_queue })
}
```

**`emit_job_items` emission changes** (lines 222–381 of `offload.rs`):

```rust
// queue field in JobRegistrarEntry inventory::submit! block (lines 365–372):
::ferro::inventory::submit! {
    ::ferro::queue::JobRegistrarEntry {
        register: |w: &mut ::ferro::queue::WorkerLoop| { w.register::<#job_ident>(); },
        name: #job_ident_str,
        queue: #queue_name_tokens,  // ADD — Some("name") or None
    }
}

// .on_queue() in Offloadable default impl (emitted after the struct):
// queue_name_tokens and on_queue_tokens derived as:
let queue_name_tokens: TokenStream2 = match info.declared_queue.as_deref() {
    Some(q) => quote! { Some(#q) },
    None    => quote! { None },
};
let on_queue_tokens: TokenStream2 = match info.declared_queue.as_deref() {
    Some(q) => quote! { .on_queue(#q) },
    None    => quote! {},
};
```

**`::ferro::*`-only emission rule** (from CLAUDE.md + `offload.rs` line 219–221):
All emitted paths must use `::ferro::queue::*`, never `::ferro_queue::*` directly.
String literals (queue name) carry no path — no issue there.

**Trybuild gate reminder:** Any change to `emit_job_items` or the service attribute
loop must be exercised by `cargo test -p ferro-macros`. The trybuild fixtures in
`ferro-macros/tests/` expand the attribute and compare against `.stderr` snapshots;
a regression in emission produces a UI gate failure, not a standard compile error.

---

### `ferro-queue/src/db.rs` — registry API (registry, CRUD)

**Analog:** self — `/Users/alberto/repositories/albertogferrario/ferro/ferro-queue/src/db.rs` lines 75–109

**Pattern to extend — `JobRegistrarEntry` struct** (lines 101–107):

```rust
pub struct JobRegistrarEntry {
    pub register: fn(&mut crate::WorkerLoop),
    pub name: &'static str,
    // ADD:
    /// Declared queue name. `None` means "default".
    pub queue: Option<&'static str>,
}

inventory::collect!(JobRegistrarEntry);
```

**Pattern to add — `Queue::registered_queue_names()`** (modeled on `has_registered_jobs` lines 75–78 and `apply_registrars` lines 80–87):

```rust
/// Derive the distinct set of queue names from all registered job types.
///
/// Used by `serve` (D-05) and `worker` with no `--queue` flag (D-03) to
/// consume all declared queues. Returns at least `["default"]`.
pub fn registered_queue_names() -> Vec<String> {
    use std::collections::BTreeSet;
    let mut names: BTreeSet<String> = BTreeSet::new();
    // Runtime path: manual Queue::register calls have no queue metadata → "default".
    if !JOB_REGISTRARS.lock().unwrap().is_empty() {
        names.insert("default".to_string());
    }
    // Inventory path: compile-time #[offload]-derived entries carry the declared queue.
    for entry in inventory::iter::<JobRegistrarEntry> {
        names.insert(entry.queue.unwrap_or("default").to_string());
    }
    if names.is_empty() {
        names.insert("default".to_string());
    }
    names.into_iter().collect()  // BTreeSet gives deterministic ordering
}
```

**Convention notes:**
- `BTreeSet` over `HashSet` for deterministic ordering — important for test
  reproducibility when the order of queues passed to `WorkerConfig::new` matters.
- `pub fn` (not `pub(crate)`) — `registered_queue_names` is called from
  `framework/src/app.rs` through the crate public API.
- Error type: `ferro-queue/src/error.rs` defines `Error`; `Queue::init` returns
  `Result<(), Error>`. `registered_queue_names` returns `Vec<String>` (no failure path).

---

### `app/src/bootstrap.rs` line 186 — transport wiring removal (boot wiring)

**Analog:** `/Users/alberto/repositories/albertogferrario/ferro/framework/tests/offload_delta_broadcast.rs` lines 370–378

**Current state** (`bootstrap.rs` lines 185–192):

```rust
// Register broadcaster so /_ferro/ws WebSocket and projection hooks share the same instance.
let broadcaster = Broadcaster::with_config(BroadcastConfig::from_env());
App::singleton(broadcaster.clone());

// Wire LiveTestProjection runtime for Phase 260 LiveFragment UAT.
let bc_arc = std::sync::Arc::new(broadcaster);
```

**Required change:** After D-06, the framework's `run_common_boot` calls
`with_transport` on the `App`-registered `Broadcaster`. The bootstrap no longer
needs to do that. The `Broadcaster::with_config(BroadcastConfig::from_env())`
construction and `App::singleton` registration remain — only the manual transport
wiring (if any were added by hand) is removed. Currently `bootstrap.rs:186–187`
does not call `with_transport` at all (the gap WR-01 addresses), so the bootstrap
change is: confirm no hand-wiring was added between the research snapshot and
implementation time, and add a comment that transport wiring is framework-managed.

**Convention notes:**
- `App::singleton(broadcaster.clone())` must remain so the framework's
  `App::get::<Broadcaster>()` call inside `run_common_boot` finds the instance.
- The `bc_arc` construction below it (for `ProjectionRuntime`) must use the
  post-`run_common_boot` broadcaster instance or the one registered by bootstrap —
  check whether `ProjectionRuntime` needs the transport-attached broadcaster.

---

## Shared Patterns

### Test DB Fixture (NamedTempFile)
**Source:** `ferro-queue/tests/race_claim_sqlite.rs` lines 36–43
**Apply to:** `ferro-queue/tests/worker_runtime.rs`, `framework/tests/worker_boot.rs`

```rust
let db_file = tempfile::NamedTempFile::new().unwrap();
let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
// _db_file must stay in scope for the full test — dropping it deletes the file.
```

### OnceLock Collapse Guard (single #[tokio::test])
**Source:** `framework/tests/offload_delta_broadcast.rs` lines 364–385
**Apply to:** both new test files

All scenarios are sub-functions (plain `async fn`, not `#[tokio::test]`) called
sequentially from one outer `#[tokio::test]`. Tables are cleared between scenarios
via `DELETE FROM <table>`. This avoids `Queue already initialized` panics and the
false-green test-collapse pitfall.

### `::ferro::*`-Only Emission (proc-macro)
**Source:** `ferro-macros/src/offload.rs` line 219 comment + CLAUDE.md
**Apply to:** all new emission in `emit_job_items`

Never emit `::ferro_queue::` or `::ferro_broadcast::` directly. All paths go through
the `::ferro::` facade so consumer crates that only depend on `ferro-rs` compile
correctly.

### Worker Loop Spawn Pattern
**Source:** `framework/src/app.rs` lines 449–455
**Apply to:** `framework/src/app.rs` `run_common_boot` and `run_worker`

```rust
tokio::spawn(async move {
    if let Err(e) = worker.run().await {
        eprintln!("WorkerLoop exited with error: {e}");
    }
});
// In run_worker (blocks until SIGTERM):
if let Err(e) = worker.run().await {
    eprintln!("Worker exited with error: {e}");
    std::process::exit(1);
}
```

The in-process worker (serve path) uses `tokio::spawn` (non-blocking). The
deployable worker (`run_worker`) uses direct `.await` (blocks main until shutdown).

### Feature-Gated Redis Block
**Source:** `framework/tests/offload_delta_broadcast.rs` lines 391–432
**Apply to:** WR-01 path inside `framework/src/app.rs`

```rust
#[cfg(feature = "redis-transport")]
{
    // ... RedisTransport::connect(url).await
}
#[cfg(not(feature = "redis-transport"))]
{
    // ... tracing::warn!(...)
}
```

The feature name is `redis-transport` (with hyphen, not underscore) in
`framework/Cargo.toml:32`. The `#[cfg]` attribute uses the hyphen form.

### Builder `with_*` Consuming Self
**Source:** `ferro-queue/src/worker.rs` lines 87–97
**Apply to:** any new builder methods on `WorkerConfig` or `WorkerLoop`

```rust
pub fn max_jobs(mut self, max: usize) -> Self {
    self.max_jobs = max;
    self
}
```

---

## No Analog Found

All Phase 248 files have direct analogs. No file requires fallback to RESEARCH.md
patterns from external documentation.

---

## Metadata

**Analog search scope:** `ferro-queue/tests/`, `framework/tests/`, `framework/src/app.rs`,
`app/src/main.rs`, `app/src/bootstrap.rs`, `ferro-macros/src/service.rs`,
`ferro-macros/src/offload.rs`, `ferro-queue/src/db.rs`, `ferro-queue/src/worker.rs`

**Files read:** 13

**Pattern extraction date:** 2026-08-14

**Key assumptions requiring verification at plan time:**
- A1: `clap::ArgAction::Append` on `Vec<String>` — verify against `app/Cargo.toml`
  pinned clap version (not checked here).
- OQ#3: `App::singleton` re-registration semantics — read `framework/src/container.rs`
  before implementing the WR-01 broadcaster swap.
