# Requirements — v16.3 MCP CRUD Data Surface (Track A)

**Milestone goal:** A projection that opts in derives a complete, safe, tenant-scoped
CRUD interface (create / read+query / update / soft-delete) as MCP tools with zero
hand-written tool code. Foundational track of the broader MCP capability program
(Tracks A–D). Anchor spec:
`docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md`.

**Builds on shipped work:** v16.0 (231/232) StateMachine-derived executor + single
`framework::write` kernel; the `tools/call` `content[]` structured envelope (Phase 205).

## v16.3 Requirements

### CRUD Derivation

- [x] **CRUD-01**: A projection opts into create via `.creatable(true)`, deriving a
  `create_<svc>` MCP tool whose input schema is auto-derived from the existing `field()`
  declarations (excludes Identifier, CreatedAt, and the tenant column; `Status` is set to
  the StateMachine initial state when an SM exists; `Sensitive` excluded).
  *(Declaration surface shipped in `5cb17d60`; tool + schema derivation pending.)*
- [x] **CRUD-02**: A projection opts into update via `.updatable(true)`, deriving
  `update_<svc>` (patch semantics, all fields optional) for **data fields only** — when a
  StateMachine exists, `Status` is never an update input (state changes go through action tools).
- [x] **CRUD-03**: A projection opts into delete via `.deletable(true)`, deriving
  `delete_<svc>` that **soft-deletes** (sets `deleted_at`), is **confirmation-gated**, and is
  filtered out of `list_<svc>` and all read/update/delete paths.

### Query

- [x] **CRUD-04**: `list_<svc>` supports range/comparison filters
  (`<field>__{gt,gte,lt,lte,ne,in}`), sort (`field` / `-field`), and `limit`/`offset`
  pagination — on top of the equality filters that already derive.

### Authorization & Dispatch

- [x] **CRUD-05**: `create`/`update`/`delete` require `read_write` key scope and pass the
  `.mcp_write_ability` policy Gate; `tenant_id` is injected server-side and is never an agent
  input (the tenant column is excluded from every write schema). Cross-tenant / soft-deleted
  targets are indistinguishable from "not found" (non-disclosure).
- [x] **CRUD-06**: CRUD verbs dispatch through the shipped `framework::write` kernel via a
  derived `derive_crud_plan` (the CRUD analog of `derive_transition_plan`), reusing the
  existing override-hook registry, idempotency, channel-parameterized audit, and confirmation —
  single-source across the MCP and visual write surfaces. Does **not** rebuild the dispatcher.
- [x] **CRUD-07**: `ServiceDef::validate()` fails fast at registration when any CRUD verb is
  enabled without `mcp_write_ability`. *(Shipped in `5cb17d60`.)*

## Future Requirements (deferred)

- Dedicated `get_<svc>` single-record tool (currently covered by `list_<svc>` + id equality filter).
- Per-field `immutable()` / `read_only()` overrides (Track A derives field sets from `FieldMeaning`).

## Out of Scope

- **Tracks B/C/D** (richer write semantics, new capability classes, agent-experience meta-tools)
  — each is its own future milestone.
- The `tools/call` `content[]` fix — already resolved (Phase 205); this milestone only extends
  the structured envelope + regression guard to the new CRUD verbs.

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| CRUD-01 | Phase 240 | partial (declaration surface done; tool + schema derivation pending) |
| CRUD-02 | Phase 240 | Complete |
| CRUD-03 | Phase 241 | Complete |
| CRUD-04 | Phase 240 | Complete |
| CRUD-05 | Phase 242 | Complete |
| CRUD-06 | Phase 241 | Complete |
| CRUD-07 | Phase 242 (verified) | done (5cb17d60) |

**Foundation/integration phases (own no requirement uniquely):**
- Phase 239 — soft-delete data model + `deleted_at` migration (substrate for CRUD-03 + CRUD-05).
- Phase 243 — app integration + e2e + envelope guard + catalog/docs (validates CRUD-01..07 end-to-end).

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

