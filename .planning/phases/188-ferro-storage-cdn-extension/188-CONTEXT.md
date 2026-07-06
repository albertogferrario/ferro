# Phase 188: ferro-storage CDN Extension - Context

**Gathered:** 2026-06-08 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend the **existing** `ferro-storage` crate (NOT a new crate) with CDN awareness:
1. `Storage::cdn_url(path)` / `Disk::cdn_url(path)` — returns the CDN edge URL for a stored
   object, configured via env (a CDN base URL), falling back to the origin `url()` when no CDN
   is configured.
2. A `PurgeApi` trait abstracting cache invalidation, with a batteries-included **DigitalOcean
   Spaces CDN** adapter (batched, rate-limited, wildcard-aware), plus Bunny and Cloudflare
   adapters behind cargo features.

**Killer feature:** the CDN cache-coherence primitive — `cdn_url()` + `PurgeApi` turn
"promote then purge" into a clean two-call sequence for any consumer, with the DO Spaces
adapter encapsulating the operationally fiddly parts (≤50-file batching, the 5 req/10s rate
limit, wildcard slot accounting, the exact `DELETE /v2/cdn/endpoints/{id}/cache` shape) so
consumers never reimplement DigitalOcean's API quirks. This is the v12.3 milestone's final
piece: gestiscilo Phase 190 composes promote (Phase 186) → purge (this phase).

