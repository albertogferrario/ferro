---
phase: 195
slug: close-the-loop-by-default
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-10
---

# Phase 195 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + `tokio` for async tests (`run_for`/`execute` become async) |
| **Config file** | none — inline `#[cfg(test)]` modules |
| **Quick run command** | `cargo test -p ferro-mcp checkpoint_projection` |
| **Full suite command** | `cargo test -p ferro-mcp` (scoped per project disk/thermal policy — not `--all-features`) |
| **Estimated runtime** | quick ~5s; ferro-mcp suite tens of seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp checkpoint_projection`
- **After every plan wave:** `cargo test -p ferro-mcp`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy -p ferro-mcp --all-targets -- -D warnings && cargo test -p ferro-mcp`
- **Max feedback latency:** ~5 seconds (scoped quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| P01-T1 | 195-01 | 1 | D-01 | — | seam names canonical (no Phase-194 names); grep gate clean | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam_names_canonical` | ❌ W0 | ⬜ pending |
| P01-T2 | 195-01 | 1 | CHK-07 | T-195-02 | `VerdictSummary` serializes without raw `seams` array (SC-1) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::verdict_summary_shape` | ❌ W0 | ⬜ pending |
| P01-T2 | 195-01 | 1 | CHK-08 | T-195-01 | `read_ambient_status` → `"unverified"` for missing file | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_missing_unverified` | ❌ W0 | ⬜ pending |
| P01-T2 | 195-01 | 1 | CHK-08 | T-195-01 | `read_ambient_status` → `clean`/`failing` from cache | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_read_clean` | ❌ W0 | ⬜ pending |
| P01-T3 | 195-01 | 1 | CHK-07 | — | checkpoint MCP handler awaits async execute (crate compiles) | unit | `cargo test -p ferro-mcp checkpoint_projection` | ❌ W0 | ⬜ pending |
| P02-T1 | 195-02 | 2 | CHK-09 | T-195-05 | seam 1 `source == "validate_projection"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam1_source_provenance` | ❌ W0 | ⬜ pending |
| P02-T1 | 195-02 | 2 | CHK-09 | T-195-05 | seam 3 `source == "json_ui_verify_action"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam3_source_provenance` | ❌ W0 | ⬜ pending |
| P02-T1 | 195-02 | 2 | D-06 | — | seams 2,3 independent of seam 1 failure | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seams_2_3_independent` | ❌ W0 | ⬜ pending |
| P02-T2 | 195-02 | 2 | CHK-09 | T-195-04 | seam 4 correct `source` per render/spec stage | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam4_source_provenance` | ❌ W0 | ⬜ pending |
| P02-T2 | 195-02 | 2 | CHK-09 | T-195-04 | seam 5 `source == "validate_contracts"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam5_source_provenance` | ❌ W0 | ⬜ pending |
| P02-T3 | 195-02 | 2 | CHK-09 | T-195-05 | SC-4: `source == "checkpoint"` only on `field_to_column` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::sc4_no_checkpoint_source_on_wrapper_seams` | ❌ W0 | ⬜ pending |
| P02-T3 | 195-02 | 2 | D-06 | — | cascade: seam 1 fail → seams 4,5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam1_fail` | ❌ W0 | ⬜ pending |
| P02-T3 | 195-02 | 2 | D-06 | — | cascade: seam 4 fail → seam 5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam4_fail` | ❌ W0 | ⬜ pending |
| P03-T1 | 195-03 | 3 | CHK-07 | T-195-07 | `generate_projection` response carries a `checkpoint` summary (omitted when None) | unit | `cargo test -p ferro-mcp generate_projection` | ❌ W0 | ⬜ pending |
| P03-T2 | 195-03 | 3 | CHK-07 | T-195-07 | `json_ui_generate` embeds `checkpoint: None` when no model anchor | unit | `cargo test -p ferro-mcp json_ui_generate` | ❌ W0 | ⬜ pending |
| P03-T3 | 195-03 | 3 | CHK-07 | T-195-09 | both generator handlers await async execute (crate compiles) | unit | `cargo test -p ferro-mcp` | ❌ W0 | ⬜ pending |
| P04-T1 | 195-04 | 4 | CHK-08 | T-195-10 | `projection_coverage` includes `checkpoint_status` (SC-2); cache-only | unit | `cargo test -p ferro-mcp projection_coverage` | ❌ W0 | ⬜ pending |
| P04-T2 | 195-04 | 4 | CHK-08 | T-195-11 | `application_info` includes `projection_checkpoint` rollup (SC-3); cache-only | unit | `cargo test -p ferro-mcp application_info` | ❌ W0 | ⬜ pending |
| P04-T3 | 195-04 | 4 | CHK-08 | — | ambient tool descriptions document new fields (crate compiles) | unit | `cargo test -p ferro-mcp` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Async test harness: `#[tokio::test]` for tests exercising the now-async `run_for`/`execute`
- [ ] New `#[cfg(test)]` cases enumerated in the Per-Task map above
- [ ] Inline-hook tests in `generate_projection.rs` and `json_ui_generate.rs`
- [ ] `projection_coverage` / `application_info` ambient-field tests
- [ ] Existing Phase 194 tests remain unchanged and continue to pass (regression — including the seam-name updates from D-01)

*No new test infrastructure files — all tests inline `#[cfg(test)]`.*

---

## Sampling Continuity Check

Every plan task carries an `<automated>` verify. No 3-consecutive-task gap without an automated test:
- Plan 01: T1 (seam_names_canonical), T2 (verdict_summary_shape + ambient), T3 (suite compile) — continuous.
- Plan 02: T1 (seam1/3 provenance), T2 (seam4/5 provenance), T3 (SC-4 + cascade) — continuous.
- Plan 03: T1 (generate_projection), T2 (json_ui_generate), T3 (full suite) — continuous.
- Plan 04: T1 (projection_coverage), T2 (application_info), T3 (full suite) — continuous.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `generate_projection`/`json_ui_generate` surface checkpoint over live MCP | CHK-07 | End-to-end async MCP wiring through a live `ferro mcp` session | Rebuild debug, restart Claude Code, call the generators and confirm a `checkpoint` summary appears |
| `application_info`/`projection_coverage` show ambient status over live MCP | CHK-08 | Cache-state-dependent introspection through live session | After a `checkpoint_projection` run, call both tools and confirm status reflects the cache |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned (tests are Wave 0 — created during execution)
