# Phase 180: Declarative action handler primitive — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `180-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-05-30
**Phase:** 180-declarative-action-handler-primitive
**Mode:** `--auto` (recommended defaults applied to the seven design-surface gray areas the original hand-crafted CONTEXT explicitly deferred to the planner)
**Areas discussed:** Flash transport, Error conversion ergonomics, ActionError shape, ActionOk shape, Macro defaults, Logging, Auth-failure routing, Migration acceptance gate

---

## Flash transport (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Query string (`?error=…&msg=…`) | Consumer's current scheme; no new infrastructure but URL pollution, length limits, leaks into logs/referer | |
| Signed cookie (`__flash`) | Stateless transport, survives without server-side store; requires HMAC infrastructure that ferro would have to invent for this purpose | |
| Session flash via `session.flash()` | Framework-native — `framework/src/session/store.rs:86` already implements this, aged on next request | ✓ |

**Choice:** Session flash + query-string back-compat fallback during the migration sweep.
**Rationale:** Ferro already owns the mechanism; consumer's query-string scheme phases out as templates become flash-aware; signed-cookie crypto would be net-new and is unnecessary when the session store already solves it.

---

## Error conversion ergonomics (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Blanket `From<E: Display> for ActionError` | Most ergonomic for `?` usage, but conflicts with `From<String>` / `From<&str>` (orphan rule); will not compile cleanly | |
| `IntoActionError` wrapper trait | Sealed-via-blanket trait with `impl<E: Display> IntoActionError for E`; explicit shim into the `?` chain; compiles on stable Rust | ✓ |
| Explicit constructors only (`ActionError::msg(err.to_string())?`) | Most ceremony, defeats the purpose of the primitive | |

**Choice:** `IntoActionError` wrapper trait.
**Rationale:** Standard Rust workaround for the `From<String>` collision; preserves `?` ergonomics without specialization; planner picks the exact compile-clean shim mechanism.

---

## ActionError shape (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: `message: String` only | Smallest surface; pushes all categorization to ad-hoc strings | |
| Full: message + kind + flash_variant + redirect_override | All four optional fields with defaults; covers NotFound / Forbidden / Unauthorized routing and Error / Warning / Info flash styling | ✓ |
| Enum variants per kind | `enum ActionError { Generic(String), NotFound(String), … }` — fewer fields but more variant churn | |

**Choice:** Full struct with `kind: ActionKind` enum field + builder methods.
**Rationale:** Matches `#[domain_error]` ergonomics; enables auth-failure routing without special-casing; small surface area that maps directly to consumer needs from the audit.

---

## ActionOk shape (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Unit type — `Ok(())` only | Macro always uses configured `redirect_to`, no success flash control | |
| Struct with `flash` + `redirect_override` | Mirrors `ActionError` surface; supports `created → /dashboard/x/{new_id}` and per-success flash messages | ✓ |
| Two type aliases (`ActionResult` + `ActionResultWithRedirect`) | Forks the API; planner would have to choose at every callsite | |

**Choice:** Struct with `flash: Option<&'static str>` and `redirect_override: Option<String>`. `From<()> for ActionOk` lets `Ok(())` stay terse for the common case.

---

## Macro defaults (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| `#[action]` no defaults — every attr required | Most explicit but verbose at every callsite | |
| `#[action(redirect_to = "...", method = "POST")]` with method default `POST` | Matches `#[handler]` discoverability convention; required `redirect_to` makes the success target obvious | ✓ |
| Method-named macros (`#[post_action]`, `#[delete_action]`) | More macros to maintain; less consistent with `#[handler]` | |

**Choice:** Single `#[action]` macro with `method = "POST"` default.

---

## Logging (D-07)

| Option | Description | Selected |
|--------|-------------|----------|
| `eprintln!` (current consumer pattern) | No structured fields, no log levels, hard to query | |
| `tracing::error!(handler, msg, source)` | Matches existing ferro convention; structured fields enable log queries | ✓ |
| `log::error!` macro | Less rich than tracing; ferro already standardized on tracing | |

**Choice:** `tracing::error!` with structured fields.

---

## Auth-failure routing (D-08)

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcode `/accedi` redirect for `Unauthorized` errors | Consumer-specific copy; violates project-agnostic crates rule from CLAUDE.md | |
| `ActionError::unauthorized()` carries `redirect_override`; ferro default `None`; consumer configures | Generalized mechanism; project-agnostic; covers auth + arbitrary error-specific routing (e.g. forbidden → `/dashboard`) | ✓ |
| Ferro inspects the source error type to decide | Magical; brittle; couples ferro to specific error types it shouldn't know about | |

**Choice:** `redirect_override` field on every `ActionError`, with `unauthorized()` constructor setting it via builder. Consumer-side default routing lives in consumer config, not in ferro.

---

## Migration acceptance gate (D-09, D-10)

| Option | Description | Selected |
|--------|-------------|----------|
| Phase ships only the ferro primitive; consumer migrates lazily | Half-migrated state, new contributors confused about which pattern to use, boilerplate regrows | |
| Phase ships ferro primitive + full consumer sweep; CI grep enforces zero `error_response!(` in POST handlers | Acceptance criterion already in the original CONTEXT; matches Alberto's "no half-migrated" preference | ✓ |
| Phase ships primitive + opt-in consumer migration with deprecation warnings | Adds deprecation infrastructure that has no other use case in ferro | |

**Choice:** Hard cut migration as part of the deliverable. CI grep enforcement.

---

## Claude's Discretion

None — every gray area resolved with an explicit auto-selected default and rationale. Planner may override any decision but must record the reason in PLAN.md.

---

## Deferred Ideas

- **CSRF integration** — out of scope (existing ferro mechanism applies before the macro runs)
- **Per-handler authorization policies** — out of scope; separate concern
- **HTMX / fetch-based `#[json_action]` sibling** — possible follow-up phase if demand arises
- **Query-string fallback removal** — back-compat shim from D-06 to be deleted in a later cleanup phase once every consumer template reads session flash
- **`From<E: Display>` blanket via stable specialization** — drop-in replacement for `IntoActionError` if/when specialization stabilizes; not blocking
