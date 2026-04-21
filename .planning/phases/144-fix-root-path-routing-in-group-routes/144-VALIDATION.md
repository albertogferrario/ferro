---
phase: 144
slug: fix-root-path-routing-in-group-routes
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-21
---

# Phase 144 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Derived from RESEARCH.md "Validation Architecture" (lines 414–458) and CONTEXT.md decisions D-09, D-10, D-11.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Built-in Rust `#[test]` + `cargo test --all-features` |
| **Config file** | None — tests live inline under `#[cfg(test)] mod tests { … }` in each module (existing convention, `framework/src/routing/macros.rs` line 1178). Integration tests live under `framework/tests/` (new file `routing_group_trailing_slash.rs`). |
| **Quick run command** | `cargo test -p ferro-rs --lib routing::` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds for routing tests, ~3–5 min for full workspace gate |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-rs --lib routing::`
- **After every plan wave:** Run `cargo test -p ferro-rs --all-features`
- **Before `/gsd-verify-work`:** Full workspace gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Max feedback latency:** 30 seconds on the inner loop (routing-only); 5 min on the full gate.

---

## Per-Task Verification Map

| Decision | Plan | Wave | Behavior | Test Type | Automated Command | File Exists | Status |
|----------|------|------|----------|-----------|-------------------|-------------|--------|
| helper | 01 | 0 | `combine_group_path(prefix, route_path)` — 8-row matrix covering all D-09 cases | unit (table-driven) | `cargo test -p ferro-rs --lib routing::path::tests::combine_group_path_matrix` | ❌ W0 — new `framework/src/routing/path.rs` | ⬜ pending |
| D-01 | 02 | 1 | `group!("/prefix", { get!("/", h) })` reaches handler at both `/prefix` and `/prefix/` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::group_root_handler_matches_both_variants` | ❌ W0 | ⬜ pending |
| D-02 | 02 | 1 | `group!("/", { get!("/", h) })` registers exactly one `/` route (no `//`) | unit | `cargo test -p ferro-rs --lib routing::macros::tests::root_prefix_root_handler_is_single_slash` | ❌ W0 | ⬜ pending |
| D-03 | 02 | 1 | `group!("/api/", { get!("/x", h) })` produces `/api/x`, not `/api//x` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::trailing_slash_prefix_is_stripped` | ❌ W0 | ⬜ pending |
| D-04 | 02 | 1 | `group!("/api", { get!("/users", h) })` still produces `/api/users` (regression) | unit | `cargo test -p ferro-rs --lib routing::macros::tests::non_root_prefix_non_root_path_unchanged` | ❌ W0 | ⬜ pending |
| D-05 | 03 | 1 | `.group("/prefix", \|r\| r.get("/", h))` builder API passes the same matrix as `group!` | unit | `cargo test -p ferro-rs --lib routing::group::tests::<mirrored matrix>` | ❌ W0 | ⬜ pending |
| D-06 | 02 | 1 | Nested `group!("/a", { group!("/b", { get!("/", h) }) })` matches `/a/b` and `/a/b/`; `group!("/a/", { group!("/b", …) })` also normalizes | unit | `cargo test -p ferro-rs --lib routing::macros::tests::nested_group_root_matches_both_variants` | ❌ W0 | ⬜ pending |
| D-07 | 04 | 2 | `get_registered_routes()` contains exactly one `RouteInfo` per logical handler after `group!("/prefix", { get!("/", h) })` | integration | `cargo test -p ferro-rs --test routing_group_trailing_slash -- no_duplicate_route_info` | ❌ W0 — new integration file | ⬜ pending |
| D-08 | 02 | 1 | Named-route: `get!("/", h).name("home")` inside `group!("/api", …)` → `route_url("home", &[])` returns `/api` (canonical, not `/api/`) | unit | `cargo test -p ferro-rs --lib routing::macros::tests::named_route_resolves_to_canonical` | ❌ W0 | ⬜ pending |
| gestiscilo | 04 | 2 | `group!("/s/{slug}", { get!("/", root), get!("/index.html", idx), get!("/{*path}", asset) })` reproducer — all four URL shapes reach the right handler with the correct `slug` param | integration | `cargo test -p ferro-rs --test routing_group_trailing_slash -- gestiscilo_reproducer` | ❌ W0 | ⬜ pending |
| regression | 02 | 1 | Top-level `get!("/", h)` (outside any group) remains single `/` | unit | `cargo test -p ferro-rs --lib routing::tests::top_level_root_route_is_single_slash` | ❌ W0 | ⬜ pending |
| middleware | 04 | 2 | Middleware on `group!("/prefix", …).middleware(Mw)` runs for BOTH `GET /prefix` and `GET /prefix/` requests | integration | `cargo test -p ferro-rs --test routing_group_trailing_slash -- middleware_runs_for_both_variants` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `framework/src/routing/path.rs` — NEW file with `combine_group_path(prefix, route_path) -> (canonical: String, alternate: Option<String>)` helper + inline `#[cfg(test)] mod tests` covering the 8-row matrix (D-09)
- [ ] Extend `framework/src/routing/macros.rs` inline `#[cfg(test)] mod tests` (around line 1178) with the D-01 through D-04, D-06, D-08 cases and the gestiscilo reproducer
- [ ] Add `#[cfg(test)] mod tests` to `framework/src/routing/group.rs` (currently has none) mirroring the macros.rs matrix (D-11)
- [ ] `framework/tests/routing_group_trailing_slash.rs` — NEW integration test file asserting D-07 (no duplicate RouteInfo) and the middleware-both-variants invariant. Use `serial_test::serial` to guard against ordering bleed from the process-global `REGISTERED_ROUTES`.
- [ ] Test helper: `fn dispatch(router: &Router, method: &str, path: &str) -> Option<(HashMap<String,String>, String)>` returning `(params, route_pattern)` — lives in a private `mod test_util` at the bottom of each affected test module, or in a shared `#[cfg(test)] pub(crate) mod test_util;` inside `routing/mod.rs`.

*No framework install needed — `serial_test` is already in `[dev-dependencies]` in `framework/Cargo.toml`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| gestiscilo-it upgrade sanity check | D-13 (changelog claim) | End-to-end production confirmation of the bug being fixed requires the downstream consumer to bump `ferro_version` and rebuild | After 0.2.13 publishes, in gestiscilo-it repo: bump `ferro_version` to 0.2.13 in `app/Cargo.toml`, `cargo update`, deploy. Visit `https://gestiscilo.it/s/amaris-experience/` and confirm the three-panel landing renders (not 404). Remove the `slug_add_trailing_slash` workaround. Record outcome in gestiscilo's phase notes, not in this phase. |

*All in-repo behaviors have automated verification. The gestiscilo field-test re-check is out of scope for this phase but tracked as a post-release sanity check.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags (`cargo watch` intentionally excluded — each invocation is a one-shot gate)
- [x] Feedback latency < 30s for inner loop (routing tests only)
- [x] `nyquist_compliant: true` set in frontmatter after planner writes tasks

**Approval:** approved — planning complete, all plans authored with full `<automated>` coverage. `wave_0_complete` flips to `true` after Plan 01 ships.
