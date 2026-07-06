# Phase 167: Embeddings & pgvector - Research

**Researched:** 2026-06-08
**Domain:** Rust embeddings helpers + pgvector/sqlx integration in a feature-gated Cargo crate
**Confidence:** HIGH (core stack verified against live docs.rs/crates.io; patterns cross-checked against existing workspace precedents)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** `ferro_ai::embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error>` — thin pass-through to `client.embed(text)`, symmetric with `complete(client, prompt)` (`complete.rs`). No batching, normalization, or retry.

**D-02:** Re-export from `lib.rs` as `pub use embed::embed;` (new `embed.rs` module), matching the `pub use complete::complete;` pattern.

**D-03:** `Error::Unsupported` propagated unchanged when the underlying client returns it (Anthropic). No special-casing.

**D-04:** `ferro_ai::cosine_similarity(a: &[f32], b: &[f32]) -> f32`, pure Rust, no new crates. Lives in new `similarity.rs` module, re-exported from `lib.rs`.

**D-05:** PANIC (not `Result`) on invalid input: panic with a clear message on empty slices and on dimension mismatch.

**D-06:** Returns value in `[-1.0, 1.0]`. Tests assert: orthogonal → `0.0`, identical → `1.0`, opposite → `-1.0` (SC#5). Epsilon tolerance in float tests.

**D-07:** `PgVectorStore::store` and `::nearest` accept `&sqlx::PgPool` as the executor type (NOT sea-orm). Note: `PgExecutor` is not dyn-compatible per docs.rs, so the concrete `&PgPool` type is the practical choice.

**D-08:** Distance metric defaults to COSINE distance (pgvector `<=>` operator). Returned score is cosine SIMILARITY (`1 - distance`) so callers reason in `[-1,1]` space matching D-04.

**D-09:** Schema is CALLER-MANAGED. `PgVectorStore` is query-only: no `CREATE EXTENSION`, `CREATE TABLE`, or DDL. Module docs include setup SQL as documentation only.

**D-10:** `store` takes `id: i64` + `embedding: &[f32]`; `nearest(pool, query: &[f32], k: i64)` returns `Vec<Neighbor>` where `Neighbor { id: i64, score: f32 }`.

**D-11:** The `pgvector` cargo feature activates two optional dependencies: `pgvector = "0.4"` AND `sqlx` (postgres + tokio runtime features). Non-`pgvector` builds pull neither.

**D-12 (SC#4 reconciliation):** SC#4 "adds only pgvector 0.4" is reconciled as: the only new vector-specific direct dependency is `pgvector 0.4`; `sqlx` is the unavoidable transport for the public API. Document in feature doc comment, not a contortion.

**D-13 (discrepancy to fix in 167):** `OllamaClient::embed` currently sends `default_model()` (`llama3.1`) to `/api/embed` — a chat model, not an embedding model. Fix: resolve embedding model independently via `FERRO_AI_EMBED_MODEL` env var, with provider-specific defaults (`nomic-embed-text` for Ollama, `text-embedding-3-small` for OpenAI). Single env knob, no new config surface.

### Claude's Discretion
- Exact executor bound for D-07 (`&PgPool` vs generic `impl PgExecutor`) — research below resolves this to `&PgPool` (concrete, dyn-incompatible trait).
- Module file layout (`embed.rs` / `similarity.rs` / `pgvector/mod.rs` vs flatter).
- Whether `Neighbor.id` is concrete `i64` or generic — bias to concrete for v1.
- Test strategy for `pgvector` (postgres integration test gated; cosine_similarity unit tests are mandatory and unconditional).

### Deferred Ideas (OUT OF SCOPE)
- Batch embedding (`embed_many`)
- Metadata/payload column on `PgVectorStore` rows + filtered nearest queries
- Generic-over-id `PgVectorStore<Id>`
- Index-management helpers
- In-memory `VectorStore` using `cosine_similarity`
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISDK-04 | Developer can generate text embeddings and compute cosine similarity (pure Rust helpers, zero extra crates). | Verified: `embed()` wraps existing `LlmClient::embed()`; `cosine_similarity` is pure arithmetic. No new crates needed for AISDK-04. |
| AISDK-05 | Developer can persist and query embeddings via pgvector (feature-gated `pgvector 0.4`, thin sqlx raw-query module). | Verified: `pgvector = "0.4"` + `sqlx = "0.8"` with `["postgres", "runtime-tokio"]` features. Feature-gating pattern confirmed from `ferro-deployments` precedent. |
</phase_requirements>

---

## Summary

Phase 167 adds three surfaces to `ferro-ai`, all of which extend the existing crate without any new crate or leaf-level architectural change:

1. **`embed()` free function** — a one-line delegate to `client.embed(text)`. The underlying provider implementations already exist (Phase 165). The only net-new work is the `embed.rs` module, the `lib.rs` re-export, fixing the Ollama embedding-model bug (D-13), and a unit test.

2. **`cosine_similarity(a, b)`** — a pure Rust function with no dependencies. The formula is standard (dot product divided by product of magnitudes). The planner's main decision is the panic contract on programmer errors vs. the `NaN` behavior on zero-magnitude vectors (SC mandates panic only on empty/dimension-mismatch, not zero-magnitude).

3. **`pgvector` feature-gated module** — wraps `pgvector 0.4` + `sqlx 0.8`. The workspace already has sqlx 0.8.6 in `Cargo.lock` (from `sea-orm`), so no version conflict arises. `pgvector` crate 0.4.2 (latest) supports `sqlx >= 0.8, < 0.10`. The concrete executor type is `&sqlx::PgPool` because `PgExecutor` is not dyn-compatible.

**Primary recommendation:** Build in wave order: (Wave 1) D-13 Ollama embed fix + `embed.rs` free function; (Wave 2) `similarity.rs` + unit tests; (Wave 3) `pgvector/mod.rs` feature-gated store; (Wave 4) integration test (gated) + `lib.rs` wiring + Cargo.toml feature.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `embed()` free function | ferro-ai leaf crate | — | Thin delegate; no HTTP, no DB — just a public entry point over existing `LlmClient::embed()` |
| `cosine_similarity()` | ferro-ai leaf crate | — | Pure arithmetic; no IO. Zero tier-crossing needed. |
| Embedding model resolution (D-13) | Provider client (OllamaClient, OpenAiClient) | AiConfig env layer | Providers own their model-resolution logic; env var is the config seam per FERRO_AI_* convention |
| `PgVectorStore` | ferro-ai leaf crate (pgvector feature) | Caller's Postgres DB | ferro-ai issues raw SQL; caller owns schema, migrations, and connection pool |
| pgvector SQL execution | Postgres DB server | — | `<=>` operator is server-side; pgvector extension must be installed by the DBA/caller |

---

## Standard Stack

### Core (AISDK-04 — no new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none) | — | `embed()` + `cosine_similarity` add zero new crates — AISDK-04 requirement is explicit | Pure Rust arithmetic and a one-line delegate |

### Feature-gated (AISDK-05 — `pgvector` feature only)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| pgvector | 0.4.2 | `Vector` type with `sqlx::Encode`/`Decode`/`Type` impls for Postgres | The only maintained Rust pgvector integration crate [VERIFIED: crates.io 2026-06-08] |
| sqlx | 0.8.6 (already in lock) | `PgPool`, `query`, `query_as`, async Postgres execution | Already in workspace Cargo.lock via sea-orm; version 0.8.6 is compatible with pgvector 0.4 (`>=0.8, <0.10`) [VERIFIED: docs.rs/pgvector 0.4.2] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `&sqlx::PgPool` | `impl sqlx::PgExecutor<'_>` | `PgExecutor` is not dyn-compatible (verified docs.rs 0.8.6). Using the concrete `PgPool` is simpler and avoids lifetime complexity for a v1 API. |
| `pgvector 0.4` | `pgvecto.rs` crate | pgvecto.rs requires a different Postgres extension (not the standard `pgvector` extension). pgvector is the de-facto standard. |
| Raw `sqlx::query` | `sqlx::query_as!` macro | The macro requires compile-time DB connection (not feasible for an optional feature in a library crate). Runtime `query` + manual `FromRow` is the correct choice here. |

**Cargo.toml additions (ferro-ai/Cargo.toml):**
```toml
[dependencies]
# ... existing deps unchanged ...
pgvector    = { version = "0.4", features = ["sqlx"], optional = true }
sqlx        = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }

[features]
pgvector = ["dep:pgvector", "dep:sqlx"]
```

**Version verification:**
- `pgvector` latest: 0.4.2 (released 2026-05-22) [VERIFIED: crates.io]
- `sqlx` in workspace Cargo.lock: 0.8.6 [VERIFIED: grep Cargo.lock]
- `sqlx` latest overall: 0.9.0 (released 2026-05-21) — workspace pins 0.8.6 via sea-orm; specifying `"0.8"` for ferro-ai matches the locked version and avoids a dual-sqlx compile.

---

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  Caller (application handler or test)                           │
│                                                                 │
│   ferro_ai::embed(client, text)  ──────────────────────────────┐│
│   ferro_ai::cosine_similarity(a, b)                            ││
│   ferro_ai::pgvector::PgVectorStore::store(pool, id, emb) ──┐  ││
│   ferro_ai::pgvector::PgVectorStore::nearest(pool, q, k) ──┐│  ││
└────────────────────────────────────────────────────────────│┼──┼┘
                                                             ││  │
                 ┌───────────────────────────────────────────┘│  │
                 │              pgvector feature gate          │  │
                 │  ferro-ai/src/pgvector/mod.rs              │  │
                 │  ┌─────────────────────────────────┐       │  │
                 │  │ PgVectorStore                   │       │  │
                 │  │   store(pool, id, emb)          │       │  │
                 │  │     sqlx::query INSERT $1,$2    │       │  │
                 │  │   nearest(pool, query, k)       │       │  │
                 │  │     sqlx::query_as SELECT       │       │  │
                 │  │     ORDER BY emb <=> $1 LIMIT $2│       │  │
                 │  └───────────────┬─────────────────┘       │  │
                 │                  │ &sqlx::PgPool            │  │
                 │                  ▼                          │  │
                 │       Postgres DB + pgvector ext            │  │
                 └─────────────────────────────────────────────┘  │
                                                                   │
            ferro-ai/src/embed.rs ─────────────────────────────────┘
            pub async fn embed(client: &dyn LlmClient, text: &str)
                           │
                           ▼
            client.embed(text)  ──► OpenAiClient  → /v1/embeddings (text-embedding-3-small)
                                ──► OllamaClient   → /api/embed (nomic-embed-text via FERRO_AI_EMBED_MODEL)
                                ──► AnthropicClient → Err(Error::Unsupported)
```

### Recommended Project Structure

```
ferro-ai/src/
├── embed.rs           # pub async fn embed(client, text) — new (mirrors complete.rs)
├── similarity.rs      # pub fn cosine_similarity(a, b) — new
├── pgvector/
│   └── mod.rs         # PgVectorStore, Neighbor — new, cfg(feature="pgvector")
├── client/
│   ├── mod.rs         # LlmClient trait — add embed_model() default resolver hint in docs
│   ├── openai.rs      # embed() — update to use embed_model() not hardcoded constant
│   └── ollama.rs      # embed() — fix: use embed_model() not default_model()
├── config.rs          # AiConfig::from_env() — read FERRO_AI_EMBED_MODEL (D-13)
├── error.rs           # Error enum — add Error::Sqlx(String) variant (pgvector feature only)
└── lib.rs             # pub use embed::embed, cosine_similarity, pgvector feature-gated re-exports
```

### Pattern 1: `embed()` free function (mirrors `complete()`)

```rust
// ferro-ai/src/embed.rs
// Source: mirrors complete.rs pattern (verified in codebase 2026-06-08)

use crate::client::LlmClient;
use crate::error::Error;

/// Generate a text embedding vector using the configured LLM provider.
///
/// Returns `Err(Error::Unsupported)` for providers without an embeddings
/// endpoint (e.g. `AnthropicClient`).
pub async fn embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error> {
    client.embed(text).await
}
```

### Pattern 2: `cosine_similarity` — pure Rust formula

```rust
// ferro-ai/src/similarity.rs
// Source: standard cosine similarity formula, verified against AISDK-04 "zero extra crates"

/// Compute cosine similarity between two embedding vectors.
///
/// Returns a value in `[-1.0, 1.0]`: `1.0` for identical direction,
/// `0.0` for orthogonal, `-1.0` for opposite direction.
///
/// # Panics
///
/// Panics if either slice is empty, or if the slices have different lengths.
/// These are programmer errors — callers are responsible for providing
/// valid, dimension-consistent embeddings from the same model.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert!(!a.is_empty(), "cosine_similarity: empty slice `a`");
    assert!(!b.is_empty(), "cosine_similarity: empty slice `b`");
    assert_eq!(
        a.len(),
        b.len(),
        "cosine_similarity: dimension mismatch ({} vs {})",
        a.len(),
        b.len()
    );

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    // Note: SC mandates panic only on empty/dim-mismatch, NOT on zero-magnitude vectors.
    // Zero-magnitude produces NaN via 0.0/0.0 — leave as-is for v1.
    dot / (mag_a * mag_b)
}
```

**Float edge cases (important for planner):**
- Empty slice → panic (SC#2, D-05) — covered by assert.
- Dimension mismatch → panic (SC#5, D-05) — covered by assert_eq!.
- Zero-magnitude vector — NOT a panic per SC/D-05. Division by 0.0 produces `NaN`. The planner may document this in the function's rustdoc as a known behavior but must NOT add a panic for it (out of scope per locked decisions).

### Pattern 3: pgvector store + nearest query

```rust
// ferro-ai/src/pgvector/mod.rs
// Source: pgvector 0.4.2 docs.rs [VERIFIED 2026-06-08] + sqlx 0.8.6 patterns

#[cfg(feature = "pgvector")]
use pgvector::Vector;
#[cfg(feature = "pgvector")]
use sqlx::PgPool;
#[cfg(feature = "pgvector")]
use crate::error::Error;

/// A row returned by `PgVectorStore::nearest`.
#[cfg(feature = "pgvector")]
pub struct Neighbor {
    pub id: i64,
    /// Cosine similarity in `[-1.0, 1.0]`; computed as `1 - cosine_distance`.
    pub score: f32,
}

#[cfg(feature = "pgvector")]
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Neighbor {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            score: row.try_get::<f32, _>("score")?,
        })
    }
}

