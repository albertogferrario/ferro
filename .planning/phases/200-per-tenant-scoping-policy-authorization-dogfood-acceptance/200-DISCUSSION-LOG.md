# Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Tenant context establishment, Tenant predicate injection, Tenant claim alignment, Policy gating, Framework/app division, Fail-closed behavior, Dogfood data substrate, Dogfood harness, Policy-deny error shape

---

## Tenant context establishment for `/mcp` (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| `TenantMiddleware` + `JwtClaimResolver` (standard stack) | Insert claims into extensions, resolve tenant via the same middleware the web surface uses | ✓ |
| Hand-rolled scope in the `/mcp` handler | Resolve tenant inline and wrap dispatch in an ad-hoc task-local | |

**Choice:** Standard middleware path — satisfies SC-3 literally (identical context source), avoids a second tenant path.
**Notes:** Load-bearing research: Phase 199 validation is inline in the handler; `JwtClaimResolver` needs claims in extensions before `TenantMiddleware` runs. Resolve ordering (prefer relocating bearer validation to a middleware ahead of `TenantMiddleware`).

## Tenant predicate injection in dispatch (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Bound `AND tenant_id = ?` in dispatch SQL | Append a server-controlled tenant predicate via the existing parameter-binding path | ✓ |
| Reuse SeaORM `TenantScope` | Apply the existing query scope | |

**Choice:** Bound predicate in dispatch. `TenantScope` operates on a typed `QueryBuilder<E>`; dispatch is raw parameterized SQL over `ServiceDef` with no entity type. Structural identity (SC-3) is in the context *source*, not the filter mechanism.
**Notes:** Research: `ServiceDef.tenant_column: Option<String>` (explicit) vs fixed `tenant_id` convention — prefer explicit.

## Tenant claim alignment (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Claim named `tenant_id` (int), default resolver | One name read by one resolver | ✓ |
| Differently-named claim + custom resolver | Second tenant vocabulary | |

**Choice:** `tenant_id` integer claim matching `JwtClaimResolver` default.
**Notes:** Load-bearing: verify what Phase 199 actually minted; reconcile name/value type if it minted `tenant`/slug.

## Policy gating mechanism (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Named `Gate` ability declared on `ServiceDef` | Load user from `sub`, `Gate::authorize(ability, &user)` | ✓ |
| Typed `Policy<M>::view_any` | Requires concrete model type `M` | |
| Bespoke MCP-only permission check | Second permission system (forbidden) | |

**Choice:** Named `Gate` ability — fits generic dispatch (needs only the user, not the model type); reuses web-surface abilities.
**Notes:** Research: concrete `User` load is app-typed → app glue performs it; `ServiceDef.mcp_ability` keeps the binding declarative. `mcp_ability = None` → fail-closed (deny).

## Framework vs app division (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Metadata on `ServiceDef` + generic dispatch scoping + app-typed user load only | Reusable concerns in framework, only `User` load in app | ✓ |
| All scoping + policy in the app handler | App-local, not inheritable | |

**Choice:** Split — `tenant_column`/`mcp_ability` metadata + generic dispatch scoping in framework; concrete-`User` load + middleware mount in app.
**Notes:** Preserves `ferro-projections` renderer-/auth-free rule; keeps the capability inheritable by any ferro app.

## Fail-closed on missing tenant (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Tenant-scoped + no context → deny/zero rows | Never unscoped `SELECT *` | ✓ |
| Fall back to unscoped read | Cross-tenant leak | |

**Choice:** Fail-closed. `tenant_column = None` is the explicit opt-out for genuinely non-tenant data.

## Dogfood data substrate (D-07)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal two-tenant fixture around the existing `order` projection | Add `tenants`+`orders(tenant_id)` tables, seed 2 tenants, wire `TenantMiddleware` on `/authorize` + `/mcp` | ✓ |
| Invent a fresh tenant-scoped projection | New projection for the dogfood | |
| Declare SC-1 covered by in-crate SQLite unit tests | No live, multi-tenant proof | |

**Choice:** Build the minimal fixture around `order`. **Discrepancy surfaced:** the sample app has no tenants, no `tenant_id`, no `TenantMiddleware`, and no `orders` table — SC-1 isolation is not provable without this. Reusing `order` keeps the walking skeleton intact.
**Notes:** Research: mirror Phase 95 tenancy schema + `TenantLookup` expectations; ensure table name `orders` matches `format!("{}s", name)`; express `User`→tenant association simply.

## Dogfood harness & GO/NO-GO (D-08)

| Option | Description | Selected |
|--------|-------------|----------|
| Scripted MCP SDK client (record of truth) + Claude Desktop (human check) | Reproducible script + human-facing confirmation, GO/NO-GO recorded in acceptance artifact | ✓ |
| Claude Desktop only | Not reproducible / not citable | |

**Choice:** Both; the script is the citable record, Claude Desktop is the "real human client also works" check.
**Notes:** Server is user-run; browser login is human-in-the-loop — acceptance procedure must treat these as manual steps. NO-GO blocks completion and triggers design revision.

## Policy-deny tool-error shape (D-09)

| Option | Description | Selected |
|--------|-------------|----------|
| JSON-RPC success envelope with `isError: true` tool error, no data | MCP tool-level error, clear message, no disclosure | ✓ |
| Transport-level `403` for policy deny | Conflates request rejection with content forbidden | |

**Choice:** MCP tool error (`isError`), no rows/columns/filter values leaked. Distinct from Phase 199's transport `401`/`403`.

## Claude's Discretion
- Seed ability name, seed tenants/orders fixtures, bearer-middleware module placement,
  deny-message wording, `tenant_column`/`mcp_ability` field grouping, scripted-client runtime.

## Deferred Ideas
- Write intents over MCP; per-tenant tool catalog variation; typed `Policy<M>` dispatch;
  generalized tenant-FK derivation; `ServiceDef.table` override.
