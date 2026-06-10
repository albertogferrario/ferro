# Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

Make an MCP `tools/call` execute **inside the token's tenant context** and **gated by the
application's existing policy layer**, then prove it end to end with a real MCP client. The
walking skeleton from Phases 197–199 (projection → tool schema → HTTP transport → OAuth token
bound to `(user, tenant)`) becomes a *secured* read: an agent's reach equals the authenticated
user's reach — no parallel permission system, no per-tool ownership filter.

**In scope:**

1. **Tenant scoping of dispatch (AMCP-10, SC-1):** a token scoped to tenant A returns only
   tenant A's rows; tenant B only B's. The tenant context is the **same** one the web surface
   uses (`current_tenant()` task-local, set by `TenantMiddleware` + a `TenantResolver`).
2. **Policy gating of dispatch (AMCP-11, SC-2):** each tool call runs through the **same
   `Gate`/`Policy` layer** as the web surface; a denied call returns an MCP **tool error** with
   a clear message and **no data disclosure**.
3. **Structural identity (SC-3):** the tenant context established for `/mcp` is structurally the
   same as the web-surface multi-tenant middleware — one tenant system, not two.
4. **Minimal multi-tenant fixture in the sample `app`** (see D-07) — currently the app has **no
   tenant infrastructure and no `orders` table at all**, so SC-1 isolation is not provable
   without this. This is a phase prerequisite, not optional polish.
5. **Dogfood GO/NO-GO (SC-4):** a real MCP client completes a browser login against a live
   consumer application, calls `tools/list` then `tools/call` for one exposed projection, and
   receives that tenant's rows. A run that fails end to end is **NO-GO** and the design is
   revised before the phase is marked complete.

**Out of scope (later milestones):**

- **Write intents** (Collect/Process projections rendered as create/submit tools with a
  confirmation step) — read-only stays the v12.6 boundary.
- **Multi-projection auto-exposure** — still one opt-in projection.
- **Per-tenant tool *catalog* variation** (different tenants seeing different tool sets) — all
  authenticated tenants see the same projection; only the *data* is scoped.
- **MCP App UI / resources / prompts** — tools only.
- **Refresh tokens, RS256/JWKS, multi-process cache** — deferred in Phase 199 and not revisited.

**Carrying forward (the seams this phase fills):**

- `app/src/controllers/mcp.rs` — `BearerCheck::Authenticated(_principal)` already carries the
  `// Phase 200 inserts principal into request extensions for JwtClaimResolver` note, and
  `let expected_tenant = ferro::current_tenant().map(|t| t.id);` is already threaded into
  `validate_bearer`. The seam is explicit and waiting.
- `ferro-mcp-server/src/dispatch.rs` — the parameterized read path with the comment
  *"No tenant or ownership filter is applied here — Phase 200 owns that seam."* The tenant
  predicate is injected here (D-02).
- Phase 199 token: minted with a tenant claim from `current_tenant()` at authorize time
  (Phase 199 D-06). The claim **name** must match `JwtClaimResolver`'s default (`tenant_id`) —
  D-03. **Caveat:** at Phase 199, `/authorize` had no `TenantMiddleware`, so `current_tenant()`
  is `None` and the token's tenant is neutralized. D-07 wires tenancy so the claim is real.

</domain>

<decisions>
## Implementation Decisions

