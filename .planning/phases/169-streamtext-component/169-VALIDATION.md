---
phase: 169
slug: streamtext-component
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 169 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (full); ~10–20s (quick, per-crate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~20 seconds (quick per-crate run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 169-01-01 | 01 | 1 | AISSE-02 SC#1 | — | N/A | unit | `cargo test -p ferro-json-ui stream_text_props_serde` | ❌ W0 | ⬜ pending |
| 169-01-02 | 01 | 1 | AISSE-02 SC#2a | T-169-01 | `data-ferro-stream-url` attribute emitted on container | unit | `cargo test -p ferro-json-ui render_streamtext_emits` | ❌ W0 | ⬜ pending |
| 169-01-03 | 01 | 1 | AISSE-02 SC#2b | T-169-01 | `sse_url` HTML-attribute-escaped via `html_escape` (no attribute breakout) | unit | `cargo test -p ferro-json-ui render_streamtext_escapes` | ❌ W0 | ⬜ pending |
| 169-02-01 | 02 | 2 | AISSE-02 SC#2c | T-169-02 | Init script emitted exactly once when ≥1 StreamText present | unit | `cargo test -p ferro-json-ui render_spec_with_stream_text_emits_init_script` | ❌ W0 | ⬜ pending |
| 169-02-02 | 02 | 2 | AISSE-02 SC#2d | — | No init script when no StreamText in spec | unit | `cargo test -p ferro-json-ui render_spec_without_stream_text` | ❌ W0 | ⬜ pending |
| 169-02-03 | 02 | 2 | AISSE-02 SC#2 | T-169-03 | Streamed tokens appended as text (no HTML parsing / XSS) — asserted in init-script content | unit | `cargo test -p ferro-json-ui render_spec_with_stream_text_emits_init_script` | ❌ W0 | ⬜ pending |
| 169-03-01 | 03 | 2 | AISSE-02 SC#3 | — | `global_catalog()` includes `StreamText` with prop descriptions | unit | `cargo test -p ferro-json-ui catalog` | ❌ W0 | ⬜ pending |
| 169-03-02 | 03 | 2 | AISSE-02 SC#5 | — | `BUILTIN_TYPES` count assertion updated (44 → 45) | unit | `cargo test -p ferro-json-ui builtin_types_count` | ✅ (count update) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `stream_text_props_serde_roundtrip` test in `ferro-json-ui/src/render/atoms.rs` `#[cfg(test)]`
- [ ] `render_streamtext_emits_data_attribute` test in `atoms.rs` `#[cfg(test)]`
- [ ] `render_streamtext_escapes_url` test in `atoms.rs` `#[cfg(test)]`
- [ ] `render_spec_with_stream_text_emits_init_script` test in `render/mod.rs` `#[cfg(test)]`
- [ ] `render_spec_without_stream_text_emits_no_init_script` test in `render/mod.rs` `#[cfg(test)]`
- [ ] `global_catalog_includes_stream_text` test in `catalog.rs` `#[cfg(test)]`

*Existing infrastructure (cargo test) covers the harness — Wave 0 here means writing the test stubs alongside each behavior, not installing a framework.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end token streaming in a live browser (EventSource opens, appends tokens, closes on `done`) | AISSE-02 SC#2 | Requires a running SSE endpoint + browser EventSource runtime; not unit-testable from Rust | Render a spec with one StreamText pointing at a test SSE route that emits `data:` frames then `event: done`; observe tokens append and the connection close (no reconnect) in browser devtools |

*The Rust unit tests assert the emitted HTML/JS string content (attribute present, escaping applied, init script present/absent, append-via-textContent and close-on-done present in the script). Runtime EventSource behavior is the one manual check.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 20s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
