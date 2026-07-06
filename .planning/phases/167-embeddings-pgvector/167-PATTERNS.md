# Phase 167: Embeddings & pgvector - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 8 (new/modified)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-ai/src/embed.rs` | utility (free fn) | request-response | `ferro-ai/src/complete.rs` | exact |
| `ferro-ai/src/similarity.rs` | utility (pure fn) | transform | `ferro-ai/src/error.rs` (inline `#[cfg(test)]` convention) | role-match |
| `ferro-ai/src/pgvector/mod.rs` | service | CRUD | `ferro-deployments/tests/race_promote_postgres.rs` + sqlx patterns in `ferro-deployments/Cargo.toml` | role-match |
| `ferro-ai/tests/pgvector_integration.rs` | test | CRUD | `ferro-deployments/tests/race_promote_postgres.rs` | exact |
| `ferro-ai/src/client/ollama.rs` (MODIFY) | service | request-response | itself (existing `embed()` impl lines 224-263) | exact |
| `ferro-ai/src/client/openai.rs` (MODIFY) | service | request-response | itself (existing `embed()` impl lines 289-328) | exact |
| `ferro-ai/src/error.rs` (MODIFY) | utility (error type) | — | itself (existing `Error` enum lines 1-67) | exact |
| `ferro-ai/src/lib.rs` (MODIFY) | config (re-exports) | — | itself (lines 44-67) + `complete.rs` pattern | exact |
| `ferro-ai/Cargo.toml` (MODIFY) | config | — | `ferro-deployments/Cargo.toml` `[features]` block (lines 31-34) | exact |

---

## Pattern Assignments

### `ferro-ai/src/embed.rs` (NEW — utility, request-response)

**Analog:** `ferro-ai/src/complete.rs`

**Module-level doc comment pattern** (complete.rs lines 1-38): the file opens with `//!` module docs
describing the public function's contract, internal flow, and error cases. `embed.rs` should follow
the same structure but is much simpler (one-line delegate, no schema steps).

**Imports pattern** (complete.rs lines 40-42):
```rust
use crate::client::LlmClient;
use crate::error::Error;
```
Only import what the function uses. `embed.rs` needs these two; no `schema` import (no schema step).

**Free function signature pattern** (complete.rs lines 57-81):
```rust
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    // ... body ...
}
```
`embed()` mirrors this shape without the generic: `pub async fn embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error>`.
Body is a single line: `client.embed(text).await`.

**Inline test module pattern** (complete.rs lines 83-143):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    use crate::client::{CompletionRequest, TokenStream};

    struct ConstClient(String);

    #[async_trait]
    impl LlmClient for ConstClient {
        fn default_model(&self) -> &str { "test" }
        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> {
            Ok(self.0.clone())
        }
        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> {
            Err(Error::Unsupported)
        }
        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> {
            Err(Error::Unsupported)
        }
    }

    #[tokio::test]
    async fn complete_returns_typed_result() { ... }
}
```
For `embed.rs` tests: define two mock `LlmClient` impls inline — one returning `Ok(vec![0.1, 0.2, 0.3])`,
one returning `Err(Error::Unsupported)`. Mock must implement ALL trait methods (including `complete`,
`complete_stream`); return `Err(Error::Unsupported)` for unused methods (see complete.rs line 113-116).
Use `#[tokio::test]` for the async tests. Note: `LlmClient` in client/mod.rs (lines 153+) also has
`complete_with_tools` with a default impl — mock only needs the required methods.

---

### `ferro-ai/src/similarity.rs` (NEW — utility, pure transform)

**Analog for module shape:** `ferro-ai/src/error.rs` (pure Rust, no async, inline `#[cfg(test)] mod tests`)

**Inline test convention** (error.rs lines 86-145):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(!Error::Provider { status: Some(400), message: "".into() }.is_retryable());
        // ...
    }
}
```
- `#[cfg(test)] mod tests` at the bottom of the file.
- `use super::*;` to import the module's public items.
- Synchronous `#[test]` (not `#[tokio::test]` — no async).
- `#[should_panic(expected = "...")]` for panic-contract tests — use the exact substring that appears
  in the panic message (e.g., `"empty slice"` and `"dimension mismatch"`).

**Float comparison pattern** (from RESEARCH.md Code Examples):
```rust
const EPSILON: f32 = 1e-6;
assert!((s - 1.0).abs() < EPSILON, "identical: expected 1.0, got {s}");
```
Do NOT use `assert_eq!` for `f32` comparisons. Use epsilon tolerance for all float assertions.