### Tenant context establishment for `/mcp` (D-01) — AMCP-10, SC-3
- **D-01:** Establish the tenant context for `/mcp` through the **same `TenantMiddleware` +
  `JwtClaimResolver` path the web surface uses**, not a hand-rolled scope in the handler.
  Sequence: a bearer-validation step parses the JWT and inserts its claims into request
  extensions via `req.insert::<serde_json::Value>(claims)`; `TenantMiddleware::new()
  .resolver(JwtClaimResolver::new("tenant_id", lookup))` reads those claims, looks up the
  `TenantContext`, and runs the rest of the request inside the `current_tenant()` task-local
  scope. Dispatch then reads `current_tenant()` exactly as a web handler would. This satisfies
  SC-3 literally: the context source is identical, so "no second permission system" is true by
  construction, not by parallel code.
  - **[auto] recommended default** — chosen over (b) manually resolving the tenant inside the
    `/mcp` handler and wrapping `dispatch` in an ad-hoc task-local scope. (b) would re-implement
    what `TenantMiddleware` already does and create a second tenant path — the exact
    duplicate-control-surface failure to avoid ([[feedback_no_duplicate_control_surface]]).
  - **RESEARCH FLAG (load-bearing — ordering):** Phase 199 placed bearer validation **inline in
    the handler**, after middleware runs. `JwtClaimResolver` reads claims from request
    extensions, which must be populated *before* `TenantMiddleware` runs. Resolve the ordering:
    either (i) relocate bearer validation into a **middleware that runs before
    `TenantMiddleware`** on the `/mcp` route (cleanest — both auth and tenancy become the
    standard middleware stack), or (ii) keep inline validation and have the handler drive
    resolution + scope itself. Prefer (i); confirm the framework lets a route mount
    `[BearerAuthMiddleware, TenantMiddleware]` in that order and that `req.insert` survives into
    the resolver. Record the chosen ordering and why.
  - **RESEARCH FLAG:** confirm what `validate_bearer` returns in `BearerCheck::Authenticated` —
    the principal must be (or be convertible to) a `serde_json::Value` carrying `tenant_id` and
    `sub` so both `JwtClaimResolver` (tenant) and the policy load (user, D-04) can read it.

### Tenant predicate injection in the dispatch read path (D-02) — AMCP-10, SC-1
- **D-02:** `dispatch` gains a **tenant predicate appended as a bound parameter**, using the
  same `Statement::from_sql_and_values` binding path as filters — never string-interpolated and
  **never sourced from the call payload** (so an agent cannot widen or override its own tenant).
  The tenant **value** comes from `current_tenant().id` (the one canonical context from D-01).
  The tenant **column** is declared on the projection (D-05) so the SQL path knows which FK to
  filter; default convention `tenant_id`. Concretely: when the projection is tenant-scoped and a
  tenant context is present, dispatch adds `AND "{tenant_col}" = ?` to both the COUNT and SELECT
  WHERE clauses with the tenant id bound.
  - **[auto] recommended default** — chosen over reusing SeaORM `TenantScope`. `TenantScope`
    applies to a `QueryBuilder<E>` over a concrete entity; dispatch is raw parameterized SQL
    over a `ServiceDef` with no entity type, so `TenantScope` cannot apply mechanically. The
    *structural identity* SC-3 requires is in the **context source** (same `current_tenant()`),
    not the filter mechanism — documented explicitly so reviewers don't read the manual SQL
    predicate as a "second system".
  - **RESEARCH FLAG:** decide where the tenant FK column name lives — a new
    `ServiceDef.tenant_column: Option<String>` (explicit, travels with the projection) vs a
    fixed `tenant_id` convention. Prefer the explicit field so a projection without a tenant FK
    can declare `None` and a non-`tenant_id` schema is expressible. Tie this to D-05/D-06.

### Tenant claim name alignment (D-03) — AMCP-10, SC-3
- **D-03:** The JWT tenant claim is named **`tenant_id`** with an integer value, matching
  `JwtClaimResolver::new("tenant_id", …)`'s default read (`claims["tenant_id"].as_i64()`). One
  claim name, read by one resolver, in one place.
  - **[auto] recommended default** — the alternative (a differently-named claim + a custom
    resolver) re-introduces a second tenant vocabulary for no benefit.
  - **RESEARCH FLAG (load-bearing):** verify what Phase 199 actually minted (`build_claims`
    second arg / `mint_token`). If the claim is named `tenant` or carries a slug rather than an
    integer id, reconcile here — either rename the minted claim to `tenant_id`(int) or
    instantiate `JwtClaimResolver::new("tenant", …)` and adapt the value type. The token-mint
    side and the resolver side **must** agree; this is the single thread tying authorize-time
    tenancy to call-time scoping.

