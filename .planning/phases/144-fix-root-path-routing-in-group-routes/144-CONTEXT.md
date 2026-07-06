# Phase 144: Fix root path routing in group routes - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected by Claude, logged below)

<domain>
## Phase Boundary

Fix the bug where `get!("/", handler)` registered inside `group!("/prefix", { ... })` is unreachable via the canonical trailing-slash URL. Today `/s/{slug}/` returns 404; only `/s/{slug}/index.html` works. The group-prefix + route-path combination in both `framework/src/routing/macros.rs` (`GroupDef::register_with_inherited`) and `framework/src/routing/group.rs` (`GroupBuilder::finalize`) collapses `/` in a group to the bare prefix, leaving the trailing-slash URL unmatched.

This phase fixes path combination in both group implementations so that `get!("/", ...)` inside a non-root group is reachable at both `/prefix` and `/prefix/`, and updates documentation and tests. No new routing features. No middleware semantics change.

</domain>

<decisions>
## Implementation Decisions

### Path Combination Semantics
- **D-01:** `get!("/", handler)` inside `group!("/prefix", { ... })` MUST be reachable at both `/prefix` and `/prefix/`. Implementation: register both variants into `matchit` (insert the handler twice under the two paths). Additive, zero breaking change.
- **D-02:** `get!("/", handler)` inside `group!("/", { ... })` MUST resolve to a single `/` (not `//`). This is the existing `group!("/", { get!("/login", ...) }) → /login` case extended — preserve current correct behavior.
- **D-03:** Trailing slash in the group prefix (e.g. `group!("/prefix/", { get!("/x", ...) })`) MUST produce `/prefix/x`, not `/prefix//x`. Strip one trailing `/` from the prefix before concatenation.
- **D-04:** Non-root route paths inside a non-root group retain current behavior: `group!("/api", { get!("/users", ...) }) → /api/users`. Only the `/` special case and the prefix-trailing-slash case change.

### Scope of Fix
- **D-05:** Fix MUST apply to BOTH group implementations:
  - `framework/src/routing/macros.rs` — `GroupDef::register_with_inherited` (macro-based `group!`, used by the default scaffold and documented in routing.md)
  - `framework/src/routing/group.rs` — `GroupBuilder::finalize` (builder-based `Router::group(prefix, |r| ...)`, used by `RouteBuilder::group` and chainable API)
- **D-06:** Nested groups (`group!` inside `group!`) MUST follow the same rules recursively. The full accumulated prefix is what gets combined with the leaf route path.

### Route Introspection
- **D-07:** When a handler is registered under both `/prefix` and `/prefix/`, `RouteInfo` (the introspection registry in `router.rs`) MUST contain ONE entry at the canonical path without trailing slash (`/prefix`). Both matchit entries point to the same handler; `list_routes`, `ferro-mcp` tools, and route-name resolution stay clean.
- **D-08:** Route naming via `.name("foo")` and `route!("foo", ...)` MUST register only the canonical (no trailing slash) path in `ROUTE_REGISTRY`. `route_url("foo", &[])` returns the canonical form — users can add trailing slash at render time if their convention requires it.

### Tests
- **D-09:** Table-driven tests MUST cover the full matrix of prefix × route-path combinations:
  - `group!("/", { get!("/", ...) })` → matches `GET /`
  - `group!("/", { get!("/x", ...) })` → matches `GET /x` (regression check)
  - `group!("/api", { get!("/", ...) })` → matches `GET /api` AND `GET /api/`
  - `group!("/api", { get!("/x", ...) })` → matches `GET /api/x` (regression check)
  - `group!("/api/", { get!("/x", ...) })` → matches `GET /api/x` (trailing-slash prefix)
  - `group!("/api/", { get!("/", ...) })` → matches `GET /api` AND `GET /api/`
  - `group!("/s/{slug}", { get!("/", ...) })` → matches `GET /s/foo` AND `GET /s/foo/` with `slug=foo` extracted (the exact gestiscilo-it field-test case)
  - Nested: `group!("/a", { group!("/b", { get!("/", ...) }) })` → matches `GET /a/b` AND `GET /a/b/`
- **D-10:** An integration test MUST assert `list_routes()` returns exactly ONE entry per (method, canonical path) pair — no duplicates from the double-insert.
- **D-11:** Both `group.rs` and `macros.rs` paths MUST have equivalent tests. The two implementations diverge today (macros.rs has the half-complete `/` special case, group.rs has none); tests lock them in sync.

### Version & Release
- **D-12:** Patch release 0.2.12 → 0.2.13 across workspace. Pure bug fix, no API changes.
- **D-13:** Changelog entry names the gestiscilo-it field test as the source so downstream users understand the upgrade motivation.

### Claude's Discretion
- Exact helper function name for path combination (e.g. `combine_prefix_and_route`) — planner/implementer picks.
- Whether to extract the combination logic into a shared helper used by both group.rs and macros.rs (recommended — the two implementations should not drift) or keep two mirrored implementations.
- Ordering of `insert_get`/`insert_post`/etc calls when registering both variants.
- Whether to short-circuit `/prefix` == `/prefix/` registration when the prefix is already `/` (no duplication needed for root prefix).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Buggy path-combination call sites (primary fix target)
- `framework/src/routing/macros.rs` §lines 644–730 — `GroupDef::register_with_inherited`: the `if converted_route_path == "/"` branch collapses the route to bare prefix, leaving `/prefix/` unmatched. Also the top of the function at lines 622–626 computes `full_prefix` by naive concatenation and does not strip a trailing slash from parent or child prefixes.
- `framework/src/routing/group.rs` §lines 62–91 — `GroupBuilder::finalize`: naive `format!("{}{}", self.prefix, route.path)` with no special case for root and no trailing-slash handling at all (worse than macros.rs — `group!("/", ...)` equivalent here would produce `//x`).

