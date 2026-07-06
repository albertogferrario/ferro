---
phase: 194
slug: core-checkpoint-tool
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 194 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`, inline `#[cfg(test)]` modules) |
| **Config file** | none — inline modules within `checkpoint_projection.rs` |
| **Quick run command** | `cargo test -p ferro-mcp checkpoint_projection` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~5s; full suite multi-minute (disk-permitting — see disk-full gate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp checkpoint_projection`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds (scoped quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | CHK-01 | — | Verdict shape (`status`/`seams`/`next_steps` + `source` provenance) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::verdict_shape` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-02 | — | Dangling field detected | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam2_dangling_field` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-02 | — | Clean projection passes seam 2 | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam2_all_pass` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-03 | — | `not_checked` when source model unresolved (never `pass`) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::not_checked_no_model` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-03 | — | `not_checked` when reconstruction fails | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::not_checked_bad_source` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-04 | — | Relationship fields never flagged | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::relationships_not_flagged` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-05 | — | Reconstruction-incomplete → `warn` not `pass` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::reconstruction_incomplete_warn` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CHK-06 | — | Mixed findings ranked + deduped | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::next_steps_ranked_deduped` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-11 | path-traversal | Cache written to `.ferro/checkpoints/{name}.json`; `name` rejects `/`, `..`, null | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cache_write` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/src/tools/checkpoint_projection.rs` — the module itself (all tests inline)
- [ ] Inline `&str` fixture constants:
  - projection source with a dangling field (field not in model)
  - projection source with a relationship + clean fields (CHK-04)
  - projection source with more builder calls than parseable fields (CHK-05)
  - fully coherent minimal projection (all fields match model)
  - "no model" scenario (service_name with no matching model)

*No new test infrastructure files needed — all tests are inline `#[cfg(test)]`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tool callable over MCP and returns the verdict | CHK-01 | End-to-end MCP wiring exercised through a live `ferro mcp` session | Build debug, restart Claude Code, call `checkpoint_projection { name }` against a real projection |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
