---
phase: 115
slug: spec-v2-data-structures
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-18
---

# Phase 115 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` (unit + integration tests) |
| **Config file** | `ferro-json-ui/Cargo.toml` (existing) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings && cargo fmt --all -- --check` |
| **Estimated runtime** | ~30 seconds (quick), ~90 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib`
- **After every plan wave:** Run `cargo test --all-features` + clippy
- **Before `/gsd-verify-work`:** Full suite must be green — fmt + clippy + test all features
- **Max feedback latency:** 30 seconds for unit tests; 90 seconds for full suite

---

## Per-Task Verification Map

Planner fills this table with one row per task once plans exist. Each task that creates or mutates a source file has an automated verify command. Wave 0 columns are populated when plan 01 writes the `tests/fixtures/` directory.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 115-01-XX | 01 | 1 | SPEC-01/02/03 | — | Spec and Element types round-trip cleanly; structural invariants hold | unit | `cargo test -p ferro-json-ui --lib spec::` | ❌ W0 | ⬜ pending |
| 115-0X-XX | 0X | X | SPEC-04 | — | v1 types removed; workspace compiles against Spec only | unit + build | `cargo build --all-targets && ! grep -r "JsonUiView" ferro-json-ui/src/` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/tests/fixtures/ok/` — 7 valid spec fixtures (minimal, three-level nested, actions, visibility, plugin-named type, builder-parity, data-field)
- [ ] `ferro-json-ui/tests/fixtures/reject/` — 11 invalid spec fixtures (missing root, dangling child, A→B→A cycle, A→A self-cycle, 4-level nesting, invalid ID space, invalid ID empty, invalid ID digit-start, invalid ID too-long, duplicate IDs in raw JSON, bad schema version)
- [ ] `ferro-json-ui/tests/round_trip.rs` — integration test that walks fixtures/ok/ and asserts parse→serialize→reparse equality
- [ ] `ferro-json-ui/tests/reject.rs` — integration test that walks fixtures/reject/ and asserts the expected `SpecError` variant

No new testing framework needed — `cargo test` is the workspace standard.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Placeholder renderer produces valid HTML | D-17 | Visual sanity check; full render pipeline arrives in Phase 116 | Run the sample `app` crate, open the rendered placeholder page, confirm `<pre><code>` block shows Spec JSON and the `<!-- ferro-json-ui v2 render pipeline arrives in Phase 116 -->` comment is present in DOM |

All other phase behaviors (type shape, structural validation, builder, JsonSchema generation, v1-deletion) have automated verification via `cargo test` and grep-based acceptance criteria.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (fixtures/ok + fixtures/reject)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (quick) / 90s (full)
- [ ] `nyquist_compliant: true` set in frontmatter after plans are reviewed by plan-checker

**Approval:** pending — planner fills the Per-Task Verification Map, then checker signs off.