**What this phase is NOT** (scope anchors — consumer concerns, not this crate's):
- It does NOT decide *which* keys to purge. Purging only non-hashed HTML (and never the
  content-hashed, immutable asset URLs) is consumer policy (gestiscilo PITFALLS B-02). The
  crate provides the mechanism; the consumer chooses the key set.
- It does NOT orchestrate promote→purge sequencing (gestiscilo Phase 190's job).
- It does NOT provision CDN endpoints / create the DO CDN endpoint (operator setup).
- No signed/temporary CDN URLs in v1 (origin `temporary_url()` already exists for that).

Requirements: STOR-F-01, STOR-F-02. Consumer: gestiscilo Phase 190. Depends on nothing —
parallel-capable with 186/187 (it landed last only by publish order).

</domain>

<decisions>
## Implementation Decisions

### cdn_url() — URL generation (STOR-F-01 / criterion 1)
- **D-01:** CDN is a presentation layer over any driver, orthogonal to the storage backend.
  Add a `cdn_url: Option<String>` field to `DiskConfig` (`ferro-storage/src/facade.rs`) with a
  `with_cdn_url()` consuming builder, mirroring the existing `url`/`with_url()` pair. Do NOT
  thread CDN into every `StorageDriver` impl — `cdn_url()` is computed at the `Disk`/`Storage`
  facade level.
- **D-02:** `Disk::cdn_url(path)` and `Storage::cdn_url(path)`: if a CDN base is configured,
  return `{cdn_base}/{path}` (joined with exactly one `/`, no double slashes); otherwise fall
  back to `self.url(path).await` (origin URL). Pure string composition — zero new dependencies,
  always available in the default feature set. Signature mirrors `url()` (async, `Result<String, Error>`)
  for API symmetry even though the CDN path is synchronous.
- **D-03:** `StorageConfig::from_env()` reads the CDN base for the S3/Spaces disk from
  `AWS_CDN_URL` (joins the existing `AWS_*` env family that already configures Spaces; the CDN
  fronts the Spaces bucket). Unset → no CDN base → `cdn_url()` falls back to origin. Per the
  project-agnostic rule this is a generic provider env var, not app identity. Planner may add a
  generic per-disk form (`FILESYSTEM_{DISK}_CDN_URL`) if cheap; `AWS_CDN_URL` is the locked minimum.

### PurgeApi trait & feature gating (STOR-F-02 / criteria 2 & 4)
- **D-04:** `PurgeApi` is a standalone `#[async_trait]` trait — `async fn purge(&self, paths: &[String])
  -> Result<(), Error>` (exact signature planner discretion; keep it minimal and provider-agnostic).
  The trait itself pulls NO new deps (async-trait is already present) and lives in the default
  feature set so consumers can implement their own purgers without enabling anything.
- **D-05:** The DigitalOcean Spaces adapter is the **default/reference** `PurgeApi` implementation.
  It requires an HTTP client (`reqwest`) for the DO REST API. Per the literal acceptance contract
  (criterion 2 names DO as the default adapter; criterion 4 names ONLY Bunny+Cloudflare as the
  feature-gated, not-in-default-graph ones), the DO adapter + `reqwest` are in the **default
  dependency graph**. Mitigate weight: `reqwest = { version = "0.12", default-features = false,
  features = ["json", "rustls-tls"] }` (no OpenSSL/C TLS, lean). The pre-existing `s3` feature
  precedent (aws-sdk gated) is NOT applied to DO — the criteria explicitly place DO in default.
- **D-06:** Bunny (`BunnyCdn`) behind cargo feature `cdn-bunny`; Cloudflare (`CloudflareCdn`)
  behind `cdn-cloudflare`. Each must COMPILE behind its feature without entering the default
  dependency graph (criterion 4). They are real, lean `PurgeApi` impls (each calls its provider's
  purge endpoint via the shared `reqwest` client), not empty stubs — but DO is the polished
  reference impl with full batching+throttle+wildcard; Bunny/CF implement their providers'
  equivalent purge call at a "works, not gold-plated" bar.

### DO Spaces adapter operational details (STOR-F-02 / criterion 2)
- **D-07:** Endpoint + auth: `DELETE https://api.digitalocean.com/v2/cdn/endpoints/{id}/cache`
  with JSON body `{"files": [<paths>]}` and `Authorization: Bearer {token}`. (DO's cache-purge
  is a DELETE-with-body; the planner verifies the exact body key against current DO API docs.)
- **D-08:** Batching: split `paths` into chunks of **≤50 files per request** (DO's documented
  limit). A wildcard path (`some/dir/*`) counts as **1 file slot** toward the 50.
- **D-09:** Rate limiting: an **internal** throttle enforcing **≤5 requests per rolling 10s**
  (DO's CDN purge limit) — the caller never manages this. Implementation primitive (token bucket
  / sliding-window timestamps + `tokio::time::sleep`) is planner discretion; ferro-storage
  already depends on tokio.
- **D-10:** Config via `DoSpacesCdnConfig::from_env()` reading `DO_SPACES_CDN_ID` +
  `DIGITALOCEAN_ACCESS_TOKEN` (the canonical DO token env var used by doctl/terraform). Both are
  generic provider env vars (project-agnostic rule — fine). **Missing `DO_SPACES_CDN_ID` →
  `purge()` is a logged no-op returning `Ok(())`** (criterion 3 — consumers without a CDN keep
  working). Missing token with an id set → a structured `Error`.

### Module layout, error handling, finalize
- **D-11:** New `cdn` module: `ferro-storage/src/cdn/mod.rs` (the `PurgeApi` trait + `DoSpacesCdn`
  adapter + `DoSpacesCdnConfig`), `src/cdn/bunny.rs` (`#[cfg(feature = "cdn-bunny")]`),
  `src/cdn/cloudflare.rs` (`#[cfg(feature = "cdn-cloudflare")]`). Re-export `PurgeApi` + adapters
  from `lib.rs` (adapters under their cfg). Add `cdn_url` to the existing facade exports surface.
- **D-12:** Extend the existing `Error` enum (`ferro-storage/src/error.rs`, thiserror **1.0** —
  match the crate's current version, do NOT bump to 2) with a `Cdn(String)` (or `Purge`) variant +
  constructor, wrapping reqwest/HTTP/DO-API failures. No `.unwrap()` on network paths; a failed
  purge returns `Err`, never panics.
- **D-13:** No new-crate chores — ferro-storage is an EXISTING published crate already in
  publish.yml Wave 1a. CI publishes it normally on the workspace version bump (publish-**update**
  token works — NO manual bootstrap needed, unlike the 185/186/187 new crates). Finalize: bump the
  workspace version (`Cargo.toml` `version = "0.2.45"` → next patch), update the ferro-storage
  docs page (`docs/src/`) with a CDN section, README note. Run the full CI-parity gate including
  `--all-features` so the Bunny/CF feature code is compiled and clippy-checked.

### Claude's Discretion
- Exact `PurgeApi::purge` signature/return detail and whether a `purge_all`/wildcard helper is added.
- Throttle primitive (token bucket vs timestamp ring) — only constraint: ≤5 req/10s, internal.
- Exact `reqwest` minor version and whether a shared internal HTTP-client helper is factored out
  for the three adapters.
- Whether `cdn_url()` also lands on the lower-level `StorageDriver` trait or stays facade-only
  (facade-only recommended — keeps drivers unchanged).
- Bunny/Cloudflare exact endpoint/auth shapes (verify against current provider docs at plan time).
- Test doubles for the HTTP calls (a mock server vs a trait-level fake) — see specifics.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Locked design (gestiscilo v7.1 — source of this milestone)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-ARCHITECTURE.md` §"D-02 — CDN provider: DigitalOcean Spaces CDN" (DO Spaces CDN bundled with the Spaces storage; cache invalidation via the DO API on deployment promote; Bunny/Cloudflare as future upgrade paths)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-PITFALLS.md` §B-02 "Stale CDN HTML Referencing Purged Assets After Promote" (the purge sequence: assets are content-hashed/immutable and never purged; only the non-hashed HTML keys are purged on promote — this is the consumer policy this crate's mechanism serves) and §B-03 (lifecycle-deleted artifacts)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/research/v7.1-INTEGRATION.md` — gestiscilo Phase 190 consumer (promote→purge two-call sequence)

### Ferro repo
- `.planning/ROADMAP.md` §"v12.3 Deployment Platform Primitives" → "Phase 188" — STOR-F-01/02 + the 4 success criteria (the acceptance contract: cdn_url+fallback, PurgeApi+DO adapter with ≤50 batch / 5-req-10s / wildcard, missing-id no-op, Bunny/CF feature-gated)
- `ferro-storage/src/facade.rs` — `DiskConfig` (add `cdn_url`), `Disk`/`Storage` (add `cdn_url()`), the `url()` delegation pattern to mirror
- `ferro-storage/src/config.rs` — `StorageConfig::from_env()` (the `FILESYSTEM_*`/`AWS_*` env-reading pattern to extend with `AWS_CDN_URL`)
- `ferro-storage/src/error.rs` — `Error` enum (thiserror 1.0) to extend with a CDN variant
- `ferro-storage/src/drivers/s3.rs` + `Cargo.toml` `[features] s3 = [...]` — the existing optional-backend feature-gating precedent
- `.planning/phases/186-ferro-deployments-immutable-deployments-atomic-promote/186-CONTEXT.md` — sibling milestone phase (the promote half this purge half pairs with)
- `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-storage/src/facade.rs` `DiskConfig.url` + `with_url()` + `Disk::url()` — the exact
  pattern `cdn_url`/`with_cdn_url()`/`cdn_url()` mirrors.
- `ferro-storage/src/config.rs` `from_env()` — extend with `AWS_CDN_URL` reading for the s3 disk.
- `ferro-storage/src/error.rs` — `Error` enum + constructor-helper pattern (thiserror 1.0).
- `Cargo.toml` `[features] s3 = ["aws-sdk-s3", ...]` — the optional-integration feature precedent
  (applied to Bunny/CF here; DO stays default per the criteria).
- ferro-storage already depends on `tokio`, `serde`, `serde_json` — reused for the adapter
  config, JSON body, and throttle timing.

### Established Patterns
- thiserror **1.0** in this crate (do not bump to 2 — keep crate-local consistency).
- `from_env()` config structs reading framework-convention env vars (project-agnostic rule).
- Async trait via `async_trait` (already a dep + re-exported).

### Integration Points
- `ferro-storage/src/lib.rs` — add `pub mod cdn;` + `pub use cdn::{PurgeApi, DoSpacesCdn, ...}`
  (Bunny/CF re-exports under their `#[cfg(feature)]`).
- `Cargo.toml` — add `reqwest` (default, lean rustls) + `[features] cdn-bunny`, `cdn-cloudflare`.
- Workspace `Cargo.toml` version bump (0.2.45 → next); ferro-storage publishes via CI normally
  (existing crate, publish-update token — no manual bootstrap).
- `docs/src/` — extend the storage docs page with a CDN section.
- Consumer: gestiscilo Phase 190 calls `storage.cdn_url()` for asset URLs and a `DoSpacesCdn`
  `PurgeApi` on promote.

</code_context>

<specifics>
## Specific Ideas

- **Promote→purge two-call sequence** is the consumer story (gestiscilo Phase 190): after
  `ferro_deployments::promote()`, call `purger.purge(&html_keys)`. This crate ships both halves'
  building blocks across the milestone (186 promote, 188 purge).
- **Purge only non-hashed HTML** (gestiscilo B-02): content-hashed asset URLs are immutable and
  never purged — this is consumer key-selection policy, but worth a doc note in the CDN section so
  consumers don't purge the whole prefix.
- **Test strategy:** the DO adapter's batching (>50 paths → multiple requests), wildcard slot
  accounting, throttle (≥6 rapid requests serialize under the 5/10s window), and missing-id
  no-op are the proof artifacts. Mock the HTTP layer (a local mock server or an injectable
  request-sender seam) so tests assert request shape/batching/throttle WITHOUT hitting the real
  DO API. `cdn_url()` fallback (configured vs unset) is a pure unit test.

</specifics>

<deferred>
## Deferred Ideas

- Signed/temporary CDN URLs — origin `temporary_url()` covers the signed-URL need today; a CDN
  signed-URL variant is future work if a consumer needs edge-signed access.
- CDN endpoint provisioning (creating the DO CDN endpoint via API) — operator/IaC concern, out
  of scope; this phase assumes the endpoint exists and `DO_SPACES_CDN_ID` is known.
- Automatic purge-on-`delete()`/`put()` — keeping purge explicit avoids surprise API calls and
  rate-limit pressure; consumers call `purge()` deliberately. Revisit only if a consumer wants it.
- Per-key purge policy helpers (e.g. "purge HTML, skip hashed assets") — consumer policy for now
  (gestiscilo B-02); promote into the crate only if a second consumer needs the same rule.
- Lifecycle-aware purge (B-03 artifact-deleted coordination) — belongs with deployment GC tooling,
  not the purge primitive.

</deferred>

---

*Phase: 188-ferro-storage-cdn-extension*
*Context gathered: 2026-06-08 (auto mode)*
