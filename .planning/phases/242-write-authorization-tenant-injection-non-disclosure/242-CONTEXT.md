# Phase 242: Write authorization, tenant injection & non-disclosure - Context

**Gathered:** 2026-06-24
**Status:** Ready for planning
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen, logged below)

<domain>
## Phase Boundary

Close the safety envelope on the CRUD write path delivered by Phases 239–241. Three
capabilities, no new dispatcher:

1. **Write authorization** — every `create_`/`update_`/`delete_<svc>` call requires
   `read_write` key scope (the scope gate already ships) **and** must pass the
   projection's `.mcp_write_ability` policy Gate before dispatch.
2. **Server-side tenant injection** — `tenant_id` is injected from context on create and
   predicated (`AND <tenant_column> = ctx`) on update/delete. The tenant column is already
   absent from every write input schema (Phase 239 `is_server_injected_field`), so an agent
   can neither set nor override it.
3. **Non-disclosure** — a cross-tenant or soft-deleted target is indistinguishable from a
   genuinely missing row: the same non-disclosing "not found" envelope, no row/column/filter
   leakage.

Plus verify (test-only) that the **shipped** CRUD-07 `validate()` write-ability fail-fast
rule (`5cb17d60`) holds at the authz/boot boundary.

**Not in scope:** any new write dispatcher, override-hook/idempotency/audit/confirmation
machinery (all reused from the `framework::write` kernel), the input-schema derivation
(Phase 240), or app-flip / e2e (Phase 243).
</domain>

<decisions>
## Implementation Decisions

### Write-ability Gate enforcement (CRUD-05, SC#1 second half)
- **D-01:** Enforce `.mcp_write_ability` via a **dedicated, fail-closed authorization
  signal** carried into the write path, checked in `ferro-mcp-server`
  (`handle_write_call`) **before** `dispatch_write`. A `None`/absent ability result denies.
- **D-02:** **Do NOT reuse `McpContext.evaluated_guards`** for this check.
  `renderer.rs:210` explicitly documents that the guard map is a *visibility* filter, **NOT**
  an authorization gate — conflating the two would erode the security boundary. The
  write-ability authorization is a separate input from the per-record visibility guards.
  *(Aligns with the "no duplicate control surface" convention — the host evaluates the real
  policy Gate; ferro-mcp-server enforces the pre-evaluated result fail-closed, exactly as the
  read path consumes pre-evaluated abilities rather than calling the Gate live.)*
- **D-03:** **Researcher must confirm** the concrete carrier: add an explicit field to
  `McpContext` (e.g. `write_authorized: Option<bool>` or an abilities map keyed by ability
  name) that the host populates by running `Gate::authorize_for(principal, ability)`. The MCP
  context principal today is `tenant_id` + `scope` (no full `User`) — the researcher resolves
  how the host derives the Gate principal and whether one boolean or an ability-keyed map is
  the right shape. Keep ferro-mcp-server free of a live policy/Gate dependency.

