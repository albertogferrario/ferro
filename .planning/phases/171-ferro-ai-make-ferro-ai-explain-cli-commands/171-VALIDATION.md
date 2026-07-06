---
phase: 171
slug: ferro-ai-make-ferro-ai-explain-cli-commands
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 171 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: 171-RESEARCH.md §"Validation Architecture" + §"Security Domain".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` |
| **Config file** | none — workspace-level, no install needed |
| **Quick run command** | `cargo test -p ferro-cli -p ferro-ai --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90 seconds (quick) / several minutes (full gate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli -p ferro-ai --lib`
- **After every plan wave:** Run the full suite command
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Task (logical) | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|----------------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `complete_with` + CompleteOptions | 1 | AICLI-02 | — | N/A | unit | `cargo test -p ferro-ai complete_delegates_to_complete_with` | ❌ W0 | ⬜ pending |
| ServiceDef-aware schema via complete_with | 1 | AICLI-02 | — | N/A | unit | `cargo test -p ferro-ai complete_with_servicedef_schema` | ❌ W0 | ⬜ pending |
| Lexical relevance filter | 2 | AICLI-01 | T-171-DoS (token exhaustion) | input bounded by relevance budget | unit | `cargo test -p ferro-cli relevance` | ❌ W0 | ⬜ pending |
| generation_context always included | 2 | AICLI-01 | — | N/A | unit | `cargo test -p ferro-cli context_always_includes_generation` | ❌ W0 | ⬜ pending |
| ServiceDef → builder-source emitter round-trip | 2 | AICLI-01 / D-03 | T-171-Tamper (gen Rust to tree) | source only to `src/projections/<name>.rs` | unit | `cargo test -p ferro-cli emitter_round_trip` | ❌ W0 | ⬜ pending |
| `ai:make --dry-run` writes nothing | 2 | AICLI-02 / D-03 | — | N/A | integration | `cargo test -p ferro-cli dry_run_no_file_write` | ❌ W0 | ⬜ pending |
| Output path sanitization (no traversal) | 2 | AICLI-01 | T-171-Path-Traversal | `is_valid_identifier` + fixed base join | unit | `cargo test -p ferro-cli ai_make_rejects_path_traversal` | ❌ W0 | ⬜ pending |
| `FERRO_AI_MAX_TOKENS_PER_COMMAND` applied | 2 | AICLI-02 | T-171-DoS | caps response tokens | unit | `cargo test -p ferro-cli max_tokens_env_applied` | ❌ W0 | ⬜ pending |
| AI-required fail-fast (no provider) | 2 | AICLI-01/03 | — | clear env-var message, no silent path | unit | `cargo test -p ferro-cli ai_make_requires_ai_config` | ❌ W0 | ⬜ pending |
| `ai:explain` resolution order service-first | 3 | AICLI-03 | — | N/A | unit | `cargo test -p ferro-cli explain_resolution_order` | ❌ W0 | ⬜ pending |
| `ai:explain --dry-run` no LLM call | 3 | AICLI-03 / D-06 | — | N/A | unit | `cargo test -p ferro-cli explain_dry_run_no_llm_call` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-ai/src/complete.rs` — add `complete_with::<T>` + `CompleteOptions` tests (delegate-with-defaults, ServiceDef-aware schema applied)
- [ ] `ferro-cli/src/commands/ai_make.rs` — new file; emitter round-trip, dry-run no-write, path-traversal rejection, max_tokens env, fail-fast tests
- [ ] `ferro-cli/src/commands/ai_explain.rs` — new file; resolution-order + dry-run-no-LLM tests
- [ ] Lexical filter unit tests (in `ai_make.rs` or a `ferro-cli/src/relevance.rs` module)
- [ ] Reuse `to_snake_case` / `is_valid_identifier` from `make_projection.rs` (extract to shared `ferro-cli/src/naming.rs` if cross-referenced)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end live `ai:make "<desc>"` against a real provider produces a sensible, ferro-consistent ServiceDef (SC#6 — references existing models/intents, not generic templates) | AICLI-01/02 | Requires a live LLM provider + API key + a real project fixture; non-deterministic output | With `FERRO_AI_*` set, run `ferro ai:make "track customer orders with pending/paid/shipped states" --dry-run` in a sample app; confirm fields use real `FieldMeaning`s, a `StateMachine` is present, and referenced model names exist in the project. |
| Live `ai:explain <existing-service>` returns projection-framed prose | AICLI-03 | Requires live provider + an existing ServiceDef in a project | Run `ferro ai:explain <service>`; confirm prose references the service's Intents / FieldMeanings / Actions+Guards / StateMachine. |

*The projection-roundtrip end-to-end automated test (NL → ServiceDef → JSON-UI) is AICLI-06, owned by Phase 173 — not this phase.*

---

## Validation Sign-Off

- [ ] All tasks have an `<automated>` verify or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
