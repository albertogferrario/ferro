---
phase: 173
slug: make-json-view-v2-projection-roundtrip-test
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 173 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`#[test]`) |
| **Config file** | none — workspace-level `cargo test` |
| **Quick run command** | `cargo test -p ferro-ai --test projection_roundtrip` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~10–30 s (quick); full suite minutes |

> The roundtrip test is **offline/deterministic** (a constructed `ServiceDef`
> fixture, no network, no LLM key, mirroring `ferro-ai/tests/projection_schema.rs`).
> The `ferro-ai` test target gains a dev-dependency on `ferro-json-ui`
> (feature `projections`) so the test can call `Spec::from_service_def`.

---

## Sampling Rate

- **After every task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings`
- **After every plan wave:** `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + clippy clean
- **Max feedback latency:** ~30 s (quick)

---

## Per-Task Verification Map

| Req / SC | Behavior | Test Type | Automated Command | File Exists | Status |
|----------|----------|-----------|-------------------|-------------|--------|
| AICLI-04 / SC1 | `catalog.component_schema()` has no role (deterministic builder selects components); documented non-use | documented (VERIFICATION.md) | N/A — vacuous; rationale recorded | after VERIFICATION write | ⬜ pending |
| AICLI-04 / SC2 | Generated spec validates against `catalog.json_schema()` before write | integration | `cargo test -p ferro-json-ui` (builder coverage) + roundtrip | ✅ existing | ⬜ pending |
| AICLI-04 / SC3 | Component selection driven by `FieldMeaning`/`Intent` via `Spec::from_service_def`, not LLM re-prompting | integration | `cargo test -p ferro-ai --test projection_roundtrip` | ❌ W0 | ⬜ pending |
| AICLI-04 / SC4 | No v1 `JsonUiView` types in the make:json-view pipeline or output | audit (grep) | `! grep -rn "JsonUiView" ferro-cli/src/commands/make_json_view.rs` → zero hits | ✅ grep | ⬜ pending |
| AICLI-06 / SC5 | Roundtrip passes via the **ServiceDef-aware** path, not a generic fallback | integration | `cargo test -p ferro-ai --test projection_roundtrip` (asserts `ColumnFormat::Currency` on a `Money` field — the deterministic observable) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**SC5 path-proof (key correctness gate):** the test seeds a `ServiceDef` with a
`FieldMeaning::Money` field; `Spec::from_service_def` deterministically maps it to
a DataTable column with `ColumnFormat::Currency` (`component_map.rs` `lookup_meaning`).
A generic LLM-only path cannot reliably produce this, so the `has_currency`
assertion pins the ServiceDef-aware path and fails if generation is ever rerouted
through a non-projection fallback.

---

## Wave 0 Requirements

- [ ] `ferro-ai/tests/projection_roundtrip.rs` — covers AICLI-06 (SC5) + AICLI-04 (SC3); offline `ServiceDef` fixture → `Spec::from_service_def` → `catalog.json_schema()` validation + `ColumnFormat::Currency` path-proof
- [ ] `ferro-ai/Cargo.toml` — add `ferro-json-ui = { path = "../ferro-json-ui", features = ["projections"] }` to `[dev-dependencies]`

*Existing `cargo test` infrastructure covers the harness; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live NL → `ServiceDef` quality | AICLI-04 (SC3, live path) | A real `ai:make` call needs a provider API key, non-deterministic | Run `ferro make:json-view <name> --description "<NL>"` against a configured `FERRO_AI_*` provider; confirm the produced view reflects the description and selects sensible components. Sign off in 173-VERIFICATION.md (mirrors Phase 171 SC4/SC6). |

> The automated roundtrip does **not** depend on a live key — the deterministic
> `ServiceDef → Spec → validate` half is the structural proof; live quality is a
> confidence check.

---

## Validation Sign-Off

- [ ] All SCs have an `<automated>` verify or a documented manual gate
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers the MISSING references (roundtrip test + dev-dep)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30 s (quick command)
- [ ] `nyquist_compliant: true` set in frontmatter (set after Wave 0 lands green)

**Approval:** pending — to be set when plans pass the checker