/// Thin pgvector query primitive.
///
/// Caller-managed schema. Run once before using this store:
///
/// ```sql
/// CREATE EXTENSION IF NOT EXISTS vector;
///
/// CREATE TABLE embeddings (
///     id      BIGINT PRIMARY KEY,
///     vec     vector(1536)   -- dimension must match your embedding model
/// );
///
/// -- Recommended index (cosine distance):
/// CREATE INDEX ON embeddings USING hnsw (vec vector_cosine_ops);
/// ```
#[cfg(feature = "pgvector")]
pub struct PgVectorStore {
    table: String,
    column: String,
}

#[cfg(feature = "pgvector")]
impl PgVectorStore {
    /// Create a store pointing at `table.column` for vectors.
    pub fn new(table: &str, column: &str) -> Self {
        Self { table: table.to_string(), column: column.to_string() }
    }

    /// Insert or replace the embedding for `id` in the table.
    pub async fn store(
        &self,
        pool: &PgPool,
        id: i64,
        embedding: &[f32],
    ) -> Result<(), Error> {
        let vec = Vector::from(embedding.to_vec());
        let sql = format!(
            "INSERT INTO {} (id, {}) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET {} = $2",
            self.table, self.column, self.column
        );
        sqlx::query(&sql)
            .bind(id)
            .bind(vec)
            .execute(pool)
            .await
            .map_err(|e| Error::Sqlx(e.to_string()))?;
        Ok(())
    }

