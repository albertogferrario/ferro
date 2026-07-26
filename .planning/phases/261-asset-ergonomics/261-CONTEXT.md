# Phase 261: `asset!()` ergonomics - Context

**Gathered:** 2026-07-26
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults; review before planning)

<domain>
## Phase Boundary

Collapse the current boot-time `ferro-bundle` builder chain to a one-line
`asset!("path")` at the use site, and add an opt-in author-time CLI fetch for
Iconify sets and Fontsource families — all flowing through the existing
content-hashed pipeline with no new infrastructure.

Deliver three things (LIVE-03):
1. **`asset!("relative/path.ext")` macro** (`ferro-macros`) — compile-time embed via
   `include_bytes!`, content-type inference from the extension, one-shot registration
   of a `ferro-bundle::Bundle`, returning the content-hashed URL.
2. **Content-type inference wiring** — a single ext→MIME source of truth the macro
   uses; unrecognized extensions pass through byte-identical.
3. **`ferro assets fetch` subcommand** (`ferro-cli`) — downloads Iconify sets /
   Fontsource families into the project asset dir, after which they flow through the
   existing `ferro-assets` transform pipeline and `asset!()`.

Scope anchors (from ROADMAP v17.0 constraints):
- **No new crates.** Macro in `ferro-macros`, over `ferro-assets`/`ferro-bundle`;
  fetch in `ferro-cli`.
- **Rust toolchain alone.** No node, no nasm, no OpenSSL. The fetch reaches the
  network at author time only.
- **Opt-in, additive.** No default feature pulls a network fetch into a normal
  `cargo build`.

Out of scope: catalog/`generation_context`/docs/publish (Phase 262 closes the loop);
`LiveFragment` (260) and `#[memoize]` (259) already shipped in-tree.

</domain>

<decisions>
## Implementation Decisions

