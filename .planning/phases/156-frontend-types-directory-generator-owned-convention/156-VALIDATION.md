---
phase: 156
slug: frontend-types-directory-generator-owned-convention
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 156 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests) |
| **Config file** | ferro-cli/Cargo.toml |
| **Quick run command** | `cargo test -p ferro-cli -- doctor 2>&1 \| tail -20` |
| **Full suite command** | `cargo test --all-features 2>&1 \| tail -30` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli -- doctor 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --all-features 2>&1 | tail -30`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 156-01-01 | 01 | 1 | D-05 | — | N/A | manual | `git ls-files app/frontend/src/types/` exits with no output | ❌ W0 | ⬜ pending |
| 156-01-02 | 01 | 1 | D-06 | — | N/A | manual | `grep "load-bearing" ferro-cli/src/templates/files/root/gitignore.tpl` | ❌ W0 | ⬜ pending |
| 156-01-03 | 01 | 1 | D-18 | — | N/A | manual | `grep "frontend/src/lib/types" ferro-cli/src/commands/generate_types.rs` | ❌ W0 | ⬜ pending |
| 156-02-01 | 02 | 2 | D-09,D-20 | — | warns on hand-written files | unit | `cargo test -p ferro-cli frontend_types_convention` | ❌ W0 | ⬜ pending |
| 156-02-02 | 02 | 2 | D-09 | — | N/A | unit | `cargo test -p ferro-cli default_checks_returns` | ❌ W0 | ⬜ pending |
| 156-03-01 | 03 | 3 | D-15,D-16,D-21 | — | FERRO_VERSION resolved | unit | `cargo test -p ferro-cli docker` | ❌ W0 | ⬜ pending |
| 156-03-02 | 03 | 3 | D-15 | — | types-gen stage present when has_frontend | unit | `cargo test -p ferro-cli types_gen_stage` | ❌ W0 | ⬜ pending |
| 156-04-01 | 04 | 4 | D-08 | — | N/A | manual | `test -f docs/src/cli/frontend-types.md` | ❌ W0 | ⬜ pending |
| 156-05-01 | 05 | 5 | D-13 | — | N/A | unit | `cargo check --all 2>&1 \| grep -c error` returns 0 | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed before Wave 1 — all tests are added alongside implementation in their respective plans.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `git status` clean after `cargo run` in reference app | D-05, D-01 | Requires running the server and checking git state | `cd app && cargo run` (brief), then `git status app/frontend/src/types/` — must show no tracked files |
| `docker build .` succeeds with gitignored types | D-15 | Requires Docker and crates.io access | From `app/` after phase: `docker build -t ferro-test .` — must exit 0 with `npm run build` succeeding |
| `ferro doctor` shows warning for hand-written file in `frontend/src/types/` | D-09 | Requires a real project with planted file | Plant `custom.ts` in `frontend/src/types/` of a test project, run `ferro doctor` — must show WARNING for `frontend_types_convention` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
