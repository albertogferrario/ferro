# Phase 204: ferro-storage provider-agnostic CDN configuration - Context

**Gathered:** 2026-06-11
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

Collapse the fragmented CDN env-var surface in **`ferro-storage`** into a single
provider-agnostic quartet, with the legacy per-provider variable names retained as
deprecated fallbacks for one release.

**The quartet (new, primary):**
- `CDN_URL` — public CDN base URL fronting the bucket (drives `Disk::cdn_url()`),
- `CDN_PROVIDER` — `none` | `digitalocean` | `bunny` | `cloudflare` (selects the `PurgeApi` adapter),
- `CDN_PURGE_TOKEN` — provider API token (DO/CF/Bunny credential),
- `CDN_PURGE_ZONE` — provider-specific zone/endpoint id (DO endpoint id, CF zone id; unused by Bunny).

**Legacy fallbacks (deprecated, one release, `tracing::warn!` when used):**
- `CDN_URL` ← `AWS_CDN_URL` (also `BUNNY_CDN_URL` / `CF_CDN_URL` per provider),
- `CDN_PURGE_ZONE` ← `DO_SPACES_CDN_ID` / `CF_ZONE_ID`,
- `CDN_PURGE_TOKEN` ← `DIGITALOCEAN_ACCESS_TOKEN` / `CF_API_TOKEN` / `BUNNY_ACCESS_KEY`.

**In scope:** the unified `ferro_storage::cdn::Config` (SC-1 name) + `from_env` reading the quartet
with per-var legacy fallback and a one-shot deprecation warn; a `CdnProvider` enum with validation
(invalid value → boot error; `none` → purge no-op); construction of the existing per-provider
`PurgeApi` adapter from the unified token/zone; wiring `CDN_URL` into the existing `Disk::cdn_url()`
display path (parity with `AWS_CDN_URL`-only callers); the CHANGELOG entry + ferro-storage minor
version bump; `app/.env.example` migrated to the quartet; the SC-7 ferro-storage gate
(`cargo test --all-features` + `clippy --all -Dwarnings`).

**Out of scope:**
- **New CDN providers or new purge behavior** — this is a *configuration-surface* refactor; the
  `DoSpacesCdn` / `BunnyCdn` / `CloudflareCdn` adapters and their purge logic (Phase 188) are
  unchanged. No new `PurgeApi` impls.
- **Removing the legacy variables** — they stay as warned fallbacks for one release (removal is a
  later phase once consumers migrate; gestiscilo Phase 205 is the consumer rename).
- **Changing `Disk::cdn_url()` output semantics** — SC-3 requires byte-identical URLs for unchanged
  callers; the only change is the *source* env var (with `AWS_CDN_URL` fallback).
- **Multi-provider-at-once** — exactly one provider is active at a time (the abstraction matches the
  code: one CDN per app).

**Carrying forward from Phase 188 (ferro-storage CDN Extension):**
- `PurgeApi` trait + `DoSpacesCdn` (default), `BunnyCdn` (`cdn-bunny` feature), `CloudflareCdn`
  (`cdn-cloudflare` feature) adapters — reused as-is; this phase only changes how their config is
  read.
- `DoSpacesCdnConfig` / `BunnyCdnConfig` / `CloudflareCdnConfig` `from_env()` read the legacy
  clusters — these become the *fallback path* (or are folded behind the unified `Config`).
- `Disk::cdn_url(path)` / `Storage::cdn_url(path)` facade (double-slash-safe, origin `url()`
  fallback) — `CDN_URL` feeds the disk's `cdn_url` exactly where `AWS_CDN_URL` does today
  (`config.rs:119`).
- API tokens are redacted in all `Debug` impls — the unified `Config` must preserve this (token is
  `<redacted>`).

</domain>

<decisions>
## Implementation Decisions

