---
phase: 253
slug: mcp-surface-docs-publish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-04
---

# Phase 253 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace) |
| **Config file** | Cargo.toml (workspace root) |
| **Quick run command** | `cargo test -p ferro-mcp design_lint` (targeted per touched crate) |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s; full suite several minutes |

**CPU-serialization rule (project convention):** never chain or parallelize cargo
runs; one test invocation at a time; reuse prior step's evidence rather than
re-running.

---

## Sampling Rate

- **After every task commit:** Run the targeted quick command for the touched crate
  (e.g. `cargo test -p ferro-mcp <module>`, `cargo test -p ferro-json-ui design`)
- **After every plan wave:** Targeted multi-crate run; full suite only at the final gate
- **Before `/gsd-verify-work` / publish push:** CI-exact gate green —
  `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, `cargo doc --no-deps` clean
- **Max feedback latency:** ~120 seconds (targeted run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (filled by planner) | | | DS-07 design_lint tool (inline + path) | path input | path is read-only, parse errors are findings not tool errors | unit | `cargo test -p ferro-mcp design_lint` | pending | pending |
| (filled by planner) | | | DS-07 catalog vocabulary | — | additive fields, 47-count mirror intact | unit | `cargo test -p ferro-mcp json_ui_catalog` | pending | pending |
| (filled by planner) | | | DS-07 generation_context summary | — | derived from registry/enums, drift-guarded | unit | `cargo test -p ferro-mcp generation_context` | pending | pending |
| (filled by planner) | | | DS-08 docs chapter | — | patterns.md ↔ `design::rules()` drift test | unit | `cargo test -p ferro-json-ui design` | pending | pending |
| (filled by planner) | | | DS-08 publish | — | CI-exact gate before push; verify via crates.io API | gate | full CI-exact command set | pending | pending |
