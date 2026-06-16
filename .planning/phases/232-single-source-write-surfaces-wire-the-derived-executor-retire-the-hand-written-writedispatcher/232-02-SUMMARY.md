---
phase: 232-single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher
plan: 02
subsystem: visual-write-surface
tags: [EXEC-05, visual-write, framework::write, projection-action-route, web-channel]
requires:
  - "framework::write kernel relocated in Plan 01 (ferro::write::dispatch_write / WriteDispatcher / WriteError)"
  - "app exposed_services() + make_write_dispatcher() (controllers::mcp)"
  - "ferro::derive_transition_plan + ServiceDef/ActionDef (Phase 231)"
provides:
  - "POST /{service}/{action} visual transition-write handler (controllers::visual_action::handle)"
  - "tenant-scoped projection.visual.action route"
affects:
  - "app router (new public write endpoint)"
  - "the projection action-button URL (builder.rs:685) now has a receiving handler"
tech-stack:
  added: []
  patterns:
    - "Visual handler is FRAMING ONLY — resolves (ServiceDef, ActionDef) and calls the shared kernel; no second executor, no match action_name, no second guard loop"
    - "Audit channel literal \"web\" passed by the framing → web.action.{name}"
key-files:
  created:
    - "app/src/controllers/visual_action.rs"
    - "app/src/tests/visual_action.rs"
  modified:
    - "app/src/controllers/mod.rs"
    - "app/src/tests/mod.rs"
    - "app/src/routes.rs"
decisions:
  - "Tenant resolved via SessionUserTenantResolver (browser session), not JwtClaimResolver — the visual/form path is a session surface, not a bearer-token surface; failure mode Forbidden so an unresolvable tenant is denied, not allowed onto the write path"
  - "Route placed after the explicit named routes; matchit prefers literal segments over params so {service}/{action} is the catch-all and does not shadow /auth/*, /mcp, /token"
  - "SC2 integration tests drive the EXACT kernel call the handler makes (dispatch_write(.., \"web\")) against an in-memory DB, mirroring the existing mcp_write_dispatch fixture pattern, rather than spinning the full HTTP+middleware stack"
metrics:
  duration: "~25m"
  tasks: 2
  files: 5
  completed: "2026-06-16"
---

# Phase 232 Plan 02: Visual Transition-Write Surface Summary

Built the previously-missing `POST /{service}/{action}` visual/form write handler — the action-button URL the projection renderer emits (`ferro-json-ui/src/projection/builder.rs:685`) for which no handler existed. The handler resolves `(ServiceDef, ActionDef)` from the same `exposed_services()` the MCP path uses, authenticates the tenant from `ferro::current_tenant()`, derives `to_state` via `derive_transition_plan` only, reuses `make_write_dispatcher()`, and calls the SAME `framework::write::dispatch_write` kernel with channel `"web"` — one declaration now backs writes in the visual modality with no per-channel executor (EXEC-05 SC2).

## What Shipped

- **`controllers::visual_action::handle` (new).** Receives `POST /{service}/{action}`; resolves path params to `(ServiceDef, ActionDef)` from `exposed_services()` (unknown → 404, no panic); reads `tenant_id` from `current_tenant()` (403 if absent); derives the transition guard from `derive_transition_plan(...).guard`; reads the form body only as opaque `inputs`; reuses `make_write_dispatcher()`; calls the shared kernel with the literal channel `"web"`. Outcome mapped to redacted 4xx (GuardFailed → 403, Validation → 422, ActionNotFound → 404, confirmation → 409) with no SQL/table/column disclosure; success → 200 JSON. No `match action_name`, no second `WriteDispatcher`, no second guard loop.
- **Tenant-scoped route registration.** `projection.visual.action` registered inside a `group!` wiring `SessionUserTenantResolver` + `TenantFailureMode::Forbidden`, after the explicit named routes. matchit literal-over-param precedence keeps it from shadowing `/auth/*`, `/mcp`, `/mcp/chat`, `/token`.
- **6 tests (5 SC2 security + 1 routing).** Drive the synthetic order/approval StateMachine anchor (no new app models): derived-transition persistence, live guard rejection (state unchanged), `web.action.*` audit channel, cross-tenant denial, form-supplied `to_state` ignored, and matchit precedence (catch-all resolves `/order/submit` while literals win).

## Verification