**`#[should_panic]` test pattern:**
```rust
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
The `expected` string must be a substring of the actual panic message. Panic messages in `embed.rs`
must use the exact phrases `"empty slice"` and `"dimension mismatch"` so these tests pass.

---

### `ferro-ai/src/pgvector/mod.rs` (NEW — service, CRUD, feature-gated)

**Analog for feature-gating:** `ferro-deployments/Cargo.toml` `[features]` block (lines 31-34)
and `ferro-deployments/tests/race_promote_postgres.rs` line 16.

**Feature gate application pattern:** every item in this file is gated. Two valid approaches:
1. `#[cfg(feature = "pgvector")]` on each `use`, `struct`, `impl` block individually.
2. A single `#[cfg(feature = "pgvector")]` wrapping the entire file content via `mod pgvector` gating in `lib.rs`.

The `lib.rs` approach (see Pattern Assignments for `lib.rs` below) uses `#[cfg(feature = "pgvector")] pub mod pgvector;`
which means the file itself need not repeat `#[cfg]` on every item — the entire module is excluded at compile time.
This is the cleaner pattern.

**sqlx raw query pattern** (from RESEARCH.md Pattern 3 — verified against pgvector docs.rs):
```rust
// INSERT with upsert (D-10: store semantics = upsert)
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

// SELECT nearest by cosine similarity (1 - distance to invert <=> operator)
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
```
Use `sqlx::query` (runtime, not `query_as!` macro) — the macro requires a live DB at compile time
which breaks optional-feature library crates.

**`FromRow` manual impl pattern** (no derive — `Neighbor` has a computed `score` column that is not
a direct table column, so `#[derive(FromRow)]` would not map `score` correctly):
```rust
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Neighbor {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            score: row.try_get::<f32, _>("score")?,
        })
    }
}
```

**Error mapping pattern** (mirrors `Error::Provider` mapping in ollama.rs lines 233-241):
```rust
.map_err(|e| {
    if e.is_timeout() {
        Error::Timeout
    } else {
        Error::Provider { status: None, message: e.to_string() }
    }
})
```
For sqlx errors from `PgVectorStore`, use `Error::Sqlx(e.to_string())` (the new variant — see
`error.rs` assignment below). No timeout check needed for sqlx (sqlx errors carry their own context).

---

### `ferro-ai/tests/pgvector_integration.rs` (NEW — integration test, gated)

**Analog:** `ferro-deployments/tests/race_promote_postgres.rs` (exact match)

**Feature gate declaration** (race_promote_postgres.rs line 16):
```rust
#![cfg(feature = "postgres-tests")]
```
Must be the FIRST line of the file (crate-level inner attribute). This makes the entire file a
no-op when the `postgres-tests` feature is not active — it compiles to an empty module.

**`DATABASE_URL` env-guard pattern** (race_promote_postgres.rs lines 44-51 and 62-63):
```rust
// Helper that returns None when DATABASE_URL is unset (graceful skip):
async fn fresh_pg_pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = sqlx::PgPool::connect(&url).await.expect("connect to postgres");
    Some(pool)
}

// In the test body — early return pattern:
#[tokio::test]
async fn store_and_nearest_roundtrip() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("DATABASE_URL not set — skipping pgvector integration test");
        return;
    }
    let pool = fresh_pg_pool().await.expect("DATABASE_URL checked above");
    // ...
}
```
Both the `Option`-returning helper AND the early-return guard in the test body are present in
the analog. Use the same two-layer guard here.

**Test header doc comment** (race_promote_postgres.rs lines 1-15):
```rust
//! SC-Nx: pgvector-gated integration tests.
//!
//! Run with:
//!   DATABASE_URL=postgres://user:pass@localhost/ferro_test \
//!     cargo test -p ferro-ai --features pgvector,postgres-tests
//!
//! Without the `postgres-tests` feature this file compiles to an empty module.
```

**Cargo.toml feature requirement:** the `postgres-tests` feature in `ferro-ai/Cargo.toml` must
enable `pgvector` (since the integration test uses `PgVectorStore`). Pattern from ferro-deployments:
```toml
[features]
postgres-tests = ["pgvector"]
```

---

### `ferro-ai/src/client/ollama.rs` (MODIFY — add `embed_model()`, fix `embed()`)

**Analog:** itself — `ferro-ai/src/client/ollama.rs`

**Existing `default_model()` pattern** (ollama.rs lines 120-122):
```rust
fn default_model(&self) -> &str {
    self.model.as_deref().unwrap_or("llama3.1")
}
```
The new `embed_model()` is a `pub(crate) fn` (not a trait method) that reads `FERRO_AI_EMBED_MODEL`
with a provider-specific default. It does NOT use `self` — it is a free associated function:
```rust
pub(crate) fn embed_model() -> String {
    std::env::var("FERRO_AI_EMBED_MODEL")
        .unwrap_or_else(|_| "nomic-embed-text".to_string())
}
```
`pub(crate)` matches the visibility of `build_body` (line 50) and `parse_ollama_line` (line 93) in the
same file.

