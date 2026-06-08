---
phase: 172
slug: mcp-tool-wrappers
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 172 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: 172-RESEARCH.md §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (tokio "1", features `full` in `ferro-mcp/Cargo.toml`) |
| **Config file** | Cargo workspace — no separate test config |
| **Quick run command** | `cargo test -p ferro-mcp --all-features` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–180 seconds (workspace test; subject to disk space — see `project_ferro_disk_full_test_gate`) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp --all-features && cargo test -p ferro-cli --all-features`
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full gate must be green (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`)
- **Max feedback latency:** ~180 seconds

---

## Per-Task Verification Map

> Planner fills exact Task IDs; rows below are the requirement→behavior contract the plans must cover.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | 1 | AICLI-05 | T-172-PI | `sanitize_description` strips XML delimiters (relocated; prompt-injection guard preserved) | unit | `cargo test -p ferro-mcp --all-features sanitize` | ❌ W0 | ⬜ pending |
| TBD | TBD | 1 | AICLI-05 | — | Relevance filter `tokenize`/`select_relevant` (relocated) behaves identically | unit | `cargo test -p ferro-mcp --all-features relevance` | ❌ W0 | ⬜ pending |
| TBD | TBD | 2 | AICLI-05 | — | `ai_scaffold` returns valid `ServiceDef` JSON (mock LLM); no disk write | unit | `cargo test -p ferro-mcp --all-features ai_scaffold` | ❌ W0 | ⬜ pending |
| TBD | TBD | 2 | AICLI-05 | — | `ai_explain` returns structured projection JSON when ServiceDef found (zero-token) | unit | `cargo test -p ferro-mcp --all-features ai_explain_structured` | ❌ W0 | ⬜ pending |
| TBD | TBD | 2 | AICLI-05 | — | `ai_explain` returns `{ "prose": ... }` fallback when no ServiceDef | unit | `cargo test -p ferro-mcp --all-features ai_explain_prose_fallback` | ❌ W0 | ⬜ pending |
| TBD | TBD | 3 | AICLI-05 | — | CLI `ai:make` / `ai:explain` thin wrappers still pass existing tests through relocated core | unit | `cargo test -p ferro-cli --all-features` | ✅ existing | ⬜ pending |
| TBD | TBD | 3 | AICLI-05 | — | Full gate green | integration | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/src/tools/relevance.rs` — relocated relevance tests (`tokenize`, `select_relevant`, `INPUT_BUDGET_CHARS`)
- [ ] `ferro-mcp` core unit tests for `sanitize_description`, `resolve_max_tokens`, candidate assembly (mock/empty introspection)
- [ ] `ferro-mcp` core unit tests for `build_*_prompt`, target-kind resolution, structured-vs-prose branch of `ai_explain`
- [ ] Migrate existing CLI tests (`sanitize_description`, `resolve_kind_priority`, `build_service_prompt`, `max_tokens_*`) to `ferro-mcp` alongside the relocated implementation

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live `ai_scaffold` / `ai_explain` quality against a real LLM provider | AICLI-05 | Requires `FERRO_AI_*` env + network + token spend; non-deterministic output | With provider configured, invoke both MCP tools against the sample app; confirm `ai_scaffold` returns a coherent `ServiceDef` and `ai_explain` returns structured projection JSON for a known service. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
