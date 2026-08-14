# Phase 248: Deployable `ferro worker` Runtime — Research

**Researched:** 2026-08-14
**Domain:** Rust CLI (clap), ferro-queue worker runtime, ferro-broadcast Redis transport wiring, framework boot factoring, proc-macro attribute parsing
**Confidence:** HIGH — all claims verified by reading current source files

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The worker is the **application's own binary** under a `worker` subcommand (`<app-bin> worker`), NOT a `ferro` CLI subcommand.
- **D-02:** Queue selector is `--queue <name>`, **repeatable** (`worker --queue reports --queue emails`), mapping directly to the existing `WorkerConfig { queues: Vec<String> }` / `WorkerConfig::new(queues)`. No `--class` flag.
- **D-03:** `worker` with **no** `--queue` consumes **all registered queues**. Derives the distinct queue set from the job registry.
- **D-04:** An `#[offload]` method declares its queue via `#[offload(queue = "name")]`, defaulting to `default` when omitted.
- **D-05:** `serve`'s in-process worker consumes **all registered queues**; `serve --no-worker` disables it.
- **D-06:** The **framework** owns transport wiring — at boot, when `BroadcastConfig.transport_redis_url` is set, construct the `RedisTransport` and attach it via `Broadcaster::with_transport(...)`. The sample app's `bootstrap.rs` stops hand-assembling this.
- **D-07:** Feature-off + URL-set → emit a warning and fall back to the in-process hub, no hard failure. With no URL, behaviour unchanged.
- **D-08:** Deploy `workers:` component emission is **deferred** to the deploy-scaffolder line. Phase 248 = worker runtime + queue routing + WR-01 wiring + SC#2/#3 verification tests.

### Claude's Discretion

- Exact factoring of the shared boot path (`run_worker(queues)` seam inside `run_server_internal`).
- How the distinct registered-queue set is derived for D-03 and D-05.
- Test construction for SC#2 (two in-process consumers split work) and SC#3 (fault-domain isolation across disjoint queue sets).

### Deferred Ideas (OUT OF SCOPE)

- Deploy `workers:` scaffolder emission (extend `[package.metadata.ferro.deploy]`).
- Multi-replica operational guidance (OTel metrics, DB_MAX_CONNECTIONS guidance, PgBouncer).
- Autonomous machine lifecycle / scale-to-zero (KEDA, CRIU, Nomad, WASM isolates).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-05 | Offloaded work runs on a deployable `ferro worker` process runnable at N replicas against the shared queue; capacity scales by adding replicas; each worker class is an independent fault domain. No framework-managed autoscaling (N is external). | D-01–D-08 plus boot-path refactor, macro attribute addition, and WR-01 wiring all detailed below. |
</phase_requirements>

---

## Summary

Phase 248 is almost entirely a **wiring and surfacing** phase: the queue mechanism, the worker loop, and the broadcast transport are all fully built (Phases 185, 244–247, 246.1). What is missing is three narrowly scoped additions:

1. A framework-provided shared boot path (DB + queue + offload hooks + broadcast) called by both `serve` and a new `worker` subcommand, so the in-process worker and the deployable worker share one boot surface.
2. A `Worker { queue: Vec<String> }` arm in the scaffolded `app/src/main.rs` `Commands` enum, plus `no_worker: bool` on `Serve`.
3. The WR-01 transport attach: when `BroadcastConfig.transport_redis_url` is set and the `redis-transport` feature is enabled, construct `RedisTransport` and call `Broadcaster::with_transport(...)` inside the shared boot step (currently nothing does this despite the config field being read).

The queue-routing mechanism (D-03/D-04) requires a minor addition: the `#[offload]` macro must accept a `queue = "name"` attribute argument and thread the declared queue into the `PendingDispatch::on_queue()` call inside `Offloadable::offload()`. The default queue name must remain `"default"` when the argument is omitted. The `WorkerConfig` already carries `Vec<String>` and the "all queues" default for D-03/D-05 is achievable by extending `JobRegistrarEntry` with an optional queue field and deriving the distinct set at boot.

**Primary recommendation:** Extract a `run_common_boot(no_worker: bool)` async function inside `framework/src/app.rs` that runs everything up to (but not including) `Server::from_config(router).run()`; then both `run_server_internal` and a new `run_worker_internal(queues)` call it. Keep the WR-01 transport attach inside that shared boot step using a `#[cfg(feature = "redis-transport")]` block.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Worker subcommand parsing | Application binary (`app/src/main.rs`) | Framework scaffold template | clap `Commands` enum lives in the generated app; framework supplies only the runtime entry point |
| Shared boot logic (DB + queue + hooks + broadcast) | Framework (`framework/src/app.rs`) | — | Enforces no-duplicate-control-surface; both `serve` and `worker` call one path |
| WorkerLoop construction and run | `ferro-queue` | Framework (thin caller) | `WorkerLoop::from_registry` already exists; framework just constructs and calls `.run()` |
| Registered-queue-set derivation | `ferro-queue` (new small API) | — | The registry owns job registrations; it should expose the queue set |
| `#[offload(queue = "name")]` parsing | `ferro-macros/src/offload.rs` | — | Proc macro attribute parsing; emits into `::ferro::queue::Offloadable::offload()` |
| Queue declared on `PendingDispatch` | `ferro-queue/src/offload.rs` | — | `Offloadable::offload()` already calls `PendingDispatch::new(self)` — add `.on_queue()` |
| WR-01 transport construction | Framework (`framework/src/app.rs`) | — | Shared boot step; `#[cfg(feature = "redis-transport")]` block reads config field |
| `redis-transport` feature forwarding | `framework/Cargo.toml` | — | Already present: `redis-transport = ["ferro-broadcast/redis-transport"]` |

