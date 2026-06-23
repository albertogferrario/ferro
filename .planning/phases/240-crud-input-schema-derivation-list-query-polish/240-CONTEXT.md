# Phase 240: CRUD input-schema derivation + `list_` query polish - Context

**Gathered:** 2026-06-23
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults; review before planning)

<domain>
## Phase Boundary

Derive **correct, safe MCP input schemas** for the write verbs and **enrich the
`list_` query surface** — purely at the schema/derivation layer. Concretely:

1. **`create_<svc>` / `update_<svc>` / `delete_<svc>` tool *listing* + auto-derived
   `inputSchema`**, emitted by `render_exposed_tools` for opted-in projections,
   from the *existing* `field()` declarations (single source of truth). Field-set
   derivation reuses the `is_server_injected_field` boundary shipped in Phase 239.
2. **`list_<svc>` query polish**: add range/comparison filters
   (`<field>__{gt,gte,lt,lte,ne,in}`) and `sort` (`field` / `-field`) to the derived
   inputSchema **and** honor them in the read dispatch path (because `list_` already
   executes reads). Equality filters stay byte-for-byte back-compatible.

**This phase is read-execution + write-schema-emission only.** The
create/update/delete tools *appear in `tools/list` with correct schemas*, but their
**call/execution path is NOT wired here** — `derive_crud_plan` + the
`framework::write` kernel wiring is Phase 241. By contrast, `list_` polish lands
fully (schema + dispatch) because `list_` already dispatches today.

**Out of scope (later phases, do not build here):**
- `derive_crud_plan` + INSERT/UPDATE/soft-delete execution through `framework::write`
  (Phase 241).
- Write authorization (`read_write` scope + `mcp_write_ability` Gate), server-side
  `tenant_id` injection on writes, and the cross-tenant/soft-deleted non-disclosure
  envelope (Phase 242).
- Soft-delete data substrate (`deleted_at` column, read-path `deleted_at IS NULL`
  predicate, resolvers) — shipped in Phase 239; this phase *consumes* it.
- App `order` projection flip + e2e + catalog/docs (Phase 243).

</domain>

<decisions>
## Implementation Decisions

### Phase scope split (schema-emission vs execution)
- **D-01:** Phase 240 delivers, for `create_/update_/delete_<svc>`: tool *listing* in
  `render_exposed_tools` + a derived `inputSchema` per verb. It does **not** make those
  tools executable — calling them is wired in Phase 241. The planner should emit the
  tools and their schemas, and either (a) leave the `handle_tools_call` write path to
  return a not-yet-implemented/`confirmation_required`-style envelope for these new
  verbs, or (b) keep them schema-only until 241, whichever keeps the existing action
  write-path and the Phase 205 structured-envelope regression guard green. The split is
  a hard boundary, not a soft preference.
  *[auto] Recommended: emit tools + schemas now; defer execution wiring to 241 — matches
  spec "Within-Track sequencing" items 3 vs 4.*
- **D-02:** `list_` query polish (range ops + sort) lands **fully** in this phase —
  schema derivation in `schema.rs` **and** WHERE/ORDER-BY assembly in
  `ferro-mcp-server/src/dispatch.rs` — because `list_` already executes. `limit`/`offset`
  already derive (`build_input_schema`) and clamp in `dispatch`; **do not re-implement
  them** — only confirm they remain and are covered.

### create_<svc> field-set derivation (CRUD-01)
- **D-03:** The creatable field set = projection data fields that are **agent-writable**,
  built by reusing `ServiceDef::is_server_injected_field` (239) to drop Identifier,
  CreatedAt, and the tenant column, plus excluding `FieldMeaning::Sensitive` and list
  fields (`is_list`). Net exclusions: Identifier, CreatedAt, tenant column, Sensitive,
  list fields. Everything else declared via `field()` is a creatable input.
