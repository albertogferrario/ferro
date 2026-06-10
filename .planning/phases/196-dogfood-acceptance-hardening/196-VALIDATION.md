---
phase: 196
slug: dogfood-acceptance-hardening
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 196 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` (tokio in dev-dependencies) |
| **Config file** | `ferro-mcp/Cargo.toml` (existing; no new config needed) |
| **Quick run command** | `cargo test -p ferro-mcp checkpoint_projection -- --nocapture` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~quick: <30s for the checkpoint module; full suite: minutes (workspace-wide) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp checkpoint_projection -- --nocapture`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds (module-scoped quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 196-SC1 | TBD | 1 | CHK-10 / SC-1 | — | N/A | unit | `cargo test -p ferro-mcp poisoned_projection_dangling_field_acceptance` | ❌ W0 | ⬜ pending |
| 196-SC2 | TBD | 1 | CHK-10 / SC-2 | — | N/A | integration | `cargo test -p ferro-mcp dogfood_app_projections` | ❌ W0 | ⬜ pending |
| 196-SC3 | TBD | 1 | CHK-10 / SC-3 | — | N/A | unit | `cargo test -p ferro-mcp next_steps_cap_at_five` | ❌ W0 | ⬜ pending |
| 196-SC4 | TBD | 2 | CHK-10 / SC-4 | — | N/A | manual code review | review `service.rs` + `docs/src/agents/checkpoint-projection.md` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Existing test to update:** `next_steps_cap_at_10` (≈line 1361/1368 in `ferro-mcp/src/tools/checkpoint_projection.rs`) must be modified to assert `== 5` in the same commit as the cap change.

---

## Wave 0 Requirements

- [ ] `poisoned_projection_dangling_field_acceptance` — new test in existing `mod tests` block (use single-word struct name, e.g. `ServiceDef::new("dangling")` + `model_src_with_fields("Dangling", ...)`, to satisfy the seam-2 lowercase matcher)
- [ ] `dogfood_app_projections` — new `#[tokio::test]` in existing `mod tests` block; resolve `app/` path via `CARGO_MANIFEST_DIR`; call seam functions directly per file (NOT `run_for`, due to the `service_def` function-name collision across all `app/` projections)
- [ ] `next_steps_cap_at_five` — new test OR in-place update of existing `next_steps_cap_at_10`
- [ ] `196-ACCEPTANCE.md` — written after the dogfood run; records per-seam finding tally + GO/NO-GO verdict

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Demoted seams documented as `not_checked`-by-default | CHK-10 / SC-4 | Documentation assertion, not a runtime property | Inspect `ferro-mcp/src/service.rs` tool description and `docs/src/agents/checkpoint-projection.md`: any wrapper seam with zero dogfood findings must be explicitly described as `not_checked`-by-default, not omitted |
| GO/NO-GO acceptance verdict reflects real run output | CHK-10 / SC-2 | The gate is a human go/no-go decision recorded in a report | Confirm `196-ACCEPTANCE.md` records the actual `app/` per-seam tally and that the verdict is GO only if ≥1 real finding was observed |

---

*Phase: 196-dogfood-acceptance-hardening*
*Validation strategy created: 2026-06-10*
