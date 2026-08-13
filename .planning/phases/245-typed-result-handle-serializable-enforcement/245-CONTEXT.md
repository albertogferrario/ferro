# Phase 245: Typed result handle + serializable-contract enforcement - Context

**Gathered:** 2026-08-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Layer two properties onto Phase 244's `#[offload]` derivation, making the offload call site
ergonomic and the contract honest:

1. **Typed result handle** — an offloaded method can be *enqueued* through a typed entrypoint
   that returns an `OffloadHandle<T>` identifying where the result will eventually land, instead
   of the bare value or a unit-returning `dispatch`.
2. **Compile-time serializable-contract enforcement** — a `#[offload]` method whose parameter
   **or return** type is not `Serialize + DeserializeOwned` fails to compile (trybuild) with a
   clear, type-naming diagnostic. This enforcement *is* the module-isolation boundary.
3. **Documentation** — the serializable boundary is documented as the module-isolation guarantee.

In scope for 245:
- A new `OffloadHandle<T>` type (identity + phantom type; **inert** in 245 — no resolve/subscribe).
- An `Offloadable` trait carrying `type Output` and the `.offload()` entrypoint, `impl`-emitted by
  the macro per offloaded method.
- Compile-time enforcement of `Serialize + DeserializeOwned` on every parameter type and on the
  return success type, via a bounded marker trait with a branded `#[diagnostic::on_unimplemented]`
  message; trybuild fixtures proving both param-side and return-side failures.

Explicitly **out of scope** (later offload phases): result → `ferro-projection` snapshot keyed by
the handle + terminal error state (246); shared broadcast transport (246.1); delta streaming and
handle subscription (247); deployable `worker` runtime (248); `ferro-mcp` introspection + docs
(249). The worker's `Job::handle()` continues to **discard** the return value in 245 — persistence
is 246.

</domain>

<decisions>
## Implementation Decisions

### Call-site surface (the killer feature)
- **D-01:** The typed handle is produced by an **`.offload()` method on the already-public derived
  Job struct**: `ReportsBuildMonthlyJob { tenant_id, month }.offload().await -> OffloadHandle<Report>`.
  The `#[offload]` **trait method itself stays `-> T` in-process** (244 D-01/D-03) — `.offload()` is
  the *enqueue* entrypoint layered on top, not a mutation of the method's own signature. This
  preserves the "one trait = in-process contract + wire spec" property from the anchor spec.
- **D-02:** The macro emits an **associated `type Output = <method return success type>`** on the
  Job so the handle's `T` is carried structurally (not inferred). `.offload()` returns
  `Result<OffloadHandle<Self::Output>, Error>` — enqueue can fail (DB insert), mirroring `dispatch`.
- **D-03:** `.offload()` enqueues through the **existing `dispatch`/`PendingDispatch` path** — no new
  enqueue mechanism. It wraps that call and mints the handle around it.

### Enforcement mechanism + diagnostic
- **D-04:** `.offload()` and `type Output` live on a new **`Offloadable` trait**
  (`trait Offloadable: Job { type Output: OffloadSerializable; async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error>; }`).
  The macro emits `impl Offloadable for <..>Job { type Output = <T>; }`; the `offload()` body is a
  provided default (enqueue + mint handle), so per-method emission stays minimal.
- **D-05:** Serializable enforcement is **structural via bounds**, not bolt-on assertions. A marker
  trait `OffloadSerializable: Serialize + DeserializeOwned` with a blanket impl
  (`impl<T: Serialize + DeserializeOwned> OffloadSerializable for T {}`) carries a
  **`#[diagnostic::on_unimplemented]`** message that names the offending type and frames the
  isolation boundary. The **return** type is enforced by `type Output: OffloadSerializable`; the
  **parameter** types by a matching bound (on the fields / a where-clause). One message style for
  both. MSRV is 1.94.1, so `#[diagnostic::on_unimplemented]` is freely available (first use in tree).
- **D-06:** Both param-side and return-side non-serializable failures are proven by **trybuild
  fixtures** (extending the 244 trybuild harness), satisfying SC#2's "type-naming message" bar.

