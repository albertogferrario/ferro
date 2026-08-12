# Phase 244: `#[offload]` macro → Job + payload derivation - Context

**Gathered:** 2026-08-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Turn a single `#[offload]` annotation on a `#[service]` trait method into a derived
`ferro-queue` Job whose fields carry the method's non-`self` parameters, such that the work is
declared once (as the method) and never re-authored as a Job wrapper. In scope for 244:

- The macro surface that recognizes `#[offload]` on a `#[service]` trait method.
- Derivation of the Job struct (= its serializable payload) from the method signature.
- Executing the original method body on a worker (round-trip test via `ferro-queue::dispatch`).
- Auto-registration of the derived Job into the worker registry.

Explicitly **out of scope** (later offload phases): the typed result handle + compile-time
serializable-contract enforcement (245); result → `ferro-projection` snapshot (246); shared
broadcast transport (246.1); delta streaming (247); deployable `worker` runtime (248);
`ferro-mcp` introspection + docs (249). New capabilities belong in those phases.

</domain>

<decisions>
## Implementation Decisions

### Call-site model (244↔245)
- **D-01:** The macro derives a **sibling Job** alongside the trait. The `#[offload]` trait
  method **stays a normal in-process synchronous call** in 244 — its return semantics are
  unchanged, preserving the spec's local sync path.
- **D-02:** The round-trip test enqueues the derived Job through the **existing
  `ferro-queue::dispatch()`** API (`dispatch(ReportsBuildMonthlyJob { .. }).await`). No new
  dispatch mechanism — consistent with "derives onto `ferro-queue`, no second job/worker path."
- **D-03:** The ergonomic handle-returning offload entrypoint (calling the method returns a
  typed handle and enqueues) is **deferred to Phase 245** and layered on top of this
  derivation. 244 does not change how the method is called.

### `#[offload]` configuration surface
- **D-04:** `#[offload]` takes **no arguments** in 244. The derived Job inherits all `Job`-trait
  defaults: queue `"default"`, `max_retries = 3`, `timeout = 60s`, full-jitter backoff.
- **D-05:** Attribute config knobs (`#[offload(queue = …, retries = …, timeout = …)]`) are
  **deferred**. Adding them later is an additive change (thread an arg into the derived Job),
  not a rewrite. The macro arg parser stays trivial this phase.

### Return value & error handling
- **D-06:** Both `-> T` and `-> Result<T, E>` method signatures are supported.
- **D-07:** For a `Result`-returning method, **`Err(e)` maps to a Job failure** so ferro-queue's
  existing retry + `failed()` path fires. `E` is stringified (via `Display`/`Debug`) — it need
  not be `Serialize` because it is not persisted in 244. For a bare `-> T` method, `handle()`
  returns `Ok(())` unless the body panics.
- **D-08:** The method's **return value is discarded in 244** (the result path is 246+). The
  return type therefore is **not** required to be serializable in this phase; that constraint
  arrives with the result handle (245) and snapshot (246).

### Derived Job naming & structure
- **D-09:** A **single** `#[derive(Serialize, Deserialize)]` struct is derived per offloaded
  method; the struct **both** carries the params as fields **and** `impl Job` (matching the
  existing ferro-queue idiom, e.g. `ProcessImage`). No separate `Payload` type.
- **D-10:** Naming scheme is **`<Trait><Method>Job`** (PascalCase), e.g. `Reports::build_monthly`
  → `ReportsBuildMonthlyJob`. Predictable from `(trait, method)` for agents and Phase 249
  introspection; collision-safe across traits that share a method name. The struct is **public**
  and referenceable (the round-trip test names it).
- **D-11:** The **`&self` receiver is excluded** from the payload; each non-`self` parameter
  becomes an owned field of the struct, keyed by the parameter name.

### Worker registration
- **D-12:** The derived Job **self-registers via `inventory`** — the macro emits an
  `inventory::submit!` entry (mirroring how `#[service(impl = …)]` auto-binds at
  `ferro-macros/src/service.rs:167`). `WorkerLoop::from_registry` gains an **inventory-collection
  path** so every `#[offload]` Job is picked up with **zero bootstrap code**. This delivers the
  "declare once, zero wiring" property.
- **D-13:** Phase 244's **scope expands into `ferro-queue`'s registration mechanism** to add the
  inventory path alongside the existing runtime `JOB_REGISTRARS` Vec (`ferro-queue/src/db.rs:60`).
  This is an accepted coherence-tax expansion, consistent with the `#[service]` precedent.

### Worker execution (consequential — follows from the above, not separately chosen)
- **D-14:** The derived `handle()` resolves the concrete service from the container
  (`App::make::<dyn Trait>()`) and calls the original method body with the payload fields. `&self`
  is the container-registered impl (`#[service(impl = …)]` / `#[injectable]`), not serialized.

### Claude's Discretion
- Exact `inventory` entry type/name for the job registrar; whether the runtime `Queue::register`
  Vec and the inventory path are unified or run side by side (as long as `from_registry` drains
  both).
- Stringification detail for `E` (`Display` vs `Debug`) and the concrete `ferro-queue::Error`
  variant used for method-`Err` mapping.