**Existing `embed()` bug to fix** (ollama.rs lines 224-226):
```rust
async fn embed(&self, text: &str) -> Result<Vec<f32>, Error> {
    let model = self.default_model().to_string();  // BUG: chat model, not embed model
    let body = serde_json::json!({
        "model": model,
        "input": text,
    });
```
Change `self.default_model().to_string()` to `Self::embed_model()`.

**AiConfig env-var pattern** (config.rs lines 43-48):
```rust
let provider = std::env::var("FERRO_AI_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
let model = std::env::var("FERRO_AI_MODEL").ok();
```
`embed_model()` follows the same `std::env::var("FERRO_AI_*").unwrap_or_else(|_| default.to_string())`
shape. No new config struct needed — single `std::env::var` call per provider.

**Test pattern for the fix** (ollama.rs lines 267-332 — extend existing `#[cfg(test)] mod tests`):
Add a test asserting `embed_model()` returns `"nomic-embed-text"` when `FERRO_AI_EMBED_MODEL` is unset,
and returns the env var value when set. Use the `ENV_LOCK: Mutex<()>` pattern from `config.rs`
(lines 79-82) to serialize env-var tests:
```rust
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn embed_model_default_is_nomic() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("FERRO_AI_EMBED_MODEL");
    assert_eq!(OllamaClient::embed_model(), "nomic-embed-text");
    // no set_var call needed — remove_var ensures clean state
}
```

---

### `ferro-ai/src/client/openai.rs` (MODIFY — add `embed_model()`, fix `embed()`)

**Analog:** itself — `ferro-ai/src/client/openai.rs`

**Existing `embed()` hardcode to fix** (openai.rs lines 289-292):
```rust
async fn embed(&self, text: &str) -> Result<Vec<f32>, Error> {
    let body = serde_json::json!({
        "model": "text-embedding-3-small",   // hardcoded — replace with Self::embed_model()
        "input": text,
    });
```

**New `embed_model()` function** — same shape as Ollama's but with OpenAI's default:
```rust
pub(crate) fn embed_model() -> String {
    std::env::var("FERRO_AI_EMBED_MODEL")
        .unwrap_or_else(|_| "text-embedding-3-small".to_string())
}
```
Place in the `impl OpenAiClient` block, alongside `build_body` (line 57). `pub(crate)` matches
the visibility of `build_body` and `parse_openai_tool_calls` in the same file.

**Existing test convention** (openai.rs lines 388-624): extend the existing `#[cfg(test)] mod tests`
block — do NOT create a new test module. Add `embed_model_default` and `embed_model_from_env` tests
using the same `ENV_LOCK` serialization pattern.

---

### `ferro-ai/src/error.rs` (MODIFY — add `Error::Sqlx` variant)

**Analog:** itself — `ferro-ai/src/error.rs`

**Existing variant shape** (error.rs lines 1-67):
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ai config error: {0}")]
    Config(String),

    #[error("ai provider error ({status:?}): {message}")]
    Provider { status: Option<u16>, message: String },

    #[error("capability not supported by this provider")]
    Unsupported,

    // ... more variants ...

    #[error("confirmation store error: {0}")]
    StoreError(String),
}
```

**New variant to add** — append after `StoreError`, before the closing brace:
```rust
/// sqlx database error from the `pgvector` store.
///
/// Only reachable when the `pgvector` feature is enabled and
/// `PgVectorStore::store` or `PgVectorStore::nearest` is called.
/// The message is `sqlx::Error::to_string()` — does not contain
/// embedding data (f32 arrays are not included in sqlx error messages).
#[error("pgvector store error: {0}")]
Sqlx(String),
```
Add it unconditionally (no `#[cfg]` on the variant — Rust allows `#[cfg]` on enum variants
but it is unusual and causes match exhaustiveness confusion). The doc comment explains the
`pgvector`-only reachability. This matches the existing `StoreError(String)` pattern (line 67)
in style and type shape.

**`is_retryable()` method** (error.rs lines 69-83): the `match self { ... }` block in `is_retryable`
must be updated to handle `Error::Sqlx(_)` — add it to the catch-all `_ => false` arm (it is
already a wildcard, so no change needed unless the method uses exhaustive matching). Verify
the current catch-all covers it.

---

### `ferro-ai/src/lib.rs` (MODIFY — add re-exports)

**Analog:** itself — `ferro-ai/src/lib.rs`