### Policy gating mechanism (D-04) — AMCP-11, SC-2
- **D-04:** Gate a tool call with a **named `Gate` ability declared on the projection**
  (`ServiceDef.mcp_ability: Option<String>`, e.g. `"view-orders"`). Before dispatch: load the
  concrete `User` by the token's `sub`, then call `Gate::authorize(ability, &user)` (or
  `Gate::allows`). On deny → return a **policy-deny tool error** (D-09), no rows, no dispatch.
  A named string ability fits the generic dispatch: `Gate` abilities take `&dyn Authenticatable`
  + `Option<&dyn Any>`, so no concrete *model* type is required — only the user. The same
  ability the web surface defines in bootstrap is reused; the agent's reach is bounded by the
  user's existing permissions.
  - **[auto] recommended default** — chosen over (b) invoking a typed `Policy<M>::view_any`,
    which would require dispatch to know the concrete model type `M` (it does not — dispatch is
    SQL-string over `ServiceDef`), and over (c) a bespoke MCP-only permission check (a second
    permission system, explicitly forbidden).
  - **RESEARCH FLAG:** loading the concrete `User` from `sub` is app-type-specific (framework
    has no concrete `User`). Decide the boundary: the **app `/mcp` glue** performs the user load
    + `Gate::authorize` (handler knows `crate::models::User`), while `ServiceDef` carries the
    ability name so the binding is declarative and reusable. Consider whether `ferro-mcp-server`
    should expose a `policy_hook: Fn(&Principal) -> Result<()>` closure so the gate call is
    framework-driven; pick the lower-coupling option and record why (likely app-glue + declared
    ability, since the user type is irreducibly app-specific).
  - **RESEARCH FLAG:** if `mcp_ability` is `None` on a projection, decide the default — **deny**
    (fail-closed, must opt into an ability to be callable) vs allow-any-authenticated. Prefer
    requiring an ability for an `mcp_exposed` projection so exposure ≠ free access.

### Division of responsibility: framework vs app (D-05) — SC-3
- **D-05:** Reusable, generic concerns live in the framework crates; only the irreducibly
  app-typed step lives in the app:
  - **Framework (`ferro-projections` `ServiceDef`):** declares `tenant_column: Option<String>`
    and `mcp_ability: Option<String>` — plain metadata, no renderer/auth dependency added
    (preserves the `ferro-projections` renderer-free rule).
  - **Framework (`ferro-mcp-server` `dispatch`):** reads `current_tenant()` and injects the
    tenant predicate (D-02). Generic, reusable, no app types.
  - **App (`/mcp` handler glue):** mounts `[BearerAuthMiddleware, TenantMiddleware]`, loads the
    concrete `User` for the policy check (D-04), maps a deny into the MCP tool error (D-09).
  - **[auto] recommended default** — keeps the killer feature (per-tenant, projection-derived
    MCP toolset) inheritable by any ferro app via declarations + mounted middleware, while
    accepting that the concrete-`User` load is the one piece only the app can supply.

### Fail-closed on missing/ambiguous tenant (D-06) — SC-1
- **D-06:** If a projection is **tenant-scoped** (`tenant_column = Some(...)`) and
  `current_tenant()` is `None`, dispatch **denies / returns zero rows** — it never falls back to
  an unscoped `SELECT *` that would leak all tenants' data. A projection with
  `tenant_column = None` is treated as genuinely non-tenant data and runs unscoped (explicit
  opt-out, not an accident).
  - **[auto] recommended default** — fail-closed is the only safe default for a data-exposure
    surface; an unscoped fallback is a cross-tenant leak.
  - **RESEARCH FLAG:** confirm whether the live dogfood token will actually carry a tenant after
    D-07 wires `/authorize` behind `TenantMiddleware`; if a single-tenant deployment legitimately
    has no tenant, that path must use `tenant_column = None`, not a `None`-context scoped read.

