# Phase 167: Embeddings & pgvector - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; rationale logged per decision)

<domain>
## Phase Boundary

Ship the embedding/similarity surface of the ferro-ai SDK on top of the `LlmClient`
foundation from Phase 165:

1. `ferro_ai::embed(client, text)` — a thin free function (the public ergonomic surface),
   mirroring `ferro_ai::complete`.
2. `ferro_ai::cosine_similarity(a, b) -> f32` — pure Rust, zero extra crates.
3. `ferro_ai::pgvector` — an OPTIONAL, feature-gated module (`PgVectorStore::store` /
   `PgVectorStore::nearest`) over raw sqlx connections for semantic search.

**Already shipped in Phase 165 (do NOT re-implement):** the `LlmClient::embed()` trait
method exists and is implemented for all three providers — `OpenAiClient::embed` hits
`/v1/embeddings`, `OllamaClient::embed` hits `/api/embed`, `AnthropicClient::embed`
returns `Error::Unsupported` (Anthropic has no embeddings endpoint). SC#1's
"providers implement `LlmClient::embed()`" is therefore satisfied at the trait level;
this phase adds the FREE-FUNCTION wrapper, not the provider HTTP paths.

This phase does NOT add SSE primitives (Phase 168), streaming text (Phase 169), or any
CLI command (later phases). It is a ferro-ai leaf-crate-only change.
</domain>

<decisions>
## Implementation Decisions

### A. `embed()` free function
- **D-01:** `ferro_ai::embed(client: &dyn LlmClient, text: &str) -> Result<Vec<f32>, Error>`
  is a thin pass-through to `client.embed(text)`, symmetric with `complete(client, prompt)`
  (`complete.rs`). It adds NO batching, NO normalization, NO retry — keeping the surface
  minimal and the trait method the single source of truth. `[auto] recommended: thin wrapper.`
- **D-02:** Re-export from `lib.rs` as `pub use embed::embed;` (new `embed.rs` module),
  matching the `pub use complete::complete;` pattern. `[auto]`
- **D-03:** When the underlying client returns `Error::Unsupported` (Anthropic), the free
  function propagates it unchanged — no special-casing. Callers choose an embedding-capable
  provider. `[auto]`

### B. `cosine_similarity`
- **D-04:** `ferro_ai::cosine_similarity(a: &[f32], b: &[f32]) -> f32`, pure Rust, no new
  crates (manual dot product + magnitudes). Lives in a new `similarity.rs` module,
  re-exported from `lib.rs`. `[auto]`
- **D-05:** Per SC#2/#5 the contract is PANIC (not `Result`) on invalid input:
  panic with a clear message on empty slices and on dimension mismatch. This is the
  locked contract from the roadmap, not a discretionary choice — a pure math helper with
  a programmer-error precondition. `[auto] recommended: follow SC, panic with clear message.`