- **D-04:** **Status under a StateMachine:** when `service.state_machine` is `Some`, the
  `Status` field (`FieldMeaning::Status`) is **excluded** from the create schema (it is
  set server-side to the SM initial state in Phase 241). When **no** SM exists, a
  `Status` field is an ordinary creatable input. This is the spec's "state-machine-
  controlled fields stay workflow-only" rule.
- **D-05:** Also exclude `FieldMeaning::UpdatedAt` from write schemas (server-managed,
  same rationale as CreatedAt). `is_server_injected_field` covers Created/Identifier/
  tenant; extend the *write-schema exclusion predicate* (not necessarily that helper) to
  also drop UpdatedAt. Keep the exclusion logic in **one** shared predicate so create and
  update agree.

### update_<svc> patch schema (CRUD-02)
- **D-06:** `update_<svc>` schema = the **identifier field (required)** + the same
  data-field set as create but **all optional** (patch semantics — no `required` array
  beyond the identifier). Reuse the create field-set predicate (D-03/D-05) so the two
  never drift.
- **D-07:** Under an SM, `Status` is **never** an update input (same as create). When no
  SM exists, `Status` is an optional patch field.

### delete_<svc> schema (CRUD-03 schema portion)
- **D-08:** `delete_<svc>` schema = **identifier (required)** + a confirmation-token
  field, with `destructiveHint=true` on the tool. The confirmation *mechanism* and the
  soft-delete *execution* are Phase 241/242; this phase only emits a schema shaped to
  carry the token so the tool surface is correct. Mirror the existing destructive-action
  / `request_confirm_*` affordance shape already in `render_exposed_tools`.
  *[auto] Note: CRUD-03 is formally a Phase 241 requirement; only the delete tool's
  schema shape is in 240's scope so the derived surface is complete and consistent.*

### list_ range/comparison filters (CRUD-04)
- **D-09:** Emit range/comparison filter params as **flat sibling keys**
  `<field>__gt`, `<field>__gte`, `<field>__lt`, `<field>__lte`, `<field>__ne`,
  `<field>__in` in the `list_` inputSchema, alongside the existing equality params (which
  stay unchanged). `__in` is typed as an array; the others share the field's scalar JSON
  type via `data_type_to_json_schema`.
- **D-10:** **Op eligibility by field type:** `__ne` and `__in` derive for every field
  that currently passes `is_filter_field` (the equality allowlist). The ordered ops
  `__gt/__gte/__lt/__lte` derive only for **ordered/comparable** fields — numeric
  (`DataType::Integer`/`Float`) and date/time (`DataType::DateTime`/`Date`) columns. This
  may require widening eligibility beyond the current `is_filter_field` *meaning*
  allowlist (which omits Money/Quantity/Percentage); introduce a dedicated
  `is_range_filter_field` (or extend the allowlist) rather than overloading equality
  eligibility. Keep equality derivation exactly as-is for back-compat.
- **D-11:** **`sort` param:** a single optional string accepting `field` (asc) or
  `-field` (desc). The base field is **allowlisted** against the projection's filterable/
  sortable fields (reuse the dispatch filter-key allowlist validation). Keep the existing
  Identifier-based deterministic ORDER BY as the **tiebreaker** appended after the
  user sort, so offset pagination stays stable. Single sort key only this phase
  (multi-key sort deferred — YAGNI).

### dispatch execution for query polish (CRUD-04, read path)
- **D-12:** Extend `dispatch`'s WHERE-clause assembly to recognize `<field>__<op>` keys:
  split each key on the **last** `__`, validate the suffix against the op allowlist
  `{gt,gte,lt,lte,ne,in}` and the base against the field allowlist; map to the SQL
  operator with a **bound parameter** (`IN (?, …)` for arrays). Unknown op or
  non-filterable base → the same non-disclosing "unknown or non-filterable filter field"
  error already used for equality keys. All values bound (no interpolation), mirroring
  the existing parameterized read path; tenant + `deleted_at IS NULL` predicates (239)
  are applied after user filters exactly as today.
- **D-13:** Parse `sort` in dispatch into a validated ORDER BY clause (asc/desc by the
  `-` prefix), placed before the deterministic Identifier tiebreaker and the
  `LIMIT/OFFSET` clause.