### Dogfood data substrate — minimal multi-tenant fixture in the sample `app` (D-07) — SC-1, SC-4
- **D-07:** The sample `app` currently has **no `tenants` table, no `tenant_id` columns, no
  `TenantMiddleware`, and no `orders` table** — only `users`, `todos`, `api_keys`,
  `oauth_clients`. The `order` projection is `mcp_exposed` and dispatch targets a table named
  `orders` that does not exist. **SC-1 (tenant A vs tenant B isolation) is therefore not
  provable today.** This phase stands up the minimal fixture that makes it provable:
  - a `tenants` table + seed of **two tenants**;
  - an `orders` table **with a `tenant_id` column** + seeded rows for **each** tenant;
  - the authenticated `User` associated with a tenant so `current_tenant()` resolves at
    `/authorize` time (token gets a real `tenant_id`) and at `/mcp` time (scoping bites);
  - `TenantMiddleware(JwtClaimResolver("tenant_id", lookup))` wired onto **`/authorize`** (to
    bind the tenant into the token) and **`/mcp`** (to scope dispatch);
  - the `order` projection's `ServiceDef` gains the `tenant_column` / `mcp_ability` declarations
    from D-05, and an `orders` migration consistent with its declared fields.
  - **[auto] recommended default** — chosen over (b) inventing a fresh tenant-scoped projection
    for the dogfood, and over (c) declaring SC-1 "verified" by the in-crate SQLite unit tests
    from Phases 197–199 (those never ran against the app's real schema or two tenants — they
    cannot satisfy a GO/NO-GO that the spec defines as a *real client against a live app*).
    Reusing the existing `order` projection keeps the walking skeleton intact and turns the
    already-exposed tool into the thing actually demonstrated.
  - **RESEARCH FLAG (load-bearing):** confirm the framework's tenant `TenantLookup` /
    `find_by_id` expectations and the minimal `tenants` schema they need; mirror an existing
    ferro tenancy example (Phase 95 multi-tenant middleware) rather than inventing a schema.
    Confirm how a `User`→tenant association is expressed (FK column on `users`, or a membership
    table) and pick the simplest that satisfies single-tenant-per-user for the dogfood.
  - **RESEARCH FLAG:** dispatch derives the table name as `format!("{}s", service.name)` →
    `"orders"`. Ensure the migration names the table `orders` (or introduce the long-standing
    `ServiceDef.table` override TODO if a mismatch arises). Align field names/types between the
    projection and the migration so `SELECT` columns resolve.

### Dogfood harness & GO/NO-GO recording (D-08) — SC-4
- **D-08:** The acceptance run uses a **scripted MCP client checked into the repo** (a small
  script using the official MCP SDK, or a documented `mcp`-CLI invocation) that drives the full
  sequence — discovery → dynamic registration → `/authorize` (browser login, human-in-the-loop)
  → `/token` → `tools/list` → `tools/call` — against the **locally-run sample `app`**, plus a
  documented **Claude Desktop** config as the human-facing confirmation path. The result is
  recorded as an explicit **GO / NO-GO** in a phase acceptance artifact
  (`200-ACCEPTANCE.md` / the phase VERIFICATION). NO-GO blocks phase completion and triggers a
  design revision (per the phase goal).
  - **[auto] recommended default** — a scripted client is reproducible and citable in the
    acceptance doc; Claude Desktop alone is not reproducible. Both are run; the script is the
    record of truth, Claude Desktop is the "a real client a human uses also works" check.
  - **OPERATOR NOTE:** the server is always run by the user (per project convention) and the
    browser login is human-in-the-loop. The plan must treat "user starts the live app + performs
    the browser login" as a manual step in the acceptance procedure, not something an agent runs
    unattended.
  - **RESEARCH FLAG:** confirm the live-app prerequisites the script needs — `APP_URL` reachable
    over a scheme/host that satisfies the Origin check and the audience (`{APP_URL}/mcp`), the
    OAuth signing secret set, and the two-tenant seed applied. List them in the acceptance
    procedure so a NO-GO is attributable to design, not setup.

### Policy-deny tool-error shape (D-09) — AMCP-11, SC-2
- **D-09:** A policy denial returns a JSON-RPC **success envelope** whose result is an **MCP
  tool error** (`isError: true` with a clear human-readable message), **not** a transport-level
  error and **not** any rows or column data. Distinguish from the transport-level `401`/`403`
  of Phase 199: those reject the *request*; a policy deny is an *authenticated, authorized-to-
  call-the-endpoint* request whose *content* is forbidden — surfaced as a tool error the agent
  can read. No table data, filter values, or row counts leak in the message.
  - **[auto] recommended default** — matches the design spec line "Policy denial: an MCP tool
    error with a clear message, no data disclosure" and the MCP convention that tool-level
    failures are `isError` results, not protocol errors.

### Claude's Discretion
- Exact name of the seeded gate ability and the two seed tenants/orders fixtures.
- Internal module placement of the bearer-auth middleware (D-01 (i)).
- Wording of the policy-deny tool-error message (must disclose nothing about the data).
- Whether `tenant_column`/`mcp_ability` are separate `Option<String>` fields or a small
  `McpAccess` sub-struct on `ServiceDef` (keep it plain metadata either way).