- Module placement of the derived struct (sibling of the trait in the same module).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design anchor & phase spec
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — milestone anchor:
  authoring surface, the fire-and-forward result path, scaling model, alternatives rejected
  (sync RPC / actors), the serializable-contract-as-isolation-boundary framing.
- `.planning/ROADMAP.md` §"v16.4 Work Distribution … Phase 244" (~L3294, L3316) — phase goal,
  dependencies, the three success criteria this phase must make TRUE.
- `.planning/REQUIREMENTS.md` — OFFLOAD-01 (this phase's requirement).

### Macro layer (`#[offload]` extends `#[service]`)
- `ferro-macros/src/service.rs` — the `#[service]` attribute macro; the `inventory::submit!`
  auto-registration precedent to mirror (`ServiceBindingEntry`, L167).
- `ferro-macros/src/lib.rs` — proc-macro entry points (where an `#[offload]` helper attribute
  or its handling is wired).
- `ferro-macros/src/injectable.rs` — `#[injectable]` container binding (how the concrete impl
  the worker resolves is registered).

### Queue layer (the Job the macro derives onto)
- `ferro-queue/src/job.rs` — the `Job` trait (`handle`, `name`, `max_retries`, `retry_delay`,
  `failed`, `timeout`, `idempotency_key`) and `JobPayload`. The derived struct implements this.
- `ferro-queue/src/dispatcher.rs` — `dispatch()` / `dispatch_to()` / `PendingDispatch` (the
  enqueue path the round-trip test uses).
- `ferro-queue/src/worker.rs` — `WorkerLoop::register::<J>()` (L175) and `from_registry` (L206);
  the inventory-collection path (D-12) is added here.
- `ferro-queue/src/db.rs` — `Queue::register` / `apply_registrars` / `JOB_REGISTRARS` (L60–87),
  the runtime registry the inventory path sits alongside (D-13).

### Framework boot & container
- `framework/src/app.rs` — worker boot via `WorkerLoop::from_registry` (L431); `App::make` /
  `App::bind` for container resolution (D-14).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Job` trait** (`ferro-queue/src/job.rs`): the derived struct implements it directly; all
  defaults (retries, timeout, backoff, idempotency) already exist — the derivation supplies only
  `handle()`.
- **`dispatch()`** (`ferro-queue/src/dispatcher.rs`): the enqueue path; no new API needed.
- **`inventory::submit!` precedent** (`ferro-macros/src/service.rs:167`): the exact auto-register
  mechanism to mirror for D-12.
- **`WorkerLoop::from_registry` / `apply_registrars`**: the registry entry point to extend with
  an inventory pass.

### Established Patterns
- ferro-queue idiom: the Job struct **is** its own serializable payload (`ProcessImage`,
  `TestJob`) — reinforces the single-struct decision (D-09).
- Job registration today is **runtime + manual** (`Queue::register::<J>()` in bootstrap) — D-12
  adds a compile-time inventory path to remove that manual step for offload jobs.

### Integration Points
- The `#[service]` macro (`service.rs`) is where `#[offload]` is most naturally recognized (it
  already parses the whole trait); the derived struct + `impl Job` + `inventory::submit!` are
  emitted from there. (Helper-attribute vs standalone is a research question — see below.)
- `from_registry` (`worker.rs` / `app.rs:431`) is where derived jobs must surface at worker boot.
- The container (`App::make`) is where the worker obtains the concrete impl to run the body.

</code_context>

<specifics>
## Specific Ideas

- Authoring surface follows the anchor spec example verbatim:
  ```rust
  #[service(impl = ReportBuilder)]
  #[async_trait]
  pub trait Reports: Send + Sync {
      #[offload]
      async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
  }
  ```
  → derives `pub struct ReportsBuildMonthlyJob { tenant_id: i64, month: Month }` + `impl Job` +
  `inventory::submit!`, enqueued in test via `dispatch(ReportsBuildMonthlyJob { .. }).await`.

### Open research questions (flagged, not user decisions)
- Whether `#[offload]` is an inert **helper attribute consumed by `#[service]`** vs. a standalone
  `#[proc_macro_attribute]` — resolve from Rust proc-macro constraints (a trait method can't
  easily carry a standalone attribute macro; the outer `#[service]` sees the whole trait).
- Mapping **reference/borrowed params** (`&str`, `&[T]`) to owned payload fields.
- Handling of **sync (non-`async`)** offloaded methods against the async `Job::handle`.

</specifics>

<deferred>
## Deferred Ideas

- Typed result handle + compile-time serializable-contract enforcement — **Phase 245**.
- Result → `ferro-projection` snapshot keyed by handle; terminal error state — **Phase 246**.
- Shared broadcast transport for multi-replica delta delivery — **Phase 246.1**.
- Read-model delta → broadcast streaming — **Phase 247**.
- Deployable `worker` subcommand runtime + deploy-metadata worker components — **Phase 248**.
- `ferro-mcp` `list_services` offload introspection + docs — **Phase 249**.
- `#[offload(queue = …, retries = …, timeout = …)]` config surface — future additive (D-05).

None of the above surfaced as scope creep during discussion; they are the already-planned
downstream offload phases.

</deferred>

---

*Phase: 244-offload-macro-job-payload-derivation*
*Context gathered: 2026-08-13*
