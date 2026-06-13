# Phase 217: Tenant Context + Per-Tenant API-Key Auth - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults grounded in the v15.0 research docs and the phase success criteria; log in `217-DISCUSSION-LOG.md`)

<domain>
## Phase Boundary

Make the consumer MCP endpoint tenant- and permission-aware, and add a per-tenant API key as a second authentication path alongside the existing OAuth JWT path.

In scope (AMCP-01, AMCP-02):
- Extend `McpContext` (today an empty `struct McpContext;`) to embed `tenant_id: Option<i64>` + `evaluated_guards: HashMap<String, bool>`, sourced from `BaseContext`. Every `tools/list` and `tools/call` path reads from this context.
- Add a SHA-256 API-key validation branch that resolves the same `tenant_id` as the equivalent OAuth JWT request; both paths produce the same `BearerCheck::Authenticated(principal)` outcome type.
- API keys carry an explicit scope (`read` / `read_write`): a `read` key lists only read tools and is rejected on any `tools/call` targeting a write tool.
- Invalid/expired keys are rejected before tool routing, identical to the OAuth invalid-token path.
- Cross-tenant isolation test: authenticated as tenant A, no listing or call surfaces tenant B data.

Out of scope (later v15.0 phases): write-tool rendering from `ActionDef` (218), write dispatch + server-side guard re-evaluation (219), confirmation gating (220), inbound NL loop (221). This phase establishes the context + auth foundation that all of those build on. Write tools do not exist yet in 217 — the scope-rejection path is wired and tested against the (still empty) write-tool set so it is correct the moment 218 adds write tools.

</domain>

<decisions>
## Implementation Decisions

### API-key validator placement (D-01)
- **D-01:** The API-key validator lives in `ferro-mcp-oauth/src/validate.rs` as `validate_api_key(header, &db, expected_tenant) -> BearerCheck`, parallel to the existing `validate_bearer`. It returns the **same `BearerCheck::Authenticated(principal)`** outcome type. `ferro-mcp-server/src/auth.rs` becomes a thin unifier (`resolve_tenant`) that branches on the token shape and delegates to one of the two `ferro-mcp-oauth` validators.
  - **Rationale:** SC#2 mandates "both paths produce the same `BearerCheck::Authenticated(principal)` outcome type." `BearerCheck` is owned by `ferro-mcp-oauth`. STACK.md (§"Crate placement") explicitly assigns the validator to the auth crate, parallel to `validate_bearer`. This supersedes the ARCHITECTURE.md build-order sketch that placed `resolve_tenant_from_api_key` inside `ferro-mcp-server` — the discrepancy is resolved in favor of the auth crate so there is one auth outcome type, not two. (Researcher: confirm both validators can share the `McpTokenClaims`-shaped principal `json!({"sub":..., "tenant_id":...})`.)

### Branch detection (D-02)
- **D-02:** Both credentials arrive as `Authorization: Bearer <token>`. The unifier branches on token shape: a `ferro_`-prefixed token routes to `validate_api_key`; otherwise it is treated as a JWT and routed to `validate_bearer`. Absent header → `Unauthenticated` (401), identical to today.

### Key hashing & storage (D-03)
- **D-03:** Keys are looked up by SHA-256 hash: `SELECT tenant_id, scope FROM api_keys WHERE key_hash = SHA256(key) AND revoked_at IS NULL` (soft-revoke via `revoked_at`/`active`). Plaintext keys are **never stored** — only `key_hash`. Uses `sha2 = "0.10"` (already a `ferro-mcp-oauth` dependency). `subtle` (also already present) is available if a constant-time comparison is wanted, but a hash-indexed lookup on a high-entropy secret is the primary defense.

### Key generation & rotation (D-04)
- **D-04:** Provide a framework helper to mint a key: generate a `ferro_`-prefixed high-entropy token via `crypto`-grade randomness, return the plaintext **once** plus the `key_hash` to persist. Rotation is a first-class operation (issue new + soft-revoke old), not delete-and-recreate. (Researcher: confirm whether the v8.1 `ferro make:api-key` CLI already ships a generator/schema to reuse before designing a new one — STACK.md §"Crate placement" flags this.)