    /// Return the `k` nearest embeddings by cosine similarity.
    ///
    /// Scores are returned as `1 - cosine_distance` (range `[-1, 1]`).
    pub async fn nearest(
        &self,
        pool: &PgPool,
        query: &[f32],
        k: i64,
    ) -> Result<Vec<Neighbor>, Error> {
        let vec = Vector::from(query.to_vec());
        let sql = format!(
            "SELECT id, (1.0 - ({} <=> $1))::float4 AS score FROM {} ORDER BY {} <=> $1 LIMIT $2",
            self.column, self.table, self.column
        );
        sqlx::query_as::<_, Neighbor>(&sql)
            .bind(vec)
            .bind(k)
            .fetch_all(pool)
            .await
            .map_err(|e| Error::Sqlx(e.to_string()))
    }
}
```

### Pattern 4: D-13 embedding-model fix in providers

```rust
// ferry-ai/src/client/ollama.rs (modified section)
// FERRO_AI_EMBED_MODEL resolution — source: D-13 decision, FERRO_AI_* convention

impl OllamaClient {
    /// The embedding model to use for `/api/embed`.
    ///
    /// Reads `FERRO_AI_EMBED_MODEL`; falls back to `"nomic-embed-text"`.
    /// Intentionally separate from `default_model()` (chat model).
    pub(crate) fn embed_model() -> String {
        std::env::var("FERRO_AI_EMBED_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string())
    }

    // In embed(&self, text): replace `self.default_model().to_string()` with `Self::embed_model()`
}

// ferry-ai/src/client/openai.rs (same pattern)
impl OpenAiClient {
    pub(crate) fn embed_model() -> String {
        std::env::var("FERRO_AI_EMBED_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string())
    }
}
```

### Pattern 5: `lib.rs` additions

```rust
// ferry-ai/src/lib.rs additions
pub mod embed;
pub mod similarity;
#[cfg(feature = "pgvector")]
pub mod pgvector;

pub use embed::embed;
pub use similarity::cosine_similarity;
#[cfg(feature = "pgvector")]
pub use pgvector::{Neighbor, PgVectorStore};
```

### Anti-Patterns to Avoid

- **Sending chat model to `/api/embed`:** `OllamaClient::embed` currently does `self.default_model()`. This sends `"llama3.1"` to `/api/embed`, which either fails or produces nonsense vectors. Fix is mandatory for correctness (D-13).
- **Using `impl PgExecutor<'_>` in public API:** `PgExecutor` is not dyn-compatible per docs.rs 0.8.6. Using it in a public API requires complex lifetime annotations and prevents object-safe wrappers. Use `&PgPool` directly.
- **Using `sqlx::query_as!` macro:** Requires a live database connection at compile time. Not suitable for optional-feature library code. Use runtime `query_as::<_, T>()`.
- **Adding `sqlx` to non-pgvector builds:** The feature gate must ensure neither `pgvector` nor `sqlx` appear in `cargo tree` without `--features pgvector`.
- **Cosine distance vs. cosine similarity:** pgvector's `<=>` returns **distance** (0 = identical, 2 = opposite). The store must compute `1 - distance` to return similarity. The free function `cosine_similarity` returns similarity directly. Both use the same `[-1, 1]` convention.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Postgres vector type encoding/decoding | Custom `ToSql`/`FromSql` for `Vec<f32>` | `pgvector::Vector` with `features = ["sqlx"]` | pgvector 0.4 provides `Encode`, `Decode`, `Type` impls — hand-rolled impls would require matching the binary wire format exactly (16-bit length prefix + f32 array) [VERIFIED: docs.rs] |
| Cosine similarity computation | — | Pure Rust formula in `similarity.rs` | Zero dependencies is the AISDK-04 requirement; the formula is 5 lines |
| Nearest-neighbor index management | DDL helpers inside `PgVectorStore` | Caller SQL in docs | D-09: schema is caller-managed; adding DDL helpers would couple `ferro-ai` to migration strategy |

**Key insight:** For `PgVectorStore`, the only non-trivial hand-rolled piece is the `FromRow` impl for `Neighbor`. Everything else (encoding, distance operators, async execution) is provided by the two deps.

---

## Common Pitfalls

### Pitfall 1: Chat model sent to embedding endpoint (D-13 bug)
**What goes wrong:** `OllamaClient::embed` uses `self.default_model()` which returns `"llama3.1"`. Ollama's `/api/embed` will accept the request but `llama3.1` has no embedding head — it either returns an error or garbage vectors.
**Why it happens:** Phase 165 implemented `embed()` mechanically by reusing the existing model field without considering that chat models and embedding models are distinct.
**How to avoid:** Introduce `embed_model()` on each provider that reads `FERRO_AI_EMBED_MODEL` with a provider-appropriate default. Fix in Wave 1 before any integration tests are written.
**Warning signs:** Integration test returns 768-dim or 4096-dim vectors instead of 768-dim `nomic-embed-text` output; similarity scores are nonsensical.

### Pitfall 2: Feature gate not watertight
**What goes wrong:** `pgvector` or `sqlx` symbols appear in a build without `--features pgvector`, causing compilation failure or unexpected dependency in the dependency graph.
**Why it happens:** Missing `#[cfg(feature = "pgvector")]` on the module declaration, the re-exports in `lib.rs`, or the error variant in `error.rs`.
**How to avoid:** Wrap EVERY item touched by the feature in `#[cfg(feature = "pgvector")]`. Run `cargo tree -p ferro-ai` without the feature and verify neither `pgvector` nor `sqlx` appears.
**Warning signs:** CI `--all-features` compiles but default build has unexpected sqlx in tree.

### Pitfall 3: Error variant leaking outside feature gate
**What goes wrong:** `Error::Sqlx(String)` variant is added to the shared `Error` enum (in `error.rs`) unconditionally. Callers not using the `pgvector` feature see an `Error::Sqlx` variant they can never trigger.
**Why it happens:** Rust enums have no feature-gated variants syntax — `#[cfg]` must be applied to the entire `Error` enum or the variant must be added unconditionally.
**How to avoid:** The recommended approach is to add `Error::Sqlx(String)` unconditionally with a doc comment noting it is only reachable when the `pgvector` feature is enabled. This is cleaner than a `#[cfg]` attribute on an enum variant (which is allowed but harder to read). Alternatively, convert sqlx errors to `Error::Provider` with no status — consistent with the existing error shape.
**Warning signs:** Pattern-match exhaustiveness warnings in non-pgvector consumers.

### Pitfall 4: Cosine distance vs. cosine similarity inversion
**What goes wrong:** `nearest` returns raw `<=>` operator output (0.0 for identical, 2.0 for opposite). Callers expect `cosine_similarity`-consistent values (1.0 for identical, -1.0 for opposite).
**Why it happens:** pgvector `<=>` computes 1 - cos(angle), not cos(angle). The `PgVectorStore` must invert.
**How to avoid:** The SQL SELECT must include `(1.0 - (col <=> $1))::float4 AS score`. See Pattern 3 above.
**Warning signs:** `nearest` returns 0.0 for the same vector that `cosine_similarity` returns 1.0 for.

### Pitfall 5: NaN from zero-magnitude vector
**What goes wrong:** `cosine_similarity([0.0, 0.0], [1.0, 0.0])` returns `NaN` because `0.0 / (0.0 * 1.0)` = `NaN`. The function does NOT panic per SC (empty/dim-mismatch are the only panic cases).
**Why it happens:** Division by zero in floating point produces NaN, not a panic, in Rust.
**How to avoid:** Document this clearly in the rustdoc. Do NOT add a panic for zero-magnitude (out of scope). Callers embedding real text will not produce zero-magnitude vectors in practice.
**Warning signs:** Test assertion `assert_eq!(result, 0.0)` fails when result is NaN.

### Pitfall 6: sqlx version conflict
**What goes wrong:** Specifying `sqlx = "0.9"` in ferro-ai's optional deps while the workspace lockfile has `0.8.6` (from sea-orm) causes Cargo to compile two versions of sqlx.
**Why it happens:** sea-orm pins sqlx 0.8.x; specifying 0.9 would add a parallel dependency.
**How to avoid:** Use `sqlx = { version = "0.8", ... }` to match the locked version. `pgvector 0.4.2` accepts `>=0.8, <0.10` — both are compatible [VERIFIED: docs.rs].

---

## Code Examples

### INSERT vector (verified from pgvector docs.rs + README 2026-06-08)
```rust
// Source: https://docs.rs/pgvector/0.4.2 + https://github.com/pgvector/pgvector-rust/README.md
use pgvector::Vector;

let embedding = Vector::from(vec![1.0, 2.0, 3.0]);
sqlx::query("INSERT INTO items (id, vec) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET vec = $2")
    .bind(id)
    .bind(embedding)
    .execute(pool)
    .await?;
```

### SELECT nearest neighbors by cosine similarity (verified operators)
```rust
// Source: pgvector docs.rs — <=> is cosine distance [VERIFIED]
// 1 - distance = cosine similarity in [-1, 1]
let rows = sqlx::query_as::<_, Neighbor>(
    "SELECT id, (1.0 - (vec <=> $1))::float4 AS score FROM items ORDER BY vec <=> $1 LIMIT $2"
)
.bind(Vector::from(query_vec))
.bind(k as i64)
.fetch_all(pool)
.await?;
```

### Cosine similarity unit test pattern
```rust
// Source: standard math + D-06 locked test contract
const EPSILON: f32 = 1e-6;

#[test]
fn identical_vectors() {
    let v = vec![1.0f32, 0.0, 0.0];
    let s = cosine_similarity(&v, &v);
    assert!((s - 1.0).abs() < EPSILON, "identical: expected 1.0, got {s}");
}

#[test]
fn orthogonal_vectors() {
    let a = vec![1.0f32, 0.0];
    let b = vec![0.0f32, 1.0];
    let s = cosine_similarity(&a, &b);
    assert!(s.abs() < EPSILON, "orthogonal: expected 0.0, got {s}");
}

#[test]
fn opposite_vectors() {
    let a = vec![1.0f32, 0.0];
    let b = vec![-1.0f32, 0.0];
    let s = cosine_similarity(&a, &b);
    assert!((s - (-1.0)).abs() < EPSILON, "opposite: expected -1.0, got {s}");
}

#[test]
#[should_panic(expected = "empty slice")]
fn panics_on_empty() {
    cosine_similarity(&[], &[]);
}

#[test]
#[should_panic(expected = "dimension mismatch")]
fn panics_on_dim_mismatch() {
    cosine_similarity(&[1.0], &[1.0, 2.0]);
}
```

### `embed()` unit test using mock client
```rust
// Source: mirrors complete.rs test pattern (verified in codebase)
#[tokio::test]
async fn embed_delegates_to_client() {
    struct ConstEmbedClient;
    #[async_trait]
    impl LlmClient for ConstEmbedClient {
        fn default_model(&self) -> &str { "test" }
        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> { Err(Error::Unsupported) }
        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> { Err(Error::Unsupported) }
        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> { Ok(vec![0.1, 0.2, 0.3]) }
    }
    let client = ConstEmbedClient;
    let result = embed(&client, "hello").await.unwrap();
    assert_eq!(result, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn embed_propagates_unsupported() {
    struct NoEmbedClient;
    #[async_trait]
    impl LlmClient for NoEmbedClient {
        fn default_model(&self) -> &str { "test" }
        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> { Err(Error::Unsupported) }
        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> { Err(Error::Unsupported) }
        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> { Err(Error::Unsupported) }
    }
    let client = NoEmbedClient;
    let result = embed(&client, "hello").await;
    assert!(matches!(result, Err(Error::Unsupported)));
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Ollama `/api/embed` with chat model | `/api/embed` with explicit embedding model (`nomic-embed-text`) | This phase (D-13 fix) | Ollama embeddings become semantically meaningful |
| OpenAI hardcoded `text-embedding-3-small` | Same default, but env-overridable via `FERRO_AI_EMBED_MODEL` | This phase (D-13 fix) | Callers can use `text-embedding-3-large` or other models |
| pgvector 0.3 (sqlx 0.7) | pgvector 0.4 (sqlx 0.8+) | May 2026 | The 0.4 release adds `>=0.8, <0.10` sqlx support — compatible with workspace's 0.8.6 |
| `sqlx` latest 0.9.0 | Must use 0.8.x to match workspace lockfile | May 2026 | Avoids duplicate sqlx compilation; sea-orm pins 0.8.x |

**Deprecated/outdated:**
- `pgvector = "0.3"`: requires sqlx 0.7, incompatible with this workspace. Do not use.
- `runtime-tokio-native-tls` combined feature: replaced in 0.8 by separate `runtime-tokio` + `tls-native-tls`. Use `runtime-tokio` only (no TLS needed for the optional dep).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `PgExecutor` is not dyn-compatible — therefore `&PgPool` is the right executor type | Standard Stack, D-07 | If wrong, the API could use a generic bound; LOW risk — docs.rs explicitly states "not dyn compatible" [CITED: docs.rs/sqlx/0.8.6] |
| A2 | The workspace's 0.8.6 sqlx (from sea-orm) will satisfy the `sqlx = { version = "0.8" }` range and Cargo will not pull a second copy | Standard Stack, Pitfall 6 | If wrong, there could be a duplicate sqlx; MEDIUM risk — needs `cargo tree` check in Wave 1 |
| A3 | `FERRO_AI_EMBED_MODEL` as the single env knob for embedding model satisfies D-13 with zero new config surface | Pattern 4, D-13 | If wrong, a separate `OllamaEmbedConfig` might be needed; LOW risk — the existing `AiConfig::from_env` reads `FERRO_AI_*` env vars with the same pattern |
| A4 | `nomic-embed-text` is a valid, commonly available Ollama embedding model (not pull-on-demand only) | D-13, Pattern 4 | LOW risk — ollama.com/library/nomic-embed-text lists it as a standard library model [CITED: ollama.com/library/nomic-embed-text] |

**If this table is empty after A4:** All claims in this research were verified or cited — no user confirmation needed for the core stack.

---

## Open Questions

1. **Error::Sqlx variant placement**
   - What we know: `error.rs` has one `Error` enum per crate (thiserror convention). Rust allows `#[cfg]` on enum variants but it is unusual.
   - What's unclear: Whether to add `Error::Sqlx(String)` unconditionally or gate it.
   - Recommendation: Add unconditionally with a doc comment. The variant is trivially additive (does not change existing match arms for existing callers). Alternative: re-use `Error::Provider { status: None, message }` for sqlx errors to avoid adding a variant at all — document the intent with a comment. Planner decides.

2. **`PgVectorStore` constructor — table/column config vs. fixed schema**
   - What we know: D-09 says schema is caller-managed. D-10 specifies `id: i64`.
   - What's unclear: Whether the constructor should accept table/column names (flexible) or assume fixed `(id, vec)` column names.
   - Recommendation: Accept `table: &str, column: &str` args to avoid hardcoding column names. This keeps the store usable for multiple tables in one application without subclassing.

3. **`store()` upsert vs. insert-only semantics**
   - What we know: D-10 says "store one row".
   - What's unclear: Whether to use `INSERT ... ON CONFLICT DO UPDATE` (upsert) or fail on duplicate id.
   - Recommendation: Upsert (`ON CONFLICT (id) DO UPDATE SET col = $2`) — matches the "store" verb semantics and is safer for idempotent callers. Planner confirms.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | 1.88.0 (workspace rust-version) | — |
| PostgreSQL + pgvector ext | Integration tests only | ✗ (not checked — CI has no Postgres service) | — | Integration tests gated behind `postgres-tests` feature + `DATABASE_URL` env check |
| `pgvector` crate | `pgvector` feature builds | N/A (optional dep, not yet added) | 0.4.2 on crates.io | — |
| `sqlx` crate | `pgvector` feature builds | ✓ (0.8.6 in Cargo.lock via sea-orm) | 0.8.6 | — |

**Missing dependencies with no fallback:**
- None for AISDK-04 (zero new deps).
- For AISDK-05: a live Postgres with `pgvector` extension is required for the integration test; gated behind feature + env var (no CI blocker).

**Missing dependencies with fallback:**
- Postgres: integration tests use `DATABASE_URL` env-guard pattern identical to `ferro-deployments/tests/race_promote_postgres.rs` — tests skip gracefully when `DATABASE_URL` is unset.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `tokio::test` (already in ferro-ai dev-dependencies) |
| Config file | None — inline `#[test]` and `#[tokio::test]` |
| Quick run command | `cargo test -p ferro-ai` |
| Full suite command | `cargo test -p ferro-ai --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AISDK-04 | `cosine_similarity` correct for identical, orthogonal, opposite vectors | unit | `cargo test -p ferro-ai -p ferro-ai similarity` | ❌ Wave 0: `ferro-ai/src/similarity.rs` (inline tests) |
| AISDK-04 | `cosine_similarity` panics on empty slice | unit (should_panic) | `cargo test -p ferro-ai` | ❌ Wave 0: same |
| AISDK-04 | `cosine_similarity` panics on dimension mismatch | unit (should_panic) | `cargo test -p ferro-ai` | ❌ Wave 0: same |
| AISDK-04 | `embed()` free function delegates to client and propagates `Unsupported` | unit (mock) | `cargo test -p ferro-ai` | ❌ Wave 0: `ferro-ai/src/embed.rs` (inline tests) |
| AISDK-04 | D-13: Ollama embed uses `nomic-embed-text` by default, not chat model | unit | `cargo test -p ferro-ai` | ❌ Wave 0: extend `ferro-ai/src/client/ollama.rs` tests |
| AISDK-05 | `PgVectorStore::store` inserts a vector row | integration (gated) | `cargo test -p ferro-ai --features pgvector,postgres-tests` | ❌ Wave 0: `ferro-ai/tests/pgvector_integration.rs` |
| AISDK-05 | `PgVectorStore::nearest` returns rows ordered by cosine similarity | integration (gated) | same | ❌ Wave 0: same file |
| AISDK-05 | SC#4 dep-graph: no pgvector/sqlx without feature flag | cargo-tree check | `cargo tree -p ferro-ai \| grep -v pgvector` (manual or in plan verification) | ❌ Wave 0: add to plan verification checklist |
| AISDK-05 | `--all-features` compilation passes | CI compatibility | `cargo clippy -p ferro-ai --all-features -- -D warnings` | ✅ CI enforces this |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-ai` (unit tests, excludes integration)
- **Per wave merge:** `cargo test -p ferro-ai --all-features` (compiles pgvector feature; integration tests skip without `DATABASE_URL`)
- **Phase gate:** `cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/src/embed.rs` — `embed()` free function + inline `#[tokio::test]` tests (REQ AISDK-04)
- [ ] `ferro-ai/src/similarity.rs` — `cosine_similarity()` + inline `#[test]` tests including `#[should_panic]` cases (REQ AISDK-04)
- [ ] `ferro-ai/src/pgvector/mod.rs` — `PgVectorStore`, `Neighbor`, `FromRow` impl, behind `#[cfg(feature = "pgvector")]` (REQ AISDK-05)
- [ ] `ferro-ai/tests/pgvector_integration.rs` — gated behind `#![cfg(feature = "postgres-tests")]` + `DATABASE_URL` env check (REQ AISDK-05)
- [ ] `ferro-ai/Cargo.toml` — add `[features] pgvector = [...]` and optional deps
- [ ] D-13 fix in `ferro-ai/src/client/ollama.rs` — `embed_model()` helper + env lookup + updated `embed()` impl
- [ ] D-13 fix in `ferro-ai/src/client/openai.rs` — same pattern, default `text-embedding-3-small`
- [ ] `ferro-ai/src/error.rs` — add `Error::Sqlx(String)` (or use `Error::Provider` — planner decides per Open Question 1)

---

## Security Domain

> `security_enforcement` not set in config — treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No auth surface in this phase |
| V3 Session Management | No | Stateless functions |
| V4 Access Control | No | Library — caller owns access control |
| V5 Input Validation | Partial | `cosine_similarity` panics on programmer errors (empty/mismatch) — this is a precondition, not user input. `PgVectorStore` uses parameterized sqlx queries — no SQL injection possible. |
| V6 Cryptography | No | No crypto |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via table/column names | Tampering | Table and column names are provided at construction time by the caller (trusted code), NOT from user input. Document this constraint. If a future version accepts user-supplied names, sanitize via whitelist. [ASSUMED — no official guideline found; standard defensive coding] |
| Embedding content leakage in error messages | Information Disclosure | `Error::Sqlx(e.to_string())` must not include the query parameters (embeddings are `Vec<f32>` — not sensitive; table/column names are low-risk). Current pattern mirrors `Error::Provider { message: e.to_string() }` [VERIFIED: error.rs] |

---

## Sources

### Primary (HIGH confidence)
- `ferro-ai/src/client/ollama.rs` — confirmed `embed()` uses `self.default_model()` which returns `"llama3.1"` (D-13 bug confirmed) [VERIFIED: codebase read 2026-06-08]
- `ferro-ai/src/client/openai.rs` — confirmed `embed()` hardcodes `"text-embedding-3-small"` [VERIFIED: codebase read 2026-06-08]
- `ferro-ai/src/complete.rs` — exact pattern to mirror for `embed.rs` [VERIFIED: codebase read 2026-06-08]
- `ferro-ai/Cargo.toml` — existing deps confirmed; no sqlx or pgvector present [VERIFIED: codebase read 2026-06-08]
- `ferro-deployments/tests/race_promote_postgres.rs` — `#![cfg(feature = "postgres-tests")]` + `DATABASE_URL` guard precedent [VERIFIED: codebase read 2026-06-08]
- `Cargo.lock` — `sqlx 0.8.6` in workspace [VERIFIED: grep 2026-06-08]
- docs.rs/pgvector/0.4.2 — `Vector` type, `features = ["sqlx"]`, sqlx version constraint `>=0.8, <0.10`, no re-exports from sqlx [VERIFIED: WebFetch 2026-06-08]
- docs.rs/sqlx/0.8.6 — `PgExecutor` "not dyn compatible" [VERIFIED: WebFetch 2026-06-08]
- crates.io — pgvector 0.4.2 (latest, 2026-05-22); sqlx 0.9.0 (latest, 2026-05-21) [VERIFIED: WebFetch 2026-06-08]

### Secondary (MEDIUM confidence)
- pgvector/pgvector README.md — sqlx INSERT and SELECT patterns [CITED: raw.githubusercontent.com/pgvector/pgvector-rust/master/README.md]
- pgvector `<=>` operator = cosine distance, `1 - <=>` = cosine similarity [CITED: context7.com/pgvector/pgvector/llms.txt]
- sqlx features: `postgres` + `runtime-tokio` for async Postgres with tokio [CITED: docs.rs/sqlx/0.8.6]

### Tertiary (LOW confidence)
- Ollama `nomic-embed-text` default embedding model recommendation [CITED: ollama.com/library/nomic-embed-text via WebSearch]

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — versions verified against crates.io and Cargo.lock; no guessing
- Architecture: HIGH — patterns verified directly from existing codebase (`complete.rs`, `ferro-deployments` precedent)
- Pitfalls: HIGH — D-13 bug confirmed by direct code read; PgExecutor dyn-compat issue verified from docs.rs
- pgvector SQL operators: HIGH — verified from pgvector official docs + README

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (30 days — pgvector 0.4.x is stable; sqlx 0.8.x series is stable in workspace context)