### Unified `cdn::Config` shape (D-01)
- **D-01:** Introduce **`ferro_storage::cdn::Config`** (the exact SC-1 name) as the single
  env-reading entry point for CDN settings: fields `url: Option<String>`,
  `provider: CdnProvider`, `purge_token: Option<String>` (redacted in `Debug`),
  `purge_zone: Option<String>`. `Config::from_env()` reads the quartet primary + legacy fallbacks
  (D-02). A method **`Config::build_purge_api() -> Result<Option<Box<dyn PurgeApi>>, Error>`**
  constructs the selected provider's existing adapter from `purge_token`/`purge_zone`
  (`None` provider → `Ok(None)`, i.e. purge no-op). The existing per-provider `*Config` structs
  are populated *from* the unified `Config` (the unified config is the new source of truth; the
  per-provider structs stay as the adapters' constructor inputs).
  - **[auto] recommended default** — chosen over (b) adding the quartet directly onto
    `StorageConfig` (mixes disk config with CDN provider selection; the CDN surface deserves its own
    `cdn::Config` named exactly as SC-1 specifies) and (c) a flat free-function returning a tuple
    (loses the redaction/`Debug` discipline and the validation seam). A dedicated struct mirrors the
    existing `*CdnConfig` convention and gives one place for fallback + validation + redaction.

### Fallback-chain mechanics + one-shot deprecation warn (D-02)
- **D-02:** Resolve each quartet var with a small **`env_with_fallback(primary, &[(alias, label)])`**
  helper: read `primary` first; if unset, try each alias in order and on first hit emit
  **`tracing::warn!`** once naming the deprecated var and its replacement (e.g. `"AWS_CDN_URL is
  deprecated; use CDN_URL"`). Fallback aliases per var:
  - `CDN_URL` ← `AWS_CDN_URL`, `CF_CDN_URL`, `BUNNY_CDN_URL`,
  - `CDN_PURGE_ZONE` ← `DO_SPACES_CDN_ID`, `CF_ZONE_ID`,
  - `CDN_PURGE_TOKEN` ← `DIGITALOCEAN_ACCESS_TOKEN`, `CF_API_TOKEN`, `BUNNY_ACCESS_KEY`.
  The warn fires at most once per legacy var per process (the alias is only read when the primary is
  absent, so a clean quartet config is silent). The token warn must **not** print the token value.
  - **[auto] recommended default** — chosen over (b) silent fallback (operators never learn to
    migrate; the deprecation window is pointless) and (c) hard error on legacy vars (breaks SC-3/SC-4
    parity for existing `AWS_CDN_URL`-only deployments — the whole point is a *graceful* one-release
    window).

### `CDN_PROVIDER` resolution + validation (D-03)
- **D-03:** `CdnProvider` enum (`None` | `DigitalOcean` | `Bunny` | `Cloudflare`),
  `#[serde(rename_all = "snake_case")]`-style string parsing. Resolution:
  - explicit `CDN_PROVIDER` parsed case-insensitively; **invalid value → boot `Error`** with a
    message listing the valid values (SC-5),
  - **`CDN_PROVIDER` unset → infer** from which legacy cluster is populated for backward-compat
    (`DO_SPACES_CDN_ID` → `digitalocean`, `CF_ZONE_ID` → `cloudflare`, `BUNNY_CDN_URL` →
    `bunny`), logged as a deprecation warn; if none populated → `None`,
  - `CDN_PROVIDER=none` → `purge()` is an **explicit logged no-op** (SC-5).
  Critically, **`CDN_URL` (display) is independent of `provider` (purge)** — an `AWS_CDN_URL`-only
  deployment with no purge creds keeps a working `cdn_url()` with `provider = None` (preserves SC-3
  parity: `cdn_url()` unchanged even when no purge provider is configured).
  - **[auto] recommended default** — inference-on-unset chosen over requiring `CDN_PROVIDER`
    explicitly (would break existing legacy-var deployments that never set it — SC-2/SC-3/SC-4
    parity). Boot-error on *invalid* (not unset) matches SC-5 exactly.

### Feature-gating interaction (D-04)
- **D-04:** `bunny`/`cloudflare` adapters remain behind the `cdn-bunny` / `cdn-cloudflare` cargo
  features (Phase 188). If `CDN_PROVIDER` (or inference) selects a provider whose feature is
  **not compiled in**, `build_purge_api()` returns a **clear boot `Error`** naming the provider and
  the feature flag to enable (e.g. `"CDN_PROVIDER=bunny requires the 'cdn-bunny' feature"`).
  `digitalocean` is always available (no feature). `none` and `digitalocean` never error on
  features.
  - **[auto] recommended default** — explicit boot error chosen over (b) silently falling back to
    no-op purge (a misconfigured production deploy would think purge works when it doesn't) and (c)
    making the providers always-compiled (contradicts Phase 188's zero-default-graph-impact proof
    via `cargo tree`). Fail loud at boot, consistent with SC-5's invalid-provider error.

### Parity preservation — `Disk::cdn_url()` (D-05)
- **D-05:** Wire `CDN_URL` into the **same place** `AWS_CDN_URL` feeds today
  (`config.rs:119` → `s3_config.with_cdn_url(...)`), via the unified `Config.url`. SC-3 parity test:
  with only `AWS_CDN_URL` set (no quartet), `Disk::cdn_url(path)` returns the byte-identical URL it
  returns today. SC-4 parity test: with only the legacy DO vars set, `purge()` authenticates
  against the same DO Spaces CDN API as today. Both go through the fallback path (D-02) so the
  warn fires but behavior is unchanged.
  - **[auto] recommended default** — reuse the exact existing wiring point rather than re-deriving
    the display URL; the only delta is the env-var source + the deprecation warn.

### CHANGELOG, version bump, `.env.example` (D-06)
- **D-06:** Bump **ferro-storage minor version** (`Cargo.toml`) and add a `## [X.Y.0]` CHANGELOG
  entry documenting the new quartet, the per-var deprecation mapping, and the one-release removal
  policy (SC-6). Migrate **`app/.env.example`** CDN section to the quartet (`CDN_URL`,
  `CDN_PROVIDER`, `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) with a short comment that the old
  `AWS_CDN_URL` / `DO_SPACES_CDN_ID` / `DIGITALOCEAN_ACCESS_TOKEN` / `BUNNY_*` / `CF_*` names are
  deprecated fallbacks. (This supersedes the interim env grouping committed in `77d360cf`.) Update
  the ferro-storage docs page if it enumerates the CDN env vars.
  - **[auto] recommended default** — minor bump (additive + deprecation, no breaking removal yet),
    matching the established ferro-storage cadence (Phase 188 was 0.2.45→0.2.46). ferro-storage is
    an **existing published crate** — CI publish-update handles the bump, no new-crate bootstrap.

### Claude's Discretion
- Exact module placement of `env_with_fallback` (private helper in `cdn/mod.rs` vs `config.rs`).
- Whether `CdnProvider` parsing uses `serde` or a hand-rolled `FromStr` (either; keep it case-
  insensitive and error-listing-valid-values).
- Whether the per-provider `*CdnConfig::from_env()` are kept (reading the unified `Config`) or made
  `pub(crate)` constructors taking explicit token/zone — as long as the public env surface is the
  quartet and tokens stay redacted.
- Exact deprecation-warn wording and the precise minor version number.
- Whether provider inference logs at `warn` (deprecation) or `info` — recommend `warn` for any
  legacy var, `info` for a clean `CDN_PROVIDER=none`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` §"Phase 204: ferro-storage provider-agnostic CDN configuration" — goal,
  SC-1…SC-7, origin (cross-repo env-files audit), consumer pairing (gestiscilo Phase 205).

### The code this phase modifies (Phase 188 surface)
- `ferro-storage/src/cdn/mod.rs` — `PurgeApi` trait, `DoSpacesCdnConfig`/`DoSpacesCdn`
  (`DO_SPACES_CDN_ID`, `DIGITALOCEAN_ACCESS_TOKEN`; redacted `Debug`); the new `cdn::Config` lives
  here.
- `ferro-storage/src/cdn/bunny.rs` — `BunnyCdnConfig::from_env` (`BUNNY_CDN_URL`,
  `BUNNY_ACCESS_KEY`); `cdn-bunny` feature.
- `ferro-storage/src/cdn/cloudflare.rs` — `CloudflareCdnConfig::from_env` (`CF_ZONE_ID`,
  `CF_API_TOKEN`, `CF_CDN_URL`); `cdn-cloudflare` feature.
- `ferro-storage/src/config.rs` — `StorageConfig::from_env`; line ~119 reads `AWS_CDN_URL` →
  `with_cdn_url` (the D-05 wiring point); the `from_env_cdn_url` test (parity baseline for SC-3).
- `ferro-storage/src/facade.rs` — `Disk::cdn_url()` (line ~407), `with_cdn_url`,
  `register_disk_with_cdn`; the display-URL path `CDN_URL` feeds.
- `ferro-storage/src/lib.rs` — re-exports (`DoSpacesCdn`, `DoSpacesCdnConfig`, `PurgeApi`); add the
  `cdn::Config` + `CdnProvider` exports.
- `ferro-storage/src/error.rs` — the `Error` enum the new invalid-provider / missing-feature boot
  errors extend (SC-5, D-04).
- `ferro-storage/Cargo.toml` — version bump (D-06) + the `cdn-bunny`/`cdn-cloudflare` feature defs.
- `ferro-storage/CHANGELOG.md` (or the crate's changelog location) — the `## [X.Y.0]` entry (SC-6).

### App + framework integration
- `app/.env.example` — CDN section to migrate to the quartet (currently grouped under
  `# CDN Settings (ferro-storage — optional)` after commit `77d360cf`).
- `CLAUDE.md` §"Project-agnostic crates" — ferro-* crates read framework conventions, must not
  hardcode app identity; the unified `Config` reads only env, no app strings.

### Prior milestone context
- `.planning/ROADMAP.md` §"v12.3 Deployment Platform Primitives (Phases 185–188)" — Phase 188 built
  the CDN purge surface this phase re-fronts; read for the `cargo tree` zero-default-graph-impact
  constraint (do not make bunny/cloudflare always-compiled — D-04).

### External
- No external specs — this is an internal env-surface refactor. Provider purge APIs (DO Spaces CDN,
  Bunny, Cloudflare) are already encapsulated in the Phase 188 adapters and unchanged here.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PurgeApi` + the three adapters (`DoSpacesCdn`/`BunnyCdn`/`CloudflareCdn`) — constructed, not
  rewritten; the unified `Config::build_purge_api()` just selects + feeds them.
- The redacted-`Debug` pattern on `DoSpacesCdnConfig`/`BunnyCdnConfig`/`CloudflareCdnConfig` —
  copy it for `cdn::Config.purge_token`.
- `config.rs:119` `AWS_CDN_URL → with_cdn_url` — the exact insertion point for `CDN_URL` (D-05).
- `facade.rs` `Disk::cdn_url()` double-slash-safe + origin fallback — unchanged; `CDN_URL` only
  changes what populates the disk's `cdn_url`.
- The existing `from_env_cdn_url` test in `config.rs` — the SC-3 parity baseline to keep green.

### Established Patterns
- Each CDN provider has a `*Config` with `from_env()` reading its own cluster — the unified
  `cdn::Config` becomes the new front; per-provider structs become its constructor inputs.
- Tokens are `<redacted>` in every `Debug` impl — non-negotiable, mirror it.
- bunny/cloudflare gated behind cargo features with proven zero default-graph impact (`cargo tree`)
  — preserve; selecting a disabled provider is a boot error (D-04), not an always-compile.
- `thiserror` `Error` enum per crate — add invalid-provider / missing-feature variants.

### Integration Points
- `cdn/mod.rs` → new `Config` + `CdnProvider` + `env_with_fallback` + `build_purge_api`.
- `config.rs::from_env` → source `cdn_url` from `cdn::Config.url` (quartet + `AWS_CDN_URL` fallback).
- `lib.rs` → export `Config as CdnConfig`(or `cdn::Config`) + `CdnProvider`.
- `error.rs` → invalid-provider + missing-feature boot-error variants.
- `Cargo.toml` + CHANGELOG → minor bump + entry.
- `app/.env.example` → quartet migration.

</code_context>

<specifics>
## Specific Ideas

- **The abstraction should match what the code does: one CDN, one provider at a time.** The whole
  motivation (roadmap Origin) is that the same DO Spaces CDN is fragmented across `AWS_*`,
  `SPACES_*`, `DO_SPACES_*` prefixes. The quartet makes the config say what the code means.
- **`CDN_URL` (display) and `CDN_PROVIDER` (purge) are orthogonal axes** — a deployment can serve
  assets via a CDN URL with no purge provider configured (`provider=none`), and that must keep
  working (SC-3). Do not couple them.
- **Parity is the load-bearing guarantee.** SC-3/SC-4 require byte-identical `cdn_url()` and the
  same DO purge auth for legacy-var-only deployments. The deprecation warn is the *only* observable
  change for those deployments — write the parity tests against an `AWS_CDN_URL`-only / legacy-DO-
  only env and assert unchanged output.
- **Fail loud on misconfig** (invalid provider value, provider selected but feature off) — a silent
  purge no-op in production is the dangerous failure mode this phase must avoid.

</specifics>

<deferred>
## Deferred Ideas

- **Removing the legacy env vars** — kept as warned fallbacks for one release; removal is a future
  phase once gestiscilo (Phase 205 consumer rename) and other consumers migrate.
- **Endpoint-level CDN purge rate-limiting / abuse controls** beyond the existing per-adapter
  throttles — unchanged from Phase 188; not in scope.
- **New CDN providers** (Fastly, CloudFront-native, etc.) — the unified surface makes adding one
  easier later, but none are added here.
- **Multi-CDN / per-disk CDN provider** — one provider per app for now; multi-provider is a larger
  abstraction change, not warranted by current usage.

None of these belong in Phase 204 — analysis stayed within scope.

</deferred>

---

*Phase: 204-ferro-storage-provider-agnostic-cdn-configuration*
*Context gathered: 2026-06-11*
