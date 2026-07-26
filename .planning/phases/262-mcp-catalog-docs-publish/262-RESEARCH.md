# Phase 262: MCP + catalog + docs + publish - Research

**Researched:** 2026-07-26
**Domain:** Ferro MCP generation_context authoring guidance, JSON-UI docs, operator-gated crates.io publish
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** SC-1 is pre-satisfied. Work = run `builtin_types_count_drift_guard` + `test_all_components_present` and record evidence. No re-implementation.
- **D-02:** Audit `json_ui_catalog` per-component output for `LiveFragment` props schema (`projection`, `key`, `template`). Fix additive gaps; existing output shape stays backward-compatible. Any static supplement drift-guarded against registry.
- **D-03:** `generation_context.rs` — add three-capability guidance: LiveFragment (when/how/channel contract/first-paint/one-binding limitation), `#[memoize]` (request-scoped dedup, complement not replace, error caching), `asset!()` (one-liner, content-hash, `ferro::bundle` mount required, `ferro assets fetch`).
- **D-04:** Style mirrors 253 D-06 / 258 D-04 — compact: ids and one-liners with pointer to `docs/src` for depth.
- **D-05:** Drift-guard all hand-written guidance: component name vs `BUILTIN_TYPES`/`BUILTIN_SPECS`, macro exports vs actual re-exports, `data-live-fragment`/`data-channel` vs `containers.rs` + `live_fragment.rs`. Pattern from `register_composition_drift_guard`.
- **D-06:** Extend existing pages first. LiveFragment → `docs/src/json-ui/components.md` + `runtime-primitives.md`; `asset!()` + `ferro assets fetch` → `docs/src/features/ferro-assets.md`; `#[memoize]` → `docs/src/features/projections.md` (or short dedicated section). Every capability ≥1 usage example. New pages wired into `SUMMARY.md`.
- **D-07:** mdBook docs build exits 0. Neutral product-documentation voice; no version-vs-version framing.
- **D-08:** NO `ferro-base.css` regen: `render_live_fragment` emits only `data-*` attributes. Grep-confirm during execution; regen only if a new class surfaces.
- **D-09:** CI-exact gate: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus `cargo doc --no-deps --all-features` with `-Dwarnings`. Serialize CPU runs; disk-check before test. Schema-export churn discarded.
- **D-10:** Operator-gated publish. Pre-publish checklist at gate. Post-publish verify via crates.io / gh API; never local `origin/*` refs; run `git update-ref refs/remotes/origin/master HEAD` after verified push.
- **D-11:** Workspace at `0.2.91` (unpublished). At gate: if crates.io < 0.2.91 → publish 0.2.91 as-is; if crates.io ≥ 0.2.91 → bump to `crates.io_max + 1`. Single publish commit, manual bump.
- **D-12:** `ferro-payments` at `0.1.6`. At gate, check crates.io: if already 0.1.6, skips (cargo handles); if behind, rides publish. No code changes.
- **D-13:** No new crates. Verify `ferro-bundle` is Wave 1a in `publish.yml` (261 D-06 moved it). Confirm `ferro-a2ui` absent.
- **D-14:** Currently on `master`, 0 ahead/behind. Assert HEAD=master from repo root before ref move. Push via gh HTTPS credential helper (SSH denied).
- **D-15:** Stage specific files only. Exclude: `app/frontend/node_modules/.vite/deps_temp_*`, `.planning/config.json` workflow-flag churn, phantom `planning/phases/158-...` deletion.

### Claude's Discretion

- Exact docs placement within D-06 constraint (which existing page; whether `#[memoize]` gets its own short page) and section ordering inside `components.md`.
- Exact `generation_context` section naming/structure for the three-capability guidance and how much detail is inline vs deferred to docs pointer.
- Whether any `json_ui_catalog` guidance gap found under D-02 is fixed in `ferro-json-ui` or `ferro-mcp`.
- Test organization for D-05 drift guards (one combined test vs per-capability).
- Pre-publish checklist composition details at the D-10 gate.

### Deferred Ideas (OUT OF SCOPE)

- Keyed live lists / collection reconciliation (v17.0 Future direction)
- Delta-granular fragment patches
- Multiple distinct fragment templates over the same projection
- Macro-emitted stable alias (`asset!("path", alias = "/app.js")`)
- Auto-wiring fetched assets into `asset!()` calls
- v16.6 / earlier milestone archival
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LIVE-04 | `ferro-mcp` catalog + `generation_context` for all three v17.0 capabilities + `docs/src` coverage + single operator-gated crates.io publish | SC-1 pre-satisfied (drift guards green at 53); SC-2 = `generation_context.rs` additions (zero v17.0 content today); SC-3 = `docs/src` additions (zero v17.0 content today); SC-4 = workspace 0.2.91 already bumped, publish.yml wave order correct |
</phase_requirements>

