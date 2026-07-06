---
phase: 124-doctor-introspection-and-ci-scaffold
plan: 02
subsystem: cli
tags: [cli, json, schema, ferro-mcp, generate-routes, serde]

requires:
  - phase: 22
    provides: generate_routes scanner (parse_routes_file, scan_routes, RouteDefinition)
provides:
  - "ferro generate-routes --json: stable JSON schema emitter"
  - "RoutesJson / RouteJson public serde types"
  - "docs/src/cli/routes-json-schema.md (stability contract)"
affects: [ferro-mcp, ci-scaffold, agent-introspection]

tech-stack:
  added: []
  patterns:
    - "Stable JSON contracts for agent-consumable CLI output (additive-only schema, documented stability table)"

key-files:
  created:
    - docs/src/cli/routes-json-schema.md
    - .planning/phases/124-doctor-introspection-and-ci-scaffold/deferred-items.md
  modified:
    - ferro-cli/src/commands/generate_routes.rs
    - ferro-cli/src/main.rs

key-decisions:
  - "middleware field is part of the stable contract from day one but always emits [] in Phase 124 — middleware parsing deferred without breaking future consumers"
  - "path_params NOT serialized — consumers parse {param} placeholders from `path` themselves, keeping schema minimal"
  - "Added new GenerateRoutes clap variant rather than overloading GenerateTypes — generate-routes was previously library-only (#[allow(dead_code)] on run)"
  - "Method strings are uppercase (GET/POST/...) to match HTTP convention; existing internal to_ts_method (lowercase) preserved for the TypeScript codepath"

patterns-established:
  - "CLI JSON output: pretty-printed JSON to stdout, errors to stderr, non-zero exit on failure"
  - "Public serde structs documented as a contract in docs/src/cli/*-schema.md with a stability table"

requirements-completed: [D-10, D-11, D-12]

duration: ~15min
completed: 2026-04-07
---

# Phase 124 Plan 02: ferro generate-routes --json with stable schema Summary

**Stable JSON schema emitter for `ferro generate-routes` consumed by ferro-mcp, with a documented additive-only contract and full serde round-trip test coverage.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-07T01:36:00Z
- **Completed:** 2026-04-07T01:51:59Z
- **Tasks:** 2
- **Files modified:** 4 (2 source, 2 docs)

## Accomplishments
- `ferro generate-routes --json` prints a stable JSON document to stdout (D-10)
- `RoutesJson` / `RouteJson` serde types published as public contract with `routes[].method/path/handler/name/middleware` fields (D-11)
- Schema documented at `docs/src/cli/routes-json-schema.md` with field-level stability table and ferro-mcp consumer hint (D-12)
- Default `ferro generate-routes` (no flag) behavior unchanged: still writes TypeScript route helpers
- `generate_routes::run` is now reachable from the CLI (was `#[allow(dead_code)]` library-only entry)

## Task Commits

1. **Task 1: JSON schema types + tests + docs** — `b102a5f9` (feat, TDD-style: types and 7 unit tests landed together)
2. **Task 2: Wire --json through clap** — `0074f5db` (feat)

## Files Created/Modified
- `ferro-cli/src/commands/generate_routes.rs` — added `RoutesJson`, `RouteJson`, `HttpMethod::as_str_upper`, `routes_to_json`, `generate_json_string`, `run_json`, 7 unit tests; removed dead_code allow on `run`
- `ferro-cli/src/main.rs` — new `GenerateRoutes { output, json }` clap variant + match arm dispatching to `run_json` or `run`
- `docs/src/cli/routes-json-schema.md` — stable schema reference, stability table, example output, ferro-mcp consumer hint
- `.planning/phases/124-doctor-introspection-and-ci-scaffold/deferred-items.md` — logged out-of-scope ferro-json-ui fmt drift

## Decisions Made
- See key-decisions in frontmatter. Notably: emit `middleware: []` from day one to lock the contract shape now and let middleware parsing land later without a breaking change.

## Deviations from Plan

None — plan executed exactly as written. Both tasks landed cleanly; the discovered fmt drift in `ferro-json-ui` is unrelated to this plan and was logged to `deferred-items.md` per scope boundary rules rather than fixed.

## Issues Encountered

- `cargo fmt --all -- --check` reports pre-existing drift in `ferro-json-ui/src/{component,render}.rs`. Out of scope for 124-02. Logged to `deferred-items.md`. Scoped checks (`cargo fmt -p ferro-cli -- --check && cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings && cargo test -p ferro-cli`) pass clean.

## Verification

- `cargo test -p ferro-cli generate_routes` → 23 tests pass (16 pre-existing + 7 new JSON tests)
- `cargo test -p ferro-cli` → all suites green (lib, golden, doc-tests)
- `cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings` → clean
- `cargo run -p ferro-cli -- generate-routes --help` → shows `--json` flag with description
- Manual schema spot-check: `routes_to_json(&[GET /users])` serializes to exactly `{"routes":[{"method":"GET","path":"/users","handler":"controllers::user::index","name":null,"middleware":[]}]}`

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- ferro-mcp can now shell out to `ferro generate-routes --json` and deserialize against the documented schema (D-12 contract live)
- Middleware extraction from `routes.rs` is the obvious follow-up — the field already exists in the contract, populating it is non-breaking
- Plans 124-03/04/05 (doctor, CI scaffold, ignore sync) are unblocked

## Self-Check: PASSED

Verified files exist:
- FOUND: ferro-cli/src/commands/generate_routes.rs (RoutesJson, run_json, middleware)
- FOUND: ferro-cli/src/main.rs (GenerateRoutes, run_json)
- FOUND: docs/src/cli/routes-json-schema.md (RoutesJson)
- FOUND: .planning/phases/124-doctor-introspection-and-ci-scaffold/deferred-items.md

Verified commits exist on master:
- FOUND: b102a5f9 (feat(124-02): add stable JSON schema for generate-routes)
- FOUND: 0074f5db (feat(124-02): wire generate-routes --json flag through clap)

---
*Phase: 124-doctor-introspection-and-ci-scaffold*
*Completed: 2026-04-07*
