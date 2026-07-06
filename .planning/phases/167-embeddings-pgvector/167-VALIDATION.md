---
phase: 167
slug: embeddings-pgvector
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-08
---

# Phase 167 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + `tokio::test` (already in `ferro-ai` dev-dependencies) |
| **Config file** | none — inline `#[test]` / `#[tokio::test]` modules |
| **Quick run command** | `cargo test -p ferro-ai` |
| **Full suite command** | `cargo test -p ferro-ai --all-features` |
| **Estimated runtime** | ~20 seconds (unit); integration tests skip without `DATABASE_URL` |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-ai`
- **After every plan wave:** Run `cargo test -p ferro-ai --all-features`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` must be green
- **Max feedback latency:** ~20 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 167-cosine | similarity | 1 | AISDK-04 | — | N/A (pure math) | unit | `cargo test -p ferro-ai similarity` | ❌ W0 | ⬜ pending |
| 167-cosine-panic | similarity | 1 | AISDK-04 | — | clear panic msg on empty / dim-mismatch | unit (should_panic) | `cargo test -p ferro-ai` | ❌ W0 | ⬜ pending |
| 167-embed-fn | embed | 1 | AISDK-04 | — | propagates `Error::Unsupported` unchanged | unit (mock client) | `cargo test -p ferro-ai` | ❌ W0 | ⬜ pending |
| 167-embed-model | embed | 1 | AISDK-04 | T-167-01 | embedding endpoint never receives chat model (D-13) | unit | `cargo test -p ferro-ai` | ❌ W0 | ⬜ pending |
| 167-pgv-store | pgvector | 2 | AISDK-05 | — | parameterized sqlx query (no SQL injection) | integration (gated) | `cargo test -p ferro-ai --features pgvector,postgres-tests` | ❌ W0 | ⬜ pending |
| 167-pgv-nearest | pgvector | 2 | AISDK-05 | — | nearest ordered by cosine similarity | integration (gated) | same | ❌ W0 | ⬜ pending |
| 167-dep-graph | pgvector | 2 | AISDK-05 | — | non-flagged build pulls neither pgvector nor sqlx | cargo-tree check | `cargo tree -p ferro-ai --no-default-features \| grep -E 'pgvector\|^sqlx'` returns empty | ❌ W0 | ⬜ pending |
| 167-all-features | pgvector | 2 | AISDK-05 | — | feature compiles under CI | compile/clippy | `cargo clippy -p ferro-ai --all-features -- -D warnings` | ✅ CI | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-ai/src/similarity.rs` — `cosine_similarity()` + inline `#[test]` (identical→1.0, orthogonal→0.0, opposite→-1.0) + `#[should_panic]` empty + dim-mismatch (AISDK-04)
- [ ] `ferro-ai/src/embed.rs` — `embed()` free fn + inline `#[tokio::test]` mock-client tests incl. `Unsupported` propagation (AISDK-04)
- [ ] `ferro-ai/src/client/ollama.rs` + `openai.rs` — `embed_model()` helper test (D-13: default `nomic-embed-text` / `text-embedding-3-small`, `FERRO_AI_EMBED_MODEL` override) (AISDK-04)
- [ ] `ferro-ai/src/pgvector/mod.rs` — `PgVectorStore`, `Neighbor`, behind `#[cfg(feature = "pgvector")]` (AISDK-05)
- [ ] `ferro-ai/tests/pgvector_integration.rs` — gated `#![cfg(feature = "postgres-tests")]` + `DATABASE_URL` env-guard (AISDK-05)
- [ ] `ferro-ai/Cargo.toml` — `[features] pgvector`, `postgres-tests`, optional `pgvector`/`sqlx` deps

*Rust built-in test framework already present — no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `PgVectorStore` against a real Postgres + `vector` extension | AISDK-05 | CI has no Postgres-with-pgvector service; integration test skips without `DATABASE_URL` | `CREATE EXTENSION vector;` on a local PG, set `DATABASE_URL`, run `cargo test -p ferro-ai --features pgvector,postgres-tests` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 20s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
