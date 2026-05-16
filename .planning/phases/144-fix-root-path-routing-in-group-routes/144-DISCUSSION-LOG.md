# Phase 144: Fix root path routing in group routes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 144-fix-root-path-routing-in-group-routes
**Mode:** `--auto` (no interactive questions; Claude selected recommended defaults)
**Areas discussed:** Matching semantics, Scope of fix, Route introspection, Edge cases, Tests, Version

---

## Matching Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Register both `/prefix` and `/prefix/` | `get!("/", ...)` in a non-root group inserts the handler under both paths in matchit. Zero breaking change, additive, catches both canonical forms. | ✓ |
| Canonical-redirect (301) | One canonical form, redirect the other. Cleaner URLs but breaking: changes wire behavior for anyone depending on current no-trailing-slash response. | |
| Pre-route normalization middleware | Strip trailing slash from `req.path()` before matching. Works but hides routing quirks in middleware and affects every request. | |

**Claude's choice (auto-recommended):** Register both variants.
**Rationale:** Minimal blast radius, no breaking change for existing users. Canonical redirect is a user-level concern, not a framework default. See D-01.

---

## Scope of Fix

| Option | Description | Selected |
|--------|-------------|----------|
| Fix both `group!` macro and `.group()` builder | Both currently diverge; macros.rs has half-working `/` special case, group.rs has none. Framework invariant is "both APIs produce equivalent registered routes." | ✓ |
| Fix only `group!` macro | The default scaffold and all docs use the macro; the builder API is less common. Smaller diff. | |
| Deprecate `.group()` builder and fix only macro | Reduces surface area long-term. Out of scope for a bug-fix phase. | |

**Claude's choice (auto-recommended):** Fix both.
**Rationale:** Silent divergence between two public APIs is the worst possible state. Unifying them in a bug fix is cheap. See D-05.

---

## Route Introspection

| Option | Description | Selected |
|--------|-------------|----------|
| One `RouteInfo` entry per canonical path (no trailing slash) | Matchit gets both paths; `REGISTERED_ROUTES` gets the canonical one. `list_routes` and ferro-mcp tools stay clean. Named routes resolve to canonical. | ✓ |
| Two `RouteInfo` entries (both `/prefix` and `/prefix/`) | Introspection sees the actual matchit state but doubles the route list and clutters MCP tool discovery. | |
| One entry with `aliases: Vec<String>` field | Most expressive but adds a new field to the public `RouteInfo` struct — API change for a bug fix. | |

**Claude's choice (auto-recommended):** One canonical `RouteInfo` entry.
**Rationale:** Introspection is for humans and agents reading the API surface; they need the canonical form. Doubling the list creates noise in `ferro mcp` tools. See D-07, D-08.

---

## Edge Cases

| Case | Decision |
|------|----------|
| `group!("/", { get!("/", ...) })` | Single `/` route (no `//` regression). |
| `group!("/", { get!("/login", ...) })` | `/login` (unchanged from today). |
| `group!("/api/", { get!("/x", ...) })` | `/api/x` (strip one trailing slash from prefix). |
| `group!("/api/", { get!("/", ...) })` | Both `/api` and `/api/`. |
| `group!("/s/{slug}", { get!("/", ...) })` | Both `/s/{slug}` and `/s/{slug}/` — THIS is the gestiscilo-it reproducer. |
| Nested: `group!("/a", { group!("/b", { get!("/", ...) }) })` | Both `/a/b` and `/a/b/`. |

**Claude's choice (auto-recommended):** All of the above. Table-driven test covers every case. See D-02, D-03, D-09.

---

## Tests

| Option | Description | Selected |
|--------|-------------|----------|
| Table-driven unit tests covering prefix × route-path matrix | One test function, explicit matrix, easy to add cases. Mirrors existing routing test conventions. | ✓ |
| Integration test only (full `routes! { ... }` macro + HTTP roundtrip) | End-to-end confidence but slow and hides which combination broke when one fails. | |
| Both unit matrix AND one integration test asserting no RouteInfo duplication | Belt-and-braces. | Partial (D-09 + D-10) |

**Claude's choice (auto-recommended):** Unit matrix (D-09) plus one integration test for the `list_routes` no-duplication invariant (D-10). Mirror the tests across `macros.rs` and `group.rs` (D-11).

---

## Version & Release

| Option | Description | Selected |
|--------|-------------|----------|
| Patch release 0.2.12 → 0.2.13 | Pure bug fix. No API change. | ✓ |
| Minor release 0.2.x → 0.3.0 | Not justified — this only fixes unreachable-route behavior. | |
| Batch with other fixes | No other urgent routing fixes in flight. Ship alone. | |

**Claude's choice (auto-recommended):** Patch 0.2.13. See D-12.

---

## Claude's Discretion

- Exact helper function name / extraction strategy for the path-combination logic (shared helper vs. mirrored implementations)
- Whether to short-circuit double-registration when prefix is `/` (no duplicates needed for root)
- Ordering of the two `insert_*` calls per handler

## Deferred Ideas

- Canonical URL enforcement (301-redirect) — opt-in middleware, future phase
- Method routing mismatch (405) for `/prefix` vs `/prefix/` — not a known issue
- Reconciling the two group implementations into one — future refactor phase