---

## Summary

Phase 262 is a closeout phase that surfaces three shipped v17.0 capabilities — `LiveFragment`, `#[memoize]`, and `asset!()` — through ferro's agent-authoring channel (`generation_context`) and human-facing docs (`docs/src`), then performs the single milestone publish.

SC-1 (catalog count lockstep at 53 including `LiveFragment`) is **pre-satisfied** in-tree: both `ferro-json-ui/src/catalog.rs:1303` and `ferro-mcp/src/tools/json_ui_catalog.rs:420` already assert 53, and both tests pass locally (`builtin_types_count_drift_guard ok`, `test_all_components_present ok`). This phase re-runs them to record evidence and checks whether `LiveFragment`'s per-component props schema in the catalog output exposes `projection`, `key`, and `template` fields adequately (D-02 audit). The `LiveFragmentProps` struct has exactly these three fields and is already wired into `BUILTIN_SPECS` — the schema derives automatically; no static supplement is expected to be needed.

SC-2 (`generation_context`) and SC-3 (`docs/src`) are the entire substantive scope. `generation_context.rs` has zero mentions of any of the three capabilities (`grep` returns empty). Docs likewise have zero coverage. The agent-authoring guidance for `LiveFragment` (the killer-feature deliverable) must be precise and drift-guarded: the container emits `data-live-fragment` and `data-channel="projection.{name}.{key}"` (sourced from `containers.rs:1678`), the client runtime subscribes on `/_ferro/ws` and swaps `innerHTML` on a `fragment` event (sourced from `runtime/live_fragment.rs`), and the one-binding-pattern non-goal must be stated. For `#[memoize]`, the key guidance facts are: request-scoped task-local (`MEMO_STORE`), coalesces concurrent callers, errors cached, graceful no-op outside request scope, complements `eager_loading`/`BatchLoad` (not a replacement), not cross-request. For `asset!()`: call-site-source-relative `include_bytes!`, lazy `OnceLock` registration, `&'static str` return, `ferro::bundle` mount required for the URL to resolve, `ferro assets fetch iconify|fontsource` for author-time downloads.

SC-4 publish is straightforward: workspace is at `0.2.91` (already bumped), `ferro-bundle` is already in Wave 1a in `publish.yml`, and the CI gate matches the local gate exactly (`cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all-features` + `cargo doc --no-deps --all-features`).

**Primary recommendation:** spend the polish budget on `generation_context` guidance quality for `LiveFragment` — precision of the data-attribute contract and the one-binding-pattern limitation. The catalog verification and publish are commodity; the docs follow naturally from the guidance content.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `generation_context` guidance | `ferro-mcp` | `ferro-json-ui`, `ferro-macros` (authoritative sources) | Guidance text lives in `ferro-mcp`; it derives/validates against the source-of-truth registries in the other crates |
| Catalog count drift guard (SC-1) | `ferro-json-ui` (canonical) | `ferro-mcp` (mirror) | `ferro-json-ui/src/catalog.rs` is the single source; `ferro-mcp` mirrors it and has a cross-crate count test |
| `LiveFragment` runtime contract | `ferro-json-ui` (server render + client runtime) | — | `containers.rs` owns the HTML contract; `runtime/live_fragment.rs` owns the client JS |
| docs/src coverage (SC-3) | `docs/src/` mdBook pages | — | Static markdown; no crate logic |
| Publish wave ordering | `.github/workflows/publish.yml` | — | Already updated in Phase 261; verify only |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-json-ui` | workspace 0.2.91 | Canonical builtin registry + `LiveFragmentProps` schema | Single source for SC-1 drift guards |
| `ferro-mcp` | workspace 0.2.91 | `generation_context` + `json_ui_catalog` tools | Agent-authoring surface |
| `ferro-macros` | workspace 0.2.91 | `#[memoize]` + `asset!()` proc-macros | Already exported as `ferro::memoize` / `ferro::asset!` |
| `ferro-bundle` | workspace 0.2.91 | Content-hashed bundle registry | Already decoupled (Wave 1a), re-exported as `ferro::bundle` |
| mdBook | system / CI | docs/src build | Config at `docs/book.toml`; build = `mdbook build docs/` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ferro_json_ui::global_catalog()` | — | Provides `components_sorted()` iterator for drift-guard tests | All D-05 tests that check component names |
| `ferro_json_ui::FERRO_RUNTIME_JS` | — | Assembled runtime bundle string | D-05 test asserting `data-live-fragment`/`data-channel` presence |
| `ferro_theme::token::ALL_TOKENS` | — | Token count for existing `token_description_count_matches_all_tokens` test | Already passing; do not touch |

## Architecture Patterns

### System Architecture Diagram

```
User/Agent
    │
    ▼
