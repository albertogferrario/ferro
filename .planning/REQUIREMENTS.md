# Requirements — v16.4 Work Distribution (`#[offload]` Service Methods)

> Formerly an accreted multi-milestone working file; every shipped section has now been
> extracted to `milestones/vX.Y-REQUIREMENTS.md`. Only **v16.4 Work Distribution** remains —
> the current milestone as of 2026-08-12 (Phases 244–249, not yet executed). Archived and
> removed: v16.3, v16.5, v16.6, v18.0 (all 2026-08-12); v16.1/v16.2/v17.0 were reconstructed
> from ROADMAP (never in this file). When v16.4 is planned, this is its requirements file.

---

## v16.4 Work Distribution

**Status:** Current milestone. Phases 244–249.2 are executed and verified. A re-audit on 2026-08-15
returned `gaps_found` again — a NEW critical dispatch-key blocker (DISPATCH-KEY-01) plus two
unexecuted hardening phases. Gap-closure / hardening Phases 249.2–249.5 precede
`/gsd-complete-milestone`: **249.5 (DISPATCH-KEY-01) is the critical blocker and executes first**,
then 249.3 (OFFLOAD-03 edges) and 249.4 (OFFLOAD-06 WR-02).

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

- [ ] **OFFLOAD-01**: A developer marks a `#[service]` trait method `#[offload]` and the
  framework derives a `ferro-queue` Job + serializable payload from the method signature — no
  hand-written Job struct, no manual enqueue.
  _Re-opened by the 2026-08-15 re-audit: the macro derivation is correct, but the derived Job's
  bare `name()` mismatches the worker's `type_name` handler key (DISPATCH-KEY-01), so the job never
  dispatches on the db path. Closed by Phase 249.5._
- [x] **OFFLOAD-02**: Calling an offloaded method returns a typed result handle; a method whose
  parameter or return type is not `Serialize`/`DeserializeOwned` fails at compile time with a
  clear, type-naming diagnostic (this enforcement is the module-isolation boundary).

### Result Delivery

- [ ] **OFFLOAD-03**: An offloaded method's return value is persisted as a `ferro-projection`
  snapshot keyed by the handle, retrievable after completion; a failed run records a terminal
  error state (no silent drop).
  _Re-opened by the 2026-08-15 re-audit: two silent-drop edges remain — sync-mode dispatch
  (`QUEUE_CONNECTION` unset default) ignores the handle key, and reaper-parked jobs record no
  terminal envelope. Closed by Phase 249.3._
- [ ] **OFFLOAD-04**: A client subscribed to a handle receives the result as a `ferro-broadcast`
  delta on completion; the originating request returns immediately and never blocks awaiting it.
  _Re-opened by the 2026-08-15 re-audit: the persist→broadcast→subscribe machinery is correct in
  isolation, but the delta never fires on the db path because the job never dispatches
  (DISPATCH-KEY-01). Unblocked by Phase 249.5._

### Scalable Execution

- [ ] **OFFLOAD-05**: Offloaded work runs on a deployable `ferro worker` process runnable at N
  replicas against the shared queue; capacity scales by adding replicas; each worker class is an
  independent fault domain. No framework-managed autoscaling (N is external).
  _Re-opened twice by the 2026-08-15 audits: (1) the default `serve` path did not auto-spawn its
  in-process worker for `#[offload]`-only apps (`has_registered_jobs()` inventory-blind) — closed by
  Phase 249.2; (2) the re-audit found the spawned worker still cannot dispatch any derived job
  because of DISPATCH-KEY-01 — unblocked by Phase 249.5._

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
| OFFLOAD-01 | Phase 244; db-path dispatch fixed by 249.5 | Pending |
| OFFLOAD-02 | Phase 245 | Complete |
| OFFLOAD-03 | Phase 246 (edges hardened by 249.3) | Pending |
| OFFLOAD-04 | Phase 247; runtime unblocked by 249.5 | Pending |
| OFFLOAD-05 | Phase 248 → 249.2; dispatch unblocked by 249.5 | Pending |
| OFFLOAD-06 | Phase 249 (scanner hardened by 249.4, WR-02) | Complete |

**Gap-closure / hardening phases (2026-08-15 audits):** 249.2 closes the OFFLOAD-05 serve-path
blocker; 249.3 hardens OFFLOAD-03 result-path terminal-state edges (sync-mode, reaper); 249.4
hardens OFFLOAD-06 MCP scanner (multi-line `#[service(...)]`, WR-02 — WR-01 already shipped in
249/249.1); **249.5 (blocker) reconciles the offload dispatch key (DISPATCH-KEY-01) so derived jobs
dispatch on the db path — restores OFFLOAD-01 and unblocks OFFLOAD-04/05 at runtime. Execute first.**
