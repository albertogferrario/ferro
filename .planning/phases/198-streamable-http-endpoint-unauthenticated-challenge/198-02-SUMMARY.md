---
phase: 198-streamable-http-endpoint-unauthenticated-challenge
plan: "02"
subsystem: app
tags: [mcp, streamable-http, bearer-challenge, handler, routes]
dependency_graph:
  requires: [198-01]
  provides: [mcp-http-endpoint, bearer-401-challenge, order-mcp-exposed]
  affects: [app]
tech_stack:
  added: [ferro-mcp-server dep in app/Cargo.toml]
  patterns: [bearer-seam-call-before-body-read, explicit-405-for-method-mismatch, exposed-services-slice]
key_files:
  created:
    - app/src/controllers/mcp.rs
  modified:
    - app/Cargo.toml
    - app/src/controllers/mod.rs
    - app/src/routes.rs
    - app/src/projections/order.rs
decisions:
  - "ServiceDef imported from ferro:: (re-export) not ferro_projections:: — app crate does not depend on ferro-projections directly"
  - "exposed_services() and challenge_response() carry #[allow(dead_code)]: #[handler] macro wraps the function body and the dead-code lint cannot trace calls through the macro expansion"
  - "Ferro router 404s on method mismatch for a registered path — verified in framework/src/server.rs match_route → None → 404; explicit get!(/mcp) returning 405 is therefore required and implemented"
metrics:
  duration: "~480s"
  completed_date: "2026-06-10"
  tasks: 2
  files_changed: 5
---

# Phase 198 Plan 02: MCP HTTP Transport Wiring Summary

**One-liner:** Thin `POST /mcp` ferro handler calling the `ferro-mcp-server` bearer seam — always returns `401 + WWW-Authenticate: Bearer resource_metadata="…"` in Phase 198 — plus an explicit `GET /mcp → 405` handler, both registered in the app router, with the `order` projection marked `mcp_exposed(true)`.

## What Was Built

### app/src/controllers/mcp.rs (new)

Thin HTTP adapter over the pure dispatch functions from Plan 01. Load-bearing ordering: `Authorization` header extracted before `req.json()` consumes the request (Ferro single-read guarantee, RESEARCH Pitfall 3). Two `#[handler]` functions:

- `handle`: reads `Authorization` header → calls `extract_bearer` seam → always `Unauthenticated` in Phase 198 → returns `401` with `WWW-Authenticate: Bearer resource_metadata="{config.app_url}/.well-known/oauth-protected-resource"` (RFC 9728 / RFC 6750). Authenticated dispatch path wired for Phase 199 (unreachable now). Carries `// TODO(phase-199): validate Origin header`.
- `method_not_allowed`: returns `405 Method Not Allowed` + `Allow: POST`.

Private helpers `exposed_services()` and `challenge_response()` carry `#[allow(dead_code)]` because the `#[handler]` macro wraps the function body and Rust's dead-code lint cannot follow calls through the macro expansion.

Two unit tests:
- `challenge_response_has_correct_header`: asserts status 401 and exact `WWW-Authenticate` header value.
- `bearer_seam_always_challenges`: asserts both `None` and bearer-token headers return `BearerOutcome::Unauthenticated`.

### app/Cargo.toml

Added `ferro-mcp-server = { path = "../ferro-mcp-server", version = "0.2" }`. Not added to `framework/Cargo.toml` — per RESEARCH §D-02 dependency-weight finding (would pull `rmcp` + `schemars` into the core crate for all consumers even when MCP is unused).

### app/src/routes.rs

Two routes added to the `routes!` block:

```
post!("/mcp", controllers::mcp::handle).name("mcp.endpoint"),
get!("/mcp", controllers::mcp::method_not_allowed).name("mcp.endpoint.get"),
```

No additional middleware group needed for Phase 198; the bearer seam self-gates inside the handler. Both routes run through the same middleware stack as all other framework routes (satisfies SC-3).

### app/src/projections/order.rs

Added `.mcp_exposed(true)` to the `ServiceDef::new("order")` builder chain so the live `tools/list` surface is non-empty.

## Router Behavior Verification (Required by Plan)

Verified in `framework/src/server.rs` lines 262-318: `router.match_route(&method, &path)` keys on both method and path and returns `Some(handler, ...)` only on exact match. For a `GET /mcp` request against a `POST /mcp`-only registration, `match_route` returns `None`. The `None` arm (line 287) tries static file serving (GET/HEAD only) and then falls through to the fallback or default `404 Not Found`. **The router does NOT emit `405 Method Not Allowed` automatically on method mismatch for a registered path.** The explicit `get!("/mcp", method_not_allowed)` handler is required to satisfy the MCP Streamable HTTP spec, which mandates `405` when the server does not offer an SSE stream on GET.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing import path] Use ferro:: re-export for ServiceDef**
- **Found during:** Task 1 first compile
- **Issue:** `use ferro_projections::ServiceDef` caused `E0432: unresolved import` — the `app` crate depends on `ferro` (which re-exports `ServiceDef` via the `projections` feature) but not on `ferro-projections` directly.
- **Fix:** Changed import to `use ferro::ServiceDef;`.
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commit:** b4d0538f

**2. [Rule 2 - Missing allow] Add #[allow(dead_code)] to private helpers**
- **Found during:** Task 1 clippy run
- **Issue:** Clippy `-D warnings` reported `exposed_services` and `challenge_response` as never used. These functions ARE called in `handle`'s body, but Rust's dead-code lint cannot trace through the `#[handler]` proc-macro expansion.
- **Fix:** Added `#[allow(dead_code)]` with explanatory comments to both helpers.
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commit:** b4d0538f

## Test Results

```
cargo test -p app

running 2 tests
test controllers::mcp::tests::bearer_seam_always_challenges ... ok
test controllers::mcp::tests::challenge_response_has_correct_header ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Validation Bar

- `cargo fmt --all -- --check` — clean
- `cargo clippy -p app --all-targets -- -D warnings` — clean, 0 warnings
- `cargo build -p app` — clean
- `cargo test -p app` — 2 tests pass
- `cargo test --all-features` — exit code 0 (full workspace, all features)

## Known Stubs

- `exposed_services()`: returns `vec![crate::projections::order::service_def()]` — explicit slice, no registry. This is intentional for Phase 198. Phase 199+ can replace with a lazy-static registry.
- `BearerOutcome::Authenticated(_principal)` arm: wired but unreachable. Phase 199 fills `extract_bearer` internals to return `Authenticated` for valid tokens.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes beyond those in the plan's threat model (T-198-06 through T-198-10). The `POST /mcp` and `GET /mcp` surfaces are exactly as modelled.

## Self-Check

### Created files exist:

- FOUND: app/src/controllers/mcp.rs

### Commits verified:

- FOUND: b4d0538f — feat(198-02): add mcp.rs handler — bearer seam, 401 challenge, JSON-RPC dispatch wiring
- FOUND: 6f573e46 — feat(198-02): register POST+GET /mcp routes; expose order projection; add 405 handler

## Self-Check: PASSED