---

## Standard Stack

### Core (all already in the workspace — no new dependencies)

| Library | Crate | Current role | Phase 248 role |
|---------|-------|-------------|----------------|
| `clap` | `app/Cargo.toml` | Parses existing `Commands` enum | Add `Worker { #[arg(long, action = ArgAction::Append)] queue: Vec<String> }` and `no_worker: bool` on `Serve` |
| `ferro_queue::WorkerLoop` | `ferro-queue` | In-process worker in `run_server_internal` | Called directly in new `run_worker_internal`; `from_registry(WorkerConfig::new(queues))` |
| `ferro_broadcast::transport::redis::RedisTransport` | `ferro-broadcast`, feature `redis-transport` | Referenced only in env-gated test | Constructed in the shared boot step when `transport_redis_url.is_some()` and feature is enabled |
| `ferro_broadcast::Broadcaster::with_transport` | `ferro-broadcast` | Exists at line 90 of `broadcaster.rs` | Called in shared boot step |
| `syn` | `ferro-macros` | Already parses `#[offload]` attributes | Extended to parse `#[offload(queue = "name")]` via `syn::parse::Parser` / `syn::meta` |

### No New Dependencies

Everything needed already exists. The `redis-transport` feature forward in `framework/Cargo.toml` line 32 is already in place: `redis-transport = ["ferro-broadcast/redis-transport"]`. [VERIFIED: /Users/alberto/repositories/albertogferrario/ferro/framework/Cargo.toml:32]

---

## Architecture Patterns

### System Architecture — Boot Path Split

```
app/src/main.rs  Commands::Serve { no_worker }
                       │
                       ▼
             framework::app::run_server_internal(no_worker)
                       │
                       ├─ shared_boot() ───────────────────────────────┐
                       │    • bootstrap_fn().await (app's register())  │
                       │    • Queue::init(conn)  (if not initialised)  │
                       │    • register_offload_hooks[_with_broadcaster] │
                       │    • WR-01: if redis-transport feature + URL   │
                       │      → RedisTransport::connect(url).await      │
                       │      → broadcaster.with_transport(transport)   │
                       │    • if !no_worker:                            │
                       │        WorkerLoop::from_registry(all_queues)   │
                       │        tokio::spawn(worker.run())              │
                       └──────────────────────────────────────────────┘
                       │
                       ▼
             Server::from_config(router).run()   ← HTTP only


Commands::Worker { queue }
        │
        ▼
framework::app::run_worker_internal(queues)
        │
        ├─ shared_boot(no_worker = true)   ← identical path, no HTTP
        │     (DB + queue + offload hooks + broadcast transport)
        │
        └─ WorkerLoop::from_registry(WorkerConfig::new(effective_queues))
              .run().await                ← blocks until SIGTERM/Ctrl-C
```

**Key invariant:** `shared_boot()` is called exactly once regardless of whether the binary entered as `serve` or `worker`. No duplication of the bootstrap control surface.

### Recommended Project Structure (files to add/modify)

```
framework/src/
  app.rs               # Extract shared_boot(); add run_worker_internal(); add no_worker flag handling
ferro-queue/src/
  db.rs                # Add Queue::registered_queue_names() → Vec<String>
  worker.rs            # (unchanged — WorkerLoop already correct)
ferro-macros/src/
  offload.rs           # Parse #[offload(queue = "name")]; thread into inventory entry + Offloadable::offload
  service_impl.rs      # (or wherever #[offload] attr is stripped — pass queue name to emit_job_items)
app/src/
  main.rs              # Add Worker { queue } command arm; add no_worker flag; call framework entry points
```

### Pattern 1: Shared Boot Extraction

**What:** Pull everything from `run_server_internal` except `Server::from_config(router).run()` into a `run_common_boot(bootstrap_fn, no_worker)` async fn inside `framework/src/app.rs`.

**Current structure** (`framework/src/app.rs:399–456`):
```rust
// [VERIFIED: framework/src/app.rs:399-470]
async fn run_server_internal(bootstrap_fn, routes_fn) {
    bootstrap_fn().await;                          // line 405
    Queue::init(conn).await;                       // line 411-414
    match App::get::<Broadcaster>() { … };         // line 426-433 (offload hooks)
    if ferro_queue::Queue::has_registered_jobs() { // line 434
        // spawn in-process WorkerLoop             // line 449-455
    }
    Server::from_config(router).run().await;       // line 466
}
```

