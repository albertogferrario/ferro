# Requirements (accreted — multi-milestone working file)

> This file accumulates one `# Requirements — vX.Y` section per milestone; it was never
> rotated per-milestone. On each milestone close its section is extracted to
> `milestones/vX.Y-REQUIREMENTS.md` and removed here. Sections currently present, in order:
> **v16.4** Work Distribution (QUEUED — not shipped), **v18.0** Projection-Native Frontend
> Substrate (CURRENT). Archived and removed so far: v16.3, v16.5, v16.6 (all 2026-08-12).
> v17.0 requirements are not in this file (tracked inline in ROADMAP).

---

# Requirements — v16.4 Work Distribution (`#[offload]` Service Methods) — QUEUED

**Status:** Queued behind v16.3 Phase 243. v16.3 remains the current milestone until 243
closes; v16.4 phases (244–249) continue the numbering and are planned/executed afterward.

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
- [ ] **OFFLOAD-02**: Calling an offloaded method returns a typed result handle; a method whose
  parameter or return type is not `Serialize`/`DeserializeOwned` fails at compile time with a
  clear, type-naming diagnostic (this enforcement is the module-isolation boundary).

### Result Delivery

- [ ] **OFFLOAD-03**: An offloaded method's return value is persisted as a `ferro-projection`
  snapshot keyed by the handle, retrievable after completion; a failed run records a terminal
  error state (no silent drop).
- [ ] **OFFLOAD-04**: A client subscribed to a handle receives the result as a `ferro-broadcast`
  delta on completion; the originating request returns immediately and never blocks awaiting it.

### Scalable Execution

- [ ] **OFFLOAD-05**: Offloaded work runs on a deployable `ferro worker` process runnable at N
  replicas against the shared queue; capacity scales by adding replicas; each worker class is an
  independent fault domain. No framework-managed autoscaling (N is external).

### Introspection & Docs