# Requirements — v16.5 JSON-UI Design System — IN PROGRESS

**Status:** Started 2026-07-03 (current milestone). Independent of v16.4 (reserved
244–249); the two share no code surface. Consumer-paired with gestiscilo Phase 232.

**Milestone goal:** Complete the design system above the token layer — density/motion/
focus-ring tokens with opinionated defaults, a canonical variant vocabulary across all
47 builtin components, and composition patterns codified as machine-readable,
intent-keyed lint rules enforced at the agent-authoring boundary. Anchor spec:
`docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md`.

## v16.5 Requirements

### Token Vocabulary v2

- [x] **DS-01**: The fixed token vocabulary grows from 23 to 30 slots — density
  (`--spacing`), motion (`--motion-duration-fast/base/slow`, `--motion-ease`), focus ring
  (`--color-ring`), display font (`--font-display`) — every new slot with a default in the
  base CSS and `default.css` (light + dark), so **every valid v1 theme remains a valid v2
  theme unchanged**. Regenerated `ferro-base.css` exposes the new utilities as `var()`
  references and collapses motion durations under `prefers-reduced-motion`.
- [x] **DS-02**: `default.css` is refreshed to the documented design language (cool-tinted
  neutrals in both modes, a single accent used sparingly, separation via spacing and
  contrast before borders, small consistent radii, one elevation treatment);
  `docs/src/features/themes.md` documents v2 plus the root-font-size type-scaling recipe.

### Component Variant Discipline

- [x] **DS-03**: All 47 builtin components use the canonical `variant`
  (primary/secondary/outline/ghost/destructive), `tone` (neutral/success/warning/
  destructive), and `size` (sm/md/lg) enums; catalog prop schemas enforce them; drift
  guards extend to the enum sets; a migration table lists every rename for consumers.
- [ ] **DS-04**: Every interactive component has hover, `focus-visible` (ring from
  `--color-ring`), and disabled states; transitions use the motion tokens at
  frequency-appropriate tiers; `ferro-base.css` regenerated after class changes.

### Pattern Layer

- [ ] **DS-05**: `Spec` gains an optional `design` field (`intent` + `allow`); a pure
  `design::lint(&Spec)` engine implements the ~10 intent-keyed rules (intent inferred
  with info-level finding when undeclared); lint never affects rendering or validation;
  each rule ships a violating/conforming unit-test pair.
- [ ] **DS-06**: `ferro design:lint [path] [--json] [--deny]` CLI — recursive over spec
  JSON files, human-readable + `--json` output, exit 0 always unless `--deny`.

### Agent Surface & Docs

- [ ] **DS-07**: ferro-mcp gains a `design_lint` tool (inline spec or path);
  `json_ui_catalog` extends with the canonical variant vocabulary and per-component
  design guidance; `generation_context` gains a design-system summary.
- [ ] **DS-08**: New `docs/src/design-system/` chapter (principles, token v2 reference,
  variant vocabulary, pattern catalog, linting guide); single crates.io publish at the
  end of the milestone.

## Out of Scope (v16.5)

- New crates — pattern rules live in ferro-json-ui; tokens stay in ferro-theme.
- Hard validation — rendering never rejects a spec on design grounds.
- Per-size type tokens or font-weight tokens (root `font-size` is the type-scale mechanism).
- Retroactive redesign of intent templates (`ThemeTemplates`).
- Consumer adoption (gestiscilo Phase 232) — separate repo, gated on the Phase 253 publish.

## Traceability (v16.5)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| DS-01 | Phase 250 | Complete (verified 2026-07-03) |
| DS-02 | Phase 250 | Complete (verified 2026-07-03) |
| DS-03 | Phase 251 | Not started |
| DS-04 | Phase 251 | Not started |
| DS-05 | Phase 252 | Not started |
| DS-06 | Phase 252 | Not started |
| DS-07 | Phase 253 | Not started |
| DS-08 | Phase 253 | Not started |
