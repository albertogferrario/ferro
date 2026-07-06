---
phase: 187
slug: ferro-assets-asset-pipeline-composer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
---

# Phase 187 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (built-in) + doc-tests |
| **Config file** | none — standard `[dev-dependencies]` in `ferro-assets/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-assets` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30-90s for the crate's own tests (image transcode tests heaviest; cfg-gate if needed per Phase 185/186 precedent) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-assets`
- **After every plan wave:** Run `cargo clippy -p ferro-assets --all-targets -- -D warnings && cargo test -p ferro-assets`
- **Before `/gsd-verify-work`:** Full workspace suite must be green (CI parity command)
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

> Filled by the planner. Each task maps to a `cargo test` target or grep-verifiable acceptance criterion.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 187-01-01 | 01 | 1 | ASSET-F-01 | — | N/A | unit | `cargo test -p ferro-assets passthrough` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Success-Criterion → Validation Map (Nyquist)

| Criterion | What must be observable | Test |
|-----------|-------------------------|------|
| 1 — content-type passthrough | A JSON file run through the full HTML/CSS/JS/image pipeline is byte-identical | unit: hash input == output bytes for a non-matching content type |
| 2 — minify built-ins + inline safety | html/css/js minifiers shrink output; inline `<script>` (template literals + JSON) and inline `<style>` survive byte-correct | unit + regression fixture from a real tenant fragment |
| 3 — image transcode, zero C deps, bounded | AVIF+JPEG variants emitted at configured widths; `cargo build` adds no C system dep; encodes bounded ≤2 | unit (decode emitted variants) + `cargo tree`/build check + concurrency assertion |
| 4 — responsive rewrite + inject/token | `<img>`→`<picture><source srcset>` references emitted variants; `inject_before_tag` + `%%TOKEN%%` substitution work | unit on rewriter output + injection/token unit |
| 5 — atomic failure | A failing transform returns structured `Error` and NO partial output set | unit: induce transform failure, assert `Err` and no returned assets |

---

## Wave 0 Requirements

- [ ] `ferro-assets/Cargo.toml` `[dev-dependencies]` (the crate itself is Wave 0)
- [ ] Verify exact `swc` umbrella crate version (`cargo search swc`) before `js_minify` is written — the one open question from RESEARCH.md
- [ ] Test fixtures dir (e.g. `ferro-assets/tests/fixtures/`) for the inline-script regression HTML + sample image

*The crate is new — its own scaffolding is the Wave 0 dependency for all tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none expected) | — | — | All phase behaviors have automated `cargo test` coverage |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (swc version, fixtures)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