**Proposed seam:**
```rust
// framework/src/app.rs — new internal entry points
async fn run_common_boot(bootstrap_fn: Option<BootstrapFn>, no_worker: bool) {
    if let Some(f) = bootstrap_fn { f().await; }

    if !ferro_queue::Queue::is_initialized() {
        let conn = Self::get_database_connection().await;
        let _ = ferro_queue::Queue::init(conn).await;
    }

    // WR-01: attach transport when configured + feature enabled.
    // The Broadcaster was registered by bootstrap_fn above.
    if let Some(bc) = crate::App::get::<ferro_broadcast::Broadcaster>() {
        let config = bc.config().clone();
        #[cfg(feature = "redis-transport")]
        if let Some(ref url) = config.transport_redis_url {
            match ferro_broadcast::transport::redis::RedisTransport::connect(url).await {
                Ok(t) => {
                    let bc_with_transport = bc.with_transport(std::sync::Arc::new(t));
                    crate::App::singleton(bc_with_transport.clone());
                    crate::offload::register_offload_hooks_with_broadcaster(
                        std::sync::Arc::new(bc_with_transport)
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "broadcast transport connect failed — using in-process hub");
                    crate::offload::register_offload_hooks_with_broadcaster(
                        std::sync::Arc::new(bc)
                    );
                }
            }
            return; // hook registered above
        }
        #[cfg(not(feature = "redis-transport"))]
        if config.transport_redis_url.is_some() {
            tracing::warn!(
                "BROADCAST_REDIS_URL is set but the `redis-transport` feature is disabled \
                 — falling back to in-process hub"
            );
        }
        // Fallback (no URL, or feature-off + URL): broadcaster-aware persist hook only.
        crate::offload::register_offload_hooks_with_broadcaster(std::sync::Arc::new(bc));
    } else {
        crate::offload::register_offload_hooks();
    }

    if !no_worker && ferro_queue::Queue::has_registered_jobs() {
        if ferro_queue::QueueConfig::is_sync_mode() {
            eprintln!("WARNING: queue jobs registered but QUEUE_CONNECTION=sync …");
        }
        let all_queues = ferro_queue::Queue::registered_queue_names();  // new API
        let config = ferro_queue::WorkerConfig::new(all_queues);
        let worker = ferro_queue::WorkerLoop::from_registry(config);
        tokio::spawn(async move {
            if let Err(e) = worker.run().await {
                eprintln!("WorkerLoop exited with error: {e}");
            }
        });
    }
}

/// New framework entry point for the worker subcommand.
pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>) {
    run_common_boot(bootstrap_fn, /*no_worker=*/true).await;
    let effective_queues = if queues.is_empty() {
        ferro_queue::Queue::registered_queue_names()   // D-03: all registered
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

**CRITICAL constraint from audit:** The `None`-fallback branch (`register_offload_hooks` vs `register_offload_hooks_with_broadcaster`) at lines 426–433 currently exists because bootstrap may not have registered a `Broadcaster`. After WR-01 is wired in the shared boot step, this fallback legitimately stays for apps without a broadcaster (e.g., headless worker-only deployments). The Phase 249.1 convergence sweep will remove the `app.rs` fallback path only when the architecture-wide clean-up is done; do not remove it in Phase 248.

### Pattern 2: `Queue::registered_queue_names()` — Deriving the Distinct Queue Set

**Problem:** `WorkerConfig::default()` hardcodes `queues: vec!["default"]`; D-03/D-05 need the union of all queues declared by registered jobs.

**Current state:** `JobRegistrarEntry` carries `register: fn(&mut WorkerLoop)` and `name: &'static str`. It does not carry a queue name field. The `Job` trait has no `fn queue()` method. [VERIFIED: ferro-queue/src/db.rs:101-109]

**Two-step solution:**

Step 1 — Add an optional queue field to `JobRegistrarEntry`:
```rust
// ferro-queue/src/db.rs
pub struct JobRegistrarEntry {
    pub register: fn(&mut crate::WorkerLoop),
    pub name: &'static str,
    /// The queue this job type routes to. `None` = "default".
    pub queue: Option<&'static str>,
}
```

Step 2 — Add `Queue::registered_queue_names()`:
```rust
// ferro-queue/src/db.rs
pub fn registered_queue_names() -> Vec<String> {
    use std::collections::BTreeSet;
    let mut names: BTreeSet<String> = BTreeSet::new();
    // Runtime registrars (manual Queue::register) always go to "default" — no queue metadata.
    // If any are present, include "default".
    if !JOB_REGISTRARS.lock().unwrap().is_empty() {
        names.insert("default".to_string());
    }
    // Inventory-collected entries carry their declared queue name.
    for entry in inventory::iter::<JobRegistrarEntry> {
        names.insert(
            entry.queue.unwrap_or("default").to_string()
        );
    }
    if names.is_empty() {
        names.insert("default".to_string());
    }
    names.into_iter().collect()
}
```

`BTreeSet` gives deterministic ordering; `Vec<String>` satisfies `WorkerConfig::new()`.

### Pattern 3: `#[offload(queue = "name")]` — Macro Attribute Parsing

**Current state:** The `#[offload]` attribute is stripped by `service_impl` as an inert helper attribute. Its presence is the only signal; no argument is parsed. [VERIFIED: ferro-macros/src/offload.rs:1-50]

