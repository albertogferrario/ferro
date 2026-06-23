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
- [ ] **CRUD-02**: A projection opts into update via `.updatable(true)`, deriving
  `update_<svc>` (patch semantics, all fields optional) for **data fields only** — when a
  StateMachine exists, `Status` is never an update input (state changes go through action tools).
- [ ] **CRUD-03**: A projection opts into delete via `.deletable(true)`, deriving
  `delete_<svc>` that **soft-deletes** (sets `deleted_at`), is **confirmation-gated**, and is
  filtered out of `list_<svc>` and all read/update/delete paths.

### Query

- [ ] **CRUD-04**: `list_<svc>` supports range/comparison filters
  (`<field>__{gt,gte,lt,lte,ne,in}`), sort (`field` / `-field`), and `limit`/`offset`
  pagination — on top of the equality filters that already derive.

### Authorization & Dispatch

- [ ] **CRUD-05**: `create`/`update`/`delete` require `read_write` key scope and pass the
  `.mcp_write_ability` policy Gate; `tenant_id` is injected server-side and is never an agent
  input (the tenant column is excluded from every write schema). Cross-tenant / soft-deleted
  targets are indistinguishable from "not found" (non-disclosure).
- [ ] **CRUD-06**: CRUD verbs dispatch through the shipped `framework::write` kernel via a
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
| CRUD-01 | — | partial (declaration surface done) |
| CRUD-02 | — | pending |
| CRUD-03 | — | pending |
| CRUD-04 | — | pending |
| CRUD-05 | — | pending |
| CRUD-06 | — | pending |
| CRUD-07 | — | done (5cb17d60) |

*(Phase column filled by the roadmap.)*