- Language/runtime of the scripted dogfood client (Python MCP SDK vs Node vs `mcp` CLI).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 200" — goal, success criteria SC-1…SC-4 (the GO/NO-GO gate).
- `.planning/REQUIREMENTS.md` — AMCP-10 (tenant scoping), AMCP-11 (policy gating).
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` §"Tenant and
  policy reuse" (≈L122–L126), §"Flow" (≈L134–L150), §"Testing / Dogfood acceptance gate"
  (≈L161–L178) — the structural-reuse mandate, the error mapping, and the acceptance discipline.

### The seams this phase fills (carry-forward)
- `ferro-mcp-server/src/dispatch.rs` — the parameterized read path; *"No tenant or ownership
  filter is applied here — Phase 200 owns that seam."* (D-02). Note `MAX_LIMIT`/`MAX_OFFSET`
  clamps and the `format!("{}s", service.name)` table derivation.
- `app/src/controllers/mcp.rs` — `BearerCheck::Authenticated(_principal)` (the principal →
  extensions seam, D-01), `expected_tenant = current_tenant().map(|t| t.id)` already threaded,
  the Origin check, and the JSON-RPC envelope splicing for the tool-error shape (D-09).
- `.planning/phases/199-oauth-browser-login/199-CONTEXT.md` — D-02 (JWT/HS256 claims), D-06
  (tenant binding at authorize time + the `current_tenant()`-is-`None` caveat), D-07 (bearer
  validation 401/403). The minted claim name/value must reconcile with D-03 here.

### Tenant system (reuse, do not fork) — Phase 95
- `framework/src/tenant/middleware.rs` — `TenantMiddleware` (resolver chain, failure mode).
- `framework/src/tenant/resolver.rs` — `JwtClaimResolver` reads `req.get::<serde_json::Value>()`
  → `claims["tenant_id"].as_i64()` → `find_by_id` (D-01, D-03). Also `TenantLookup` contract.
- `framework/src/tenant/context.rs` — `current_tenant()` task-local + `with_tenant_scope` (the
  context dispatch reads, D-02).
- `framework/src/tenant/scope.rs` — `TenantScope` (why it does **not** mechanically apply to the
  SQL dispatch path; documents the structural-identity argument for D-02).
- `framework/src/tenant/lookup.rs` — `TenantLookup::find_by_id`/`find_by_slug` (fixture, D-07).
- `.planning/phases/95-multi-tenant-middleware/95-CONTEXT.md` — the original multi-tenant
  decisions to mirror for the sample-app fixture schema (D-07).

### Authorization layer (reuse, do not fork) — AMCP-11
- `framework/src/authorization/gate.rs` — `Gate::define`/`allows`/`authorize`, the global
  registry, `before` hooks (D-04). Abilities take `&dyn Authenticatable`.
- `framework/src/authorization/policy.rs` — `Policy<M>` trait (`view_any`, `before`); why the
  typed policy path needs a concrete `M` dispatch lacks (D-04 rationale).
- `framework/src/authorization/response.rs` — `AuthResponse` (`allow`/`deny`/`deny_silent`),
  the deny message dispatch maps into the tool error (D-09).
- `framework/src/auth/authenticatable.rs`, `framework/src/auth/guard.rs` — `Authenticatable`,
  `Auth::id`; the concrete-`User` load from `sub` (D-04 boundary).

### Sample-app fixture targets (D-07)
- `app/src/projections/order.rs` — the `mcp_exposed` projection to extend with
  `tenant_column`/`mcp_ability`.
- `app/src/migrations/mod.rs`, `app/src/migrations/m20251208_160100_create_users_table.rs`,
  `app/src/migrations/m20260611_create_oauth_clients_table.rs` — migration patterns + the
  registration list to add `tenants`/`orders` migrations to.
- `app/src/models/users.rs`, `app/src/models/mod.rs` — model patterns + the `User`→tenant
  association.
- `app/src/routes.rs` — where `TenantMiddleware` mounts onto `/authorize` and `/mcp`.
- `ferro-projections/src/service.rs` — `ServiceDef` (where `tenant_column`/`mcp_ability` are
  added as plain metadata; keep the crate renderer-free).

### External specs (no repo file — read upstream)
- **MCP specification** — tool-call result shape, `isError` tool errors vs JSON-RPC protocol
  errors (D-09); protected-resource / OAuth 2.1 resource-server expectations for the live run.
- **RFC 6750** Bearer Token Usage — `401`/`403` distinction Phase 199 set, relative to a policy
  deny (D-09).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TenantMiddleware` + `JwtClaimResolver` + `current_tenant()` — the entire tenant context path;
  reused verbatim so `/mcp` tenancy is structurally the web surface (D-01, SC-3).