### Testing (success criterion #4)
- **D-14:** **Table tests** (in `schema.rs` / `service.rs`): create field-set and update
  field-set derivation; `Status` inclusion/exclusion with vs without an SM; presence of
  the full `__{gt,gte,lt,lte,ne,in}` param set per eligible field type and the `sort`
  param; identifier-required-on-update and all-data-fields-optional.
- **D-15:** **sqlite in-memory dispatch tests** (matching existing read-path tests):
  range filters return the correct rows; `__in` array filtering; `sort=field` /
  `sort=-field` ordering; `limit`/`offset` still clamp; **back-compat** — pre-existing
  equality params produce identical results. Tool-listing tests assert the three write
  tools appear with correct schemas (extending the existing `render_exposed_tools` tests).

### Claude's Discretion
- Exact names of new schema builders (`build_create_input_schema` /
  `build_update_input_schema` / `build_delete_input_schema`) and the shared write-field
  exclusion predicate / `is_range_filter_field` helper — follow existing `schema.rs`
  naming.
- Whether the write-field exclusion predicate lives in `ferro-projections`
  (next to `is_server_injected_field`) or in `ferro-mcp-server/src/schema.rs` — planner's
  call, as long as create and update share one source and it composes with the 239
  helper. *Prefer co-locating the projection-level field-classification in
  `ferro-projections` so non-MCP renderers can reuse it.*
- JSON Schema niceties (per-op `description` strings, `format` propagation to range
  params) — follow the equality-filter precedent.
- Whether delete-tool emission is feature-gated behind `confirmation` like the existing
  destructive-action path.

### Considered but NOT adopted
- Reworking the equality `is_filter_field` allowlist to add numeric meanings globally —
  rejected; range eligibility is a *separate* concern (D-10), and changing equality
  eligibility would alter existing `list_` schemas (back-compat risk). Add a dedicated
  range predicate instead.
- Multi-key sort (`sort=a,-b`) — deferred (YAGNI); single key covers the milestone.
- Wiring create/update/delete execution in this phase — out of scope (Phase 241).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Anchor spec (read first)
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — Track A
  design. Especially "Derived tool surface" (§120–132, the per-verb input-schema table),
  "Declaration surface" (§99–118), "Query polish" decision (§91), "Within-Track
  sequencing" items 3 (schema derivation, this phase) vs 4 (`derive_crud_plan`, Phase
  241), and "Non-goals" (§239–244).

### Roadmap & requirements
- `.planning/ROADMAP.md` §"Phase 240" (lines ~3432–3463) — goal + 4 success criteria.
- `.planning/REQUIREMENTS.md` — **CRUD-01** (create tool + schema derivation),
  **CRUD-02** (update patch schema, data fields only), **CRUD-04** (list range/sort/
  pagination). CRUD-01's declaration surface already shipped (`5cb17d60`).

### Prior-phase context (substrate this phase consumes)
- `.planning/phases/239-soft-delete-data-model-deleted-at-migration/239-CONTEXT.md` —
  D-07 (`resolved_table`/`resolved_soft_delete_column`), D-11 (`is_server_injected_field`
  classification boundary), and the `deleted_at IS NULL` read predicate this phase's
  query polish runs on top of.

