---
phase: 263
slug: projection-native-inertia-substrate
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-27
---

# Phase 263 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `263-RESEARCH.md` → ## Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test --all-features`) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p <changed_crate>` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | full workspace build+test dominated; quick per-crate ~seconds |

CI-exact gate (must be green before verify): `cargo fmt --all -- --check` +
`cargo clippy --all --all-targets -- -D warnings` + `cargo test --all-features`.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <changed_crate>`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full CI-exact gate must be green
- **Max feedback latency:** per-crate test (seconds); full workspace on wave merges

---

## Per-Task Verification Map

Task IDs are assigned by the planner (`/gsd-plan-phase` → PLAN.md). This map is keyed by
requirement with the concrete automated signal each task must satisfy; the planner attaches
the matching Task ID / Plan / Wave when plans are created.

| Requirement | Secure/Correct Behavior | Test Type | Automated Command | File Location | Status |
|-------------|-------------------------|-----------|-------------------|---------------|--------|
| SUBST-01 | `schema_contract(&ServiceDef)` returns correct field names, meanings, validations, action defs; serde round-trip holds | snapshot + unit | `cargo test -p ferro-projections schema_contract` | `ferro-projections/tests/schema_contract.rs` (Wave 0 gap) | ⬜ pending |
| SUBST-02 | `permitted_actions` hides an action when its guard is `Some(false)`; shows it when guard passes | unit | `cargo test -p framework permitted_actions` | `framework/src/permitted_actions.rs` (inline `#[cfg(test)]`) | ⬜ pending |
| SUBST-02 | After the lift, MCP `tools/list` returns the same tools as before the refactor (no regression) | regression | `cargo test -p ferro-mcp-server` | `ferro-mcp-server/src/renderer.rs` (extend existing tests) | ⬜ pending |
| SUBST-03 | Data query is tenant-scoped; cross-tenant rows excluded (cross-tenant ids not found) | integration | `cargo test -p app data_tenant_scoping` | `app/src/tests/data_tenant_scoping.rs` (Wave 0 gap; check `crud_e2e.rs` first) | ⬜ pending |
| SUBST-03 | `Inertia::from_projection` serializes `{ schema, data, permitted_actions }` props correctly | unit | `cargo test -p ferro-inertia from_projection` | `ferro-inertia/src/projection.rs` | ⬜ pending |
| SUBST-04 | Inertia `POST /{service}/{action}` reaches the same `dispatch_write` kernel as MCP; audit differs only by channel tag (`web.` vs `mcp.`) | integration | `cargo test -p app single_source_inertia` | `app/src/tests/single_source.rs` (extend) | ⬜ pending |
| SUBST-05 | Changing an action-precondition guard state changes both the Inertia `permitted_actions` set and the MCP `tools/list` set identically | integration | `cargo test -p app permitted_actions_parity` | `app/src/tests/permitted_actions_parity.rs` (Wave 0 gap) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-projections/tests/schema_contract.rs` — SUBST-01 snapshot fixture
- [ ] `app/src/tests/permitted_actions_parity.rs` — SUBST-02 / SUBST-05 parity test (mirrors the
  existing `single_source_both_channels` pattern)
- [ ] `app/src/tests/data_tenant_scoping.rs` — SUBST-03 tenant isolation (verify it is not already
  partially covered by `app/src/tests/crud_e2e.rs` before creating)

*Existing infrastructure (`cargo test --all-features`) covers execution; only the fixtures above
are new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | — |

*All phase behaviors have automated verification (schema snapshot, tenant-scoping, permitted-actions
parity vs MCP `tools/list`, write-parity vs `dispatch_write`).*

---

## Validation Sign-Off

- [ ] All tasks have an `<automated>` verify command or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all Wave-0-gap references above
- [ ] No watch-mode flags
- [ ] Feedback latency acceptable (per-crate quick run on task commits)
- [ ] `nyquist_compliant: true` set in frontmatter (after planner maps task IDs)

**Approval:** pending
