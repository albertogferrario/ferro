# Phase 244: `#[offload]` macro → Job + payload derivation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-13
**Phase:** 244-offload-macro-job-payload-derivation
**Areas discussed:** Call-site model, Config surface, Return & error handling, Job naming, Worker registration

---

## Gray area selection

| Option | Description | Selected |
|--------|-------------|----------|
| Call-site model (244↔245) | Whether the method call triggers offload or a derived sibling does | ✓ |
| #[offload] config surface | Bare attribute vs. attribute-arg config now | ✓ |
| Return & error handling scope | How method outcome maps to Job success/failure; return discarded | ✓ |
| Derived Job & payload naming | Predictable referenceable names vs. hidden identifiers | ✓ |

User selected all four; a fifth area (Worker registration) was surfaced mid-discussion after
scouting `ferro-queue`'s registration mechanism, and was also discussed.

---

## Call-site model (244↔245)

| Option | Description | Selected |
|--------|-------------|----------|
| Derived sibling; method stays sync | Macro derives a named Job; method stays in-process sync in 244; test enqueues the Job; handle-returning entrypoint deferred to 245 | ✓ |
| Method call auto-enqueues now | Calling the method enqueues in 244 (returns ()/placeholder); 245 upgrades return to a handle | |
| Generated companion fn | Macro emits a companion entrypoint now; method untouched | |

**User's choice:** Derived sibling; method stays sync.
**Notes:** Cleanest 244↔245 sequencing; preserves the spec's in-process sync path unchanged.
Consequence recorded: enqueue via existing `ferro-queue::dispatch()` (no new mechanism).

---

## `#[offload]` configuration surface

| Option | Description | Selected |
|--------|-------------|----------|
| Bare attribute, defaults only | No args; inherit all Job-trait defaults; config deferred | ✓ |
| Queue override only | Support `#[offload(queue = …)]` now (relevant to 248 worker classes) | |
| Full config now | queue + retries + timeout on the attribute | |

**User's choice:** Bare attribute, defaults only.
**Notes:** Maximal zero-config feel; adding knobs later is additive.

---

## Return & error handling scope

| Option | Description | Selected |
|--------|-------------|----------|
| Wire Err → Job failure now | Support `-> T` and `-> Result`; `Err` → Job failure/retry; value discarded in 244 | ✓ |
| Always Ok in 244, defer errors | Job always returns Ok; defer error semantics to 246 | |
| Restrict to -> T only in 244 | Reject `-> Result` until 246 | |

**User's choice:** Wire Err → Job failure now.
**Notes:** Honest retry semantics from day one; return value discarded (return type not required
to be serializable in 244); `E` stringified via Display/Debug.

---

## Derived Job & payload naming

| Option | Description | Selected |
|--------|-------------|----------|
| `<Trait><Method>Job` | `Reports::build_monthly` → `ReportsBuildMonthlyJob`; predictable + collision-safe | ✓ |
| `<Method>Job` | `BuildMonthlyJob`; cleanest but collides across traits sharing a method name | |
| Module-namespaced | `reports::BuildMonthlyJob`; collision-safe via module path, adds a generated module | |

**User's choice:** `<Trait><Method>Job`.
**Notes:** Structure assumed and unopposed — a single `#[derive(Serialize,Deserialize)]` struct
that also `impl Job` (ferro-queue idiom), public/referenceable. No separate Payload type.

---

## Worker registration (surfaced during discussion)

| Option | Description | Selected |
|--------|-------------|----------|
| Inventory auto-registration | Macro emits `inventory::submit!`; `from_registry` gains inventory path; zero bootstrap wiring; 244 scope expands into ferro-queue | ✓ |
| Generated aggregate registrar | Per-crate `register_offload_jobs()` called once in bootstrap; no ferro-queue change | |
| Document manual registration | Consumer calls `Queue::register::<…>()` per job; manual wiring | |

**User's choice:** Inventory auto-registration.
**Notes:** Delivers the "declare once, zero wiring" killer property; mirrors the existing
`#[service(impl=…)]` inventory precedent; accepted coherence-tax expansion into `ferro-queue`.

## Claude's Discretion

- Exact inventory entry type/name for the job registrar; unify vs. run beside the runtime
  `JOB_REGISTRARS` Vec (as long as `from_registry` drains both).
- `Display` vs `Debug` for `E` stringification and the concrete `ferro-queue::Error` variant.
- Module placement of the derived struct.

## Deferred Ideas

- All downstream offload phases (245–249, 246.1) as noted in CONTEXT.md `<deferred>`.
- `#[offload(queue/retries/timeout)]` config surface — future additive.