- `Gate::authorize` / `Gate::allows` + the global ability registry — the policy check; the same
  abilities the web surface defines bound the agent's reach (D-04).
- `dispatch`'s existing `Statement::from_sql_and_values` binding path — the tenant predicate
  rides the same bound-parameter mechanism as filters; no new SQL-assembly surface (D-02).
- `validate_bearer` / `BearerCheck::Authenticated(principal)` (Phase 199, `ferro-mcp-oauth`) —
  yields the `(user, tenant)` principal feeding extensions (D-01) and the policy user-load (D-04).
- Existing migration + model patterns (`users`, `oauth_clients`) — the shape for the
  `tenants`/`orders` fixture (D-07).

### Established Patterns
- One tenant system: tenancy is set by middleware into a task-local and read by handlers — never
  re-derived per surface ([[feedback_no_duplicate_control_surface]]).
- `ferro-projections` stays renderer-/auth-free: new `ServiceDef` fields are **plain metadata**
  (`tenant_column`, `mcp_ability`), mirroring how `mcp_exposed` was added in Phase 197 (D-05).
- Fail-closed on a data-exposure surface (D-06): the bearer seam already fails closed on missing
  config (Phase 199), and dispatch already clamps limit/offset — tenant scoping continues that
  posture.

### Integration Points
- `/mcp` route middleware stack: `[BearerAuthMiddleware?, TenantMiddleware(JwtClaimResolver)]`
  before the handler (D-01 ordering research).
- `/authorize` route gains `TenantMiddleware` so the minted token carries a real `tenant_id`
  (D-07) — without it, Phase 199's tenant claim stays neutralized.
- `dispatch(service, filters, limit, offset, db)` reads `current_tenant()` and consults
  `service.tenant_column` to inject the predicate (D-02) — signature may gain the tenant value
  explicitly if reading the task-local from inside the crate is undesirable (research).
- `app/src/controllers/mcp.rs` `tools/call` arm: user-load + `Gate::authorize` before
  `handle_tools_call`, deny → MCP tool error (D-04, D-09).

</code_context>

<specifics>
## Specific Ideas

- The phase delivers the *secured* form of the milestone's killer capability: a standard MCP
  client logs in through the consumer app's own browser login and reads exactly the tenant data
  the authenticated user can already see — enforced by the **same** middleware and policy layer
  as the web UI, with no MCP-specific permission code. "An agent's reach equals the user's
  reach" is the acceptance feeling, proven by the dogfood, not asserted.
- The single most fragile thread is claim-name agreement (D-03): the token minted in Phase 199
  and the `JwtClaimResolver` read in Phase 200 must name the tenant claim identically, or there
  are silently two tenant systems. Treat reconciliation as the first research task.
- The dogfood gate is real, not ceremonial: the sample app **cannot** demonstrate tenant
  isolation today (no tenants, no `orders` table). The honest path is to build the minimal
  two-tenant fixture (D-07) and let a real client prove A≠B — anything less is "tasks cleared,
  killer not delivered."

</specifics>

<deferred>
## Deferred Ideas

- **Write intents over MCP** (create/submit tools with confirmation) — next milestone; this
  phase stays read-only.
- **Per-tenant tool catalog variation** (tenants seeing different tool *sets*) — only data is
  scoped now; differing catalogs is a later concern.
- **Typed `Policy<M>` dispatch** (resolving the concrete model for richer per-row policies) —
  the named-ability `Gate` check (D-04) is sufficient for a list read; revisit when write
  intents or per-row authorization arrive.
- **Generalized tenant-FK derivation** (auto-detecting the tenant column from model metadata)
  — explicit `tenant_column` declaration now; automate only if a third consumer hits the
  friction.
- **`ServiceDef.table` override** for irregular plurals — only if the `orders` fixture surfaces
  the existing `format!("{}s", name)` mismatch TODO.

None of these belong in Phase 200 — analysis stayed within scope.

</deferred>

---

*Phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance*
*Context gathered: 2026-06-10*
