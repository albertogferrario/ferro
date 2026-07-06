---
phase: 155
slug: ferro-projection-crate-live-read-model-from-domain-events-wi
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 155 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution of `ferro-projection` (Wave 1b crate).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 2021 edition, workspace test harness) |
| **Config file** | `Cargo.toml` (workspace + `ferro-projection/Cargo.toml` after Plan 02) |
| **Quick run command** | `cargo test -p ferro-projection --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~120 seconds (full); ~10 seconds (quick on single crate) |

The full suite command is **the exact CLI command CI executes** — non-negotiable per `feedback_ci_clippy_command_match.md`. Plans MUST verify with `--all --all-targets -- -D warnings` (not just `--all` or `-- -D warnings`).

---

## Sampling Rate

- **After every task commit:** Run `cargo build -p ferro-projection` (compile-only — catches type errors immediately).
- **After every plan completes:** Run `cargo test -p ferro-projection` (unit + integration tests for the crate).
- **Before `/gsd-verify-work`:** Full suite command must be green across the entire workspace (-D warnings catches drift from other crates that import ferro-projection during plan 07).
- **Max feedback latency:** 120 seconds (full suite). Quick path is < 15 seconds.

---

## Per-Task Verification Map

Each plan's tasks land here as they're authored. The matrix below is the planner-supplied skeleton — every D-decision in CONTEXT.md maps to at least one row.

| Task ID | Plan | Wave | Decision Ref | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|--------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 155-01-* | 01 | 1 | D-01..D-05, D-28..D-30 | — | crate scaffolded with thin Wave 1b shape; no panics in error paths | unit (compile) | `cargo build -p ferro-projection` | ✅ W0 | ⬜ pending |
| 155-02-* | 02 | 2 | D-04, D-05, D-54, D-55 | — | workspace member registered; publish.yml WAVE1B_CRATES list updated; CLAUDE.md table row with disambiguation | manual + grep | `grep -q '"ferro-projection"' Cargo.toml && grep -q 'ferro-projection' .github/workflows/publish.yml && grep -q '\| `ferro-projection` \|' CLAUDE.md` | ❌ W0 → ✅ after 02 | ⬜ pending |
| 155-03-* | 03 | 3 | D-23..D-27 | — | migration creates table + composite PK; entity round-trips through SeaORM JSON column | unit (sqlite) | `cargo test -p ferro-projection --lib entity::tests` | ❌ W0 → ✅ after 03 | ⬜ pending |
| 155-04-* | 04 | 4 | D-06..D-12, D-28..D-30 | — | `Projection` trait compiles for a minimal demo impl; `ProjectionKey::new`/`as_str`/`Display`/`Serde` round-trip; trait method defaults (`snapshot_interval=100`, `broadcast_event_name="delta"`) | unit | `cargo test -p ferro-projection --lib projection::tests key::tests` | ❌ W0 → ✅ after 04 | ⬜ pending |
| 155-05-* | 05 | 5 | D-13..D-22, D-31..D-37, D-45 | — | `apply_event` per-key serialization; snapshot persisted with `version` increment; broadcast frame on `projection.{name}.{key}`; `read` returns the persisted state; `register` wires listener into `global_dispatcher` | unit | `cargo test -p ferro-projection --lib runtime::tests listener::tests` | ❌ W0 → ✅ after 05 | ⬜ pending |
| 155-06-* | 06 | 6 | D-17, D-38..D-44, D-46..D-50 | — | `rebuild` discards + folds + broadcasts `"rebuild"` frame; cross-crate showcase (`ferro-reservation` events → projection state) green; concurrent_apply test passes (20 tasks × 5 keys → exactly-4-per-key); 3 proptest properties green | integration + proptest | `cargo test -p ferro-projection --test event_bus_integration --test projection_over_reservation_events --test concurrent_apply --test proptest_properties` | ❌ W0 → ✅ after 06 | ⬜ pending |
| 155-07-* | 07 | 7 | D-51..D-56 | — | docs page lands; CHANGELOG entry placed at top; workspace pre-release gate green; manual first-publish bootstrap captured as human-action checkpoint | manual + gate | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features && test -f docs/src/features/live-read-models.md` | ❌ W0 → ✅ after 07 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Coverage matrix completeness:** Every D-decision from D-01 through D-56 in `155-CONTEXT.md` maps to at least one task above. The breakout matrix detailing decision-to-test-name traceability is enumerated in `155-RESEARCH.md §Validation Architecture`.

