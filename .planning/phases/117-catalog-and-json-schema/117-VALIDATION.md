---
phase: 117
slug: catalog-and-json-schema
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-18
---

# Phase 117 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `117-RESEARCH.md` §10 "Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `cargo test` (workspace convention) |
| **Config file** | `Cargo.toml` per crate (no separate test runner) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib catalog::` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~10s quick (catalog module only); ~90s full CI-parity |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-json-ui --lib catalog::`
- **After every wave merge:** `cargo test -p ferro-json-ui --lib && cargo test -p ferro-mcp --lib && cargo test -p ferro-cli --lib`
- **Before `/gsd-verify-work`:** Full CI-parity suite must be green.
- **Max feedback latency:** 90s (full suite)

---

## Per-Task Verification Map

Mapped to the 8 ROADMAP success criteria + supporting drift/scope guards. Task IDs provisional — planner refines when slicing plans.

| Task ID | Plan | Wave | Success Criterion | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------------|-----------|-------------------|-------------|--------|
| 117-01-01 | 01 | 1 | Prereq — dep + skeleton | unit | `cargo build -p ferro-json-ui --lib` | ❌ W0 (creates `catalog.rs`) | ⬜ pending |
| 117-01-02 | 01 | 1 | Drift guard `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` | unit | `cargo test -p ferro-json-ui --lib catalog::builtin_specs_len_matches_dispatch` | ❌ W0 | ⬜ pending |
| 117-02-01 | 02 | 2 | SC-1 — Catalog::build populates all builtins | unit | `cargo test -p ferro-json-ui --lib catalog::build_populates_all_builtins` | ❌ W0 | ⬜ pending |
| 117-02-02 | 02 | 2 | SC-1 — plugin discovery | unit | `cargo test -p ferro-json-ui --lib catalog::build_discovers_plugins` | ❌ W0 | ⬜ pending |
| 117-02-03 | 02 | 2 | H-3 risk — BuildFailed on invalid plugin schema | unit | `cargo test -p ferro-json-ui --lib catalog::build_fails_on_invalid_plugin_schema` | ❌ W0 | ⬜ pending |
| 117-03-01 | 03 | 3 | SC-4 — json_schema() is valid JSON Schema | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_is_valid` | ❌ W0 | ⬜ pending |
| 117-03-02 | 03 | 3 | SC-4 — oneOf covers every built-in via `"type": const` | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_oneof_covers_all_builtins` | ❌ W0 | ⬜ pending |
| 117-03-03 | 03 | 3 | SC-4 — $defs/Action + $defs/Visibility present | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_has_action_and_visibility_defs` | ❌ W0 | ⬜ pending |
| 117-04-01 | 04 | 4 | SC-3 — validate positive per type | unit | `cargo test -p ferro-json-ui --lib catalog::validate_positive_per_type` | ❌ W0 | ⬜ pending |
| 117-04-02 | 04 | 4 | SC-3 — UnknownType error | unit | `cargo test -p ferro-json-ui --lib catalog::validate_unknown_type` | ❌ W0 | ⬜ pending |
| 117-04-03 | 04 | 4 | SC-3 — PropsInvalid on missing required | unit | `cargo test -p ferro-json-ui --lib catalog::validate_missing_required_prop` | ❌ W0 | ⬜ pending |
| 117-04-04 | 04 | 4 | SC-3 — SpecInvalid on bad $schema | unit | `cargo test -p ferro-json-ui --lib catalog::validate_bad_schema_version` | ❌ W0 | ⬜ pending |
| 117-04-05 | 04 | 4 | SC-3 — pre-dispatch short-circuit | unit | `cargo test -p ferro-json-ui --lib catalog::validate_pre_dispatch_short_circuits` | ❌ W0 | ⬜ pending |
| 117-04-06 | 04 | 4 | SC-8 — validator compiled once | unit (timing) | `cargo test -p ferro-json-ui --lib catalog::validator_is_cached_not_recompiled` | ❌ W0 | ⬜ pending |
| 117-05-01 | 05 | 5 | SC-2 — prompt ≤ 8 KB | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_under_size_budget` | ❌ W0 | ⬜ pending |
| 117-05-02 | 05 | 5 | SC-2 — prompt mentions every built-in | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_mentions_every_builtin` | ❌ W0 | ⬜ pending |
| 117-05-03 | 05 | 5 | SC-2 — prompt deterministic | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_is_deterministic` | ❌ W0 | ⬜ pending |
| 117-05-04 | 05 | 5 | SC-5 — component_schema returns Props-only | unit | `cargo test -p ferro-json-ui --lib catalog::component_schema_returns_props_only` | ❌ W0 | ⬜ pending |
| 117-05-05 | 05 | 5 | SC-5 — component_schema(unknown) → None | unit | `cargo test -p ferro-json-ui --lib catalog::component_schema_none_for_unknown` | ❌ W0 | ⬜ pending |
| 117-06-01 | 06 | 6 | SC-6 — `ferro json-ui:schema` to stdout | integration (smoke) | `cargo run --quiet -- json-ui:schema \| jq .` | ❌ W0 (new CLI cmd) | ⬜ pending |
| 117-06-02 | 06 | 6 | SC-6 — `--output <path>` writes file | integration | shell smoke test | ❌ W0 | ⬜ pending |
| 117-06-03 | 06 | 6 | SC-6 — `--component <name>` prints Props | integration | shell smoke test | ❌ W0 | ⬜ pending |
| 117-06-04 | 06 | 6 | SC-7 — `COMPONENT_CATALOG` grep returns 0 | CI grep | `! rg "COMPONENT_CATALOG" --type rust` | ❌ W0 | ⬜ pending |
| 117-06-05 | 06 | 6 | consumer migration — ferro-mcp json_ui_catalog + json_ui_generate + ferro-cli/ai.rs | unit | `cargo test -p ferro-mcp --lib && cargo test -p ferro-cli --lib` | ⚠ existing tests | ⬜ pending |
| 117-07-01 | 07 | 7 | All SCs — framework-level integration | integration | `cargo test -p ferro-rs --lib --features json-ui` (if catalog is wired into framework) | ⚠ | ⬜ pending |
| 117-07-02 | 07 | 7 | Phase gate | full CI | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | ✅ (always runs) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/Cargo.toml` — add `jsonschema = "0.46"` (per RESEARCH H-1 override of CONTEXT D-09).
- [ ] `ferro-json-ui/src/catalog.rs` — NEW file. Hosts `Catalog`, `CatalogError`, `ComponentSpec`, `global_catalog()`, `BUILTIN_SPECS` static table, all unit tests inline.
- [ ] `ferro-json-ui/src/lib.rs` — re-export catalog types; delete `COMPONENT_CATALOG` const (lines 88-168).
- [ ] `ferro-cli/src/commands/json_ui_schema.rs` — NEW file. CLI command with `--output`, `--pretty`, `--component` flags.
- [ ] `framework/src/app.rs` — add `"json-ui:schema"` subcommand dispatch (corrected from CONTEXT D-22's stale `framework/src/bin/ferro.rs` path).
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — rewrite body to pull from `global_catalog()`; preserve public `JsonUiCatalog` shape.
- [ ] `ferro-mcp/src/tools/json_ui_generate.rs` — swap `COMPONENT_CATALOG` reference to `global_catalog().prompt()`.
- [ ] `ferro-cli/src/ai.rs` — migrate `COMPONENT_CATALOG` reference (discovered by RESEARCH grep; not in CONTEXT D-26).

---

## Manual-Only Verifications

| Behavior | Success Criterion | Why Manual | Test Instructions |
|----------|-------------------|------------|-------------------|
| `ferro json-ui:schema` output usable by external IDE tooling | SC-6 | End-to-end LSP integration is out of scope for Phase 117; the smoke test ("jq parses the output") is sufficient for phase completion. Real IDE consumption belongs to a future phase. | Run `cargo run --quiet -- json-ui:schema > /tmp/schema.json && jq . /tmp/schema.json > /dev/null` — exit code 0 is the only assertion. |
| Prompt quality for downstream LLMs | SC-2 | LLM prompt quality is a Phase 120 (two-tier AI) concern. Phase 117 guarantees size + coverage; quality evaluation belongs with the two-tier strategy. | None required for Phase 117. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING file references (jsonschema dep, catalog.rs, CLI cmd, app.rs dispatch)
- [ ] No watch-mode flags (cargo test is one-shot)
- [ ] Feedback latency < 90s (full CI-parity suite)
- [ ] `nyquist_compliant: true` set in frontmatter once plans are written

**Approval:** pending
