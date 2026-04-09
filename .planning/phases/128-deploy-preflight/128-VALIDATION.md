---
phase: 128
slug: deploy-preflight
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-09
---

# Phase 128 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `ferro-cli/Cargo.toml`, `ferro-mcp/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-cli doctor` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~90s full, ~15s scoped |

---

## Sampling Rate

- **After every task commit:** Run the task's `<automated>` verify command.
- **After every plan wave:** Run `cargo test -p ferro-cli` + `cargo test -p ferro-mcp`.
- **Before `/gsd:verify-work`:** Full suite + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings`.
- **Max feedback latency:** 90 seconds.

---

## Per-Task Verification Map

| Plan | Task | Automated Command | Wave 0 dep? |
|------|------|-------------------|-------------|
| 128-01 | 1 — CheckCategory enum + trait default | `cargo test -p ferro-cli doctor::check -- --nocapture` | none |
| 128-01 | 2 — Extract read_path_dep_version | `cargo test -p ferro-cli doctor::checks::cargo_docker_toml_staleness` | none |
| 128-02 | 1 — copy_dirs_dockerignore_collision | `cargo test -p ferro-cli doctor::checks::copy_dirs_dockerignore_collision` | 128-01 Task 1 (CheckCategory) |
| 128-02 | 2 — ferro_version_skew | `cargo test -p ferro-cli doctor::checks::ferro_version_skew` | 128-01 Task 1 + Task 2 |
| 128-02 | 3 — Registry + --deploy flag + staleness category | `cargo test -p ferro-cli doctor` | 128-02 Task 1 + Task 2 |
| 128-03 | 1 — deploy_init compute + persist + execute | `cargo test -p ferro-cli commands::deploy_init` | 128-01 Task 1 (CheckCategory only for --deploy doc link; compilation independent) |
| 128-03 | 2 — CLI dispatcher wiring | `cargo build -p ferro-cli` | 128-03 Task 1 |
| 128-04 | 1 — deploy_check MCP tool | `cargo test -p ferro-mcp deploy_check` | 128-02 Task 3 (binary must be buildable to shell out) |
| 128-04 | 2 — Docs update | `test -f "$(find docs/src -name '*.md' | xargs grep -l 'ferro deploy:init' 2>/dev/null | head -1)"` | 128-03 + 128-02 (to document accurately) |

---

## Wave 0 Requirements

None beyond what Plan 128-01 establishes. All new check tests use existing
`tempfile::TempDir` fixtures. The `default_checks_returns_eleven_in_declared_order`
test is updated in-place during Plan 128-02 Task 3 (no separate Wave 0 step).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Interactive `ferro deploy:init` prompt flow | REPORT item 15 | `dialoguer` TTY interaction | Run `ferro deploy:init` in a sample project, answer prompts, verify Cargo.toml write preserves unrelated content |
| `ferro deploy:init` existing-table Select menu | D-09 | `dialoguer::Select` interactive only | Pre-populate `[package.metadata.ferro.deploy]`, run `ferro deploy:init`, confirm abort/overwrite/merge menu appears |
| MCP `deploy_check` tool shape in real MCP client | D-03 | Requires live MCP client | Launch `ferro mcp`, call `deploy_check`, inspect JSON Report shape |
| `ferro doctor --deploy` end-to-end on a real project | D-02 | Requires a real project checkout | Run in `gestiscilo-it/app` or similar; confirm only three checks run and exit code is meaningful |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or documented Wave 0 dependency
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covered by Plan 128-01 (foundation)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true`

**Approval:** planner-signed 2026-04-09