---

## Wave 0 Requirements

- [ ] `ferro-projection/Cargo.toml` — declares package metadata + `[dependencies]` block (Plan 01)
- [ ] `ferro-projection/src/lib.rs` — module-level rustdoc with disambiguation lead paragraph + `pub use` re-exports of stub modules (Plan 01)
- [ ] `ferro-projection/src/error.rs` — full `ProjectionError` enum body with five variants (Plan 01)
- [ ] `ferro-projection/src/projection.rs`, `key.rs`, `entity.rs`, `migration.rs`, `runtime.rs`, `listener.rs` — stub modules with `// TODO: Plan-NN` markers (Plan 01)
- [ ] `Cargo.toml` (workspace root) — `[workspace.members]` includes `"ferro-projection"`; `[workspace.package] version = "0.2.33"` (Plan 02)
- [ ] `.github/workflows/publish.yml` — `WAVE1B_CRATES` includes `ferro-projection` (Plan 02)
- [ ] `CLAUDE.md` — `### Workspace Structure` table has the new `ferro-projection` row with disambiguation framing (Plan 02)
- [ ] `README.md` (workspace root) — workspace crates table row with disambiguation (Plan 02)

*Wave 0 is the minimum surface required to make plans 03-07 testable. Every plan after Plan 02 builds on this scaffold.*

---

## Manual-Only Verifications

| Behavior | Decision Ref | Why Manual | Test Instructions |
|----------|--------------|------------|-------------------|
| First-publish bootstrap of `ferro-projection 0.2.33` to crates.io | D-55, D-56 | CI publish token has publish-update scope only; new crate needs personal `publish-new`-scoped token from a local terminal | (1) `cargo login <personal-token>` (local terminal). (2) `cargo publish -p ferro-projection`. (3) After success, push commit to master; subsequent versions auto-publish via Wave 1b in `publish.yml`. Mirror Phase 154 PLAN-07 checkpoint. |
| Disambiguation copy is consistent across rustdoc + doc page + README + CLAUDE.md + CHANGELOG | D-02, D-51, D-52, D-56 | Subjective phrasing variants pass grep equality but fail prose-quality review | Read each artifact end-to-end. Confirm the "Not to be confused with `ferro-projections` (plural)" phrasing appears verbatim or near-verbatim in: `ferro-projection/src/lib.rs` opening paragraph, `docs/src/features/live-read-models.md` lede, `README.md` workspace crates table row, `CLAUDE.md` workspace structure table row, `CHANGELOG.md` ferro-projection section opening line. |
| Cross-crate showcase test reads coherently as a v11.11 milestone-completion demonstration | D-47 | Test code can pass while reading as gibberish; the test's didactic value is part of the deliverable | After Plan 06, read `tests/projection_over_reservation_events.rs` as a tutorial. Confirm: (1) the test name communicates the milestone story, (2) inline comments explain why each step composes the four primitives, (3) the assertions verify both the projection state AND the broadcast frames, (4) a maintainer reading this test understands the four-primitive composition in under five minutes. |

---

## Validation Sign-Off

- [ ] All tasks have automated `cargo` verify commands OR are in the Manual-Only matrix above
- [ ] Sampling continuity: every plan ends with `cargo test -p ferro-projection` passing
- [ ] Wave 0 covers all MISSING references (scaffold-then-fill arc; no plan after 02 has unresolved Wave-0 deps)
- [ ] No watch-mode flags — every test is one-shot
- [ ] Feedback latency: < 15s on quick path; < 120s on full suite
- [ ] `nyquist_compliant: true` set in frontmatter AFTER all 7 plans pass their verification

**Approval:** pending (will flip to approved YYYY-MM-DD when Plan 07's release gate passes)

---

## Decision-to-Test Coverage Reference

For the full D-XX → test-name traceability matrix, see `155-RESEARCH.md §Validation Architecture (Nyquist Dimension 8) — Coverage Matrix`. That matrix is the authoritative source; this VALIDATION.md is the planner-facing summary that downstream agents consult when wiring `<automated>` verify steps into each task.