### Tenant injection wiring (CRUD-05, SC#2) — design locked by D-09
- **D-04:** `derive_crud_plan` fills `tenant_column: Some(TenantColumn { column })` from
  `svc.tenant_column` when the projection declares one (today it always emits `None`). When
  `svc.tenant_column` is unset, the plan stays `None` (non-tenant projection — unscoped, as
  on the read path's explicit non-tenant case).
- **D-05:** `framework::write::execute_crud_plan` binds the **runtime** `tenant_id` (already
  a parameter, currently `_tenant_id`) when `tenant_column` is `Some`:
  - *Create* → append `(tenant_column, tenant_id)` to the INSERT column list.
  - *Update* → add `AND <tenant_column> = ?` to the existing
    `WHERE id=? AND <soft_delete_column> IS NULL` predicate.
  - *Delete* → add `AND <tenant_column> = ?` to the soft-delete `WHERE` predicate.
- **D-06:** The runtime `tenant_id` is **never** stored in the serializable `CrudPlan`
  (only the column name is) — it flows from auth via the `dispatch_write` `tenant_id` param,
  consistent with the idempotency/audit tenant handling already in the kernel.
- **D-07:** Create defense-in-depth: the executor injects the tenant column from
  `tenant_id`; the derive path never copies it from agent inputs (it is write-excluded). A
  debug assertion that the tenant column is not already present in `columns` is acceptable
  but not required.

### Non-disclosure envelope (CRUD-05, SC#3)
- **D-08:** A cross-tenant or soft-deleted update/delete target falls through to
  **0 rows affected** because of the `AND <tenant_column> = ?` (and existing
  `AND <soft_delete_column> IS NULL`) predicate, which maps to the **existing**
  `WriteError::RecordNotFound` → `error_kind: "not_found"`,
  message `"record not found or already deleted"`. **No new error kinds, no distinct
  cross-tenant signal.** The SQL predicate is the non-disclosure mechanism — a foreign-tenant
  row is unaddressable, so it reads identically to a missing row.

### Authorization-deny vs target non-disclosure (boundary clarification)
- **D-09:** Two distinct response classes, deliberately kept separate:
  - **Authorization denial** (scope-deny: read key on a write tool — already shipped at
    `jsonrpc.rs:77`; and write-ability Gate-deny — new, D-01) is an **explicit** error: the
    caller lacks permission for the *verb class*. It is fine for the agent to learn it cannot
    write.
  - **Target non-disclosure** (cross-tenant / soft-deleted specific row, D-08) is **opaque**:
    the agent must NOT learn whether a specific out-of-tenant row exists. Returns the
    not-found envelope.
  - Rationale: permission-to-write-at-all is not secret; existence-of-a-specific-foreign-row
    is. Do not collapse these into one envelope.

### CRUD-07 verification (SC#4) — verify-only
- **D-10:** No new validation code. Add a **boot-time test** asserting
  `ServiceDef::validate()` rejects a projection that enables any CRUD verb
  (`.creatable`/`.updatable`/`.deletable`) without `.mcp_write_ability` — proving it is a
  config error at registration, never a silent deny at call time. The rule itself shipped in
  `5cb17d60`; this phase only verifies it at the authz boundary.

### Claude's Discretion
- Exact test-fixture layout, the choice between one `write_authorized` boolean vs an
  ability-keyed map (D-03, pending researcher confirmation), SQL placeholder/dialect details
  (SQLite `?` vs Postgres `$n` — follow the existing `execute_crud_plan` dual-dialect
  pattern), and the precise `McpContext` field name.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Spec & requirements
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — Track A
  design. **Authorization matrix §lines 176–186** (scope gate × policy Gate × tenant
  injection per tool class), **SQL shapes §lines 144–146 & 170** (create injects
  `tenant_id=ctx`; update/delete carry `AND tenant_id=ctx AND deleted_at IS NULL`),
  **non-disclosure §lines 184–186 & 193–194**, **validate() fail-fast §line 117**.
- `.planning/REQUIREMENTS.md` — **CRUD-05** (write authz + tenant injection + non-disclosure,
  this phase's sole owned requirement) and **CRUD-07** (validate() fail-fast — shipped,
  verified here).
- `.planning/ROADMAP.md` §"Phase 242" (lines 3491–3516) — goal + four success criteria.

### Extension points left by Phase 241 (fill these, do not rebuild)
- `ferro-projections/src/executor.rs` — `CrudPlan` (lines 156–196), `TenantColumn`
  (lines 130–136), `derive_crud_plan` (line 209). Every variant's `tenant_column` is `None`
  today (D-09 contract, documented at lines 151–155 & 204); Phase 242 fills it from
  `svc.tenant_column`.
- `framework/src/write/mod.rs` — `dispatch_write` (line 600, already takes `tenant_id`),
  `execute_crud_plan` (line 272, currently `_tenant_id` + `tenant_column: _` ignored — wire
  both here). SQL shapes documented at lines 258–270.
- `ferro-mcp-server/src/write_dispatch.rs` — `handle_write_call` CRUD path (lines 157–260);
  the Phase 242 marker at line 423 ("wires mcp_write_ability / per-record …"). The
  write-ability Gate check goes in front of `dispatch_write` here.
- `ferro-mcp-server/src/jsonrpc.rs` — scope gate (lines 72–87, **already enforces**
  read-key-rejects-write); the write-ability check is the missing second half of SC#1.
- `ferro-mcp-server/src/renderer.rs` — `McpContext` (lines 18–22: `tenant_id`,
  `evaluated_guards`, `scope`); **the visibility-not-auth warning at line 210** (the reason
  for D-02).

### Service declaration surface
- `ferro-projections/src/service.rs` — `tenant_column` field/builder (lines 88, 161–162),
  `is_server_injected_field` / write-exclusion (Phase 239), `mcp_write_ability` builder, and
  the shipped `validate()` write-ability rule (referenced by the lines around 2205/2285).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`framework::write::dispatch_write`** — single write kernel; already receives `tenant_id`
  from auth and threads it through guard re-eval, idempotency lookup/store, audit, and the
  override hook. Tenant injection extends `execute_crud_plan` *inside* this kernel — no new
  dispatch path.
- **`WriteError::RecordNotFound`** — already mapped to the non-disclosing
  `error_kind: "not_found"` envelope in `handle_write_call` (write_dispatch.rs ~line 240).
  The tenant predicate reuses it verbatim (D-08).
- **Scope gate** (`jsonrpc.rs:77`) — read-key-rejects-write already shipped; SC#1's scope
  half is done. Only the write-ability Gate half is net-new.
- **`TenantColumn` + `derive_crud_plan`** — purpose-built `None` slot waiting to be filled.

### Established Patterns
- **Pre-evaluated authorization** — ferro-mcp-server consumes authorization *results* in
  `McpContext`, it does not call the Gate live (read path uses `evaluated_guards`). Mirror
  this for writes with a *separate* signal (D-01/D-02), keeping the host as the policy owner.
- **Dual-dialect SQL** in `execute_crud_plan` (SQLite `?` / Postgres `$n`) — the tenant
  predicate must follow the same branching already present for the soft-delete predicate.
- **`tenant_id` from auth, never from payload** — reinforced throughout the kernel
  (write/mod.rs comments lines 92/138; idempotency scoping lines 479–502).

### Integration Points
- `McpContext` gains the write-authorization carrier (D-03) — touches the host that builds
  the context (consumer-MCP endpoint) plus `ferro-mcp-server`.
- `handle_write_call` (write_dispatch.rs) — insert the fail-closed Gate check before the
  `dispatch_write` call in the CRUD prefix loop.
- `execute_crud_plan` (framework write kernel) — the only place SQL changes.
</code_context>

<specifics>
## Specific Ideas

- The non-disclosure mechanism is the SQL predicate itself, not an envelope branch: a
  foreign-tenant row produces 0 affected rows → already-existing not-found envelope. Resist
  adding any cross-tenant-specific code path (that would itself be a disclosure vector).
- Keep "permission to write" (explicit deny) and "this specific row exists" (opaque
  not-found) on separate response paths (D-09). This is the one boundary most likely to be
  blurred during implementation.
- The write-ability Gate is authorization, **not** the per-record visibility guard — do not
  route it through `evaluated_guards` (D-02).
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. (App flip, e2e drive, structured-envelope
regression-guard extension, and catalog/docs are Phase 243, already roadmapped.)
</deferred>

---

*Phase: 242-write-authorization-tenant-injection-non-disclosure*
*Context gathered: 2026-06-24*