### Macro embedding & path resolution
- **D-01:** `asset!("relative/path.ext")` expands to code containing
  `include_bytes!("relative/path.ext")`, so path resolution uses Rust's native
  **call-site-source-relative** semantics — matching the spec (§3: "expands … to
  `include_bytes!` of the path") and author intuition. The macro does **not** read
  the file itself at proc-macro expansion time (that would resolve against
  CWD/`CARGO_MANIFEST_DIR` and be fragile). Hashing happens on the embedded
  `&'static [u8]` at registration, exactly as `Bundle::new` does today.
  - `[auto]` recommended default — spec-literal, least-surprise.

### Registration lifetime & caching (use-site safety)
- **D-02:** **Lazy register-once at the use site.** The expansion emits a block with a
  private `static U: ::std::sync::OnceLock<String>` that, on first evaluation,
  registers the `Bundle` and caches its hashed URL; every later evaluation returns the
  cached value. This makes `asset!()` safe inside a per-request / hot render path —
  `Bundle::new` is process-global and panics on duplicate name, so naive re-registration
  would panic on the second request.
  - Trade-off (documented): a duplicate-path collision surfaces on **first evaluation**
    (runtime first-hit) rather than at boot. Acceptable — content hashing of embedded
    bytes is cheap and the panic message is a clear developer error.
- **D-03:** **Return type `&'static str`** (via `static OnceLock<String>` →
  `get_or_init(...).as_str()`), avoiding a per-call allocation. `asset!()` is an
  expression usable inline in a template/handler (e.g. `src={asset!("assets/app.js")}`).
- **D-04:** **Bundle name derived deterministically from the sanitized asset path**
  (path separators/dots → underscores), giving a stable, debuggable URL
  (`/bundles/assets_app.<sha8>.js`). The path is the natural unique key; a
  post-sanitization collision is developer error caught by `Bundle`'s existing
  duplicate-name panic.

### Content-type inference (single source of truth)
- **D-05:** Add a `pub fn` **ext→MIME helper in `ferro-bundle`** — the crate that owns
  the `content_type` concept and already carries the reverse `ext_from_content_type`
  (ct→ext) map. Cover the extensions `ferro-bundle` already recognizes (`js`, `css`,
  `html`, `json`, `txt`, `png`, `jpg`/`jpeg`, `svg`, `gif`, `webp`, `woff2`, `woff`,
  `wasm`) plus unknown → `application/octet-stream`. The macro expansion calls it, then
  chains `.content_type(...)`. Unknown extensions serve byte-identical with
  octet-stream (SC #2 passthrough preserved). No inference logic duplicated inside the
  macro.
  - `ferro-assets::infer_content_type` stays as-is — it returns a transform-oriented
    enum (`ContentType`), not a MIME string, so it is the wrong surface for bundle
    registration. Do not force the two together.

### Crate wiring / re-export
- **D-06:** **Re-export `ferro-bundle` from `framework` as `ferro::bundle`** so the
  macro can emit `::ferro::bundle::Bundle` and compile wherever `ferro` is a dependency
  — mirroring the `::ferro::memo::*` path `#[memoize]` reaches via
  `crate::utils::ferro` (`ferro-macros/src/memoize.rs:44`). The `asset` macro lives in
  `ferro-macros` (`#[proc_macro] pub fn asset`), re-exported as `ferro::asset!`, and
  reuses the same `crate::utils::ferro` root-path helper so it works both inside the
  workspace (`crate`) and downstream (`::ferro`).

### `ferro assets fetch` subcommand
- **D-07:** **Command shape:** an `assets` command group with a `fetch` subcommand,
  typed by source — `ferro assets fetch iconify <set>[/<icon>…]` and
  `ferro assets fetch fontsource <family>` — authored with clap-derive `Subcommand`,
  matching the existing CLI style (`ferro-cli/src/main.rs`, `commands/mod.rs`).
- **D-08:** **Reuse the existing `reqwest` blocking + rustls-tls dependency** already in
  `ferro-cli/Cargo.toml` (`reqwest = { default-features = false, features =
  ["blocking", "json", "rustls-tls"] }`). Pure-Rust TLS — no OpenSSL, nasm, or node —
  satisfies "Rust toolchain alone." No new HTTP crate, no new default feature. Opt-in is
  **inherent**: an author-run CLI command is never part of `cargo build`.
- **D-09:** **Output:** write **individual servable files** into the project asset dir —
  default `assets/` at project root (create if missing; `--out` override) — so each
  fetched file then flows through `asset!("assets/…")` and the existing `ferro-assets`
  pipeline. Iconify icons land as `.svg`; Fontsource families as `.woff2` (+ face `.css`
  if needed). Fetch only downloads + writes files; it does **not** auto-generate
  `asset!()` calls or auto-wire routes.
  - `assets/` is a **new consumer-app convention** — no such dir exists in the sample
    app today (existing `assets/` dirs are crate-internal). Establish it here.

### Claude's Discretion
- Exact Iconify + Fontsource **endpoint URLs and response formats** — pin during
  research/planning. Starting points: Iconify API `https://api.iconify.design/{set}.json`
  or per-icon `https://api.iconify.design/{set}/{icon}.svg`; Fontsource via jsDelivr
  `https://cdn.jsdelivr.net/npm/@fontsource/{family}/…`. Which weights/subsets to pull by
  default is a planning detail.
- `OnceLock` vs `LazyLock` for the use-site cache; the exact name-sanitization function;
  whether the macro accepts an optional stable alias argument (recommend **no** for
  v17.0 — keep minimal, see Deferred).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase spec & scope
- `docs/superpowers/specs/2026-07-21-live-projection-surface-design.md` §3 "Asset
  declaration ergonomics" (lines ~116–128), "Phase decomposition" (Phase 261 bullet),
  "Testing" (asset tests), "Honest limitations" (opt-in fetch) — the anchor spec.
- `.planning/ROADMAP.md` v17.0 "Phase 261: `asset!()` ergonomics" (Goal, Depends on,
  Success Criteria #1–3) + the milestone "Architectural constraints" block
  (no-new-crates, Rust-toolchain-only, single publish at 262).

### Requirement
- LIVE-03 (`asset!()` macro + Iconify/Fontsource fetch) → Phase 261, per the ROADMAP
  v17.0 Requirement→Phase mapping table.

### Prior-phase precedent (consistency)
- `.planning/phases/259-request-scoped-memoization/259-CONTEXT.md` — the `#[memoize]`
  proc-macro + `crate::utils::ferro` root-path pattern this phase mirrors.
- `.planning/phases/260-live-reactive-fragment/260-CONTEXT.md` — bundle/client-runtime
  interplay context (the client runtime that 262 may ship via `asset!()`-style embed).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ferro-bundle` (`ferro-bundle/src/lib.rs`)** — `Bundle::new(name, &'static [u8])`
  (SHA-256 hash + process-global registry, panics on duplicate name),
  `.content_type(ct)` (re-keys hashed URL), `.with_alias(path)`, `.hashed_url()`,
  `.serve(req)`, and the private `ext_from_content_type` (ct→ext) to invert for D-05.
  This is exactly the boot-time chain `asset!()` collapses.
- **`ferro-assets` (`ferro-assets/src/{lib,asset,pipeline}.rs`)** — `Pipeline`/`Transform`
  with byte-identical passthrough for unknown types (the SC #2 guarantee source);
  `infer_content_type` (ext→`ContentType` enum). Fetched files flow through this
  unchanged — no pipeline change needed.
- **`ferro-macros` (`ferro-macros/src/{lib,memoize,utils}.rs`)** — proc-macro machinery;
  `#[memoize]` (Phase 259) is the direct sibling precedent for registering a new macro
  and reaching the crate root via `crate::utils::ferro`.
- **`ferro-cli`** — clap-derive `Subcommand` enum in `src/main.rs`, `pub mod` list in
  `src/commands/mod.rs`; existing `reqwest` blocking+rustls-tls dep (D-08).

### Established Patterns
- Process-global `OnceLock<DashMap>` registries with panic-on-duplicate (ferro-bundle).
- Use-site `static OnceLock<_>` lazy init (idiomatic target for the macro expansion, D-02).
- Macro crate-root resolution via `crate::utils::ferro` so expansions compile both in the
  workspace (`crate`) and downstream (`::ferro`).

### Integration Points
- **`framework/src/lib.rs`** — needs a `pub use ferro_bundle as bundle;` (or equivalent)
  re-export so `::ferro::bundle::Bundle` resolves (D-06); add `ferro-bundle` to
  `framework/Cargo.toml` deps if not already present.
- **`ferro-macros/src/lib.rs`** — register `#[proc_macro] pub fn asset`; re-export as
  `ferro::asset!` from `framework`.
- **App boot** — the app must still mount `Bundle::serve` on `/bundles/{filename}` (+
  aliases) for the URL `asset!()` returns to resolve. `asset!()` does not change routing.
- **`ferro-cli/src/main.rs` + `commands/mod.rs`** — register the `assets fetch` command
  (D-07); new module `commands/assets_fetch.rs` (or `assets.rs`).

</code_context>

<specifics>
## Specific Ideas

- The one-liner target, verbatim from the spec: collapse
  `Bundle::new(name, bytes).content_type(ct).with_alias(path)` → `asset!("path")` at
  the use site, reusing the same immutable-cache machinery underneath.
- SC framing to preserve: hashed-URL **stability across builds for unchanged bytes**,
  content-type inference table, byte-identical passthrough for unrecognized extensions.

</specifics>

<deferred>
## Deferred Ideas

- **Macro-emitted stable alias** (`asset!("path", alias = "/app.js")`) — the boot chain's
  `.with_alias` equivalent. Out of scope for v17.0's minimal one-liner; add only if a
  consumer needs a stable non-hashed URL.
- **Auto-wiring fetched assets into `asset!()` calls / route generation** — `ferro assets
  fetch` downloads + writes only; generating the reference sites is future work.
- **Delta-granular / list-diffing directions** — belong to the live-fragment track, not
  assets.

None of these blocked the phase — scope stayed within the three LIVE-03 deliverables.

</deferred>

---

*Phase: 261-asset-ergonomics*
*Context gathered: 2026-07-26*
