---
phase: 167-embeddings-pgvector
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - ferro-ai/Cargo.toml
  - ferro-ai/src/client/ollama.rs
  - ferro-ai/src/client/openai.rs
  - ferro-ai/src/config.rs
  - ferro-ai/src/embed.rs
  - ferro-ai/src/error.rs
  - ferro-ai/src/lib.rs
  - ferro-ai/src/pgvector/mod.rs
  - ferro-ai/src/similarity.rs
  - ferro-ai/tests/pgvector_integration.rs
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 167: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 167 adds cosine similarity helpers, an embed entry point, and a feature-gated pgvector store module to `ferro-ai`. The implementation is structurally sound: SQL injection is not a concern (all dynamic values are bound via `$1`/`$2` positional parameters; `table`/`column` are constructor-supplied trusted values), the D-13 embed-model fix is correct (both `OllamaClient::embed_model()` and `OpenAiClient::embed_model()` read `FERRO_AI_EMBED_MODEL` separately from `default_model()`), and feature gating is clean (`pgvector` and `sqlx` are `optional = true` with `dep:` syntax, and both the `pub mod pgvector` and the re-export in `lib.rs` are gated under `#[cfg(feature = "pgvector")]`). The project-agnostic crate rule is met — only `FERRO_AI_*` environment variables are read in new code. The cosine similarity formula is algebraically correct; the panic-on-empty and panic-on-mismatch contracts are sound for a trusted-caller API.

Two warning-level issues require fixes before release: the `k: i64` parameter type in `nearest()` allows negative values that are semantically invalid and produce a runtime database error rather than an early Rust-level rejection, and the `"id"` column name being hardcoded in `store()`/`nearest()` is an undocumented API constraint that will silently produce a runtime SQL error for callers whose primary key column is not named `"id"`. No critical issues were found.

## Warnings

### WR-01: `nearest()` accepts negative `k` — silent API contract violation

**File:** `ferro-ai/src/pgvector/mod.rs:130`
**Issue:** The public `nearest` signature is `k: i64`. Negative values (`k = -1`) pass Rust type-checking and compile cleanly, but Postgres rejects them at runtime with `ERROR: LIMIT must not be negative`, surfacing as `Error::Sqlx`. Callers cannot distinguish "connection error" from "I passed a nonsensical k" without inspecting the error string. The intent is clearly "number of neighbors to return" which is semantically a non-negative integer. Using `i64` for the public API implies all 64-bit signed integers are valid.

**Fix:** Either add an early guard or change the parameter type to `u32` (safe to cast to `i64` since `u32::MAX` fits in i64 without loss):

```rust
// Option A: guard (minimal signature change)
pub async fn nearest(
    &self,
    pool: &PgPool,
    query: &[f32],
    k: u32,
) -> Result<Vec<Neighbor>, Error> {
    let vec = Vector::from(query.to_vec());
    let sql = format!(
        "SELECT id, (1.0 - ({} <=> $1))::float4 AS score FROM {} ORDER BY {} <=> $1 LIMIT $2",
        self.column, self.table, self.column
    );
    sqlx::query_as::<_, Neighbor>(&sql)
        .bind(vec)
        .bind(k as i64)   // safe: u32::MAX = 4_294_967_295 fits in i64
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Sqlx(e.to_string()))
}
```

---

### WR-02: Hardcoded `"id"` column name in `store()` and `nearest()` — undocumented schema constraint

**File:** `ferro-ai/src/pgvector/mod.rs:104` and `134`

**Issue:** `store()` generates `INSERT INTO {table} (id, {column})` and `nearest()` generates `SELECT id, ... FROM {table}`. The `"id"` column name is hardcoded in the SQL string. `PgVectorStore::new(table, column)` accepts a configurable vector column but not a configurable id column. Callers whose table uses a different primary key name (e.g. `entity_id`, `record_id`) will get a runtime SQL error (`column "id" does not exist`) with no indication at construction time. The module-level doc example shows `id BIGINT PRIMARY KEY` but `PgVectorStore::new()`'s own rustdoc does not state this constraint.

**Fix (documentation-only, minimal):** Add the id-column constraint to `PgVectorStore::new()`'s rustdoc:

```rust
/// Creates a new store targeting `table.column` for vector operations.
///
/// `table` and `column` must be names of an existing Postgres table and
/// vector column. They are interpolated into SQL strings — supply only
/// trusted, application-controlled values (not user input).
///
/// # Schema requirements
///
/// The target table must have a `BIGINT` column named exactly `"id"` as its
/// primary key. The vector column is identified by the `column` parameter.
/// See the module-level documentation for the required `CREATE TABLE` statement.
pub fn new(table: &str, column: &str) -> Self {
```

Or (if wider table compatibility is needed), expose an `id_column` field:

```rust
pub struct PgVectorStore {
    table: String,
    column: String,
    id_column: String,  // defaults to "id"
}

impl PgVectorStore {
    pub fn new(table: &str, column: &str) -> Self {
        Self {
            table: table.to_string(),
            column: column.to_string(),
            id_column: "id".to_string(),
        }
    }

    pub fn with_id_column(mut self, id_column: &str) -> Self {
        self.id_column = id_column.to_string();
        self
    }
}
```

---

## Info

### IN-01: `Error::Sqlx` variant is not feature-gated — dead code in non-pgvector builds

**File:** `ferro-ai/src/error.rs:75`

**Issue:** `Error::Sqlx(String)` is always compiled regardless of the `pgvector` feature flag. In builds without `--features pgvector`, this variant is unreachable (nothing constructs it) and constitutes dead code. It does not cause a compile error or warning at default lint levels, but it adds a variant that pattern-matching callers see even when they have no pgvector feature enabled.

**Fix:** Gate the variant:

```rust
/// sqlx database error from the `pgvector` store.
#[cfg(feature = "pgvector")]
#[error("pgvector store error: {0}")]
Sqlx(String),
```

---

### IN-02: Zero-magnitude vector produces silent `NaN` from `cosine_similarity` — no test for documented behavior

**File:** `ferro-ai/src/similarity.rs:14-16`

**Issue:** The docstring correctly documents that a zero-magnitude vector yields `NaN` via `0.0 / 0.0` and explicitly states "Documented, not guarded." However there is no test asserting this documented behavior. Without a test, a future change (e.g. replacing `0.0 / 0.0` with a guard) could inadvertently break the documented contract in the other direction.

**Fix:** Add a test pinning the NaN contract:

```rust
#[test]
fn zero_magnitude_yields_nan() {
    let zero = vec![0.0f32, 0.0, 0.0];
    let other = vec![1.0f32, 0.0, 0.0];
    let s = cosine_similarity(&zero, &other);
    assert!(s.is_nan(), "zero-magnitude vector must yield NaN, got {s}");
}
```

---

### IN-03: `reqwest::Client::builder().expect()` in public constructors — panic on TLS init failure

**File:** `ferro-ai/src/client/ollama.rs:37`, `ferro-ai/src/client/openai.rs:43`

**Issue:** Both `OllamaClient::new()` and `OpenAiClient::new()` call `.expect("failed to build reqwest client")` on `reqwest::Client::builder().build()`. The `build()` call returns `Err` only when the TLS backend fails to initialize, which is extremely rare and essentially a misconfigured build. The `new()` signatures return `Self` (not `Result<Self>`), making `.expect()` the only option without a signature change. This is a pre-existing pattern — not new in phase 167 — and the behavior is acceptable for a framework where TLS initialization failure is a deployment-time misconfiguration. Flagged for awareness only.

**Fix (if desired):** Change constructors to `new() -> Result<Self, Error>` and propagate via `Error::Config`. Not required for this phase.

---

### IN-04: `FERRO_AI_EMBED_MODEL` env var undocumented in `AiConfig::from_env()` module-level table

**File:** `ferro-ai/src/config.rs:10-19`

**Issue:** The module-level doc table in `config.rs` lists four environment variables (`FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`, `FERRO_AI_BASE_URL`) but omits `FERRO_AI_EMBED_MODEL`, which is now a real operator-facing knob introduced in phase 167. A developer reading `config.rs` to understand the full env var surface will miss it.

**Fix:** Add the row to the doc table:

```
//! | `FERRO_AI_EMBED_MODEL` | No | provider default | Embedding model override (`nomic-embed-text` for Ollama, `text-embedding-3-small` for OpenAI) |
```

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