### Router internals
- `framework/src/routing/router.rs` §lines 14–102 — `REGISTERED_ROUTES` introspection registry; `insert_get`/`insert_post`/etc are called per path. Phase 144 must avoid duplicating `RouteInfo` entries when inserting a handler under two paths.
- `framework/src/routing/router.rs` §lines 104–112 — `register_route_name` / `ROUTE_REGISTRY` for named-route URL generation; must point at the canonical (no trailing slash) path.

### Default scaffold (ships the broken pattern to every new ferro project)
- `ferro-cli/src/templates/files/backend/routes.rs.tpl` — uses `group!("/", { ... })` for guest and authenticated route groups. The root-prefix case happens to already work for non-root child routes (the `full_prefix == "/"` branch in macros.rs handles it); the fix must not regress this.

### Documentation (must be updated to reflect new semantics)
- `docs/src/the-basics/routing.md` §lines 54–60, 117, 150 — `group!` usage examples. Add a section clarifying "/" inside a group matches both with and without trailing slash.
- `docs/src/the-basics/middleware.md` §lines 179, 187 — group + middleware examples.
- `docs/src/features/stripe.md` §line 149 — `group!` example inside stripe feature page; scan for regressions.
- `docs/src/features/authentication.md` §lines 382, 489, 497 — uses `group!("/")` pattern that must keep working.

### Bug source
- `.planning/STATE.md` §line 146 — roadmap entry pinning the bug to gestiscilo-it field test: `/s/{slug}/` returns 404, `/s/{slug}/index.html` works.

### Roadmap entry
- `.planning/ROADMAP.md` §line 1268 — Phase 144 header; depends on Phase 143.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `convert_route_params` in `macros.rs` (line 56+) — already normalizes `:id` → `{id}`. The path-combination fix slots in naturally next to it as a sibling helper.
- `Box::leak(full_path.into_boxed_str())` pattern at `macros.rs:665` — used to satisfy `matchit`'s `&'static str` requirement. When the fix inserts a second variant, it needs a second `Box::leak` (or one helper that returns both).
- Existing `RouteInfo` / `REGISTERED_ROUTES` registry already handles the one-canonical-entry invariant via "find the most recent route with this path" logic — Phase 144 needs to call `register_route` only once per handler despite the double `insert_get` / `insert_post` call.

### Established Patterns
- Two parallel group APIs exist today (macros-based `group!` and builder-based `.group()`) and have drifted: macros.rs has a half-working `/` special case, group.rs has none. Any cross-cutting routing fix must touch both — the framework invariant is "both APIs produce the same registered routes for equivalent definitions." Phase 144 restores that invariant.
- Unit tests for routing live inline in their module files (`#[cfg(test)] mod tests { ... }` at the bottom of macros.rs and group.rs). Integration tests that exercise a full `routes! { ... }` macro live in `framework/tests/`.

### Integration Points
- `framework/src/lib.rs` — public re-exports: `group!`, `get!`, `post!`, `routes!`. No surface change from Phase 144.
- `ferro-mcp` introspection tools `list_routes`, `describe_route` consume `get_registered_routes()`. They must see exactly the same number of routes after the fix — double-registration in matchit MUST NOT produce two introspection entries (see D-07, D-10).
- `framework/src/middleware/*` — middleware is keyed by full path (`router.add_middleware(full_path, mw)`). When a handler is registered at both `/prefix` and `/prefix/`, middleware must be added for BOTH paths or the trailing-slash request will skip middleware. This is an easy-to-miss correctness trap — the planner must call it out explicitly.

</code_context>

<specifics>
## Specific Ideas

- The exact gestiscilo-it reproducer: `group!("/s/{slug}", { get!("/", pages::serve_root), get!("/index.html", pages::serve_index), get!("/{*path}", pages::serve_asset) })`. After the fix, `/s/amaris-experience/` must reach `serve_root` with `slug=amaris-experience`.
- `slug_add_trailing_slash` in the gestiscilo-it codebase (referenced in the ferro field test) currently exists as a workaround layer — after Phase 144 ships and gestiscilo upgrades, that workaround should be removable (not this phase's concern, but note it in the release changelog).

</specifics>

<deferred>
## Deferred Ideas

- Canonical URL enforcement (301-redirect `/prefix/` → `/prefix` or vice versa) — NOT this phase. Phase 144 accepts both; a future phase can add opt-in canonical-redirect middleware.
- Method routing mismatch (405 Method Not Allowed) for `/prefix` vs `/prefix/` — deferred, not a known issue today.
- Wildcard `/{*path}` interaction with group root-path handlers — already works per the gestiscilo field test (`/s/{slug}/lite/` reaches the wildcard). Phase 144 does not change wildcard semantics; tests cover the coexistence case only as regression protection.
- Reconciling the two group implementations into one — deferred. Phase 144 fixes both; a future refactor can pick a single implementation if the maintenance burden justifies it.

</deferred>

---

*Phase: 144-fix-root-path-routing-in-group-routes*
*Context gathered: 2026-04-21*
*Auto-mode decision log: see 144-DISCUSSION-LOG.md*
