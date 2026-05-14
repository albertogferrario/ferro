---
phase: 157
slug: migration-deploy-safety-backend-portable-backfill-helpers-fe
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 157 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test -p ferro-migration -p ferro-cli --all-features 2>&1 \| tail -20` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli 2>&1 | tail -20`
- **After wave complete:** Run full suite

---

## Validation Dimensions

| Dimension | Check |
|-----------|-------|
| Migration helpers compile | `cargo build -p ferro-migration` exits 0 |
| Backfill SQL correct per backend | unit tests in ferro-migration assert correct SQL per DbBackend |
| do:init emits jobs block | `render_app_yaml` test asserts `jobs:` present |
| migrate_gate Error on missing gate | `check_impl` test returns `CheckStatus::Error` |
| Silent runner aborts on failure | `run_migrations_silent` calls `process::exit(1)` |
| Registry test updated | `default_checks_returns_twelve_in_declared_order` passes |
| debug_assert on unresolved tokens | `render_app_yaml` leaves no `{{` in output |