- **D-06:** Returns a value clamped/landing in `[-1.0, 1.0]`. Unit tests assert: orthogonal
  → `0.0`, identical → `1.0`, opposite → `-1.0` (SC#5). Use an epsilon tolerance in tests
  for float comparison. `[auto]`

### C. `PgVectorStore` API surface
- **D-07:** `PgVectorStore::store` and `::nearest` accept a RAW sqlx Postgres executor
  (`&sqlx::PgPool` — or `impl sqlx::PgExecutor<'_>` if the planner finds it ergonomic),
  NOT a sea-orm connection. ferro-ai stays decoupled from sea-orm here. `[auto]`
- **D-08:** Distance metric defaults to COSINE distance (pgvector `<=>` operator) for
  consistency with the pure-Rust `cosine_similarity` helper. The returned score is the
  cosine SIMILARITY (`1 - distance`) so callers reason in the same `[-1,1]` space as D-04.
  `[auto] recommended: cosine for cross-consistency.`
- **D-09:** Schema is CALLER-MANAGED. `PgVectorStore` is query-only: it does NOT run
  `CREATE EXTENSION vector`, `CREATE TABLE`, or index DDL. The module docs include the
  one-time setup SQL (`CREATE EXTENSION vector;` + an example table + `ivfflat`/`hnsw`
  index hint) as documentation, but the store performs only row insert + nearest-N query.
  Keeps the primitive thin and migration-agnostic. `[auto]`
- **D-10:** Minimal typed result. `store` takes an id + embedding (`&[f32]` / `pgvector::Vector`);
  `nearest(conn, query, k)` returns `Vec<Neighbor>` where `Neighbor { id, score }`
  (id type concrete — recommend `i64` for v1, with a planner note if a generic id is cheap).
  Optional payload/metadata column is OUT of scope for v1 (deferred). `[auto]`

### D. `pgvector` feature & dependency wiring
- **D-11:** The `pgvector` cargo feature activates two optional dependencies:
  `pgvector = "0.4"` AND `sqlx` (postgres + a tokio runtime feature). Non-`pgvector`
  builds pull NEITHER. `[auto]`
- **D-12 (SC#4 reconciliation — flag for planner):** SC#4 says the feature "adds only
  `pgvector 0.4` to the dependency graph." A raw-sqlx-connection public API (D-07)
  structurally REQUIRES `sqlx` types in ferro-ai's own signatures, so `sqlx` is an
  unavoidable second dependency under the feature (pgvector 0.4 itself depends on sqlx but
  does not re-export `PgPool`). Treat SC#4 as: *the only NEW vector-specific direct
  dependency is `pgvector 0.4`; its `sqlx` companion is the transport the API is defined
  over.* The planner should document this in the feature's doc comment rather than contort
  the API to avoid a direct sqlx dep. Per the audit-and-surface convention, do not silently
  work around the wording — note the reconciliation in the plan. `[auto]`

### E. Embedding-model resolution (latent discrepancy — flag for planner)
- **D-13 (discrepancy to surface):** `OllamaClient::embed` currently sends
  `default_model()` (the CHAT model, `llama3.1`) to `/api/embed` — `llama3.1` is not an
  embedding model, so live embeds would fail or return poor vectors. `OpenAiClient::embed`
  hardcodes `text-embedding-3-small`. This is inconsistent and partly wrong. Minimal
  recommended fix: have the providers resolve an embedding model independent of the chat
  `default_model()` (e.g. a per-provider embedding default such as Ollama `nomic-embed-text`
  / OpenAI `text-embedding-3-small`, overridable via an env var). Avoid inventing a broad
  new config surface — reuse the existing `FERRO_AI_*` convention (e.g.
  `FERRO_AI_EMBED_MODEL`) and keep it a single knob. Planner decides whether this is in-scope
  for 167 or a fast-follow; at minimum the limitation must be documented. `[auto] recommended:
  fix minimally in 167 since embed() is the phase's headline surface.`

### Claude's Discretion
- Exact executor bound for D-07 (`&PgPool` vs generic `impl PgExecutor`).
- Module file layout (`embed.rs` / `similarity.rs` / `pgvector/mod.rs` vs flatter).
- Whether `Neighbor.id` is concrete `i64` or generic (D-10) — bias to concrete for v1.
- Test strategy for `pgvector` (the postgres integration test is gated; cosine_similarity
  unit tests are mandatory and run unconditionally).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 167: Embeddings & pgvector" — goal + 5 Success Criteria (the boundary)
- `.planning/ROADMAP.md` §"v12.1 AI — ferro-ai SDK" — milestone build order; note `pgvector 0.4` is the only declared new workspace dep
- `.planning/REQUIREMENTS.md` AISDK-04 (embeddings + cosine, zero extra crates) and AISDK-05 (pgvector, feature-gated, thin sqlx raw-query module)

### Existing crate (extend, not greenfield)
- `ferro-ai/src/client/mod.rs` — `LlmClient` trait incl. existing `async fn embed(&self, text) -> Result<Vec<f32>, Error>` (D-01 wraps this)
- `ferro-ai/src/client/openai.rs` §`embed` — hits `/v1/embeddings`, model hardcoded `text-embedding-3-small` (D-13)
- `ferro-ai/src/client/ollama.rs` §`embed` — hits `/api/embed` with `default_model()` — the chat-model discrepancy (D-13)
- `ferro-ai/src/client/anthropic.rs` §`embed` — returns `Error::Unsupported` (correct; no Anthropic embeddings endpoint)
- `ferro-ai/src/complete.rs` — the free-function + `lib.rs` re-export pattern `embed()` must mirror (D-01/D-02)
- `ferro-ai/src/error.rs` — `Error` enum (`Unsupported`, `Provider { status, message }`); add a `PgVector`/`Sqlx` variant if the store needs one
- `ferro-ai/src/lib.rs` — public re-exports (add `embed`, `cosine_similarity`, and feature-gated `pgvector` surface)
- `ferro-ai/Cargo.toml` — add `[features] pgvector = [...]` + optional `pgvector`/`sqlx` deps (D-11/D-12)

### Prior-phase context
- `.planning/phases/165-llmclient-trait-provider-implementations/165-CONTEXT.md` — `LlmClient`, `Error::Unsupported`, `Error::Provider { status, message }`, project-agnostic `FERRO_AI_*` rule (D-07 of 165)
- `.planning/phases/166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer/166-CONTEXT.md` — `complete::<T>()` surface conventions the `embed` free fn mirrors

### Conventions
- `CLAUDE.md` (project) §"Project-agnostic crates" — only `FERRO_AI_*` env vars, no app identity (constrains D-13's env knob)
- `.github/workflows/publish.yml` — ferro-ai is in `WAVE1B_CRATES`; CI runs `--all-features`, so the `pgvector` feature MUST compile under `--all-features` (a CI postgres service or `#[ignore]`d integration test is the gate, not a live DB requirement)
- Workspace: `thiserror` one Error enum per crate; serde `rename_all = "snake_case"`; builder `with_*` consuming `self`

### Provider / library docs (fetch live during research — do not rely on training cutoff)
- `pgvector` crate 0.4 (docs.rs) — `Vector` type, sqlx `Encode`/`Decode`/`Type` integration, distance operators `<->` `<#>` `<=>`
- `sqlx` 0.8 Postgres — `PgPool`, `query`/`query_as`, `PgExecutor` bound (the executor type for D-07)
- OpenAI Embeddings API (`/v1/embeddings`, `text-embedding-3-small`)
- Ollama Embeddings API (`/api/embed`, embedding-capable models e.g. `nomic-embed-text`)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_ai::complete` (`complete.rs`) — exact free-function + `lib.rs` re-export shape to mirror for `embed`
- `LlmClient::embed()` — already implemented; the free fn is a one-line delegate
- `Error` enum (thiserror) — extend with at most one pgvector/sqlx error variant; `Unsupported` already covers Anthropic
- Phase 165/166 established that ferro-ai owns its public types and never re-exports transport crates (cf. `reqwest-eventsource` is `pub(crate)`) — apply the same to sqlx (only `PgPool` appears in signatures, no broad sqlx re-export)

### Established Patterns
- `async_trait` already present; provider `embed` impls already use the `Error::Provider { status, message }` mapping
- Optional-dependency-behind-feature pattern exists in the workspace (`ferro-deployments` `sqlx-postgres`/`postgres-tests` features) — a direct precedent for D-11

### Integration Points
- `lib.rs` re-exports are the public surface; new symbols: `embed`, `cosine_similarity`, and (feature-gated) `pgvector::{PgVectorStore, Neighbor}`
- CI `--all-features` path compiles the `pgvector` feature — the integration test must be runnable-or-ignorable without a mandatory live DB
</code_context>

<specifics>
## Specific Ideas

- Keep `embed`/`cosine_similarity` dependency-free and obvious — they are the AISDK-04
  "zero extra crates" promise made literal.
- The pgvector module is deliberately a thin query primitive, not an ORM: store one row,
  find the nearest k. Anything richer (metadata filtering, hybrid search, index management)
  is a future phase, not this one.
</specifics>

<deferred>
## Deferred Ideas

- Batch embedding (`embed_many`) — multiple texts in one provider call. New capability; future phase.
- Metadata/payload column on `PgVectorStore` rows + filtered nearest queries. Future phase.
- Generic-over-id `PgVectorStore<Id>` if a second consumer needs non-`i64` keys.
- Index-management helpers (ivfflat/hnsw creation) as a `ferro-migration` integration. Future phase.
- Re-using `cosine_similarity` inside an in-memory `VectorStore` (no Postgres) for small datasets.

None of the above is required by SC#1–#5. Discussion stayed within phase scope.
</deferred>

---

*Phase: 167-embeddings-pgvector*
*Context gathered: 2026-06-08*
