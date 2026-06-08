//! Feature-gated pgvector store primitives for semantic search over Postgres.
//!
//! Enabled with the `pgvector` cargo feature. This module is a thin query
//! primitive — it does not manage schema, extensions, or indexes. Schema
//! is caller-managed (D-09).
//!
//! # One-time setup SQL (caller's responsibility)
//!
//! ```sql
//! CREATE EXTENSION IF NOT EXISTS vector;
//!
//! CREATE TABLE embeddings (
//!     id     BIGINT PRIMARY KEY,
//!     vec    vector(1536)   -- dimension must match your embedding model
//! );
//!
//! -- HNSW index for fast approximate nearest-neighbor search
//! CREATE INDEX ON embeddings USING hnsw (vec vector_cosine_ops);
//! -- Alternative: ivfflat (faster to build, slightly lower recall)
//! -- CREATE INDEX ON embeddings USING ivfflat (vec vector_cosine_ops) WITH (lists = 100);
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use ferro_ai::pgvector::PgVectorStore;
//!
//! let store = PgVectorStore::new("embeddings", "vec");
//! store.store(&pool, 42, &embedding_vec).await?;
//! let neighbors = store.nearest(&pool, &query_vec, 5).await?;
//! for n in neighbors {
//!     println!("id={} score={:.4}", n.id, n.score);
//! }
//! ```

use crate::error::Error;
use pgvector::Vector;
use sqlx::PgPool;

/// A single result from [`PgVectorStore::nearest`].
///
/// `score` is the cosine similarity: `1 - cosine_distance`, in the range `[-1, 1]`.
/// Higher values indicate greater similarity. Identical vectors yield `score = 1.0`;
/// orthogonal vectors yield `0.0`; opposite vectors yield `-1.0`.
pub struct Neighbor {
    /// The row identifier (matches the `id` column supplied to [`PgVectorStore::store`]).
    pub id: i64,
    /// Cosine similarity score in `[-1, 1]`. Higher = more similar.
    pub score: f32,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Neighbor {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            score: row.try_get::<f32, _>("score")?,
        })
    }
}

/// Thin query primitive for storing and searching vector embeddings in Postgres.
///
/// Operates over a raw `sqlx::PgPool` — independent of sea-orm. Schema and index
/// management are caller-managed (see module-level docs for the one-time setup SQL).
///
/// # Security
///
/// Dynamic values (`id`, `embedding`, `k`) are always bound via sqlx `$1`/`$2`
/// parameters — never string-interpolated. The `table` and `column` names are
/// interpolated into the SQL string, but they originate exclusively from the
/// [`PgVectorStore::new`] constructor (trusted application code), never from
/// request or user input. Do not pass untrusted strings to `new()`.
pub struct PgVectorStore {
    table: String,
    column: String,
}

impl PgVectorStore {
    /// Creates a new store targeting `table.column` for vector operations.
    ///
    /// `table` and `column` must be names of an existing Postgres table and
    /// vector column. They are interpolated into SQL strings — supply only
    /// trusted, application-controlled values (not user input).
    pub fn new(table: &str, column: &str) -> Self {
        Self {
            table: table.to_string(),
            column: column.to_string(),
        }
    }

    /// Upserts a vector embedding for the given `id`.
    ///
    /// Inserts a new row; if a row with the same `id` already exists, the
    /// embedding is updated (`ON CONFLICT (id) DO UPDATE`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlx`] if the database operation fails (e.g., dimension
    /// mismatch between `embedding` and the column definition, connection error).
    pub async fn store(&self, pool: &PgPool, id: i64, embedding: &[f32]) -> Result<(), Error> {
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

    /// Returns the `k` nearest neighbors to `query` ordered by cosine similarity (descending).
    ///
    /// Uses the pgvector `<=>` cosine distance operator. The returned [`Neighbor::score`]
    /// is `1 - cosine_distance` so it falls in `[-1, 1]` and aligns with the
    /// `ferro_ai::cosine_similarity` convention.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlx`] if the database operation fails (e.g., dimension
    /// mismatch, connection error).
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
