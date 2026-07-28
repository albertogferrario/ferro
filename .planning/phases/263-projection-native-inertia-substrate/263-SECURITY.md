---
phase: 263
slug: projection-native-inertia-substrate
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-28
---

# Phase 263 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| declaration → contract | `ServiceDef` (developer-authored, trusted) transformed into `SchemaContract`. No external/untrusted input. | Developer-declared field names/meanings |
| pre-computed guard map → visibility filter | `evaluated_guards: HashMap<String,bool>` populated from live DB once per request; `permitted_actions` reads it to decide what to SHOW. | Boolean guard results |
| surface listing → write execution | Actions shown by `permitted_actions` are advisory. Authorization happens at `dispatch_write` with live guard re-evaluation. | Action names (display only) |
| caller filters/limit → SQL query | `filters: serde_json::Value` and `limit`/`offset` are untrusted request input; query uses bound parameters. | Untrusted filter values |
| tenant_id → row visibility | `tenant_id: Option<i64>` scopes every read; cross-tenant rows excluded. | Tenant-scoped row data |
| client → Inertia read | `ProjectionQuery` (filters/limit/offset) and authenticated `tenant_id` cross into tenant-scoped data query. | Request-supplied query params |
| client → Inertia write | Form input crosses into `dispatch_write(channel="web")` where guards are re-evaluated server-side. | Form field values |
| props → frontend | `permitted_actions` in props is advisory display data, NOT an authorization grant. | Action name list |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-263-01 | Information disclosure | `SchemaContract` echoes `FieldMeaning::Sensitive` field names | accept | Names already exposed via MCP `tools/list` inputSchema and visual renderer; field VALUES remain governed by tenant-scoped data query | closed |
| T-263-02 | Tampering | Impure derivation leaks runtime state into "pure" contract | mitigate | `schema_contract.rs` has no `async`/`tokio`/`sea_orm` (grep-verified per VERIFICATION truth #2); `ferro-projections` is a leaf with zero runtime deps | closed |
| T-263-03 | Elevation of privilege | `permitted_actions` treated as an authorization gate | mitigate | Rustdoc marks it visibility-only; writes go through `dispatch_write` which re-evaluates guards via live `GuardEvaluatorFn`; no new write path added | closed |
| T-263-04 | Elevation of privilege | Lift accidentally widens MCP tool set | mitigate | 1:1 extraction filtering only on `action.preconditions`; `guard_visibility_unchanged_after_lift` regression test + 72 ferro-mcp-server tests green | closed |
| T-263-05 | Repudiation / drift | Two guard-visibility sites diverge | mitigate | Exactly one `== Some(&false)` evaluation site: `framework/src/permitted_actions.rs:29` (grep-verified per VERIFICATION truth #5) | closed |
| T-263-06 | Information disclosure | `dispatch` returns rows outside caller's tenant | mitigate | Tenant predicate injected as bound parameter; `tenant_scoping`, `tenant_isolation`, `tenant_fail_closed` tests green | closed |
| T-263-07 | Tampering | Malicious `filters` keys/values reach raw SQL | mitigate | Values bound via `json_to_sea_value`/`placeholder`; keys validated via `is_filter_field`/`is_range_filter_field` allowlists; invalid filters return `InvalidFilter` | closed |
| T-263-08 | Denial of service | Unbounded page request | mitigate | `MAX_LIMIT=100` hard cap enforced inside `dispatch` regardless of caller (grep-verified per VERIFICATION truth #9) | closed |
| T-263-09 | Regression | Error-type swap changes query semantics | mitigate | `ProjectionReadError` maps 1:1 back to `crate::Error` in wrapper; full `cargo test -p ferro-mcp-server` 64/64 green | closed |
| T-263-10 | Information disclosure | `from_projection` returns rows for another tenant | mitigate | Data load goes through `framework::projection_read::dispatch` with authenticated `tenant_id`; `data_tenant_scoping` tests green | closed |
| T-263-11 | Elevation of privilege | Frontend treats `permitted_actions` as permission | mitigate | Writes ONLY through existing `POST /{service}/{action}` → `dispatch_write(channel="web")` re-evaluating guards; exactly one `dispatch_write` call site (Task 2 verified) | closed |
| T-263-12 | Elevation of privilege | Dependency cycle forces wrong placement | mitigate | Task 0 `cargo tree` self-check confirms cycle-free: `ferro-inertia` has no `framework`/`ferro-rs` or `ferro-mcp-server` dep | closed |
| T-263-13 | Denial of service | Unbounded `limit` in `ProjectionQuery` | mitigate | `dispatch` enforces `MAX_LIMIT=100` regardless of `ProjectionQuery.limit`; default is 25 | closed |
| T-263-14 | Elevation of privilege | Inertia and MCP surfaces diverge on visible actions | mitigate | `permitted_actions_parity` test asserts SET EQUALITY between both surfaces; guard flip changes both identically | closed |
| T-263-15 | Information disclosure | Cross-tenant row leakage regression | mitigate | `cross_tenant_id_not_found` test: id=3 (tenant 2) scoped to tenant 1 returns empty; runs under `--all-features` | closed |
| T-263-16 | Elevation of privilege | Inertia write path diverges from guarded kernel | mitigate | `single_source_inertia_reuses_web_channel` asserts identical `to_state`; audit channel tag is the ONLY divergence | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-263-01 | T-263-01 | `SchemaContract` echoes field names/meanings already disclosed by MCP `tools/list` inputSchema and the visual renderer. Field VALUES for Sensitive fields remain governed by the tenant-scoped data query (`readable` gate). No new disclosure surface. | Alberto | 2026-07-28 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-28 | 16 | 16 | 0 | Claude (gsd-security-auditor workflow) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log (AR-263-01)
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-28