**What must change:**

1. In the `service_impl` macro (wherever `#[offload]` is detected and stripped), parse the optional `queue = "name"` argument from the attribute's token tree.
2. Pass the extracted queue name (as `Option<&'static str>`) into `collect_info()` and `emit_job_items()`.
3. In `emit_job_items`, thread the queue name into two places:
   - The `inventory::submit!` entry: set `queue: Some("name")` on the `JobRegistrarEntry`.
   - The `Offloadable::offload()` method body: emit `.on_queue("name")` on the `PendingDispatch`.

**Parsing sketch using `syn::meta`:**
```rust
// In service_impl, where #[offload] is matched:
let mut declared_queue: Option<String> = None;
for attr in &method.attrs {
    if attr.path().is_ident("offload") {
        // Parse optional arguments: #[offload] or #[offload(queue = "name")]
        if !attr.meta.require_path_only().is_err() {
            // Has arguments — parse key = value pairs
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("queue") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    declared_queue = Some(value.value());
                    Ok(())
                } else {
                    Err(meta.error("unknown #[offload] argument"))
                }
            })?;
        }
    }
}
```

**Emission change in `emit_job_items`** (queue_name: `Option<&str>`):
```rust
let queue_name_tokens: TokenStream2 = match queue_name {
    Some(q) => quote! { Some(#q) },
    None => quote! { None },
};
let on_queue_tokens: TokenStream2 = match queue_name {
    Some(q) => quote! { .on_queue(#q) },
    None => quote! {},
};

// In the inventory::submit! block:
::ferro::inventory::submit! {
    ::ferro::queue::JobRegistrarEntry {
        register: |w: &mut ::ferro::queue::WorkerLoop| { w.register::<#job_ident>(); },
        name: #job_ident_str,
        queue: #queue_name_tokens,
    }
}

// In Offloadable::offload() default body:
async fn offload(self) -> Result<::ferro::queue::OffloadHandle<Self::Output>, ::ferro::queue::Error> {
    let key = ::ferro::queue::HandleKey::new();
    ::ferro::queue::PendingDispatch::new(self)
        .with_handle_key(key.as_str().to_string())
        #on_queue_tokens
        .dispatch()
        .await?;
    Ok(::ferro::queue::OffloadHandle::new(key))
}
```

**Macros gate blindspot:** Any change to `emit_job_items` or the service macro's attribute-stripping loop MUST be covered by the trybuild UI gate. Run `cargo test -p ferro-macros` after changes — trybuild exercises the fixture expansion. Emit only `::ferro::*` paths. [CITED: .planning/MEMORY.md — project_ferro_macros_gate_blindspot]

### Pattern 4: WR-01 Transport Construction (Detailed)

**Current state:**
- `BroadcastConfig::from_env()` reads `BROADCAST_REDIS_URL` (or `REDIS_URL` fallback) into `transport_redis_url: Option<String>`. [VERIFIED: ferro-broadcast/src/config.rs:104-107]
- `Broadcaster::with_transport(transport: Arc<dyn BroadcastTransport + Send + Sync>)` exists. [VERIFIED: ferro-broadcast/src/broadcaster.rs:90-119]
- `RedisTransport::connect(url)` async constructor exists under `#[cfg(feature = "redis-transport")]`. [VERIFIED: ferro-broadcast/src/transport/redis.rs:58-61]
- The feature gate chain: `framework/Cargo.toml:32` — `redis-transport = ["ferro-broadcast/redis-transport"]`. [VERIFIED]
- Nothing in `framework/src/` currently calls `with_transport()` or constructs `RedisTransport`. [VERIFIED: grep returned no results]
- `app/src/bootstrap.rs:186-187` builds `Broadcaster::with_config(BroadcastConfig::from_env())` — does NOT call `with_transport`. [VERIFIED]

