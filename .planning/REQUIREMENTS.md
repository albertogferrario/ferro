# Requirements — v16.4 Work Distribution (`#[offload]` Service Methods)

> Formerly an accreted multi-milestone working file; every shipped section has now been
> extracted to `milestones/vX.Y-REQUIREMENTS.md`. Only **v16.4 Work Distribution** remains —
> the current milestone as of 2026-08-12 (Phases 244–249, not yet executed). Archived and
> removed: v16.3, v16.5, v16.6, v18.0 (all 2026-08-12); v16.1/v16.2/v17.0 were reconstructed
> from ROADMAP (never in this file). When v16.4 is planned, this is its requirements file.

---

## v16.4 Work Distribution

**Status:** Current milestone. Phases 244–249.1 are executed and verified. The 2026-08-15
milestone audit returned `gaps_found` (a serve-path integration blocker) plus selected tech debt;
gap-closure / hardening Phases 249.2–249.4 were added and precede `/gsd-complete-milestone`.

**Milestone goal:** A `#[service]` trait method marked `#[offload]` becomes a distributable
unit of work with zero hand-written queue plumbing — the framework derives the `ferro-queue`
Job, serializable payload, and a typed result handle from the method signature, runs it on a
horizontally scalable worker, and streams the result back via the read-model + broadcast path.
Anchor spec: `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md`.

**Scope decision:** Build the scalable primitive; defer the auto-deciding. Capacity scales by
running more workers (external/operator/k8s-managed N). Autonomous machine lifecycle /
scale-to-zero / KEDA / CRIU is **out of scope** (cost-optimization, not capacity — parked as a
2.0 direction in the spec).

## v16.4 Requirements

### Offload Primitive

- [x] **OFFLOAD-01**: A developer marks a `#[service]` trait method `#[offload]` and the
  framework derives a `ferro-queue` Job + serializable payload from the method signature — no
  hand-written Job struct, no manual enqueue.
- [x] **OFFLOAD-02**: Calling an offloaded method returns a typed result handle; a method whose
  parameter or return type is not `Serialize`/`DeserializeOwned` fails at compile time with a
  clear, type-naming diagnostic (this enforcement is the module-isolation boundary).

### Result Delivery

- [x] **OFFLOAD-03**: An offloaded method's return value is persisted as a `ferro-projection`
  snapshot keyed by the handle, retrievable after completion; a failed run records a terminal
  error state (no silent drop).
- [x] **OFFLOAD-04**: A client subscribed to a handle receives the result as a `ferro-broadcast`
  delta on completion; the originating request returns immediately and never blocks awaiting it.

### Scalable Execution

- [x] **OFFLOAD-05**: Offloaded work runs on a deployable `ferro worker` process runnable at N
  replicas against the shared queue; capacity scales by adding replicas; each worker class is an
  independent fault domain. No framework-managed autoscaling (N is external).
  _Re-opened by the 2026-08-15 audit: satisfied for the dedicated `worker` process, but the default
  `serve` path does not auto-spawn its in-process worker for `#[offload]`-only apps
  (`has_registered_jobs()` is inventory-blind). Closed by Phase 249.2._

### Introspection & Docs

- [x] **OFFLOAD-06**: Offloadable methods are introspectable through `ferro-mcp` (`list_services`,
  derived payload schema); docs cover the authoring surface, result path, scaling model
  (stateless tier + replicable workers + cache + queue), and the non-goals / deferred elastic
  direction.

## Out of Scope (v16.4)

- **Synchronous, request-path, cross-machine RPC** (the rejected "Approach A" — see spec
  Alternatives).
- **Autonomous machine lifecycle / scale-to-zero** (KEDA, CRIU warm-start, Nomad, WASM isolates)
  — 2.0 direction; the queue-consumer model does not preclude it.

## Traceability (v16.4)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| OFFLOAD-01 | Phase 244 | Complete |
| OFFLOAD-02 | Phase 245 | Complete |
| OFFLOAD-03 | Phase 246 (edges hardened by 249.3) | Complete |
| OFFLOAD-04 | Phase 247 | Complete |
| OFFLOAD-05 | Phase 248 → 249.2 | Complete |
| OFFLOAD-06 | Phase 249 (scanner hardened by 249.4) | Complete |

**Gap-closure / hardening phases (2026-08-15 audit):** 249.2 closes the OFFLOAD-05 serve-path
blocker; 249.3 hardens OFFLOAD-03 result-path terminal-state edges (sync-mode, reaper); 249.4
hardens OFFLOAD-06 MCP scanner (`#[service(impl = X)]`, multi-line attributes).