### api_keys table ownership (D-05)
- **D-05:** The framework defines the **canonical `api_keys` schema** (columns: `id`, `tenant_id`, `key_hash`, `scope`, `revoked_at`/`active`, timestamps); the consumer app runs the migration. `ferro-mcp-oauth` ships only the lookup contract (`validate_api_key`) + the generator helper, never hardcoding app identity. `ferro-mcp-server` integration tests create rows directly against this schema.
  - **Rationale:** ARCHITECTURE build-order says "migration belongs in the consumer app"; STACK.md says "reuse whatever v8.1 `make:api-key` generates." Reconciled: framework owns the schema definition (single source of truth, project-agnostic), consumer owns running it. Avoids each consumer hand-rolling a divergent table.

### Scope model (D-06)
- **D-06:** API keys carry an explicit `scope: read | read_write` enum field **on the key record**, present from the first migration (retrofitting scope later means rotating every key — PITFALLS §4). `tools/list` filters tools to the key's scope; `tools/call` re-checks the key scope before dispatching a write tool, independently of the listing filter. In 217 there are no write tools yet, so the read-scope filter is a no-op on listing and the write-rejection branch is exercised by a test that asserts a `read_write`-only operation is rejected for a `read` key once 218 lands (or via a synthetic write-tool fixture in 217's test).
  - **Not a duplicate control surface:** scope governs *the credential's* permission (a read-only key issued to a third-party agent for a tenant who can write); `ServiceDef.mcp_ability` governs *the tenant's* ability. Orthogonal axes — both legitimately exist (re: no-duplicate-control-surface convention).

### McpContext threading (D-07)
- **D-07:** `McpContext { tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool> }`, `#[derive(Debug, Clone, Default)]`. Constructed at the top of the request handler after auth resolves `tenant_id`. `handle_tools_call` passes `tenant_id` into the existing fail-closed `dispatch()` (already takes `Option<i64>`). `evaluated_guards` is embedded now but populated in the write-tool phases (218/219) where guards gate action tools — in 217 it may be an empty map; the field and its read sites exist so later phases don't reshape the context.

