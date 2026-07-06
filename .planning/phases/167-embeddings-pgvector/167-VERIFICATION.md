---
phase: 167-embeddings-pgvector
verified: 2026-06-08T12:20:40Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 167: Embeddings & pgvector — Verification Report

**Phase Goal:** Ship pure-Rust embedding helpers and cosine similarity, plus an optional pgvector integration for semantic search.
**Verified:** 2026-06-08T12:20:40Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro_ai::embed(client, text)` calls the provider's embedding endpoint and returns `Vec<f32>`; Anthropic, OpenAI, and Ollama all implement `LlmClient::embed()` | VERIFIED | `ferro-ai/src/embed.rs` exists with `pub async fn embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error>` delegating via `client.embed(text).await`; re-exported in `lib.rs` as `pub use embed::embed`; Ollama hits `/api/embed` via `Self::embed_model()`, OpenAI hits `/v1/embeddings` via `Self::embed_model()`, Anthropic returns `Err(Error::Unsupported)` |
| 2 | `ferro_ai::cosine_similarity(a: &[f32], b: &[f32]) -> f32` is a pure Rust function with no extra crates; returns a value in [-1.0, 1.0]; panics with a clear message on empty or dimension-mismatched inputs | VERIFIED | `ferro-ai/src/similarity.rs` implements `pub fn cosine_similarity` with pure arithmetic; panics with `"cosine_similarity: empty slice \`a\`"` / `"cosine_similarity: empty slice \`b\`"` and `"cosine_similarity: dimension mismatch ({} vs {})"` messages; `cargo tree --no-default-features` shows zero new dependencies; re-exported as `pub use similarity::cosine_similarity` in `lib.rs` |
| 3 | `ferro_ai::pgvector` module exists behind the `pgvector` cargo feature; `PgVectorStore::store` and `PgVectorStore::nearest` accept raw sqlx connections and return typed results | VERIFIED | `ferro-ai/src/pgvector/mod.rs` defines `pub struct PgVectorStore` with `store(&self, pool: &PgPool, id: i64, embedding: &[f32]) -> Result<(), Error>` and `nearest(&self, pool: &PgPool, query: &[f32], k: u32) -> Result<Vec<Neighbor>, Error>`; module gated by `#[cfg(feature = "pgvector")] pub mod pgvector;` in `lib.rs`; `Neighbor { id: i64, score: f32 }` is a typed result with manual `FromRow` impl |
| 4 | Feature flag `pgvector` adds only `pgvector 0.4` to the dependency graph; non-flagged builds do not pull pgvector | VERIFIED | `cargo tree -p ferro-ai --no-default-features \| grep -E 'pgvector\|sqlx'` returns empty — no pgvector or sqlx in default build. SC#4 wording reconciliation (D-12, documented in `[features]` comment): sqlx is an unavoidable second direct dep under the feature because pgvector 0.4 does not re-export `PgPool`; this is documented in `Cargo.toml` with a version-rationale comment and reconciled in the plan. Non-flagged builds pull neither. |
| 5 | Unit tests for `cosine_similarity`: orthogonal vectors return 0.0, identical vectors return 1.0, opposite vectors return -1.0 | VERIFIED | `cargo test -p ferro-ai similarity` exits 0 with 6 tests: `identical_vectors` (→1.0), `orthogonal_vectors` (→0.0), `opposite_vectors` (→-1.0), `panics_on_empty` (should_panic "empty slice"), `panics_on_dim_mismatch` (should_panic "dimension mismatch"), `zero_magnitude_yields_nan` |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-ai/src/similarity.rs` | `pub fn cosine_similarity` + inline tests | VERIFIED | Exists, substantive (89 lines), re-exported from `lib.rs` |
| `ferro-ai/src/embed.rs` | `pub async fn embed` + tokio tests | VERIFIED | Exists, substantive (104 lines), delegates to `LlmClient::embed`, re-exported from `lib.rs` |
| `ferro-ai/src/error.rs` | `Error::Sqlx(String)` variant | VERIFIED | `Sqlx(String)` variant present at line 75; `is_retryable()` uses `_ => false` wildcard — no match arm edit needed |
| `ferro-ai/src/lib.rs` | `pub use embed::embed; pub use similarity::cosine_similarity; #[cfg(feature = "pgvector")]` gates | VERIFIED | All three re-exports present; `#[cfg(feature = "pgvector")] pub mod pgvector;` and `pub use pgvector::{Neighbor, PgVectorStore};` are feature-gated |
| `ferro-ai/src/client/ollama.rs` | `embed_model()` reading `FERRO_AI_EMBED_MODEL`, default `nomic-embed-text` | VERIFIED | `pub(crate) fn embed_model() -> String` at line 51; `embed()` uses `Self::embed_model()` at line 234; env tests `embed_model_default_is_nomic` and `embed_model_from_env` present |
| `ferro-ai/src/client/openai.rs` | `embed_model()` reading `FERRO_AI_EMBED_MODEL`, default `text-embedding-3-small` | VERIFIED | `pub(crate) fn embed_model() -> String` at line 57; `embed()` uses `Self::embed_model()` at line 300; env tests present |
| `ferro-ai/src/pgvector/mod.rs` | `PgVectorStore` + `Neighbor` + manual `FromRow`, feature-gated | VERIFIED | Exists (150 lines); `pub struct PgVectorStore`, `pub struct Neighbor`, manual `impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Neighbor`; `ON CONFLICT (id) DO UPDATE` upsert and `(1.0 - (col <=> $1))::float4 AS score` cosine-distance inversion both present |
| `ferro-ai/Cargo.toml` | `[features]` with `pgvector = ["dep:pgvector", "dep:sqlx"]` and optional deps | VERIFIED | `[features]` section exists; `pgvector = ["dep:pgvector", "dep:sqlx"]` and `postgres-tests = ["pgvector"]` present; optional deps declared with `dep:` syntax |
| `ferro-ai/tests/pgvector_integration.rs` | `#![cfg(feature = "postgres-tests")]` first line + `DATABASE_URL` guard | VERIFIED | First line is `#![cfg(feature = "postgres-tests")]`; two-layer guard (compile-time feature + runtime `DATABASE_URL` env-var early-return); full roundtrip test body present |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-ai/src/embed.rs` | `LlmClient::embed` | `client.embed(text).await` | WIRED | Line 36: `client.embed(text).await` — exact pattern matches |
| `ferro-ai/src/client/ollama.rs` | `FERRO_AI_EMBED_MODEL` | `embed_model()` helper, default `nomic-embed-text` | WIRED | `std::env::var("FERRO_AI_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string())` at line 52; called in `embed()` at line 234 |
| `ferro-ai/src/client/openai.rs` | `FERRO_AI_EMBED_MODEL` | `embed_model()` helper, default `text-embedding-3-small` | WIRED | `std::env::var("FERRO_AI_EMBED_MODEL")` at line 58; called in `embed()` at line 300 |
| `ferro-ai/src/pgvector/mod.rs` | `pgvector::Vector` | `Vector::from(embedding.to_vec())` bound into sqlx query | WIRED | Lines 107, 138: `let vec = Vector::from(...)` bound via `$2` |
| `ferro-ai/src/pgvector/mod.rs` | Postgres cosine distance operator | `SELECT (1.0 - (col <=> $1))::float4 AS score ... ORDER BY col <=> $1 LIMIT $2` | WIRED | Lines 140-141 match the expected SQL pattern |
| `ferro-ai/src/lib.rs` | pgvector module | `#[cfg(feature = "pgvector")] pub mod pgvector;` | WIRED | Line 55-56 in `lib.rs` |

---

### Data-Flow Trace (Level 4)

Not applicable — no dynamic data rendered to a UI. All artifacts are library primitives (free functions, struct impls). Data-flow path: caller → `embed()` → `LlmClient::embed()` → provider HTTP — fully connected.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| cosine_similarity unit tests (identical, orthogonal, opposite, panics) | `cargo test -p ferro-ai similarity` | 6 passed, 0 failed | PASS |
| No pgvector/sqlx in default dep graph | `cargo tree -p ferro-ai --no-default-features \| grep -E 'pgvector\|sqlx'` | Empty output | PASS |
| pgvector/sqlx present under feature flag | `cargo tree -p ferro-ai --features pgvector \| grep -E '^[├└]── (pgvector\|sqlx)'` | `pgvector v0.4.1` and `sqlx v0.8.6` both present | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AISDK-04 | 167-01-PLAN.md | Developer can generate text embeddings and compute cosine similarity (pure Rust helpers, zero extra crates). | SATISFIED | `ferro_ai::embed` and `ferro_ai::cosine_similarity` implemented; no new non-feature-gated dependencies; 6 passing unit tests |
| AISDK-05 | 167-02-PLAN.md | Developer can persist and query embeddings via pgvector (feature-gated `pgvector 0.4`, thin sqlx raw-query module). | SATISFIED | `ferro_ai::pgvector::{PgVectorStore, Neighbor}` behind `pgvector` feature; `store` + `nearest` fully implemented; integration test compiles and skips gracefully without live DB |

---

### Anti-Patterns Found

No blockers or stubs detected in phase files.

| File | Pattern | Severity | Notes |
|------|---------|----------|-------|
| `ferro-ai/src/pgvector/mod.rs` | `nearest` uses `k: u32` vs. plan-specified `k: i64` | Info | Deliberate improvement: `u32` rejects negative limits at the type level. Internally cast to `i64::from(k)` for sqlx bind. Integration test passes literal `2` (valid u32). Not a defect. |

---

### Human Verification Required

None. All five success criteria verified programmatically.

---

### Gaps Summary

No gaps. All 5/5 ROADMAP success criteria are satisfied. AISDK-04 and AISDK-05 are fully covered.

**SC#4 note:** The roadmap says "adds only pgvector 0.4 to the dependency graph" but D-12 (documented in both CONTEXT.md and Cargo.toml) reconciles this: `sqlx` is a structural necessity because pgvector 0.4 does not re-export `PgPool`, which appears in `PgVectorStore`'s public API. Non-flagged builds pull neither `pgvector` nor `sqlx` — the intent of SC#4 (isolation of optional deps) is fully satisfied.

---

_Verified: 2026-06-08T12:20:40Z_
_Verifier: Claude (gsd-verifier)_