ferro-mcp `generation_context` tool
    │  (SC-2: add LiveFragment / #[memoize] / asset!() sections)
    │
    ├──derives from──► ferro-json-ui `global_catalog()` ──► BUILTIN_TYPES/BUILTIN_SPECS (LiveFragment props)
    │
    ├──derives from──► ferro-json-ui `FERRO_RUNTIME_JS` (data-live-fragment, data-channel)
    │
    └──derives from──► ferro-macros (memoize, asset proc-macros)
                            │
                            └──re-exported as──► ferro::memoize, ferro::asset!

ferro-mcp `json_ui_catalog` tool (SC-1 — pre-satisfied)
    │
    └──derives from──► ferro-json-ui `global_catalog()` [53 components incl. LiveFragment]

docs/src/ (SC-3)
    ├── json-ui/components.md       ← LiveFragment component section
    ├── json-ui/runtime-primitives.md ← LiveFragment WebSocket behavior
    ├── features/ferro-assets.md    ← asset!() + ferro assets fetch
    └── features/projections.md     ← #[memoize] render-dedup section

publish.yml (SC-4)
    Wave 1a: ferro-json-ui, ferro-bundle, ferro-macros, ...
    Wave 1b: ferro-projections, ...
    Wave 2:  ferro-rs (framework), ferro-mcp, ...
    Wave 3:  ferro-cli
```

### Recommended Project Structure

No structural changes needed. All work is additive within existing files.

```
ferro-mcp/src/tools/
├── generation_context.rs   ← SC-2: add LiveProjectionGuidance struct + 3 sections + drift-guard tests
└── json_ui_catalog.rs      ← SC-1: evidence only (tests already pass)

docs/src/
├── json-ui/
│   ├── components.md           ← add LiveFragment component section
│   └── runtime-primitives.md   ← add LiveFragment WebSocket behavior section
├── features/
│   ├── ferro-assets.md         ← add asset!() section + ferro assets fetch
│   └── projections.md          ← add #[memoize] render-dedup section
└── SUMMARY.md                  ← update if any page is added
```

### Pattern 1: generation_context Section — New Guidance Field (D-03/D-04)

Mirror the `RegisterCompositionGuidance` pattern exactly: a new `pub struct LiveProjectionGuidance` (or similar name — planner's discretion) with typed `&'static str` fields for each sub-topic, plus derived fields for anything coming from a registry.

```rust
// Source: ferro-mcp/src/tools/generation_context.rs (existing RegisterCompositionGuidance pattern)
#[derive(Debug, Serialize)]
pub struct LiveProjectionGuidance {
    /// When to use LiveFragment vs polling/Inertia/manual reload.
    pub when_to_use: &'static str,
    /// Container + channel contract (data-live-fragment, data-channel format).
    pub container_contract: &'static str,
    /// First-paint behavior when no snapshot exists.
    pub first_paint: &'static str,
    /// Explicit non-goals / limitations.
    pub limitations: &'static str,
    // ... similar for memoize and asset fields
}
```

The `execute()` function in `generation_context.rs` assembles these similarly to how `register_composition` is assembled today. The `GenerationContext` struct gains a new `live_projection: LiveProjectionGuidance` (or equivalent) field.

**Compact style rule (D-04):** each field is one or two sentences max. The field value ends with a pointer like `"See docs/src/json-ui/components.md#livefragment for a usage example."`. Do NOT write a manual in `generation_context`; that belongs in `docs/src`.

### Pattern 2: Drift-Guard Test — Component Name + Attribute + Macro Export

Mirror `register_composition_drift_guard` (line 559 in `generation_context.rs`). Three assertions per capability:

1. **Component names** mentioned in prose exist in `ferro_json_ui::global_catalog().components_sorted()`.
2. **Data attributes** mentioned exist in `ferro_json_ui::FERRO_RUNTIME_JS` (for `data-live-fragment` and `data-channel`).
3. **Macro re-exports** are accessible — verify `ferro::memoize` and `ferro::asset!` compile (can use a `#[allow(unused)]` invocation or a doc-test, or simply assert the exported symbol names are stable by referencing the modules).

```rust
// Source: ferro-mcp/src/tools/generation_context.rs:559 (register_composition_drift_guard)
#[test]
fn live_projection_drift_guard() {
    let ctx = execute();

    // 1. LiveFragment is a builtin
    let builtins: HashSet<String> = ferro_json_ui::global_catalog()
        .components_sorted()
        .map(|c| c.name.clone())
        .collect();
    assert!(builtins.contains("LiveFragment"), "LiveFragment must be a builtin");

    // 2. Data attributes appear in the assembled runtime bundle
    for attr in ["data-live-fragment", "data-channel"] {
        assert!(
            ferro_json_ui::FERRO_RUNTIME_JS.contains(attr),
            "runtime bundle missing `{attr}`"
        );
    }

    // 3. Prose mentions the attribute names
    let prose = format!("{}", ctx.live_projection.container_contract);
    for attr in ["data-live-fragment", "data-channel"] {
        assert!(prose.contains(attr), "guidance prose no longer mentions `{attr}`");
    }
}
```

### Pattern 3: docs/src Component Section Format (from `components.md`)

Each component section in `docs/src/json-ui/components.md` follows this format:

```markdown
### LiveFragment

[One-sentence description framing the use case]

| Prop | Type | Description |
|------|------|-------------|
| `projection` | `string` | ferro-projection NAME (`Projection::NAME` const) |
| `key` | `string` | Per-key channel selector |
| `template` | `object` | Child spec rendered against snapshot as data scope |

**Usage example:**

```json
{
  "live_stock": {
    "type": "LiveFragment",
    "props": {
      "projection": "inventory",
      "key": "warehouse-a",
      "template": {
        "schema": "ferro-json-ui/v2",
        "root": "count",
        "elements": { "count": { "type": "Text", "props": { "content": "$count" } } }
      }
    }
  }
}
```
```

The `docs/src/json-ui/runtime-primitives.md` section for LiveFragment follows the existing `data-lazy-hero` section format: Contract table → Selector/Behavior description → Usage HTML snippet → Browser support note.

### Anti-Patterns to Avoid

- **Re-implementing SC-1 work:** The count and mirror are already at 53 with `LiveFragment`. Do not change these values or re-add drift guards that already exist. Re-run them; record the result.
- **Marketing language in docs:** Docs must be neutral product documentation. No "killer feature", "revolutionary", "unprecedented".
- **Version comparison framing:** No "v1 vs v2", "old vs new", "legacy" framing per `feedback_json_ui_naming`.
- **Commerce/domain nouns in ferro-* crates:** Component names, guidance prose, and attribute names in ferro-mcp/ferro-json-ui must be domain-neutral per `feedback_catalog_vocabulary_structural_nouns`. Examples are OK when explicitly framed as samples (e.g., `"projection": "inventory"` in a usage example is fine).
- **Skipping `fmt --check` after hand edits:** The recurrent publish-blocker. Run `cargo fmt --all -- --check` after EVERY file edit, even small ones. `cargo clippy` and `cargo test` do not catch rustfmt drift.
- **Mid-phase publishes:** One publish only, at the final gate with operator go. No interim crates.io pushes.
- **Trusting local `origin/*` refs:** Always verify remote state via `gh api` or `curl crates.io` — local refs are recurrently stale.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Component-name validation in guidance tests | Hand-coded name list checked against prose | `ferro_json_ui::global_catalog().components_sorted()` | Catalog IS the registry; a separate list creates two sources of truth |
| Runtime-attribute validation | Manual string search | `ferro_json_ui::FERRO_RUNTIME_JS.contains(attr)` | The assembled runtime bundle IS the truth; grep on source is fragile |
| Prop schema for LiveFragment | Hand-written JSON in catalog tool | `serde_json::from_value(schema_for!(LiveFragmentProps))` already in `BUILTIN_SPECS` | Schema derives from the struct; no duplication needed |
| mdBook build verification | Custom HTML scrape / content check | `mdbook build docs/` exit code = 0 | mdBook validates SUMMARY.md links + missing files (D-06 `create-missing = false`) |

**Key insight:** For this phase, "don't hand-roll" means "don't duplicate registries". Every authoritative fact about `LiveFragment`, `#[memoize]`, and `asset!()` already lives in a concrete artifact (struct definition, proc-macro source, runtime JS). The guidance/docs are thin pointer layers, not re-implementations.

## Common Pitfalls

### Pitfall 1: Claiming SC-1 Work as This Phase's Output
**What goes wrong:** Commit message or verification report attributes the count/mirror bump to Phase 262.
**Why it happens:** It looks like the natural scope.
**How to avoid:** The Phase 260 Plan 04 commit already owns that work. Phase 262's SC-1 contribution is "ran both tests, recorded green result". State this explicitly in the verification.
**Warning signs:** If you find yourself changing `catalog.rs:1303` or `json_ui_catalog.rs:420`, stop — those are already correct.

### Pitfall 2: `cargo fmt` After Any Hand-Edit
**What goes wrong:** A hand-written `&'static str` or doc-string with non-canonical whitespace causes `fmt --check` to fail at CI time, blocking publish.
**Why it happens:** `cargo clippy` and `cargo test` pass regardless of fmt state; the error only surfaces at `fmt --check`. Local rustfmt 1.8.0-stable matches CI (toolchain 1.88.0).
**How to avoid:** Run `cargo fmt --all -- --check` immediately after each file edit. Do not batch fmt until the end.
**Warning signs:** Any hand-edited `.rs` file without a subsequent fmt check.

### Pitfall 3: `cargo doc` Warnings Breaking the CI Docs Gate
**What goes wrong:** A new pub struct field or doc-comment in `generation_context.rs` triggers a `missing_docs` or broken-link warning, and `RUSTDOCFLAGS: -Dwarnings` (ci.yml:74) turns it into an error.
**Why it happens:** `cargo test` does not run `cargo doc`; the doc gate is a separate CI job.
**How to avoid:** Run `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS=-Dwarnings` locally before the publish gate. Every new `pub` item must have a doc comment.
**Warning signs:** Any new `pub struct`, `pub fn`, or `pub field` without a `///` doc comment.

### Pitfall 4: `schema-export` Test Dirtying the Tree
**What goes wrong:** `cargo test --all-features` regenerates `docs/protocol/schemas/*.json`, making `git status` dirty and potentially including schema churn in the publish commit.
**Why it happens:** The Phase 94 schema-export test always rewrites the JSON files if their content diverges from what the current code emits.
**How to avoid:** After running the full test gate, check `git diff docs/protocol/schemas/`. If only that path is dirty, run `git checkout docs/protocol/schemas/` to discard (D-09 / `project_schema_export_test_dirties_tree` memory). Only include schema changes if a genuine schema diff results from this phase's code changes (not expected).
**Warning signs:** `git status` showing modified `docs/protocol/schemas/*.json` after tests.

### Pitfall 5: Version-Bump Double-Counting with CI Auto-Bump
**What goes wrong:** Pushing a manually-bumped 0.2.91 triggers the CI `bump-version` job (because 0.2.91 is already tagged), resulting in CI auto-bumping to 0.2.92 before publishing — fine, but creates confusion if the operator expected exactly 0.2.91 on crates.io.
**Why it happens:** `publish.yml` checks if `v0.2.91` is already a git tag. If it is (because a prior CI run tagged it), it bumps to 0.2.92 before publishing.
**How to avoid:** At gate time, read crates.io ferro-rs current version. If crates.io is already at 0.2.91, bump the workspace to 0.2.92 manually before pushing the publish commit (D-11: the manual-bump-so-CI-publishes-directly pattern, same as 0.2.75/0.2.85/0.2.89).
**Warning signs:** `git tag | grep v0.2.91` returns a result — means the tag exists and CI will auto-bump.

### Pitfall 6: Vite Cache Deletions in the Stage
**What goes wrong:** `git add -A` accidentally stages 36 `app/frontend/node_modules/.vite/deps_temp_*` path deletions, polluting the publish commit.
**Why it happens:** These tracked-path deletions accumulate in the working tree and match `-A` staging.
**How to avoid:** Always stage specific files by name (`git add ferro-mcp/src/tools/generation_context.rs docs/src/...`). Never `git add -A` or `git add .`. (D-15)

## Code Examples

### Current `generation_context.rs` Struct Shape (add new field)

```rust
// Source: ferro-mcp/src/tools/generation_context.rs:7-18
pub struct GenerationContext {
    pub naming_conventions: NamingConventions,
    pub file_structure: FileStructure,
    pub common_patterns: CommonPatterns,
    pub avoid: Vec<String>,
    pub imports: ImportTemplates,
    pub design_system: DesignSystemSummary,
    pub register_composition: RegisterCompositionGuidance,
    // ADD:
    pub live_projection: LiveProjectionGuidance,  // or equivalent name
}
```

### `LiveFragmentProps` (authoritative source for guidance and docs)

```rust
// Source: ferro-json-ui/src/component.rs:753-763
pub struct LiveFragmentProps {
    pub projection: String,  // ferro-projection NAME (Projection::NAME const)
    pub key: String,         // per-key channel selector
    pub template: serde_json::Value,  // child Spec as JSON
}
```

### `render_live_fragment` Output Contract (authoritative)

```rust
// Source: ferro-json-ui/src/render/containers.rs:1678
format!(r#"<div data-live-fragment data-channel="{channel}">{inner_html}</div>"#)
// where channel = format!("projection.{}.{}", props.projection, props.key)
```

### Client Runtime Subscription Channel Format (authoritative)

```javascript
// Source: ferro-json-ui/src/runtime/live_fragment.rs:22,44-54
// Subscribe: ws.send(JSON.stringify({ type: 'subscribe', channel: "projection.{name}.{key}" }))
// Receive: { type: "event", event: "fragment", channel: "...", data: { html: "..." } }
// Action: channelMap[msg.channel].innerHTML = msg.data.html
```

### `asset!()` Expansion Contract (authoritative)

```rust
// Source: ferro-macros/src/asset.rs — what asset!("assets/app.js") expands to
{
    static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String> = ::std::sync::OnceLock::new();
    __FERRO_ASSET_URL.get_or_init(|| {
        static __FERRO_ASSET_BYTES: &[u8] = include_bytes!("assets/app.js");
        ::ferro::bundle::Bundle::new("assets_app_js", __FERRO_ASSET_BYTES)
            .content_type(::ferro::bundle::mime_from_ext("js"))
            .hashed_url()
    }).as_str()
}
// Return type: &'static str (content-hashed URL, e.g. "/bundles/assets_app.a1b2c3d4.js")
```

### `#[memoize]` Usage Contract (authoritative)

```rust
// Source: ferro-macros/src/memoize.rs:1-37, ferro/src/lib.rs:360
// Import: use ferro::memoize;  (re-exported from ferro-macros)
// Apply to any async fn whose value arguments implement Hash:
#[memoize]
async fn fetch_stock(warehouse_id: String) -> Result<StockLevel, AppError> {
    // body runs at most once per (callsite, warehouse_id) per request
}
// Outside request scope (no MEMO_STORE task-local): runs un-memoized, no panic (D-02)
// Errors are cached — a transient error is returned to all coalesced callers (D-04)
```

### Framework Re-Exports (agents use these)

```rust
// Source: framework/src/lib.rs:134,354,360 + framework/src/bundle.rs:23
pub use memo::{current_memo_store, MemoKey, MemoStore};  // ferro::memo::*
pub use ferro_macros::asset;   // ferro::asset!
pub use ferro_macros::memoize; // ferro::memoize
pub mod bundle;                // ferro::bundle::{Bundle, BundleResponse, mime_from_ext, serve}
```

### mdBook Build Command

```bash
# SC-3 gate: mdBook exits 0
mdbook build docs/

# Config: docs/book.toml
# [build] create-missing = false  ← missing pages are errors, not auto-created
```

### Prior Closeout Publish Choreography (from Phase 258 pattern)

```bash
# 1. Assert HEAD=master from main repo root
git -C /path/to/ferro rev-parse --verify HEAD
git -C /path/to/ferro branch --show-current   # must be "master"

# 2. Read crates.io current version (never trust local origin/* refs)
curl -s https://crates.io/api/v1/crates/ferro-rs | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['crate']['newest_version'])"
# OR: gh api https://crates.io/api/v1/crates/ferro-rs

# 3. Version decision (D-11)
# If crates.io version < 0.2.91 → use 0.2.91 as-is (workspace already at 0.2.91)
# If crates.io version >= 0.2.91 → bump workspace to crates.io_max + 1

# 4. Push via HTTPS credential helper (SSH denied)
git -c credential.helper='!gh auth git-credential' push https://github.com/albertogferrario/ferro.git master

# 5. Post-push fix stale ref
git update-ref refs/remotes/origin/master HEAD

# 6. Post-publish verify
curl -s "https://crates.io/api/v1/crates/ferro-rs/versions" | python3 -c "import sys,json; vs=json.load(sys.stdin)['versions']; print([v['num'] for v in vs[:3]])"
```

### CI-Exact Gate Commands (in order, serialized)

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --all-features
# (cargo-deny runs as a separate CI job; not local-gate-blocking but note deny.toml)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Bundle::new(...).content_type(...).hashed_url()` boot chain | `asset!("path")` one-liner | Phase 261 (v17.0) | Single-site embed with lazy registration and content-hashed URL |
| Polling / manual reload for live data | `LiveFragment` builtin binding per-key projection snapshot | Phase 260 (v17.0) | No-WASM live updates via server-push HTML over existing broadcast channel |
| Re-fetch on every intent render | `#[memoize]` on fetch fn / service method | Phase 259 (v17.0) | N intents over one key = one fetch per request; concurrent callers coalesce |
| `ferro-bundle` in Wave 3 (depended on ferro-rs) | `ferro-bundle` in Wave 1a (leaf, framework-agnostic) | Phase 261 (v17.0) | Enabled `framework` → `ferro-bundle` dep + `ferro::bundle` re-export |

**Deprecated/outdated:**
- The old `Bundle::serve(Request)` signature (breaking change in Phase 261 decoupling; ferro-bundle is `publish = false` — wait, it IS published. The decoupled `serve_path`/`BundleResponse` API is the new contract; `framework/src/bundle.rs` provides the `Request → HttpResponse` adapter as `ferro::bundle::serve`).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | mdBook can be invoked as `mdbook build docs/` from the repo root | Standard Stack, Code Examples | Risk LOW: `docs/book.toml` exists and points to `src = "src"`; command is standard mdBook CLI |
| A2 | `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS=-Dwarnings` matches the CI Docs job exactly | Common Pitfalls / CI gate | Risk LOW: verified against `ci.yml:72-74` — exact match |
| A3 | `ferro-payments 0.1.6` on crates.io is current (will not need re-publish) | D-12 | Risk LOW: memory confirms 0.1.6 shipped with v16.6; no code changes in v17.0 |

## Open Questions

1. **D-02: Does the LiveFragment per-component catalog output (props schema) need any supplemental description?**
   - What we know: `LiveFragmentProps` has `projection` (String), `key` (String), `template` (serde_json::Value). The `BUILTIN_SPECS` entry at `catalog.rs:382` has a description string: "Binds a child template to a ferro-projection per-key snapshot; re-renders in place on each delta via server-push HTML over the ferro-broadcast WebSocket." The `schemars`-derived schema for `LiveFragmentProps` gives types but minimal field descriptions.
   - What's unclear: Whether agents reading the catalog JSON Schema for `LiveFragment` get enough guidance on `template` (it's a `Value` → `{}` in JSON Schema, not a nested Spec schema). A `BUILTIN_SPECS` description line for the `template` field may help.
   - Recommendation: During execution, inspect the actual catalog output for `LiveFragment` (call `execute(None)` and print `component_schemas["LiveFragment"]`). If `template` shows as `{}` or `true`, add a supplemental description in the `BUILTIN_SPECS` entry; if it shows a nested structure, no change needed. This is a D-02 discretion call.

