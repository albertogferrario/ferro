---
phase: 195
slug: close-the-loop-by-default
status: draft
nyquist_compliant: false
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
| TBD | TBD | TBD | CHK-07 | — | `generate_projection` response carries a `checkpoint` summary | unit | `cargo test -p ferro-mcp generate_projection` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-07 | — | `json_ui_generate` embeds `checkpoint: None` when no model anchor | unit | `cargo test -p ferro-mcp json_ui_generate` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-07 | — | `VerdictSummary` serializes without raw `seams` array (SC-1) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::verdict_summary_shape` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-08 | — | `read_ambient_status` → `"unverified"` for missing file | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_missing_unverified` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-08 | — | `read_ambient_status` → `clean`/`failing` from cache | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_read_clean` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-08 | — | `projection_coverage` includes `checkpoint_status` (SC-2) | unit | `cargo test -p ferro-mcp projection_coverage` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-08 | — | `application_info` includes `projection_checkpoint` summary (SC-3) | unit | `cargo test -p ferro-mcp application_info` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-09 | — | seam 1 `source == "validate_projection"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam1_source_provenance` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-09 | — | seam 3 `source == "json_ui_verify_action"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam3_source_provenance` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-09 | — | seam 4 correct `source` per render/spec stage | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam4_source_provenance` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-09 | — | seam 5 `source == "validate_contracts"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam5_source_provenance` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-09 | — | SC-4: `source == "checkpoint"` only on `field_to_column` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::sc4_no_checkpoint_source_on_wrapper_seams` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-01 | — | seam names canonical (no Phase-194 names) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam_names_canonical` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-06 | — | cascade: seam 1 fail → seams 4,5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam1_fail` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-06 | — | cascade: seam 4 fail → seam 5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam4_fail` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-06 | — | seams 2,3 independent of seam 1 failure | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seams_2_3_independent` | ❌ W0 | ⬜ pending |

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

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `generate_projection`/`json_ui_generate` surface checkpoint over live MCP | CHK-07 | End-to-end async MCP wiring through a live `ferro mcp` session | Rebuild debug, restart Claude Code, call the generators and confirm a `checkpoint` summary appears |
| `application_info`/`projection_coverage` show ambient status over live MCP | CHK-08 | Cache-state-dependent introspection through live session | After a `checkpoint_projection` run, call both tools and confirm status reflects the cache |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
