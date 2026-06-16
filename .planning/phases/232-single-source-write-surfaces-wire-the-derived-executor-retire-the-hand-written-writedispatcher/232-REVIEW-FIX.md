---
phase: 232-single-source-write-surfaces
fixed_at: 2026-06-16T00:00:00Z
review_path: .planning/phases/232-single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher/232-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 232: Code Review Fix Report

**Fixed at:** 2026-06-16T00:00:00Z
**Source review:** 232-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (fix_scope = critical_warning; 0 critical, 2 warning)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Form-urlencoded bodies break id extraction — the endpoint's primary path fails

**Files modified:** `app/src/controllers/mcp.rs`, `app/src/tests/visual_action.rs`, `app/Cargo.toml`, `Cargo.lock`
**Commit:** 3b7fc160
**Applied fix:** Coerced the shared id extraction in `make_write_dispatcher()` to accept
string-encoded numerics — `inputs["id"].as_i64().or_else(|| inputs["id"].as_str().and_then(|s| s.parse::<i64>().ok()))`.
This is the safest fix site: it keeps BOTH channels correct (MCP sends JSON
integers, the visual/form path sends `application/x-www-form-urlencoded` bodies
that `serde_urlencoded` decodes as JSON strings — `id=1` → `{"id": "1"}`). The
same coercion was mirrored in the test dispatcher (`app/src/tests/visual_action.rs`)
so the test path exercises the real form shape rather than the JSON-int shape the
existing tests used.

Added two regression tests:
- `form_urlencoded_id_decodes_as_string` — drives the actual `serde_urlencoded`
  decode (the decoder `req.input::<Value>()` uses) and asserts `id=1` yields the
  JSON string `"1"` with `as_i64() == None`, locking the premise WR-01 rests on.
  `serde_urlencoded` was added as an app dev-dependency to drive this exact path.
- `visual_action_accepts_form_string_id` — drives the shared kernel with the
  form-shaped body (`{"id": "1"}`) and asserts the transition succeeds and
  persists the derived `to_state`. This is the primary (HTML form) path the prior
  tests missed (they passed JSON `{"id": 1}` only).

### WR-02: Silent `.insert(...).ok()` route registration masks future param-route conflicts

**Files modified:** `framework/src/routing/router.rs`
**Commit:** 34e15e7e
**Applied fix:** Introduced a private `insert_route()` helper that calls
`MatchitRouter::insert` and, on `Err`, emits a `tracing::error!` naming the method,
path, and matchit error instead of `.ok()`-discarding the rejection. Routed all
five internal group-insert helpers (`insert_get/post/put/patch/delete`) and the
five public builder methods (`get/post/put/patch/delete` — the path
`app/src/routes.rs:119` actually uses) through it.

Conservative scope: the success path is byte-for-byte identical (the route is
still inserted), only a CONFLICT is made loud. It logs rather than panics, so
legitimate idempotent re-registration of the same pattern stays non-fatal while
remaining visible. Today's clean registration produces no conflict — confirmed by
the framework lib suite (493 passed, including all `routing::*` tests) and by the
phase's own `visual_route_registered_without_shadowing` test (still green). The
alias-insert methods were intentionally left untouched (out of finding scope;
they point at already-registered canonical routes).

## Gate Results

Run one CPU op at a time (one-CPU-op-at-a-time rule). Host disk was tight
(~3.1Gi free); no ENOSPC occurred.

| Step | Command | Result |
|------|---------|--------|
| Framework tests (router change) | `cargo test -p ferro-rs --lib` | PASS — 493 passed, 0 failed; all `routing::*` green |
| App tests (incl. new form-body tests) | `cargo test -p app` | PASS — 30 passed, 0 failed; `form_urlencoded_id_decodes_as_string` + `visual_action_accepts_form_string_id` green |
| Clippy | `cargo clippy --all --all-targets -- -D warnings` | PASS — exit 0, no warnings |
| Format | `cargo fmt --all -- --check` | PASS — clean (after `cargo fmt --all` reflowed the new code) |

Note: `cargo test -p app --all-features` enables the `confirmation` feature,
which gates out the `visual_action.rs` test module
(`#[cfg(all(test, not(feature = "confirmation")))]`). The new WR-01 tests were
therefore exercised via `cargo test -p app` (no `--all-features`), where they
pass. No `docs/protocol/schemas/*.json` churn was produced. The only `Cargo.lock`
change is the legitimate addition of the `serde_urlencoded` dev-dependency to the
`app` crate; it was committed with WR-01.

---

_Fixed: 2026-06-16T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