### Handle type & key identity
- **D-07:** The handle key is **always a fresh UUID v4 minted at enqueue**, **decoupled** from
  `Job::idempotency_key()`. The handle is the identity of *this offload call*, not of the result
  content. Reconciling a deduped job (same `idempotency_key`) that hands back multiple handles is a
  **246/247 concern**, not 245's — kept out of scope deliberately to keep 245 minimal.
- **D-08:** `OffloadHandle<T>` holds a `HandleKey` (UUID-backed string newtype) + `PhantomData<T>`.
  It is **inert in 245**: it exposes a read-only key accessor (`.key()` / `.id()`) for tests and
  downstream phases, and has **no** `.await` / `.subscribe()` resolve methods (those arrive with the
  result path in 246 and streaming in 247).

### Return-type contract precision
- **D-09:** `type Output = T` is the **success type**. For a `Result<T, E>` method the handle is
  `OffloadHandle<T>`; **`E` keeps its 244 D-07 treatment** — `Display`-stringified → job failure,
  **not** required to be `Serialize`. For a bare `-> T` method, `Output = T`; for `-> ()` (or no
  return), `Output = ()`. Enforcement (`OffloadSerializable`) targets `Output` (the success type)
  and the parameters, never `E`.
- **D-10:** The worker's `Job::handle()` **still discards the value in 245** (unchanged from 244).
  245 locks the *typed contract* and the *compile-time enforcement* before the result path exists;
  value capture / snapshot persistence is Phase 246. The enforcement therefore runs "ahead of"
  persistence by design — exactly SC#2's intent.

### Claude's Discretion
- Exact module home for `OffloadHandle`, `Offloadable`, `OffloadSerializable`, `HandleKey` (a new
  `offload` module in `ferro-queue`, vs `framework` — pick per re-export ergonomics so generated
  `::ferro::*` paths resolve in any consumer crate, as in 244).
- Whether `OffloadHandle<T>` derives `Serialize, Deserialize, Clone, Debug` (recommended: yes —
  `PhantomData<T>` is always serde-safe regardless of `T`, and 247 needs the handle to travel to the
  client as the subscription key; `T` need not be `Serialize` for the handle itself to be).
- The exact `#[diagnostic::on_unimplemented]` wording (message + note), as long as it names `{Self}`
  and frames the offload isolation boundary.
- How the parameter-side bound is expressed (per-field bound vs a generated `where` assertion) so
  the branded message fires for params identically to the return type.
- The concrete UUID crate/path used for `HandleKey` generation (align with any existing framework
  dependency).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design anchor & phase spec
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` §"[serializable
  contract]" (~L65–73) — the isolation-boundary framing (one constraint, both properties: wire
  contract + sealed module) that SC#3 must document; §"Result path (fire-and-forward)" (~L75–88) —
  the handle "identifies where the result will land" / "the handle is the projection key the client
  subscribes to" definition that shapes `OffloadHandle`.
- `.planning/ROADMAP.md` §"Phase 245" (~L3334) — phase goal, dependency (244), and the three
  Success Criteria this phase must make TRUE.
- `.planning/REQUIREMENTS.md` — **OFFLOAD-02** (this phase's requirement).
- `.planning/phases/244-offload-macro-job-payload-derivation/244-CONTEXT.md` — the immediate
  predecessor's locked decisions (D-01/D-03 method-stays-sync; D-06/D-07 Result/E handling;
  D-08 return-not-serializable-until-245; D-09/10/11 Job struct shape). 245 builds directly on these.

### Macro layer (extend the 244 derivation)
- `ferro-macros/src/offload.rs` — the 244 derivation to extend: `collect_info` (add return-type
  capture for `type Output`), `emit_job_items` (emit `impl Offloadable` + `type Output` +
  enforcement bounds), `owned_type` (param owned-mapping already present). `returns_result` /
  return-type detection already exists (L170–185) and must now feed `Output`.
- `ferro-macros/src/service.rs` — where offloaded methods are collected and emitted (L183–254);
  the return type must be threaded through to the emitted `Output`.
- 244 trybuild fixtures (locate under `ferro-macros` tests) — extend with a non-`Serialize`
  parameter case and a non-`Serialize`/`DeserializeOwned` return case (SC#2).

### Queue layer (where the new types most likely live)
- `ferro-queue/src/job.rs` — the `Job` trait (L44) the derived struct implements; `idempotency_key`
  (L86), explicitly **decoupled** from the handle key (D-07). `Offloadable`/`OffloadHandle` most
  naturally sit alongside.
- `ferro-queue/src/dispatcher.rs` — `dispatch()` / `PendingDispatch` (L204) that `.offload()` wraps.
- `ferro-queue/src/worker.rs` — `WorkerLoop::register` / `from_registry` (dispatch keyed by
  `type_name`); unchanged in 245 but read for the derived-Job registration context.

### Framework boot & re-export
- `framework/src/app.rs` — `App::make` container resolution (worker execution path, unchanged);
  the public re-export surface where `OffloadHandle` / `Offloadable` must surface as `::ferro::*`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **244 derivation** (`ferro-macros/src/offload.rs`): `OffloadMethodInfo` already carries
  `returns_result` and the return-type plumbing; `owned_type` already maps borrowed params to owned
  serializable fields. 245 adds an `Output` type + an `impl Offloadable` + enforcement bounds to the
  existing `emit_job_items`.
- **`dispatch` / `PendingDispatch`** (`ferro-queue/src/dispatcher.rs`): `.offload()` reuses this to
  enqueue — no new enqueue path (mirrors 244 D-02).
- **`#[derive(Serialize, Deserialize)]` on the derived struct** (present since 244): already forces
  *parameter* fields to be serializable — but with serde's default message. 245 upgrades the
  diagnostic (branded, isolation-framed) and adds the currently-unchecked **return-type** coverage.