### Code to extend
- `ferro-mcp-server/src/schema.rs` — `build_input_schema` (equality + limit/offset
  already here; add range/sort), `is_filter_field`, `data_type_to_json_schema`,
  `build_action_input_schema` (the write-schema precedent to mirror for create/update/
  delete).
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools` (~line 69): emits `list_`
  + per-`ActionDef` write tools; add `create_/update_/delete_<svc>` emission here.
- `ferro-mcp-server/src/dispatch.rs` — `dispatch` read path: filter-key allowlist
  (~line 128), tenant predicate (~line 150), deterministic ORDER BY (~line 197),
  LIMIT/OFFSET (~line 214). Extend for `__op` filters + `sort`.
- `ferro-projections/src/service.rs` — `ServiceDef::is_server_injected_field` (~line 236),
  `resolved_table`/`resolved_soft_delete_column`, `creatable`/`updatable`/`deletable`/
  `mcp_write_ability`/`state_machine`. Basis for the write-field exclusion predicate.
- `ferro-projections/src/field.rs` — `FieldMeaning` enum + `infer_meaning`
  (Identifier, CreatedAt, UpdatedAt, Status, Sensitive, ForeignKey, Category, Boolean,
  Money/Quantity/Percentage, …). Basis for range-op eligibility (D-10) and write
  exclusions (D-03/D-05).
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_list` / `handle_tools_call`; Phase
  205 structured-envelope regression guard context (`jsonrpc.rs:215`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`is_server_injected_field`** (`service.rs:236`, Phase 239): already excludes
  Identifier/CreatedAt/tenant — the create/update field-set predicate composes with it
  instead of re-deriving.
- **`build_action_input_schema`** (`schema.rs:111`): the precedent for a write-tool
  inputSchema — injects the identifier field + maps declared inputs, excludes Sensitive.
  Create/update/delete schema builders mirror its shape.
- **`build_input_schema`** (`schema.rs:67`): equality filters + `limit`/`offset` already
  derive here; range/sort params are added to the same builder.
- **`render_exposed_tools`** (`renderer.rs:69`): the single place tool emission happens
  (list_ + ActionDef writes + confirm tools); the destructive `request_confirm_*` /
  `destructiveHint` pattern is the model for `delete_`.
- **`dispatch`** filter-key allowlist + parameterized SQL + tenant predicate
  (`dispatch.rs`): the `__op` filters and `sort` reuse the exact allowlist+bound-value
  shape; nothing in the new ops is interpolated.

### Established Patterns
- Tool schemas are **derived, never hand-declared** — adding a field changes the schema
  (AMCP-02 single-source). Create/update schemas must obey the same: derive from
  `field()`, no separate schema declaration.
- Read SQL is parameterized; filter KEYS are allowlisted against `service.fields` before
  any SQL. New ops and sort keys follow the same allowlist-then-bind discipline.
- "Always-on" predicates (tenant; `deleted_at IS NULL`) live in the shared read builder,
  not per-tool. Query polish must not disturb them.

### Integration Points
- `ServiceDef` (ferro-projections) → consumed by `schema.rs` + `renderer.rs` +
  `dispatch.rs` (ferro-mcp-server). The write-field exclusion predicate crosses this
  boundary (prefer authoring it in ferro-projections so other renderers reuse it).
- New write tools appear in `tools/list` this phase but only become callable in Phase
  241 — keep the boundary explicit so the envelope regression guard stays green.

</code_context>

<specifics>
## Specific Ideas

- Mirror `build_action_input_schema` line-for-line in structure for the create/update/
  delete builders — identifier injection, `data_type_to_json_schema` mapping, Sensitive
  exclusion — so the four write-schema builders read identically.
- Keep `limit`/`offset` untouched (already correct); the only new pagination work is
  ensuring `sort` composes cleanly with the existing deterministic ORDER BY tiebreaker.
- Add range eligibility as a **new** `is_range_filter_field` predicate; do not mutate
  `is_filter_field` (equality back-compat is a stated success criterion).

</specifics>

<deferred>
## Deferred Ideas

- **`derive_crud_plan` + create/update/delete execution** through `framework::write` —
  Phase 241 (spec sequencing item 4).
- **Write authorization + tenant injection + non-disclosure envelope** — Phase 242.
- **App `order` projection flip + e2e + catalog/docs** — Phase 243.
- **Multi-key sort** (`sort=a,-b`) — YAGNI this milestone.
- **Dedicated `get_<svc>` tool; per-field `immutable()`/`read_only()` write overrides** —
  spec non-goals (§239–244).

</deferred>

---

*Phase: 240-crud-input-schema-derivation-list-query-polish*
*Context gathered: 2026-06-23 (--auto)*