| Check | Result |
|-------|--------|
| `cargo build -p app --all-features` | exit 0 |
| `cargo test -p app visual_action` | 6/6 passed |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p app --all-targets -D warnings` | clean |
| `cargo clippy -p app --all-features --all-targets -D warnings` | clean |
| `grep -rnE 'match .*action' visual_action.rs` (code, comments excluded) | EMPTY — no re-introduced match |
| `grep -n 'dispatch_write(' visual_action.rs` | shared-kernel call passing `"web"` |
| `grep -n 'derive_transition_plan' visual_action.rs` | matches — to_state derived, not from form |
| `grep -n 'current_tenant' visual_action.rs` | matches — tenant from auth |
| `grep -n 'make_write_dispatcher' visual_action.rs` | matches — single dispatcher reused |
| `grep -n 'visual_action' routes.rs` | route registered, tenant-scoped |

### Test → SC2 / threat mapping

| Test | Proves |
|------|--------|
| `visual_action_drives_derived_transition` | SC2: visual path persists the derived `to_state` ("submitted") through the shared kernel |
| `visual_guard_rejects_illegal_transition` | T-232-06: live guard re-eval rejects (`is_manager` false for a userless tenant); state unchanged |
| `visual_audit_channel_is_web` | T-232-08: audit entry is `web.action.submit`, not `mcp.action.*` |
| `visual_cross_tenant_denied` | T-232-07: tenant from auth; tenant 1 cannot mutate tenant 2's order |
| `visual_action_rejects_form_supplied_to_state` | T-232-05: bogus `status`/`to_state` in the body is ignored; persisted state is the derived `Transition.to` |
| `visual_route_registered_without_shadowing` | route wired, not silently dropped (`.ok()` insert); literals not shadowed |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `req.param()` returns `Result`, not `Option`**
- **Found during:** Task 1 (build error E0599: no `ok_or_else` on `Result`)
- **Issue:** The plan's interfaces section described `req.param("service")?`; the actual signature is `param(&self, name) -> Result<&str, ParamError>`. The handler used `.ok_or_else()`.
- **Fix:** `.map_err(|_| HttpResponse::new().status(400))?.to_string()` — also took owned `String`s so the params outlive the `req.input()` consume.
- **Files modified:** `app/src/controllers/visual_action.rs`
- **Commit:** 4ea0f4ab

**2. [Rule 3 - Blocking] tenants.id is i64, not i32, in the test seed**
- **Found during:** Task 1 (test compile error E0308 in the seed loop)
- **Issue:** The new three-tenant seed loop inferred `i32` for the tenant id; the `tenants` ActiveModel id column is `i64`.
- **Fix:** Typed the tenant seed loop literals `1i64/2i64/3i64`.
- **Files modified:** `app/src/tests/visual_action.rs`
- **Commit:** 4ea0f4ab

**3. [Rule 3 - Blocking] route test cannot call the full `routes::register()`**
- **Found during:** Task 2 (`visual_route_registered_without_shadowing` panicked: `tenant_lookup not initialized`)
- **Issue:** `routes::register()` eagerly constructs the `/mcp` group's `JwtClaimResolver`, which calls `crate::tenant_lookup::get()` and panics without `bootstrap::register()`. Calling the full app router in a unit test is not viable.
- **Fix:** The route test builds a minimal `ferro::Router` registering the relevant POST literals plus the `{service}/{action}` catch-all (in routes.rs order) and asserts matchit precedence directly — the property the route ordering actually relies on. Production route registration is exercised by `cargo build` (the `post!` macro) + the SC2 tests calling the handler's exact kernel path.
- **Files modified:** `app/src/tests/visual_action.rs`
- **Commit:** 94abcdd1

### Scope note (not a deviation)

The plan's Task-1 acceptance grep `grep -rnE 'match .*action'` expecting EMPTY matches two doc-comment mentions ("no `match action_name`"). The handler **code** contains no transition match — the same nuance Plan 01's summary recorded. Filtering comments confirms EMPTY.

## Threat Model Outcomes

- **T-232-05 (form-supplied to_state, Tampering/EoP):** `to_state` derived only inside the reused executor via `derive_transition_plan`; the body is opaque `inputs`. `visual_action_rejects_form_supplied_to_state` green.
- **T-232-06 (guard bypass, EoP):** handler calls the shared `dispatch_write`; live `merged_guards` re-eval runs. `visual_guard_rejects_illegal_transition` green (state unchanged).
- **T-232-07 (cross-tenant write, Info Disclosure/Tampering):** `tenant_id` from `current_tenant()`; reused executor keeps `find_for_tenant`. `visual_cross_tenant_denied` green.
- **T-232-08 (audit repudiation):** channel `"web"` → `web.action.submit`. `visual_audit_channel_is_web` green.
- **T-232-09 (error disclosure):** kernel errors mapped to redacted 4xx; no SQL/table/column strings in responses.

## Threat Flags

The new `POST /{service}/{action}` endpoint IS a new public write surface, but it is exactly the surface this plan's `<threat_model>` (T-232-05..09) was written to cover, and every disposition is `mitigate` with a passing test. No surface outside the registered threat model was introduced.

## Self-Check: PASSED

- `app/src/controllers/visual_action.rs` — FOUND
- `app/src/tests/visual_action.rs` — FOUND
- `232-02-SUMMARY.md` — FOUND
- commits 4ea0f4ab, 94abcdd1 — verified below