- [ ] **OFFLOAD-06**: Offloadable methods are introspectable through `ferro-mcp` (`list_services`,
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
| OFFLOAD-01 | Phase 244 | Not started |
| OFFLOAD-02 | Phase 245 | Not started |
| OFFLOAD-03 | Phase 246 | Not started |
| OFFLOAD-04 | Phase 247 | Not started |
| OFFLOAD-05 | Phase 248 | Not started |
| OFFLOAD-06 | Phase 249 | Not started |

---

# Requirements — v18.0 Projection-Native Frontend Substrate — CURRENT

**Status:** Current milestone, created 2026-07-27 from committed anchor spec. Phase
numbering continues at 263, following v17.0 Live Projection Surface (259–262).

**Milestone goal:** Derive a custom (Inertia) frontend's data, field schema, and
permitted-actions from the same `ServiceDef`/`ActionDef` declaration that already drives
the visual (`JsonUiRenderer`) and MCP (`McpRenderer`) surfaces, and deliver them
Inertia-first in one `Inertia::from_projection` call. Route that frontend's writes through
the existing channel-agnostic `dispatch_write` kernel. The single structural change lifts
guard-visibility (`permitted_actions`) out of `ferro-mcp-server` into `framework` so guards
are evaluated in exactly one place. These are missing derivations, not new coupling — the
operability layer is already renderer-independent.

**Anchor spec:** `docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md`.

**Builds on shipped work:** `framework::write::dispatch_write` (Phase 231/232) channel-agnostic
transition kernel; the `ferro-mcp-server` `tools/list` guard filter (the extraction source);
`ferro-projections` (`ServiceDef`/`ActionDef`, `derive_intents()`); `ferro-inertia` (delivery
target); the `ferro-json-ui` `GET /data/{service}` read convention.

## v18.0 Requirements

### Derivation core

- [x] **SUBST-01**: A pure `schema_contract(&ServiceDef) -> SchemaContract` derivation in
  `ferro-projections` returns the service's field set, meanings, validations, and action
  definitions. It is schema-level and dependency-free (renders nothing — a sibling of
  `derive_intents()`) and snapshot-tested against the declaration.
- [x] **SUBST-02**: The "which actions are permitted for a record" logic is lifted out of
  `ferro-mcp-server` (`tools/list` filter) into `framework` as
  `permitted_actions(service, record, tenant, ctx) -> Vec<ActionName>`, evaluated by the
  same `GuardEvaluatorFn` the write path uses. After the refactor `ferro-mcp-server` **and**
  `ferro-inertia` both call `framework`'s `permitted_actions(...)`; guard-visibility is
  evaluated in exactly one place (no duplicate control surface).

### Inertia delivery

- [x] **SUBST-03**: `Inertia::from_projection(req, service, query) -> InertiaResponse` in
  `ferro-inertia` loads the tenant-scoped data shaped by the `ServiceDef` field set, attaches
  the `SchemaContract` and per-record `permitted_actions`, and serializes them as Inertia
  props `{ schema, data, permitted_actions }`. The helper lives on the framework-side Inertia
  facade (`framework/src/inertia/projection.rs`), not in the pure `ferro-projections` crate.
  (Corrected 2026-07-27, operator-approved: `framework` already depends on `ferro-inertia`, so the
  spec's literal `ferro-inertia` placement is a hard Cargo cycle — same class as Phase 261's
  `ferro-bundle`; the renderer-location intent is preserved since the helper is in the framework's
  Inertia delivery layer, not the pure projection crate.) Data reads are tenant-scoped
  (cross-tenant ids are not found) with filter/limit shaping matching the field set.

### Writes

- [x] **SUBST-04**: An Inertia form `POST /{service}/{action}` routes through the existing
  `dispatch_write(.., channel = "web")` kernel (guard re-eval + persist + audit). No new write
  path, no new failure semantics — the substrate adds read derivation only. Errors reuse the
  framework JSON error envelope and `WriteError` mapping across channels.

### Single source (tests)

- [x] **SUBST-05**: Parity is proven by test: (a) permitted-actions parity — for a record in
  a given state the set from `permitted_actions(...)` equals the guard-filtered set MCP
  `tools/list` returns for the same record and tenant, and changing state changes both
  identically (mirrors `single_source_both_channels`); (b) write parity — an Inertia
  `POST /{service}/{action}` reaches the same `dispatch_write` kernel MCP reaches, differing
  only by channel tag; (c) schema snapshot (SUBST-01); (d) data tenant-scoping (SUBST-03).

## Open Questions (resolve in planning)

- `SchemaContract` location — `ferro-projections` (pure schema, leaning) vs. `ferro-inertia`.
- Opt-in surface — a projection builder flag vs. a registration list mirroring MCP
  `exposed_services()`.
- Permitted-actions placement in props — per-record inline vs. a separate lookup.
- Pagination contract for the data query — cursor vs. offset; default/max limit.
- Confirm the `tools/list` filter can be lifted cleanly into `framework` without regressing
  the MCP surface.

## Out of Scope (v18.0)

- **Projection-native JSX/React component derivation** (scaffolds, typed hooks) and any
  **design-lint on generated frontends** — a separate later spec, gated on a coherence-at-scale
  validation.
- **Generic JSON delivery mode** (`GET /project/{service}`, `GET /data/{service}` for decoupled
  SPA / native clients) — additive over the identical derivation core, deferred until a consumer
  needs it; its token/CORS auth story lands with it.
- **New authentication model** — Inertia reuses the app's same-origin session/cookie + CSRF.
- **Any change to JSON-UI's role** — it remains the first-class velocity renderer; this
  substrate is the customization path.

## Traceability (v18.0)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| SUBST-01 | Phase 263 | Complete |
| SUBST-02 | Phase 263 | Complete |
| SUBST-03 | Phase 263 | Complete |
| SUBST-04 | Phase 263 | Complete |
| SUBST-05 | Phase 263 | Complete |