2. **D-13 publish.yml verification: is `ferro-bundle` Wave 1a?**
   - What we know: `publish.yml:217` contains `WAVE1A_CRATES="... ferro-bundle"` — `ferro-bundle` is already in Wave 1a. [VERIFIED: publish.yml line 217]
   - No action needed. D-13 is pre-satisfied.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | CI gate | ✓ | 1.88.0 (CI-pinned) | — |
| mdBook | SC-3 gate | CHECK at execution | — | Install: `cargo install mdbook` |
| `gh` CLI | HTTPS push, post-publish verify | ✓ (per project memory) | — | — |
| crates.io API (read) | D-11 version resolution | Network at gate time | — | `cargo search ferro-rs` as fallback |

**Missing dependencies with no fallback:**
- mdBook: confirm `mdbook --version` before running the docs gate; install if absent (`cargo install mdbook`).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[test]`) |
| Config file | none (workspace-level cargo test) |
| Quick run command | `cargo test -p ferro-mcp generation_context && cargo test -p ferro-json-ui catalog` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LIVE-04 (SC-1) | `builtin_types_count_drift_guard` asserts 53 | unit | `cargo test -p ferro-json-ui -- catalog::tests::builtin_types_count_drift_guard --all-features` | ✅ exists (catalog.rs:1294) |
| LIVE-04 (SC-1) | `test_all_components_present` asserts 53 incl. LiveFragment | unit | `cargo test -p ferro-mcp -- tools::json_ui_catalog::tests::test_all_components_present --all-features` | ✅ exists (json_ui_catalog.rs:411) |
| LIVE-04 (SC-2) | `live_projection_drift_guard` asserts component name + runtime attributes + macro names match authoritative sources | unit | `cargo test -p ferro-mcp -- tools::generation_context::tests::live_projection_drift_guard --all-features` | ❌ Wave 0 — create in this phase |
| LIVE-04 (SC-2) | `test_generation_context_has_all_sections` asserts new `live_projection` field is non-empty | unit | `cargo test -p ferro-mcp -- tools::generation_context::tests::test_generation_context_has_all_sections --all-features` | ✅ exists (update to cover new field) |
| LIVE-04 (SC-3) | mdBook build exits 0 | smoke | `mdbook build docs/` | ✅ (docs/book.toml exists) |
| LIVE-04 (SC-4) | `cargo publish -p ferro-rs --dry-run` exits 0 | gate check | `cargo publish -p ferro-rs --dry-run` | ✅ (workspace Cargo.toml) |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp -- tools::generation_context 2>&1 | tail -5`
- **Per wave merge:** `cargo test --all-features 2>&1 | grep "test result"`
- **Phase gate:** Full CI-exact gate green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-mcp/src/tools/generation_context.rs` — add `LiveProjectionGuidance` struct + `live_projection` field + `execute()` assembly + `live_projection_drift_guard` test

*(All other test infrastructure exists — only the new drift-guard test needs creating)*

## Security Domain

The phase adds documentation and MCP guidance text. No new network endpoints, no new authentication surfaces, no new data validation paths, no cryptographic operations.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no (docs only) | — |
| V6 Cryptography | no | — |

The `generation_context` guidance prose may describe `data-channel` values that are HTML-escaped by the server (verified: `containers.rs:1672` `html_escape(&props.projection)` and `html_escape(&props.key)`). The docs should note that channel values are server-controlled and HTML-escaped — not user-injectable — to prevent misunderstanding of the client runtime's `channelMap` lookup pattern (WR-01 in `live_fragment.rs:51`).

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/generation_context.rs` — section structure, drift-guard test pattern, existing `RegisterCompositionGuidance` shape to mirror
- `ferro-mcp/src/tools/json_ui_catalog.rs:405-483` — SC-1 test, `test_all_components_present` (verified passing locally)
- `ferro-json-ui/src/catalog.rs:1289-1303` — `builtin_types_count_drift_guard` (verified passing locally at 53), `BUILTIN_SPECS` LiveFragment entry at :382
- `ferro-json-ui/src/component.rs:753-763` — `LiveFragmentProps` struct definition (authoritative field names)
- `ferro-json-ui/src/render/containers.rs:1632-1678` — `render_live_fragment` (authoritative HTML output contract)
- `ferro-json-ui/src/runtime/live_fragment.rs` — `setupLiveFragments` client runtime (authoritative data-attribute and event contract)
- `ferro-macros/src/memoize.rs` — `#[memoize]` implementation details (D-04 guidance source)
- `ferro-macros/src/asset.rs` — `asset!()` expansion (D-04 guidance source)
- `framework/src/lib.rs:134,354,360` + `framework/src/bundle.rs` — framework re-exports (`ferro::memoize`, `ferro::asset!`, `ferro::bundle`)
- `.github/workflows/publish.yml:217` — `ferro-bundle` Wave 1a confirmed (D-13 pre-satisfied)
- `.github/workflows/ci.yml:62-74` — docs CI gate (`cargo doc --no-deps --all-features` + `RUSTDOCFLAGS=-Dwarnings`)
- `docs/src/json-ui/components.md` — per-component format anchor (props table + usage example pattern)
- `docs/src/json-ui/runtime-primitives.md` — runtime primitive section format (contract table + behavior note pattern)
- `docs/src/features/ferro-assets.md` — existing Asset Pipeline page where `asset!()` section lands
- `docs/src/SUMMARY.md` — TOC to update if a page is added
- `docs/book.toml` — mdBook config (`create-missing = false` → missing pages = error)
- `Cargo.toml:47` — workspace version `0.2.91` (D-11)
- `ferro-payments/Cargo.toml:3` — `0.1.6` (D-12)
- `.planning/phases/262-mcp-catalog-docs-publish/262-CONTEXT.md` — all locked decisions D-01 through D-15

