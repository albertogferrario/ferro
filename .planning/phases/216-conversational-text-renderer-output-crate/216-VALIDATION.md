---
phase: 216
slug: conversational-text-renderer-output-crate
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 216 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) + `insta` 1.x snapshots (yaml feature; already a workspace dev-dep) |
| **Config file** | none — workspace `Cargo.toml`; new `ferro-text/Cargo.toml` adds `insta` to `[dev-dependencies]` |
| **Quick run command** | `cargo test -p ferro-text` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~quick: <30s for `-p ferro-text`; full: minutes (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-text` (and `cargo test -p ferro-projections` after the `render_hint` schema task)
- **After every plan wave:** Run `cargo build --workspace` + `cargo test -p ferro-text -p ferro-projections`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green + `cargo doc --no-deps -Dwarnings` clean
- **Max feedback latency:** ~30 seconds (per-crate quick run)

---

## Per-Task Verification Map

> Task IDs are illustrative — the planner assigns real IDs. Each phase behavior maps to an automated check below.

| Behavior | Requirement | Secure/Correct Behavior | Test Type | Automated Command | Status |
|----------|-------------|-------------------------|-----------|-------------------|--------|
| `FieldDef.render_hint` field + `RenderHint{AltText,Skip}` added; all 11 `FieldDef {}` literal sites updated to `render_hint: None` | CHAN-03 | ferro-projections compiles; existing tests unchanged | unit/compile | `cargo test -p ferro-projections` | ⬜ pending |
| `render_hint` default `None` preserves behavior | CHAN-03 | absent hint = today's output | unit | `cargo test -p ferro-projections` | ⬜ pending |
| New `ferro-text` crate exists with `TextRenderer: Renderer<Output=String, Context=BaseContext>` | CHAN-04 | crate builds, impls trait | compile | `cargo build -p ferro-text` | ⬜ pending |
| ferro-projections adds NO renderer | CHAN-04 / SC-1 | grep finds no `impl Renderer` outside sketch in ferro-projections src | grep | `! grep -rn "impl Renderer" ferro-projections/src --include=*.rs \| grep -v sketch` | ⬜ pending |
| Process anchor: lists only guard-passing actions (empty guards → all 4; `{is_approver:false}` → approve/reject hidden) | CHAN-04 / SC-2 | snapshot diff between the two guard states | snapshot | `cargo test -p ferro-text` (insta) | ⬜ pending |
| Verbosity Brief vs Full differ on anchor fixture | CHAN-04 / SC-2 | two distinct snapshots | snapshot | `cargo test -p ferro-text` | ⬜ pending |
| Per-intent render: Browse/Collect/Process/Summarize/Track each produce intent-appropriate text | CHAN-04 | snapshot per intent fixture | snapshot | `cargo test -p ferro-text` | ⬜ pending |
| `ImageUrl`/`Url` with `AltText(s)` → alt text; `Skip` → omitted; `None` → useful label (not raw URL) | CHAN-03 / SC-3 | snapshot/string asserts per hint variant | snapshot/unit | `cargo test -p ferro-text` | ⬜ pending |
| Focus + Analyze fallback present and tested | CHAN-04 / SC-3 | snapshot of degraded render + note | snapshot | `cargo test -p ferro-text` | ⬜ pending |
| Empty intent slice → `Error::NoIntents` (not `"unknown"`) | CHAN-04 | typed error returned | unit | `cargo test -p ferro-text` | ⬜ pending |
| `TextRenderer` (+ `RenderHint`) reachable from `ferro` facade | CHAN-04 / SC-4 | `framework/src/lib.rs` re-export; doc test or grep | grep/doc | `grep -n "ferro_text::TextRenderer" framework/src/lib.rs` | ⬜ pending |
| Crate in workspace members + publish.yml Wave 1b | CHAN-04 | registered | grep | `grep -n "ferro-text" Cargo.toml .github/workflows/publish.yml` | ⬜ pending |
| Docs clean | SC-4 | no doc warnings | build | `cargo doc --no-deps -p ferro-text -Dwarnings` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase validation — `cargo test` + `insta` (already a workspace dev-dep) are sufficient.
- The new `ferro-text/Cargo.toml` `[dev-dependencies]` must include `insta = { version = "1", features = ["yaml"] }` (mirrors ferro-projections). This is part of the crate-scaffold task, not a separate framework install.
- Snapshot baseline files (`*.snap`) are created on first `cargo insta review` / first green run and committed.

*No new test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Conversational text "reads naturally" (not a debug dump) | CHAN-04 (killer-feature polish) | Subjective readability is not grep-checkable; snapshots pin exact strings but not quality | Human reads the committed `*.snap` per-intent outputs; confirm each reads like a channel reply, not a struct printout |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have automated verify or are listed Manual-Only
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none — existing infra)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
