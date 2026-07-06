---
phase: 192
slug: ferro-mcp-template-validation-docs
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 192 — Validation Strategy

> Per-phase validation contract. This is a docs/template phase — verification is
> grep-based assertions on the template source and the docs page, plus the
> workspace compile/lint gate (the template code must reference real public API).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | grep assertions + `cargo build`/`cargo doc` (no new test framework) |
| **Config file** | none |
| **Quick run command** | grep checks below + `cargo build -p ferro-mcp` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | seconds (grep) + a ferro-mcp build |

---

## Per-Requirement Verification Map

| Req / SC | Behavior | Verify (grep / build) | Status |
|----------|----------|------------------------|--------|
| VALID-06 / SC1 | ferro-mcp `action_handler` template shows BOTH layers; no template shows `unique` without a downstream `ConstraintMap` | `grep -q 'AsyncValidator' ferro-mcp/src/tools/code_templates.rs` AND `grep -q 'ConstraintMap' ferro-mcp/src/tools/code_templates.rs`; AND audit: every template `code` block referencing `unique(` also references `ConstraintMap`/`map_constraint` (no `unique`-only template) | ⬜ |
| VALID-06 / SC1b | template imports/placeholders updated; compiles | `cargo build -p ferro-mcp` exits 0; template `imports` reference `AsyncValidator`/`ConstraintMap` | ⬜ |
| VALID-06 / SC2 | docs has a dedicated async-rules section (`unique` with + without `.ignore`) AND a dedicated constraint-mapping section (`ConstraintMap` + two-layer rationale) | `grep -qiE 'async rule|unique' docs/src/features/validation.md` for the async section AND `grep -q 'ConstraintMap' docs/src/features/validation.md` for the mapping section; both `.ignore(` and `.sqlite(`/`try_map`/`map_constraint` present | ⬜ |
| VALID-06 / SC3 | the two docs sections are cross-referenced (each links to the other) | grep both new section anchors appear as in-page links (`](#...)`) from the other section | ⬜ |
| accuracy | template + docs example code matches the real public API (190/191) | names exist in `framework/src/lib.rs` re-exports: `AsyncValidator`, `unique`, `ConstraintMap`, `MapConstraintExt` | ⬜ |

*Status: ⬜ pending · ✅ green*

---

## Wave 0 Requirements

- None — edits land in existing files (`ferro-mcp/src/tools/code_templates.rs`, `docs/src/features/validation.md`). No new test harness.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All SCs are grep/build-verifiable; no manual gate. |

---

## Validation Sign-Off

- [ ] SC1/SC1b/SC2/SC3 grep + build checks all green
- [ ] Template code references real, current public API (compiles in a sample handler shape)
- [ ] `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings` clean (ferro-mcp template is a string, but the file must lint)
- [ ] `nyquist_compliant: true` set after checks pass

**Approval:** pending — set when plans pass the checker
