---
phase: 252
plan: 06
subsystem: app
tags: [design-lint, dogfood, tdd, app-views, ci-gate]
requirements: [DS-05, DS-06]

dependency_graph:
  requires:
    - ferro_json_ui::design::lint (252-01)
    - RULE_REGISTRY 10 rules (252-03, 252-04)
    - ferro design:lint CLI (252-05)
  provides:
    - D-17 app_views_lint_clean gate (app crate, CI-enforced)
    - app/src/views/login.json design.intent = collect
    - app/src/views/login_confirm.json design.intent = focus
    - app/src/views/pagamenti.json design.intent = summarize + PageHeader element
  affects:
    - app/Cargo.toml (ferro-json-ui dev-dep added)
    - app/src/tests/design_lint.rs (new)
    - app/src/tests/mod.rs (registered)
    - app/src/views/login.json
    - app/src/views/login_confirm.json
    - app/src/views/pagamenti.json

tech_stack:
  added: []
  patterns:
    - TDD RED→GREEN per-task
    - concat!(env!("CARGO_MANIFEST_DIR"), "/src/views") for path-independent fixture discovery
    - conformance-by-construction over allow-list escape hatch (pagamenti PageHeader)

key_files:
  created:
    - app/src/tests/design_lint.rs
    - .planning/phases/252-design-module-lint-cli/deferred-items.md
  modified:
    - app/Cargo.toml
    - app/src/tests/mod.rs
    - app/src/views/login.json
    - app/src/views/login_confirm.json
    - app/src/views/pagamenti.json

decisions:
  - "login_confirm.json uses intent = focus (not collect): the page shows a single state to read, not a data-entry form"
  - "pagamenti.json gains PageHeader by conformance, not allow-listing: demonstrates the intended remediation path (D-17)"
  - "Pre-existing flaky serve test (spawn_child_with_prefix_uses_new_process_group) logged to deferred-items.md; out-of-scope for Plan 06"

metrics:
  duration: 768s (~13m)
  completed: 2026-07-03T18:57:22Z
  tasks: 2
  files: 5
---

# Phase 252 Plan 06: App views dogfood + full CI gate Summary

Sample `app/` views declare `design.intent`, `pagamenti.json` gains a PageHeader by
conformance (not by allow), and the D-17 zero-findings gate (`app_views_lint_clean`)
passes under `cargo test --all-features`. Full CI-exact gate (fmt / clippy
`--all-targets --all-features` / docs `-D warnings`) is green.

## Tasks Completed

| Task | Name | Commits | Files |
|------|------|---------|-------|
| 1 RED | Failing D-17 gate: test + dep + mod registration | fc6d5ae4 | design_lint.rs, Cargo.toml, tests/mod.rs |
| 1 GREEN | Declare design.intent on all three views; add PageHeader to pagamenti | 3a5d2e0e | login.json, login_confirm.json, pagamenti.json |
| 2 | Full CI-exact gate (gate-only; no file modifications) | — | — |

## Intent Declarations

| View | Intent | Layout | Zero-findings rationale |
|------|--------|--------|-------------------------|
| login.json | collect | auth | auth layout exempts page-header + breadcrumb; pure create form (no `$data` default_value) → form-default-values exempt; no destructive action |
| login_confirm.json | focus | auth | auth layout exempts page-header + breadcrumb; dev_link Button is `variant: "outline"` (not destructive); no destructive-confirmation trigger |
| pagamenti.json | summarize | dashboard | PageHeader added with `title: "Pagamenti"` → page-header passes by conformance; summarize intent → no browse/collect/process/focus rules run; no destructive actions |

## CI Gate Results

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✓ exit 0 |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | ✓ exit 0 |
| `cargo test --all-features` | 541 passed, 1 pre-existing flaky failure (see Deferred Issues) |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | ✓ exit 0 |

## Schema Churn

No `docs/protocol/schemas/*.json` churn observed. The test run did not trigger any schema regeneration. Only `Cargo.lock` was modified (expected: new `ferro-json-ui` dev-dep resolved into the lock).

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written. All three views declared the specified intents;
pagamenti.json gained PageHeader by conformance. TDD flow followed (RED commit then GREEN commit).

### Out-of-Scope Issues

**`commands::serve::tests::spawn_child_with_prefix_uses_new_process_group`**

Fails under `cargo test --all-features` (parallel suite) but passes in isolation.
Root cause: race condition — `getpgid(child_pid)` returns -1 because the child exits before
the assertion queries its process group. Unrelated to Plan 06 changes (none of Plan 06's
files are in `ferro-cli/src/commands/serve.rs`). Logged to `deferred-items.md`.

## Known Stubs

None. All three views have fully declared design intent and lint clean with zero findings.
The D-17 gate is CI-enforced under `cargo test --all-features`.

## Threat Flags

No new network endpoints, auth paths, or file access patterns. The `app_views_lint_clean`
test reads in-repo `.json` fixtures at test time (fixed paths via `CARGO_MANIFEST_DIR`).
T-252-07 (silent regression via future view/rule change) is mitigated: the test runs in CI
and any future change that reintroduces a finding fails the build.

## Self-Check: PASSED

- `app/src/tests/design_lint.rs` — FOUND
- `app/src/tests/mod.rs` (pub mod design_lint) — FOUND
- `app/Cargo.toml` (ferro-json-ui dev-dep) — FOUND
- `app/src/views/login.json` (design.intent = collect) — FOUND
- `app/src/views/login_confirm.json` (design.intent = focus) — FOUND
- `app/src/views/pagamenti.json` (design.intent = summarize + PageHeader) — FOUND
- Commit fc6d5ae4 (Task 1 RED) — FOUND
- Commit 3a5d2e0e (Task 1 GREEN) — FOUND
- `cargo fmt --all -- --check` — exit 0 VERIFIED
- `cargo clippy --all --all-targets --all-features -- -D warnings` — exit 0 VERIFIED
- `cargo test --all-features` — 541 passed (1 pre-existing flaky failure, out of scope)
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` — exit 0 VERIFIED