### Error handling (D-08)
- **D-08:** Add an `Auth(String)` variant to `ferro-mcp-server/src/error.rs`. Invalid/expired API key → rejected before any tool routing, mapped to the **same** JSON-RPC error envelope the OAuth invalid-token path already returns (SC#4: "identical to the existing OAuth invalid-token path"). No new error code class.

### Cross-tenant isolation test (D-09)
- **D-09:** Extend the existing **non-ignored** integration test `ferro-mcp-server/tests/mcp_tenant_isolation.rs` in the same commit: authenticate via API key as tenant A, assert no `tools/list` entry or `tools/call` result surfaces tenant B data; assert tenant-A-key and tenant-A-JWT resolve the same `tenant_id`. (PITFALLS §1 requires the fixture update to land with the auth change, not as a follow-up.)

### Claude's Discretion
- Exact column names/types in the `api_keys` migration (within D-05's shape).
- Whether the `read`/`read_write` scope is a SeaORM enum, a `TEXT` check-constrained column, or a small int — researcher/planner pick per existing ferro migration conventions.
- Token entropy length and the exact `ferro_` prefix format, provided it is unambiguously distinguishable from a JWT.
- Whether `validate_api_key` takes `expected_tenant: Option<i64>` (mirroring `validate_bearer`'s signature) or resolves tenant purely from the row.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v15.0 design (the de-facto spec for this milestone)
- `.planning/research/ARCHITECTURE.md` §"Decision (d): Per-Tenant API-Key Auth", §"McpContext" extension, §"Build order — Phase 1" — the primary design; note the validator-placement discrepancy resolved by D-01.
- `.planning/research/STACK.md` §"(b) API-Key Auth for the MCP Endpoint" — SHA-256 hex lookup, `sha2`/`subtle` already present, validator belongs in `ferro-mcp-oauth`, confirm v8.1 `make:api-key` schema.
- `.planning/research/PITFALLS.md` §1 (cross-tenant tool leak), §2 (server-side guard bypass — relevant for the scope re-check pattern), §4 (API-key scope creep — scope on the key from day one).
- `.planning/research/FEATURES.md` — "Per-tenant API-key auth" / "Tenant scoping" rows; "Separate MCP permission model" anti-pattern (reuse policy layer, no second permission system).
- `.planning/REQUIREMENTS.md` — AMCP-01, AMCP-02 (the two requirements this phase closes).

### v12.6 foundation
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — the OAuth browser-login design the API-key path runs alongside.

### Code touch-points (read before editing)
- `ferro-mcp-oauth/src/validate.rs` — `BearerCheck` enum + `validate_bearer` (the pattern `validate_api_key` parallels; same `Authenticated(principal)` shape).
- `ferro-mcp-oauth/src/jwt.rs` — `McpTokenClaims { sub, tenant_id }` (principal shape both paths must produce).
- `ferro-mcp-server/src/auth.rs` — stub `BearerOutcome` to replace with the `resolve_tenant` unifier.
- `ferro-mcp-server/src/renderer.rs` — `struct McpContext;` (the extension point, line ~10).
- `ferro-mcp-server/src/dispatch.rs` — `dispatch()` already takes `Option<i64>` tenant_id, fail-closed; thread context tenant_id here.
- `ferro-mcp-server/src/error.rs` — add `Auth(String)`.
- `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — extend with the API-key cross-tenant fixture.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `BearerCheck` enum (`ferro-mcp-oauth/src/validate.rs`): `Unauthenticated | Invalid | Forbidden | Authenticated(principal)`. The API-key path reuses this exact outcome type — no new auth-result enum.
- `validate_bearer(header, config, expected_tenant) -> BearerCheck`: the template signature/structure for `validate_api_key`.
- `McpTokenClaims { sub, tenant_id }` + `Authenticated` principal `json!({"sub":..., "tenant_id":...})`: the principal both auth paths emit.
- `dispatch()` (`ferro-mcp-server/src/dispatch.rs`): already fail-closed tenant-scoped (`Option<i64>` param, injects `AND tenant_id=?`, denies when a tenant-scoped service has no tenant). The context only needs to *supply* the tenant_id — the scoping enforcement already exists.
- `sha2 = "0.10"` and `subtle` already in `ferro-mcp-oauth/Cargo.toml` — no new dependency.
- Integration test scaffolding: `ferro-mcp-server/tests/{dispatch_integration,jsonrpc_integration,mcp_tenant_isolation}.rs`.

### Established Patterns
- Auth crate owns auth outcome types; the server crate unifies/routes (D-01).
- Fail-closed tenant predicate injection at the SQL layer (do not re-implement scoping in the auth or context layer).
- `BaseContext.evaluated_guards` semantics (v14.0/Phase 215): absent key = allow, explicit `false` = deny — the same semantics `McpContext.evaluated_guards` will use for write-tool filtering in 218.

### Integration Points
- `ferro-mcp-server`'s JSON-RPC request handler: where `resolve_tenant` runs and where `McpContext` is constructed and threaded into `tools/list` and `tools/call`.
- Consumer-app DB (same SeaORM connection): hosts the `api_keys` table the validator queries.

</code_context>

<specifics>
## Specific Ideas

- "One endpoint, two validation branches, same downstream behavior" (STACK.md) — do **not** add a separate MCP endpoint for API-key auth.
- The scope-rejection path must be a hard server-side check at `tools/call`, mirroring the guard re-evaluation pattern — not merely a `tools/list` filter (PITFALLS §2/§4).
- Reuse the existing policy/ability layer; no MCP-specific permission system (FEATURES anti-pattern row).

</specifics>

<deferred>
## Deferred Ideas

- Write-tool rendering from `ActionDef`, guard filtering, `destructiveHint` annotations — Phase 218.
- `dispatch_write()` + server-side guard re-evaluation at execution + idempotency keys + audit log — Phase 219.
- `ferro-ai` confirmation gating for destructive actions — Phase 220.
- Inbound NL intent loop + replay/smoke CI path — Phase 221.
- Fine-grained `abilities[]` per-action scoping on the key (beyond `read`/`read_write`) — future v15.x; 217 ships the coarse scope the success criteria require.
- DB-backed confirmation store, per-call audit trail / key-usage logging — production hardening, deferred per REQUIREMENTS.

</deferred>

---

*Phase: 217-tenant-context-per-tenant-api-key-auth*
*Context gathered: 2026-06-13*