**WR-01 implementation location:** Inside `run_common_boot()` in `framework/src/app.rs`, AFTER `bootstrap_fn().await` (so the app's registered `Broadcaster` is available via `App::get::<Broadcaster>()`), BEFORE the offload hook registration.

**D-07 feature-flag matrix:**

| `redis-transport` feature | `transport_redis_url` | Behaviour |
|---|---|---|
| on | set | Connect `RedisTransport`, call `with_transport`, continue |
| on | not set | Skip, use in-process hub (no warning needed) |
| off | set | `warn!("redis-transport feature disabled but BROADCAST_REDIS_URL is set — in-process hub only")`, continue |
| off | not set | Normal in-process hub, no warning |

**Tech debt WR-02 (from audit):** The SUBSCRIBE loop is spawned detached with no readiness signal; tests paper over the attach window with `sleep(20ms)`. This is a pre-existing issue in `broadcaster.rs:with_transport`; Phase 248 does not fix it — just wires the call. Do not introduce additional detached spawns.

**Tech debt WR-03 (from audit):** `with_transport` constructs a fresh `BroadcasterInner`, discarding pre-existing clients/channels. Safe only if transport wiring happens before `add_client` — i.e., before the HTTP server accepts WebSocket connections. The shared boot step runs before `Server::from_config(router).run()`, so ordering is correct as long as `with_transport` is called inside `run_common_boot()`, not after.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic single-claim | Custom locking | `ferro_queue::db::claim` (`FOR UPDATE SKIP LOCKED` / `BEGIN IMMEDIATE`) | Already implemented and tested (Phase 185). Rewiring would break the existing exactly-once guarantee. |
| SIGTERM / Ctrl-C drain | Custom signal handler | `WorkerLoop::run()` already installs one | The loop uses an `Arc<AtomicBool>` shutdown flag; duplicate handlers would fight. |
| Worker ID for requeue | Custom UUID | `WorkerLoop::worker_id` (auto-generated UUID v4) | Needed for `requeue_claimed_by` to scope resets to this worker only. |
| Redis pub/sub client | `redis` crate directly | `ferro_broadcast::transport::redis::RedisTransport` | Already implemented with `ConnectionManager` for PUBLISH and dedicated `get_async_pubsub()` for SUBSCRIBE. |
| Queue enumeration | Runtime DB query | Inventory-collected `JobRegistrarEntry` records | Compile-time registration avoids a DB round-trip at boot; consistent with the existing `from_registry` pattern. |

---

## Runtime State Inventory

Phase 248 is a framework extension, not a rename or migration. No stored data keys, live service config, OS registrations, secrets, or build artifacts carry any renamed string. **Step 2.6 skipped for this phase.**

---

## Common Pitfalls

### Pitfall 1: SQLite In-Memory vs File for Multi-Consumer Tests (SC#2)

**What goes wrong:** A test that creates two `WorkerLoop` instances sharing one DB connection string over `sqlite::memory:` — each connection sees a different isolated in-memory database. Jobs enqueued on one connection are invisible to the other; the test vacuously passes (no claims, no duplicates, empty queue).

**Why it happens:** SQLite `:memory:` databases are scoped to the connection, not the file.

**How to avoid:** Use `tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc`, exactly as `ferro-queue/tests/race_claim_sqlite.rs` does. [VERIFIED: ferro-queue/tests/race_claim_sqlite.rs:1-15]

**Warning signs:** Both workers claim 0 jobs; the test still passes; `unique.len() == N` with N=0.

### Pitfall 2: OnceLock Queue Init Collision in Multi-Scenario Test Suites

**What goes wrong:** Calling `Queue::init(conn)` twice in a test — once per scenario — panics because `GLOBAL_CONNECTION` is a `OnceLock`. [VERIFIED: ferro-queue/src/db.rs:22-52]

**Why it happens:** `OnceLock::set` returns `Err` if already set; `Queue::init` maps this to `Error::custom("Queue already initialized")`.

**How to avoid:** Follow the established pattern from `offload_delta_broadcast.rs`: put all scenarios under one `#[tokio::test]` function and clear tables between scenarios. Alternatively check `Queue::is_initialized()` before calling `Queue::init()`.

**Warning signs:** `Queue already initialized` error in second test scenario; `OFFLOAD_BROADCASTER` OnceLock also has this property.

### Pitfall 3: Test-Suite Collapse False Green (SC#2 / SC#3)

**What goes wrong:** Multiple `#[tokio::test]` functions in the same crate binary collapse into one suite function (Ferro's OnceLock race pattern). A `-- <fn_name>` filter then silently matches 0 tests and exits 0.

**Why it happens:** The ferro test suite pattern collapses N tokio tests into one suite fn with a `OnceLock`; a filter for an individual function name finds nothing and reports success. [CITED: .planning/MEMORY.md — project_ferro_test_suite_collapse_filter_falsegreen]

**How to avoid:** Keep SC#2 and SC#3 as sub-functions invoked from a single outer `#[tokio::test]`. Verify the test command resolves to at least one test: run `cargo test -p <crate> -- --list` first.

**Warning signs:** `test result: ok. 0 passed` is the canary.

### Pitfall 4: `with_transport` Discards Existing Clients

**What goes wrong:** Calling `Broadcaster::with_transport` after clients have subscribed discards all existing channels and clients because it constructs a fresh `BroadcasterInner`. [VERIFIED: ferro-broadcast/src/broadcaster.rs:90-98 — `clients: DashMap::new(), channels: DashMap::new()`]

**Why it happens:** `with_transport` is a consuming builder that returns a new `Self`, not a mutation.

**How to avoid:** The shared boot step calls `with_transport` before `Server::from_config(router).run()`. No WebSocket clients can have connected yet. If for any reason the order must change, re-register the new `Broadcaster` instance via `App::singleton()` so subsequent `App::get::<Broadcaster>()` calls see the transport-attached version.

### Pitfall 5: `--queue` Parsed by Clap, Not by Framework

**What goes wrong:** Using `#[arg(long)]` without `action = ArgAction::Append` on a `Vec<String>` field gives only the last occurrence of `--queue`. `worker --queue a --queue b` would set `queues = ["b"]`.

**How to avoid:** Use `#[arg(long, action = clap::ArgAction::Append)]` or `#[arg(long, num_args = 1..)]`. The `Vec<String>` clap type with `action = Append` accumulates all occurrences. [ASSUMED — clap docs; verify against clap version in Cargo.lock before implementing]

### Pitfall 6: Macro Emit Path Must Stay `::ferro::*`

**What goes wrong:** Emitting `::ferro_queue::` or `::ferro_broadcast::` paths from proc-macro code causes compilation failures in consumer crates that only depend on `ferro-rs` (not on the internal crates directly). [CITED: CLAUDE.md — ferro-macros emission convention; MEMORY.md — project_ferro_macros_gate_blindspot]

**How to avoid:** All paths in `emit_job_items` remain `::ferro::queue::*`. The queue name string emitted into `JobRegistrarEntry { queue: Some("name") }` is a string literal — no path issue.

---

## Code Examples

### SC#2 Test — Two In-Process Workers Split Work Without Double-Processing

The existing `race_claim_sqlite.rs` already proves exactly-once claim at the DB level. Phase 248's SC#2 test should prove the same at the `WorkerLoop` level — two `WorkerLoop` instances running concurrently against one queue drain all jobs with no duplicates.

Pattern (derived from existing tests, adapted for `WorkerLoop`):
```rust
// Source: ferry-queue/tests/race_claim_sqlite.rs (adapted)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_worker_loops_split_work_without_duplicates() {
    // CRITICAL: NamedTempFile, not sqlite::memory: (Pitfall 1)
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn, None).await.unwrap();
    ferro_queue::Queue::init(conn).await.unwrap();

    const N: usize = 20;
    let now = chrono::Utc::now();
    for _ in 0..N {
        ferro_queue::enqueue(
            ferro_queue::Queue::connection(),
            "default", "TestJob", "{}", 3, None, None, None, now,
        ).await.unwrap();
    }

    let executed = Arc::new(Mutex::new(Vec::<i64>::new()));
    // Register a handler that records job IDs before deleting.
    // ... (register via inventory or Queue::register)

    // Two drain loops running concurrently
    let (h1, h2) = tokio::join!(
        tokio::spawn(drain_loop("w1", executed.clone())),
        tokio::spawn(drain_loop("w2", executed.clone())),
    );

    // Assert exactly-once coverage (same pattern as race_claim_sqlite.rs)
    let all = executed.lock().unwrap();
    let unique: HashSet<i64> = all.iter().cloned().collect();
    assert_eq!(unique.len(), all.len(), "no duplicates");
    assert_eq!(unique.len(), N, "all jobs executed");
}
```

**Human UAT caveat:** SC#2 above is an in-process test (two `WorkerLoop` instances in one binary over a shared temp DB). True multi-process claim isolation (two separate OS processes) is proven by the existing `race_claim_sqlite.rs` + `race_claim_postgres.rs` DB-level tests — those already certify `FOR UPDATE SKIP LOCKED` / `BEGIN IMMEDIATE`. Flagged for human UAT only if the requirement extends to testing two separately-launched binaries.

### SC#3 Test — Saturating One Queue Does Not Stall Another

```rust
// Source: derived from offload_delta_broadcast.rs pattern (sub-functions in one #[tokio::test])
async fn queue_fault_isolation() {
    // Setup: two distinct queues — "media" and "reports"
    // Worker A: WorkerConfig::new(vec!["media"])
    // Worker B: WorkerConfig::new(vec!["reports"])
    // Enqueue N_slow slow jobs on "media" (simulate saturation via semaphore / slow handler)
    // Enqueue N_fast fast jobs on "reports"
    // Run both workers concurrently
    // Assert: "reports" jobs complete within tight deadline regardless of "media" queue load
    //
    // Use tokio::sync::Barrier or a channel to synchronize start without time.Sleep.
}
```

**Determinism note:** Avoid `time::Sleep` for synchronization. Use a `tokio::sync::barrier::Barrier` (N workers reach it before any proceeds) or `Arc<Semaphore>` as the coordination primitive. The slow handler in "media" can use a `tokio::sync::OnceLock` / channel to signal when it has started, allowing the "reports" drain to start concurrently.

**Human UAT note:** Two separate OS processes genuinely cannot be tested deterministically in a `cargo test` run. The in-process simulation (two `WorkerConfig` instances with disjoint queue sets in one binary) covers the fault-isolation claim at the WorkerLoop level. Cross-process isolation is an architectural property of the DB-level claim already proven by the existing DB tests.

### WR-01 Transport Wiring (Illustrative)

```rust
// Source: framework/src/app.rs — new run_common_boot() block
// Runs after bootstrap_fn().await, before Server::from_config().run()
#[cfg(feature = "redis-transport")]
if let (Some(bc), Some(url)) = (
    crate::App::get::<ferro_broadcast::Broadcaster>(),
    bc.config().transport_redis_url.clone(),  // <-- BroadcastConfig::from_env already read it
) {
    match ferro_broadcast::transport::redis::RedisTransport::connect(&url).await {
        Ok(transport) => {
            let bc_with_transport = bc.with_transport(std::sync::Arc::new(transport));
            crate::App::singleton(bc_with_transport.clone()); // replace the singleton
            crate::offload::register_offload_hooks_with_broadcaster(
                std::sync::Arc::new(bc_with_transport)
            );
        }
        Err(e) => {
            tracing::warn!(error = %e,
                "BROADCAST_REDIS_URL set but Redis connect failed — in-process hub only");
            crate::offload::register_offload_hooks_with_broadcaster(
                std::sync::Arc::new(bc)
            );
        }
    }
} else {
    // No transport URL, or transport URL but feature disabled — existing path.
    // Feature-disabled + URL-set warning emitted in #[cfg(not(feature = "redis-transport"))] block.
}
```

---

## State of the Art

| Old Approach | Current Approach | Phase 248 Change |
|--------------|------------------|-----------------|
| Worker must be a separate `ferro` CLI subcommand | Worker is the app binary under `worker` subcommand | Adds `Worker { queue }` arm to scaffolded `main.rs` |
| In-process `serve` worker hardcoded to `["default"]` queue | `WorkerConfig::default()` still hardcodes `["default"]` | Changes `serve` worker to use `registered_queue_names()` (D-05) |
| `transport_redis_url` is read but never consumed | `BroadcastConfig::from_env()` populates the field | `run_common_boot()` constructs and attaches the transport |
| `#[offload]` has no arguments | `#[offload]` only marks presence | New: `#[offload(queue = "name")]` parsed; default = `"default"` |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Clap's `action = ArgAction::Append` on `Vec<String>` accumulates multiple `--queue` occurrences | Pitfall 5 | Minor: would need `num_args = 1..` or similar; behavior change, not a correctness failure |
| A2 | The sample `app/src/main.rs` is the canonical template for consumer apps and changes there become scaffold output | Architecture patterns | Low: even if the scaffold template lives elsewhere, the change is straightforward in both places |

All other claims were verified directly from source files in this session.

---

## Open Questions (RESOLVED)

> All three were resolved at plan time by reading the pinned manifests and container source; resolutions are recorded here and in the corresponding PLAN interfaces blocks.

1. **`syn::meta` API compatibility with current `syn` version in ferro-macros**
   - What we know: `ferro-macros` uses `syn` and `quote`; the exact `syn` version determines whether `attr.parse_nested_meta()` or `attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)` is the right API.
   - **RESOLVED:** `ferro-macros/Cargo.toml` pins `syn = "2"`. Use `attr.parse_nested_meta(|meta| { … })`; the syn-1 `NestedMeta` path must NOT be used. Recorded in `248-02-PLAN.md` interfaces block.

2. **`run_worker` as a public framework API vs scaffolded `main.rs` direct call**
   - What we know: The `Application` builder in `framework/src/app.rs` exposes a builder API. The sample `app/src/main.rs` currently bypasses the builder and calls `bootstrap::register()` + `Server::from_config()` directly.
   - **RESOLVED:** Expose as a standalone `pub async fn run_worker(bootstrap_fn, queues)` at the `framework` / `ferro-rs` facade level, parallel to today's `Server::from_config()` call pattern (not through the builder). Adopted uniformly across all plans; the `worker` CLI arm calls it directly.

3. **`App::singleton` replacement semantics for the transport-attached Broadcaster**
   - What we know: `App::singleton(x)` registers `x` in the container. If a `Broadcaster` was already registered by `bootstrap.rs:187`, calling `App::singleton` again may silently overwrite or panic depending on the implementation.
   - **RESOLVED:** `App::singleton` uses `HashMap::insert` keyed by `TypeId` (`framework/src/container/mod.rs:80-84`) → re-registration OVERWRITES silently, no panic. Safe to attach the transport-bearing `Broadcaster` before the HTTP accept loop. Recorded in `248-01-PLAN.md` interfaces block.

---

## Environment Availability

Step 2.6: This phase builds only Rust code and tests. The only external dependency relevant to the optional Redis transport path is `redis-server`, needed only for the feature-gated `redis_cross_replica` integration test. That test was already written in Phase 246.1 and is env-gated on `REDIS_URL`.

| Dependency | Required By | Available | Fallback |
|------------|------------|-----------|---------|
| `redis-server` | `redis_cross_replica` env-gated test | Not checked (CI-only) | Test skips when `REDIS_URL` unset |
| Rust toolchain | All compilation | Assumed present | — |

---

## Validation Architecture

> `workflow.nyquist_validation` absent from config — treating as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio::test` + `cargo test` |
| Config file | No separate config — Cargo.toml `[[test]]` sections |
| Quick run command | `cargo test -p ferro-queue --test race_claim_sqlite 2>&1 \| tail -5` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| SC # | Behavior | Test Type | Automated Command | Notes |
|------|----------|-----------|-------------------|-------|
| SC#1 | `<app-bin> worker --queue X` consumes only queue X | Integration (in-process) | `cargo test -p ferro-queue worker_consumes_only_selected_queue` | New test in `ferro-queue/tests/`; verifies jobs on queue Y are not claimed by a worker configured for X only |
| SC#2 | Two replicas split work, no double-processing | Integration (in-process) | `cargo test -p ferro-queue two_worker_loops_split_work_without_duplicates` | In-process simulation; cross-process isolation already proven by `race_claim_sqlite.rs` |
| SC#3 | Saturating one queue does not stall another | Integration (in-process) | `cargo test -p ferro-queue queue_fault_isolation` | Two `WorkerLoop`s with disjoint queue sets; channel/barrier sync, no `time::Sleep` |
| SC#4 | No autoscaling introduced; N is external | Documentation / grep | `grep -r "autoscal\|scale_to_zero\|KEDA" framework/src/` must return 0 matches | Structural — confirm no autoscaling code is introduced |
| OFFLOAD-05 | Deployable worker process runnable at N replicas | Human UAT | `<app-bin> worker --queue default` in a real shell, then `<app-bin> serve --no-worker` in another | Multi-process, cannot be deterministically automated in `cargo test` |
| WR-01 | `transport_redis_url` set → `RedisTransport` attached | Env-gated integration | Existing `redis_cross_replica` test (feature `redis-transport`, env `REDIS_URL`) | Already written in Phase 246.1; WR-01 wiring makes it exercise the real bootstrap path |
| D-07 | Feature-off + URL-set → warning, no hard failure | Unit | `cargo test -p framework transport_url_no_feature_warns` | Assert `tracing::warn!` fires; no panic |

### SC#2/SC#3 Test Collapse Guard

Both SC#2 and SC#3 tests should be sub-functions of a **single** `#[tokio::test]` in `ferro-queue/tests/worker_runtime.rs`, not separate `#[tokio::test]` functions, to avoid the OnceLock collision (Pitfall 2) and the false-green test-collapse issue (Pitfall 3). Verify the command resolves to a real test:
```bash
cargo test -p ferro-queue --test worker_runtime -- --list
# Must show at least one test name
```

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-queue` (queue tests only, fast)
- **Per wave merge:** `cargo test -p ferro-queue && cargo test -p framework`
- **Phase gate:** Full suite `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-queue/tests/worker_runtime.rs` — covers SC#1, SC#2, SC#3 as sub-functions of one `#[tokio::test]`
- [ ] `framework/tests/worker_boot.rs` — covers WR-01 bootstrap path (feature-gated) and D-07 warning behavior

---

## Security Domain

This phase adds no authentication, authorization, or sensitive data paths. The worker subcommand runs under the same process model as `serve`; all existing security controls (DB credentials via env, Redis URL via env) apply without change. ASVS V5 input validation: `--queue` values are strings used only to filter the `WorkerConfig.queues` Vec — they are never interpolated into SQL directly (the DB claim path uses parameterized statements). No additional ASVS controls required beyond what the existing framework already provides.

---

## Sources

### Primary (HIGH confidence — verified by reading source files)

- `ferro-queue/src/worker.rs` — `WorkerConfig`, `WorkerLoop::from_registry`, shutdown drain, `requeue_claimed_by`, `drain_for_test`
- `framework/src/app.rs:399-470` — `run_server_internal` full boot sequence
- `ferro-broadcast/src/config.rs` — `BroadcastConfig::from_env`, `transport_redis_url` field
- `ferro-broadcast/src/broadcaster.rs:90-119` — `Broadcaster::with_transport` implementation
- `ferro-broadcast/src/transport/redis.rs` — `RedisTransport::connect`, feature gate
- `ferro-broadcast/Cargo.toml` — `redis-transport = ["redis"]` feature
- `framework/Cargo.toml:32` — `redis-transport = ["ferro-broadcast/redis-transport"]` forwarding
- `ferro-macros/src/offload.rs` — `collect_info`, `emit_job_items`, `OffloadMethodInfo`
- `ferro-queue/src/db.rs` — `Queue`, `JobRegistrarEntry`, `has_registered_jobs`
- `ferro-queue/src/dispatcher.rs:181` — `let queue = self.queue.unwrap_or("default")`
- `ferro-queue/src/offload.rs:118-125` — `Offloadable::offload()` default body
- `ferro-queue/tests/race_claim_sqlite.rs` — existing SC-1 two-consumer exactly-once test pattern
- `app/src/main.rs` — current `Commands` enum, `run_server` function
- `app/src/bootstrap.rs:186-187` — current Broadcaster construction (no transport attach)
- `.planning/v16.4-MILESTONE-AUDIT.md` — WR-01 gap evidence and consequences
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — anchor spec

### Tertiary (LOW confidence — needs human validation before execution)

- Clap `action = ArgAction::Append` behavior for `Vec<String>` (A1 above) — not verified against the Cargo.lock-pinned clap version
- `App::singleton` re-registration semantics (OQ #3) — container implementation not read in this session

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in workspace; feature gates confirmed
- Architecture (boot-path factoring): HIGH — current code read line by line; seam identified precisely
- Macro attribute parsing: HIGH — `offload.rs` and `emit_job_items` fully read; pattern clear
- WR-01 transport wiring: HIGH — all four relevant files read; the gap is confirmed absent code
- Test construction (SC#2/SC#3): HIGH — existing test patterns followed exactly; SQLite pitfall documented
- Clap `action = Append`: LOW (one assumption, low blast radius)

**Research date:** 2026-08-14
**Valid until:** 2026-09-14 (stable Rust ecosystem; 30 days)
