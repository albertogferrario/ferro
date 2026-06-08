---
phase: 170
slug: ferro-cli-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 170 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — workspace test runner |
| **Quick run command** | `cargo test -p ferro-cli --lib` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~10s (ferro-cli lib only); full suite ~minutes (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli --lib`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 170-01-* | 01 | 1 | AISDK-06 | — / — | `ai.rs` deleted; no `reqwest::blocking::Client` and no direct Anthropic API call in the AI path (compile-time fact) | compilation | `cargo build -p ferro-cli` | ✅ verified by build | ⬜ pending |
| 170-01-* | 01 | 1 | AISDK-06 | — / — | `make_json_view` pure-function tests still pass after helper relocation | unit | `cargo test -p ferro-cli --lib -- make_json_view` | ✅ exists | ⬜ pending |
| 170-01-* | 01 | 1 | AISDK-06 | — / — | static fallback produces catalog-valid spec when no provider configured | unit | `cargo test -p ferro-cli --lib -- static_fallback_produces_valid_spec` | ✅ exists | ⬜ pending |
| 170-01-* | 01 | 1 | AISDK-06 | — / — | no new compilation warnings in ferro-cli (SC#5) | lint | `cargo clippy -p ferro-cli --all-targets -- -D warnings` | N/A — lint run | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

None — existing test infrastructure covers all phase requirements. No new test files are needed for AISDK-06; the SDK swap is verified by compilation (no blocking client remains), the surviving `make_json_view` unit tests, and the existing `static_fallback_produces_valid_spec` test.

- The optional regression test asserting the static-fallback path under `AiConfig::from_env()` error is **discretionary** (CONTEXT.md D-05/Discretion) — not required for AISDK-06.

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `ferro make:json-view <name> --description "..."` produces a catalog-valid `src/views/<name>.json` end-to-end against a live provider | AISDK-06 / SC#3 | Requires a real LLM provider API key + network; not run in CI | In a scratch ferro app: `export FERRO_AI_PROVIDER=anthropic FERRO_AI_API_KEY=sk-ant-...`; run `ferro make:json-view dashboard --description "ops dashboard"`; confirm the written JSON parses via `Spec::from_json` and validates against the catalog. Repeat with `FERRO_AI_PROVIDER=openai` to prove provider-agnosticism (SC#4). |
| Static-template fallback when no provider configured | AISDK-06 / SC#3 | Behavioral (stderr UX) — covered structurally by the `Err` branch + existing static-fallback unit test | Unset all `FERRO_AI_*` and `ANTHROPIC_API_KEY`; run `ferro make:json-view foo --description "x"`; confirm the info/warning message prints and a valid static template is written. |

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none — covered)
- [ ] No watch-mode flags
- [ ] Feedback latency < ~10s (quick run)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