- **`Job::idempotency_key`** (`ferro-queue/src/job.rs:86`): exists, but the handle key is
  deliberately independent of it (D-07).

### Established Patterns
- Generated code emits only `::ferro::*` paths so it resolves in any crate depending on `ferro-rs`
  (244 convention) — `OffloadHandle` / `Offloadable` / `OffloadSerializable` must be re-exported
  accordingly.
- ferro-queue idiom: the Job struct **is** its own serializable payload — the `Offloadable` impl is
  an additional trait impl on that same struct, not a second type.
- No `OffloadHandle`/`Offloadable`/`OffloadSerializable` type exists yet — all new in 245.

### Integration Points
- `emit_job_items` (`offload.rs`) is where the new `impl Offloadable { type Output = T }` and the
  enforcement bounds are emitted.
- The trybuild harness added in 244 is where the two new negative fixtures (bad param, bad return)
  attach.
- `framework/src/app.rs` / `framework/src/lib.rs` re-exports are where the new public types surface.

</code_context>

<specifics>
## Specific Ideas

Target call-site shape (extends the 244 anchor example):
```rust
#[service(impl = ReportBuilder)]
#[async_trait]
pub trait Reports: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report; // stays in-process
}

// 245 adds the enqueue entrypoint on the derived Job:
let handle: OffloadHandle<Report> =
    ReportsBuildMonthlyJob { tenant_id, month }.offload().await?;
let key = handle.key(); // inert in 245; 246/247 add resolve/subscribe
```

Branded diagnostic sketch (exact wording is discretion):
```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` crosses the #[offload] isolation boundary and must be \
               Serialize + DeserializeOwned",
    note = "offloaded parameters and return types travel as a queue payload; \
            make `{Self}` serializable to seal the module across the boundary"
)]
pub trait OffloadSerializable: serde::Serialize + serde::de::DeserializeOwned {}
impl<T: serde::Serialize + serde::de::DeserializeOwned> OffloadSerializable for T {}
```

</specifics>

<deferred>
## Deferred Ideas

- Result → `ferro-projection` snapshot keyed by the handle; terminal error state on
  failure/panic; `handle()` capturing the value — **Phase 246**.
- Reconciling a deduped job (same `idempotency_key`) that yields multiple distinct handles —
  **Phase 246/247** (consequence of D-07's random-UUID decision).
- Handle `.await` / `.subscribe()` resolve + streaming semantics — **Phase 247**.
- `#[offload(queue = …, retries = …, timeout = …)]` config surface — future additive (244 D-05).

None surfaced as scope creep; these are the already-planned downstream offload phases.

</deferred>

---

*Phase: 245-typed-result-handle-serializable-enforcement*
*Context gathered: 2026-08-13*
