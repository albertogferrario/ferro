# Phase 167: Embeddings & pgvector - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 167-embeddings-pgvector
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per question)
**Areas discussed:** embed() free function, cosine_similarity contract, PgVectorStore API, pgvector feature/dependency wiring, embedding-model resolution

---

## A. `embed()` free function

| Option | Description | Selected |
|--------|-------------|----------|
| Thin pass-through | `embed(client, text)` delegates to `client.embed(text)`, symmetric with `complete()` | ✓ |
| Wrapper with normalization | Free fn L2-normalizes the returned vector | |
| Wrapper with retry/batching | Free fn adds retry loop and/or batches inputs | |

**Auto choice:** Thin pass-through.
**Notes:** Keeps the trait method as the single source of truth; matches `complete.rs` exactly. Anthropic's `Unsupported` propagates unchanged.

## B. `cosine_similarity` contract

| Option | Description | Selected |
|--------|-------------|----------|
| Panic on invalid (SC-mandated) | Panic with clear message on empty / dimension-mismatch | ✓ |
| Return `Result` | `Result<f32, Error>` for invalid input | |
| Return `Option` | `None` on invalid input | |

**Auto choice:** Panic — this is locked by SC#2/#5, not discretionary. Pure Rust, no crates.
**Notes:** Tests assert orthogonal→0, identical→1, opposite→-1 with float epsilon.

## C. `PgVectorStore` API surface

| Option | Description | Selected |
|--------|-------------|----------|
| Raw sqlx `&PgPool`, cosine `<=>`, caller-managed schema, `(id, score)` results | Thin query primitive | ✓ |
| sea-orm connection | Couple to the app's ORM | |
| Store owns DDL (CREATE EXTENSION/TABLE/index) | Self-managing schema | |
| Default L2 `<->` distance | Match pgvector's most common default | |

**Auto choice:** Raw sqlx + cosine distance + caller-managed schema + minimal typed `Neighbor { id, score }`.
**Notes:** Cosine chosen for consistency with the pure-Rust helper. Metadata column deferred. `id` concrete `i64` for v1.

## D. `pgvector` feature & dependency wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Feature pulls `pgvector 0.4` + optional `sqlx` (postgres) | Honest about the transport dep | ✓ |
| Avoid direct sqlx (only pgvector) | Attempt to satisfy SC#4 literally | |

**Auto choice:** Pull both; document the SC#4 reconciliation in the feature doc comment.
**Notes:** The raw-connection API structurally needs `sqlx::PgPool` in signatures; pgvector 0.4 does not re-export it. Surface, don't hide, per the audit-and-surface convention.

## E. Embedding-model resolution (latent discrepancy)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal fix in 167 | Providers resolve an embedding-specific model (env-overridable), independent of chat `default_model()` | ✓ |
| Leave as-is, document only | Ship the free fn over the current (buggy Ollama) impls | |
| Broad new embedding-config struct | Full configuration surface | |

**Auto choice:** Minimal fix using the existing `FERRO_AI_*` convention (single `FERRO_AI_EMBED_MODEL` knob); planner may defer to fast-follow but must document.
**Notes:** `OllamaClient::embed` currently sends the chat model `llama3.1` to `/api/embed` — wrong for embeddings. `OpenAiClient::embed` hardcodes `text-embedding-3-small`. Avoid a duplicate control surface.

## Claude's Discretion

- Executor bound (`&PgPool` vs `impl PgExecutor`).
- Module file layout.
- `Neighbor.id` concrete vs generic.
- pgvector integration test gating strategy under CI `--all-features`.

## Deferred Ideas

- `embed_many` batch embedding.
- Metadata/payload column + filtered nearest queries.
- Generic-over-id `PgVectorStore<Id>`.
- ivfflat/hnsw index-management helpers.
- In-memory `VectorStore` reusing `cosine_similarity`.
