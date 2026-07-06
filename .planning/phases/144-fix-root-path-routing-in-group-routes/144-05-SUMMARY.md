---
phase: 144-fix-root-path-routing-in-group-routes
plan: 05
subsystem: release
tags: [docs, changelog, release]

requires:
  - 144-01
  - 144-02
  - 144-03
  - 144-04

provides:
  - "ferro-rs 0.2.13 release: docs, CHANGELOG entry, version bump"
  - "docs/src/the-basics/routing.md — root-in-group subsection documenting dual-variant semantics and the root-in-root degenerate case"
  - "docs/src/the-basics/middleware.md — invariant note clarifying that middleware attached to a group runs for both `/prefix` and `/prefix/`"
  - "framework/src/routing/macros.rs — rustdoc updates on group! reflecting the new semantics"

affects: []

tech-stack:
  added: []
  patterns:
    - "Neutral release voice — CHANGELOG describes what changed, not internal strategy"

key-files:
  created: []
  modified:
    - docs/src/the-basics/routing.md
    - docs/src/the-basics/middleware.md
    - framework/src/routing/macros.rs
    - CHANGELOG.md
    - Cargo.toml

key-decisions:
  - "Version bump to 0.2.13 (patch) — the change is a bug fix with no public-API break. `combine_group_path` is pub(crate) and the `insert_{method}_alias` methods are pub(crate). The only externally visible effect is that grouped root routes now resolve at both `/prefix` and `/prefix/`, which is a strictly additive behavior."
  - "Full `cargo test --all-features` was NOT run as part of this plan's acceptance criterion because it exhausted disk space on the thermally-stressed development machine. Substituted: targeted `cargo test -p ferro-rs --lib --features json-ui routing::` (22/22 pass), `cargo test -p ferro-rs --test routing_group_trailing_slash --features json-ui` (5/5 pass), `cargo fmt --all -- --check` (clean), `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings` (clean). The full workspace gate should be run on CI before publication."

verification:
  - "cargo fmt --all -- --check — clean"
  - "cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings — 0 warnings"
  - "cargo test -p ferro-rs --lib --features json-ui routing:: — 22 passed, 0 failed"
  - "cargo test -p ferro-rs --test routing_group_trailing_slash --features json-ui — 5 passed, 0 failed"

deviations:
  - "Acceptance criterion `cargo test --all-features` deferred to CI due to disk exhaustion during thermal-stressed local execution. All lighter verification commands pass cleanly."
---

# 144-05: Release artifacts for ferro-rs 0.2.13

## What shipped

Five commits across master:

| Commit | Scope |
|--------|-------|
| `49b91d29` | docs/src/the-basics/routing.md — root-in-group subsection |
| `e816e217` | docs/src/the-basics/middleware.md + framework/src/routing/macros.rs rustdoc |
| `3fc85883` | CHANGELOG.md — 0.2.13 entry in neutral voice |
| `307a1737` | Cargo.toml version bump 0.2.12 → 0.2.13 + cargo fmt |

## Semantics now documented

`group!("/prefix", { get!("/", h) })` reaches `h` at both `/prefix` and `/prefix/`. Middleware attached to the group runs for both variants. The root-in-root case `group!("/", { get!("/", h) })` yields exactly one route at `/`. Non-root leaves inside a group do not emit a trailing-slash alternate.

## Verification

All targeted test gates pass:

- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings` — 0 warnings
- `cargo test -p ferro-rs --lib --features json-ui routing::` — 22 passed
- `cargo test -p ferro-rs --test routing_group_trailing_slash --features json-ui` — 5 passed

`cargo test --all-features` was not run locally because the workspace's full-feature compilation (async-stripe, sea-orm, sqlx) exceeded available disk space during thermal-stressed local execution. Run it on CI before `cargo publish`.

## Self-Check: PASSED

The four tasks in the plan all completed. Task 4 (version bump + fmt) was committed on master directly by the orchestrator after the worktree agent was blocked by disk exhaustion. All content matches the plan specification.
