---
phase: 160
slug: remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-17
---

# Phase 160 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Phase 160 is a deletion/cleanup phase; validation is dominated by **grep gates** and **cargo invocations**, not new unit tests.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace; existing harness — no new framework introduced) |
| **Config file** | `Cargo.toml` workspace + per-crate `Cargo.toml` (none added by this phase) |
| **Quick run command** | `cargo test -p ferro-mcp --lib` (targeted at the only crate gaining code changes vs deletions) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~30s; full suite ~5-10min (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo build -p {crate_touched}` to confirm the touched crate still compiles.
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Before `/gsd-verify-work`:** Full suite must be green, plus the D-10 grep gate must report zero matches.
- **Max feedback latency:** quick check < 60s; full wave gate < 10min.

---

## Per-Task Verification Map

> Filled by the planner during PLAN.md authoring. Skeleton rows below.

| Task ID | Plan | Wave | Decision | Secure Behavior | Test Type | Automated Command | Status |
|---------|------|------|----------|-----------------|-----------|-------------------|--------|
| 160-XX-YY | XX | 1 | D-02 / D-03 | doc-comment rewrites do not change emitted HTML | grep + cargo test | `grep -rnE '\b(v1\|legacy\|Port of)\b' ferro-json-ui/src \| wc -l` (expect 0 outside whitelist) + `cargo test -p ferro-json-ui` | ⬜ pending |
| 160-XX-YY | XX | 1 | D-04 | migration_v1_to_v2_templates fn + registration + test removed; remaining template surface unchanged | grep + cargo test | `! grep -n 'migration_v1_to_v2_templates' ferro-mcp/src/tools/code_templates.rs` + `cargo test -p ferro-mcp --lib code_templates` | ⬜ pending |
| 160-XX-YY | XX | 1 | D-05 | application_info::scan_json_ui_specs counts v2 JSON specs; field names preserved or migrated per Phase 115 SUMMARY allowance | unit | `cargo test -p ferro-mcp --lib application_info` | ⬜ pending |
| 160-XX-YY | XX | 1 | D-06 | json_ui_inspect test either renamed (fixture neutral name) or deleted; no new failures | unit | `cargo test -p ferro-mcp --lib json_ui_inspect` | ⬜ pending |
| 160-XX-YY | XX | 1 | Pattern 6 | ferro-json-ui/README.md example compiles against current crate API | mdbook | `cargo doc --no-deps -p ferro-json-ui 2>&1 \| grep -i 'error\|fail' \| head` (expect none) | ⬜ pending |
| 160-XX-YY | XX | 1 | Pattern 7 | docs/src/reference/cli.md make:json-view example matches current ferro-cli output shape | manual diff | regenerate sample via `ferro make:json-view sample` and diff against doc snippet | ⬜ pending |
| 160-XX-YY | XX | 2 | D-07 | docs/protocol + docs/src/features rewrites build via mdbook and contain no `\bv1\b` narrative | grep + mdbook | `cd docs && mdbook build` + `! grep -nE '\bv1\b\|legacy' src/features/projections.md src/json-ui/*.md` | ⬜ pending |
| 160-XX-YY | XX | 2 | D-08 | full sweep audit produced; every remaining match falls in a whitelisted category (API-version examples, planning files) | grep | `node scripts/d08-sweep.sh` (or inline grep with whitelist) — exit 0 | ⬜ pending |
| 160-XX-YY | XX | 3 | D-10 | final grep gate: zero matches outside `.planning/` | grep | `! grep -rnE '\b(JsonUiView\|ComponentNode\|PluginProps)\b' ferro-json-ui/src framework/src ferro-mcp/src` + `! grep -rn 'ferro-json-ui/v1' ferro-json-ui/src framework/src ferro-mcp/src docs/src docs/protocol/src` | ⬜ pending |
| 160-XX-YY | XX | 3 | D-09 (ferro) | workspace fmt + clippy + tests all green on v12.0/json-ui-v2 | cargo | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ⬜ pending |
| 160-XX-YY | XX | 3 | D-09 (gestiscilo) | gestiscilo suite green against local-path ferro | cargo | `cd /Users/alberto/repositories/gestiscilo-it/app && cargo test --all-features` (or the suite command the gestiscilo STATE.md prescribes) | ⬜ pending |
| 160-XX-YY | XX | 3 | D-09 (ferro-code) | DESCOPED — ferro-code repo is empty per research Open Q2 | n/a | none — record in SUMMARY | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

None. Phase 160 introduces **no new code paths** that would need stub coverage; it deletes and rewrites existing surface. Existing `cargo test --all-features` covers the regression surface.

- [x] Existing infrastructure covers all phase verification needs.

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Doc-comment rewrites read as neutral prose (no historical narrative) | D-02 / D-03 | Style/tone judgment beyond what grep can catch (a comment could be free of `v1` keyword while still being a port narrative — e.g., "Differences from previous version") | Read each rewritten doc-comment block; confirm it describes function purpose + props + output, no historical/comparative framing |
| Public-doc rewrites read as neutral prose | D-07 | Same — `mdbook build` succeeding does not prove the prose is in the right voice | Read `docs/protocol/src/{terminology,architecture,rendering}.md` and `docs/src/features/projections.md` post-rewrite; apply the CLAUDE.md "trigger phrases for review" checklist |
| gestiscilo end-to-end smoke (post-cargo-test browser sanity) | D-09 | A green `cargo test` does not exercise the live JSON-UI render path | Start gestiscilo server, hit 2-3 key dashboard routes, confirm rendering matches Phase 159 verification expectations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references — N/A, no Wave 0 needed
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s for quick checks
- [ ] `nyquist_compliant: true` set in frontmatter once planner fills the table above with concrete task IDs

**Approval:** pending