**Existing module declaration + re-export pattern** (lib.rs lines 44-67):
```rust
pub mod classifier;
pub mod client;
pub mod complete;
pub mod config;
pub mod confirmation;
pub mod error;
pub mod schema;
pub mod tools;

pub use classifier::anthropic::AnthropicProvider;
// ...
pub use complete::complete;
pub use config::AiConfig;
// ...
pub use error::Error;
```

**Additions to make** — insert after `pub mod tools;` (line 51) and after `pub use complete::complete;`
(line 60):
```rust
// Module declarations (add to the pub mod block):
pub mod embed;
pub mod similarity;
#[cfg(feature = "pgvector")]
pub mod pgvector;

// Re-exports (add to the pub use block):
pub use embed::embed;
pub use similarity::cosine_similarity;
#[cfg(feature = "pgvector")]
pub use pgvector::{Neighbor, PgVectorStore};
```
The `#[cfg(feature = "pgvector")]` attribute on the `pub mod pgvector;` line gates the entire
module tree. This means `pgvector/mod.rs` items need no per-item `#[cfg]` annotations — they
are unreachable when the feature is off. The re-exports under `#[cfg]` follow the same pattern.

---

### `ferro-ai/Cargo.toml` (MODIFY — add `[features]` + optional deps)

**Analog:** `ferro-deployments/Cargo.toml` lines 31-34:
```toml
[features]
sqlx-postgres = ["sea-orm/sqlx-postgres"]
postgres-tests = ["sqlx-postgres"]
```

**Additions to make:**

Under `[dependencies]` (after the existing deps, before `[dev-dependencies]`):
```toml
pgvector = { version = "0.4", features = ["sqlx"], optional = true }
sqlx     = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }
```

New `[features]` section (does not exist yet in ferro-ai/Cargo.toml — add after `[dev-dependencies]`):
```toml
[features]
pgvector      = ["dep:pgvector", "dep:sqlx"]
postgres-tests = ["pgvector"]
```
`dep:pgvector` syntax (Rust 1.60+) avoids the implicit `pgvector` feature being activated by the
crate name. This is the correct form when `optional = true` is used with a dep-name that matches
a potential feature name.

**Version rationale (document in a comment above the deps):**
```toml
# pgvector 0.4 supports sqlx >= 0.8, < 0.10 (docs.rs/pgvector/0.4.2).
# Workspace Cargo.lock pins sqlx 0.8.6 via sea-orm; "0.8" matches that pin
# and avoids a second sqlx compilation. Do NOT bump to "0.9".
```

---

## Shared Patterns

### Env-var reading (FERRO_AI_* convention)
**Source:** `ferro-ai/src/config.rs` lines 43-48
**Apply to:** `embed_model()` in `ollama.rs` and `openai.rs`
```rust
std::env::var("FERRO_AI_EMBED_MODEL")
    .unwrap_or_else(|_| "provider-specific-default".to_string())
```
No `?` propagation — use `unwrap_or_else` to fold into a `String` with a sensible default.
This matches how `FERRO_AI_PROVIDER` is read in `config.rs` line 44.

### Error mapping from external crates
**Source:** `ferro-ai/src/client/ollama.rs` lines 233-241, openai.rs lines 211-219
**Apply to:** `pgvector/mod.rs` sqlx error mapping
```rust
.map_err(|e| Error::Provider { status: None, message: e.to_string() })
// OR for sqlx specifically:
.map_err(|e| Error::Sqlx(e.to_string()))
```
Never leak internal library error types through the crate boundary. Always convert to `crate::error::Error`.

### Feature-gated integration test guard
**Source:** `ferro-deployments/tests/race_promote_postgres.rs` lines 16, 44-51, 62-63
**Apply to:** `ferro-ai/tests/pgvector_integration.rs`

Two-layer guard:
1. `#![cfg(feature = "postgres-tests")]` at file top — compile-time exclusion.
2. `if std::env::var("DATABASE_URL").is_err() { eprintln!(...); return; }` in each test body — runtime skip when no DB is available.

### Inline test structure
**Source:** `ferro-ai/src/complete.rs` lines 83-143, `ferro-ai/src/error.rs` lines 86-145
**Apply to:** `embed.rs`, `similarity.rs`, provider file test extensions
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // imports needed only in tests (async_trait, etc.)

    #[test]           // for sync tests
    #[tokio::test]    // for async tests
    fn test_name() { ... }
}
```

---

## No Analog Found

None. All files have direct analogs in the codebase.

---

## Metadata

**Analog search scope:** `ferro-ai/src/`, `ferro-deployments/` (Cargo.toml + tests/)
**Files scanned:** 9 source files + 2 Cargo.toml files
**Pattern extraction date:** 2026-06-08