### Secondary (MEDIUM confidence)
- `.planning/phases/259-request-scoped-memoization/259-CONTEXT.md` — `#[memoize]` semantics (D-01 through D-05)
- `.planning/phases/261-asset-ergonomics/261-CONTEXT.md` — `asset!()` D-01 through D-09, ferro-bundle decoupling (D-06)
- Project memory `feedback_ci_matrix_wider_than_local_gate.md`, `feedback_one_cpu_op_at_a_time.md`, `project_schema_export_test_dirties_tree.md` — gate gotchas

## Metadata

**Confidence breakdown:**
- SC-1 status (pre-satisfied): HIGH — two tests verified passing locally, source lines pinned
- `generation_context` structure: HIGH — full source read, pattern from existing drift guard
- LiveFragment contract: HIGH — source read at `containers.rs:1678` and `live_fragment.rs`
- `#[memoize]` contract: HIGH — source read at `ferro-macros/src/memoize.rs`
- `asset!()` contract: HIGH — source read at `ferro-macros/src/asset.rs`
- Publish choreography: HIGH — verified against `publish.yml` wave list and prior closeout memory
- mdBook build command: MEDIUM — inferred from `docs/book.toml`; `mdbook` binary availability not checked (A1)

**Research date:** 2026-07-26
**Valid until:** 2026-08-26 (no external dependencies that change; all sources are in-tree)
