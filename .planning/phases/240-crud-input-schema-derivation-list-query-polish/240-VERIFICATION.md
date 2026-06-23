---
phase: 240-crud-input-schema-derivation-list-query-polish
verified: 2026-06-23T18:30:00Z
status: passed
score: 4/4
overrides_applied: 0
---

# Phase 240: CRUD Input-Schema Derivation + List Query Polish — Verification Report

**Phase Goal:** Derive correct, safe MCP input schemas for `create_`/`update_`/`delete_<svc>` from the existing `field()` declarations (single source of truth) and extend the already-derived `list_<svc>` equality filters with range/comparison ops, sort, and pagination — so a projection authored for reads yields correct write schemas and a richer query surface for free.
**Verified:** 2026-06-23T18:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | An opted-in projection lists a `create_<svc>` tool whose input schema contains exactly the creatable data fields — Identifier, CreatedAt, tenant column, and Sensitive absent; Status absent when SM exists, present without SM | VERIFIED | `build_create_input_schema` (schema.rs lines 249-276) calls `service.is_write_excluded_field(field, exclude_sm_status)` where `exclude_sm_status = service.state_machine.is_some()`. Gate chain: server-injected → UpdatedAt → Sensitive → is_list → SM-Status. `render_create_tool` emitted only when `service.creatable` (renderer.rs lines 94-98). Tests `test_create_schema_exclusions` and `test_create_schema_status_sm` pass. |
| 2 | `update_<svc>` requires the identifier and exposes data fields as optional (patch); Status never an update input under an SM | VERIFIED | `build_update_input_schema` (schema.rs lines 284-336): Identifier injected first into `properties` + `required`; data fields added to `properties` only (never `required`). Explicit Identifier skip in the patch loop (WR-03 hardening at line 314). Status excluded when SM present via same `is_write_excluded_field` predicate. `required[]` is exactly `["id"]`. Tests `test_update_schema_patch_semantics` and `test_update_schema_status_sm` pass. |
| 3 | `list_<svc>` accepts `<field>__gt/gte/lt/lte/ne/in`, `sort=field` / `sort=-field`, and `limit`/`offset`, while pre-existing equality params remain unchanged (back-compat) | VERIFIED | Schema: `build_input_schema` extended with `__ne`/`__in` for every `is_filter_field` field, `__gt/__gte/__lt/__lte` for every `is_range_filter_field` field, and a `sort` string param (schema.rs lines 132-178). Execution: `dispatch` restructured with `split_op_key` (rfind at dispatch.rs line 56), op-to-SQL mapping, `__in` parameterized expansion, sort parsed and validated with `is_filter_field || is_range_filter_field` (WR-01 fix at dispatch.rs line 156). Equality path byte-for-byte unchanged. Tests `range_filter_returns_correct_rows`, `in_filter_returns_correct_rows`, `sort_orders_rows`, `equality_filter_backcompat` all pass against SQLite in-memory seeded data. |
| 4 | Field-set and query-param derivation are covered by table tests asserting Status inclusion/exclusion with vs without an SM and the full range/sort/pagination param set | VERIFIED | ferro-projections: `is_write_excluded_field_gates` (service.rs line 2167) — 9-case table covering all 5 gates including SM-conditional Status pair. ferro-mcp-server: 8 schema tests (exclusions, SM-Status pair, patch semantics, range/ne/in/sort/backcompat), 4 dispatch integration tests, 4 renderer emission tests, Phase 205 NTI guard + WR-04 regression. Full suite: 56 ferro-mcp-server lib tests + 277 ferro-projections — 0 failures. |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projections/src/service.rs` | `ServiceDef::is_write_excluded_field` predicate + table test | VERIFIED | `pub fn is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool` at line 254; doc-commented; 5 gates in spec order; delegates Gate A to `is_server_injected_field`. Table test `is_write_excluded_field_gates` at line 2167 covers 9 cases. |
| `ferro-mcp-server/src/schema.rs` | `is_range_filter_field` + three write-schema builders + extended `build_input_schema` | VERIFIED | `is_range_filter_field` at lines 52-69 (DataType-based, separate from `is_filter_field`); `build_create_input_schema` at 249; `build_update_input_schema` at 284; `build_delete_input_schema` at 343; `build_input_schema` extended at 132-178. |
| `ferro-mcp-server/src/renderer.rs` | `render_create_tool` / `render_update_tool` / `render_delete_tool` + flag-gated emission | VERIFIED | Three private helpers at lines 244, 270, 297 call respective Plan 02 builders. Emission block at lines 90-108 gated on `service.creatable/updatable/deletable`. `disambiguate_write_tool_collisions` unchanged. `delete_` has `destructive(true)`; create/update have `destructive(false)`. |
| `ferro-mcp-server/src/write_dispatch.rs` | CRUD verb NTI detection before `find_action`, returns `CallToolResult::structured` | VERIFIED | NTI detection block at lines 155-180 using `crud_verb_opted_in` closure gated on matching flag. Appears before `find_action` call at line 192. Returns `CallToolResult::structured({error_kind: "not_yet_implemented", ...})`. Never reaches -32601. |
| `ferro-mcp-server/src/dispatch.rs` | `split_op_key` + `__op` filter loop + sort parsing/ORDER BY | VERIFIED | `split_op_key` at line 56 uses `rfind("__")`. Filter loop at lines 172-259 routes equality vs `__op` in a single pass. `__in` expansion with parameterized placeholders at lines 221-228. Sort extracted before filter loop at lines 135-165. ORDER BY four-arm match at lines 321-328. |
| `ferro-mcp-server/src/jsonrpc.rs` | Phase 205 guard extended with CRUD verb test; WR-04 regression test | VERIFIED | `crud_tool_call_nti_parses_as_valid_mcp_content` at line 415 asserts NTI envelope shape (is_error=false, structured_content.error_kind="not_yet_implemented"). `crud_nti_not_returned_when_verb_flag_disabled` at line 487 asserts unflagged verb returns -32601. Both pass. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `schema.rs::build_create_input_schema` | `service.rs::is_write_excluded_field` | field-set exclusion | WIRED | `service.is_write_excluded_field(field, exclude_sm_status)` at schema.rs line 255 |
| `schema.rs::build_update_input_schema` | `service.rs::is_write_excluded_field` | same shared predicate | WIRED | Called at schema.rs line 317 — no drift possible between create and update |
| `renderer.rs::render_create_tool` | `schema.rs::build_create_input_schema` | inputSchema construction | WIRED | `crate::schema::build_create_input_schema(service)` at renderer.rs line 249 |
| `renderer.rs::render_update_tool` | `schema.rs::build_update_input_schema` | inputSchema construction | WIRED | `crate::schema::build_update_input_schema(service)` at renderer.rs line 275 |
| `renderer.rs::render_delete_tool` | `schema.rs::build_delete_input_schema` | inputSchema construction | WIRED | `crate::schema::build_delete_input_schema(service)` at renderer.rs line 302 |
| `write_dispatch.rs` NTI block | `rmcp::CallToolResult::structured` | NTI envelope | WIRED | `CallToolResult::structured(serde_json::json!({...}))` at write_dispatch.rs line 173 |
| `dispatch.rs::dispatch` | `schema.rs::is_range_filter_field` | range-op base-field allowlist | WIRED | `use crate::schema::{is_filter_field, is_range_filter_field}` at dispatch.rs line 1; applied at lines 156 (sort) and 194-199 (gt/gte/lt/lte allowlist) |
| `dispatch.rs::dispatch` IN clause | `json_to_sea_value` + `placeholder` | parameterized IN | WIRED | `(0..arr.len()).map(|i| placeholder(backend, idx + i))` at dispatch.rs line 221; elements bound via `json_to_sea_value` at line 227 |

### Data-Flow Trace (Level 4)

Not applicable — this phase derives and emits input schemas (read by agents) and extends the list read path. There are no new UI components rendering dynamic data. The dispatch path is verified by SQLite in-memory integration tests.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All ferro-mcp-server tests pass | `cargo test -p ferro-mcp-server` | 56 lib + 5 dispatch integration + 5 tools_list + 4 tenant_isolation tests: 0 failures | PASS |
| All ferro-projections tests pass | `cargo test -p ferro-projections` | 277 lib tests + 8 doc-tests: 0 failures | PASS |
| NTI block is before `find_action` | line ordering grep | `not_yet_implemented` at line 155-180, `find_action` call at line 192 — NTI block precedes it | PASS |
| Unflagged verb returns -32601 not NTI | `crud_nti_not_returned_when_verb_flag_disabled` test | Passes — asserts `response["error"]["code"] == -32601` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CRUD-01 | Plans 01, 02, 03 | `create_<svc>` tool with auto-derived input schema excluding Identifier/CreatedAt/tenant/Sensitive; Status excluded under SM | SATISFIED | `build_create_input_schema` + `render_create_tool` + flag-gated emission; 2 table tests cover SM-conditional Status |
| CRUD-02 | Plans 01, 02, 03 | `update_<svc>` patch schema; Status never update input under SM | SATISFIED | `build_update_input_schema` + `render_update_tool`; patch semantics verified by `test_update_schema_patch_semantics` |
| CRUD-04 | Plans 02, 04 | `list_<svc>` range/comparison filters, sort, limit/offset on top of equality filters | SATISFIED | Schema extension in `build_input_schema`; dispatch execution in extended `dispatch()`; 4 SQLite integration tests + 3 schema tests + 1 backcompat test |
| CRUD-03 | Phase 241 | `delete_<svc>` actual soft-delete execution | NOT IN SCOPE | `delete_<svc>` tool is listed with schema (Phase 240) but execution wired in Phase 241 |
| CRUD-05 | Phase 242 | `create`/`update`/`delete` require `read_write` scope + Gate; tenant injection | NOT IN SCOPE | Authorization is Phase 242 |
| CRUD-06 | Phase 241 | CRUD verbs dispatch through `framework::write` kernel via `derive_crud_plan` | NOT IN SCOPE | Execution is Phase 241 |

### Scope Boundary Checks (CONTEXT.md D-01/D-02)

| Check | Expected | Status | Evidence |
|-------|----------|--------|---------|
| CRUD verb call path returns NTI envelope, not -32601 | `CallToolResult::structured` with `error_kind=not_yet_implemented` | VERIFIED | write_dispatch.rs lines 155-180; asserted by `crud_tool_call_nti_parses_as_valid_mcp_content` test |
| NTI detection before `find_action` | Line number of NTI block < line number of `find_action` call | VERIFIED | NTI at lines 155-180, `find_action` call at line 192 |
| No `derive_crud_plan` | Not present in write_dispatch.rs | VERIFIED | Grep confirms no `derive_crud_plan` in write_dispatch.rs |
| No write execution (INSERT/UPDATE/soft-delete) reached | No such SQL in write path for CRUD verbs | VERIFIED | NTI block returns before tenant check; no write path code added for CRUD verbs |
| No write authorization implementation | `read_write` scope check + Gate deferred to Phase 242 | VERIFIED | The NTI block runs before the tenant check, and `mcp_write_ability` enforcement is not added in Phase 240 |
| Tenant predicate unchanged after query-polish extension | `tenant_column` guard and `deleted_at IS NULL` predicate remain | VERIFIED | dispatch.rs lines 262-290 (tenant) and 280-290 (soft-delete) unchanged; Phase 239 behavior confirmed by `tenant_scoping` / `soft_delete_excluded` tests continuing to pass |
| All new `__op`/sort values are bound parameters + allowlisted | Op suffix mapped to fixed SQL constant; field names from service.fields; values via `json_to_sea_value` | VERIFIED | dispatch.rs lines 177-237: exhaustive match on op suffix, field validated from `service.fields` before SQL assembly, values bound via `json_to_sea_value` + `placeholder` |

### Code-Review Findings (240-REVIEW-FIX.md) — All Resolved

| ID | Finding | Status | Evidence |
|----|---------|--------|---------|
| WR-01 | Sort must accept range-filterable fields (Money/Quantity/Percentage can be sorted) | RESOLVED | dispatch.rs line 156: `is_filter_field(f) \|\| is_range_filter_field(f)` |
| WR-02 | Schema property descriptions must be informative | RESOLVED | schema.rs: `"Value for the {field} field"` (create), `"New value for the {field} field"` (update), `"ID of the {svc} record to delete"` (delete) |
| WR-03 | `build_update_input_schema` must explicitly skip Identifier in patch loop | RESOLVED | schema.rs line 314: `if matches!(field.meaning, FieldMeaning::Identifier) { continue; }` |
| WR-04 | NTI envelope gated on matching opt-in flag; unflagged verb returns -32601 | RESOLVED | write_dispatch.rs `crud_verb_opted_in` closure at line 161; `crud_nti_not_returned_when_verb_flag_disabled` regression test at jsonrpc.rs line 487 |

### Anti-Patterns Found

No blockers. The NTI detection block (`error_kind: "not_yet_implemented"`) is an intentional, documented stub representing the Phase 240 scope boundary. It is not a content stub — it advertises the tools with correct schemas and returns a valid MCP response. Phase 241 removes it and wires execution.

### Human Verification Required

None. All success criteria are verifiable programmatically and confirmed by passing test suites.

### Gaps Summary

No gaps. All 4 roadmap success criteria are verified in the actual code, all 3 plan-level requirements (CRUD-01, CRUD-02, CRUD-04) are satisfied, all code-review findings (WR-01 through WR-04) are resolved, and all scope boundary constraints from CONTEXT.md (D-01/D-02) are honored. The test suites (277 ferro-projections + 56 ferro-mcp-server lib tests) pass with 0 failures.

---

_Verified: 2026-06-23T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
