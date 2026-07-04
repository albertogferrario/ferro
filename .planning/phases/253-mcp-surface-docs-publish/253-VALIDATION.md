---
phase: 253
slug: mcp-surface-docs-publish
status: planned
nyquist_compliant: true
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
  `cargo test --all-features`, `cargo doc --no-deps -D warnings` clean
- **Max feedback latency:** ~120 seconds (targeted run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 253-01-T1 | 01 | 1 | DS-07 design_lint tool (inline + path + parse-error + XOR) | T-253-01/02/03 | path read-only; parse/read/XOR errors are Warning findings, not tool errors | unit | `cargo test -p ferro-mcp design_lint` | ❌ Wave 1 | pending |
| 253-01-T2 | 01 | 1 | DS-07 tool registration + stale count fix | — | tool description is framework surface; "39"→"47" corrected | build | `cargo build -p ferro-mcp` | ❌ Wave 1 | pending |
| 253-02-T1 | 02 | 1 | DS-07 catalog design_system vocabulary | T-253-04 | variant/tone/size derived from canonical enums (no duplicate); 47-count mirror intact | unit | `cargo test -p ferro-mcp json_ui_catalog` | ❌ Wave 1 | pending |
| 253-02-T2 | 02 | 1 | DS-07 generation_context summary + token drift guard | T-253-04 | token count guarded against ALL_TOKENS; sections test extended | unit | `cargo test -p ferro-mcp generation_context` | ❌ Wave 1 | pending |
| 253-03-T1 | 03 | 1 | DS-08 docs chapter (5 pages + SUMMARY.md) | T-253-07 | neutral voice; cross-link not duplicate | file existence | `ls docs/src/design-system/` | ❌ Wave 1 | pending |
| 253-03-T2 | 03 | 1 | DS-08 patterns.md ↔ registry drift test | T-253-06 | both-direction drift guard | unit | `cargo test -p ferro-json-ui patterns_md_matches_rule_registry` | ❌ Wave 1 | pending |
| 253-04-T1 | 04 | 1 | DS-08 IN-01 dead FIELD_TYPES entry | T-253-08 | RichTextEditor retained; no rule regression | unit | `cargo test -p ferro-json-ui design` | ✅ (edit) | pending |
| 253-04-T2 | 04 | 1 | DS-08 IN-02 zero-files vs all-clean message | T-253-09 | zero-files case made explicit | unit | `cargo test -p ferro-cli design_lint` | ✅ (edit) | pending |
| 253-05-T1 | 05 | 2 | DS-08 CI-exact gate + version bump | T-253-10/11 | full gate green before push; version verified via crates.io API | gate | `cargo fmt --all -- --check` + clippy + test + doc (all --all-features) | ✅ (edit) | pending |
| 253-05-T2 | 05 | 2 | DS-08 pre-publish UAT + operator approval | T-253-10 | operator-gated; no irreversible action pre-approval | checkpoint | (human-verify) | — | pending |
| 253-05-T3 | 05 | 2 | DS-08 publish + verify + gestiscilo brief | T-253-11/12/13 | verify via crates.io/gh API; consumer handoff brief only | smoke | `curl -s https://crates.io/api/v1/crates/ferro-rs \| jq -r .crate.max_version` | — | pending |
