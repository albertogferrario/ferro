# Phase 262: MCP + catalog + docs + publish - Pattern Map

**Mapped:** 2026-07-26
**Files analyzed:** 6 (1 new struct+test in existing file, 4 doc page extensions, 1 manifest)
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/generation_context.rs` | MCP tool (struct + test) | request-response | same file — `RegisterCompositionGuidance` (lines 99–130) + `register_composition_drift_guard` (lines 559–638) | exact |
| `docs/src/json-ui/components.md` | mdBook docs page | — | same file — `### Tile` section (lines 1413–1444) | exact |
| `docs/src/json-ui/runtime-primitives.md` | mdBook docs page | — | same file — `## data-lazy-hero` section (lines 8–62) | exact |
| `docs/src/features/ferro-assets.md` | mdBook docs page | — | same file — existing sections (e.g. `### HtmlMinify`, lines 80–92) | role-match (page extension) |
| `docs/src/features/projections.md` | mdBook docs page | — | same file — `## MCP CRUD Opt-In` section (lines 646–702), additive section pattern | role-match (page extension) |
| `Cargo.toml` | workspace manifest | — | same file — line 47 `version = "0.2.91"` | exact (single-field edit) |

---

## Pattern Assignments

### `ferro-mcp/src/tools/generation_context.rs` — new `LiveProjectionGuidance` struct + field + drift-guard test

This is the primary substantive change. Mirror the `RegisterCompositionGuidance` pattern exactly.

**Analog:** same file, `RegisterCompositionGuidance` struct (lines 99–130) and its `register_composition_drift_guard` test (lines 559–638).

---

#### Struct declaration pattern (lines 99–130):

```rust
/// Register composition guidance for POS-style sale screens (D-03). Everything derivable is derived;
/// prose is drift-guarded by `register_composition_drift_guard`.
#[derive(Debug, Serialize)]
pub struct RegisterCompositionGuidance {
    /// (a) When to use Register layout template vs. a form-only Collect spec. Also states that Numpad
    /// and a standalone FilterTabs are author-composable additions, not part of the v1 register
    /// template (D-06 / 257 D-07).
    pub when_to_use: &'static str,
    /// (b) Form-state selection contract: hidden-input qty accumulation (`data-qty-input`), ONE confirm
    /// POST, single Form common ancestor, TileGrid.form_id == SelectionPanel.form_id, SelectionPanel is
    /// a live client-side view of form state — never a second source of truth.
    pub form_state_contract: &'static str,
    /// (c) Runtime data attributes for filter + numpad + qty wiring (format: `"attr — role"`).
    pub data_attributes: &'static [&'static str],
    /// (d) fill_viewport requirement: required when a spec has TileGrid/SelectionPanel/Numpad; root Grid
    /// needs `fill: true`; supported shell layouts are "app" and "dashboard" ONLY.
    pub fill_viewport_requirement: &'static str,
    /// (e) The four register-composition lint rule ids (three `register-*` rules plus
    /// `fill-viewport-layout-unknown`) to check via design_lint, derived from design::rules().
    pub lint_rules: Vec<RegisterRuleRef>,
    /// (f) Pointer to register_template() (ferro-json-ui/src/projection/intent_layout.rs) — the
    /// one-call Collect->Register override; the projection-derived /cassa sample is the reference.
    pub template_helper: &'static str,
}
```

**New struct to add** — copy this shape, substitute fields for the three v17.0 capabilities:

```rust
/// Live projection surface guidance for v17.0 capabilities (D-03). Everything derivable
/// is derived; prose is drift-guarded by `live_projection_drift_guard`.
#[derive(Debug, Serialize)]
pub struct LiveProjectionGuidance {
    /// (a) LiveFragment — when to use, projection/key/template contract, channel format,
    /// first-paint behavior, one-binding-pattern limitation.
    pub live_fragment: &'static str,
    /// (b) Container contract — data-live-fragment marker + data-channel value format.
    pub container_contract: &'static str,
    /// (c) #[memoize] — when to annotate, request-scoped dedup, coalescing, error caching,
    /// graceful no-op outside request scope, complement to eager_loading/BatchLoad.
    pub memoize: &'static str,
    /// (d) asset!() — one-line embed, content-hashed URL, lazy register-once, &'static str
    /// return, ferro::bundle mount required, ferro assets fetch CLI.
    pub asset_macro: &'static str,
    /// Pointer to docs/src for depth (D-04 compact style rule).
    pub docs: &'static str,
}
```

**`GenerationContext` struct field to add** (line 17, after `register_composition`):

```rust
// existing line 17:
pub register_composition: RegisterCompositionGuidance,
// ADD:
/// Live projection surface guidance for v17.0 capabilities (D-03). Prose drift-guarded
/// by `live_projection_drift_guard`.
pub live_projection: LiveProjectionGuidance,
```

---

#### Static data pattern — `REGISTER_DATA_ATTRIBUTES` (lines 261–275):

```rust
/// Runtime data attributes for the register composition (filter, tile-qty, numpad, form-guard).
/// Drift-guarded by `register_composition_drift_guard` — each attribute must appear in FERRO_RUNTIME_JS.
static REGISTER_DATA_ATTRIBUTES: &[&str] = &[
    "data-filter-scope — scoping container for a filter group (TileGrid root)",
    "data-filter-tab=\"<token>\" — filter tab button; empty value = 'All' (FilterTabs)",
    // ... 11 more entries ...
];
```

For `LiveProjectionGuidance` there are only two data attributes (`data-live-fragment`, `data-channel`) — inline them directly in the prose field strings rather than extracting a static array, since they are few and the prose is short. Follow the same quoting style used in `form_state_contract`.

---

#### `execute()` assembly pattern (lines 336–381):

```rust
// ── Register composition guidance (D-03) ─────────────────────────────────
let register_rule_ids = [
    "register-fill-viewport",
    "register-grid-fill",
    "register-selection-present",
    "fill-viewport-layout-unknown",
];
let lint_rules: Vec<RegisterRuleRef> = design_rules()
    .iter()
    .filter(|r| register_rule_ids.contains(&r.id))
    .map(|r| RegisterRuleRef {
        id: r.id,
        title: r.title,
        rationale: r.rationale,
    })
    .collect();

let register_composition = RegisterCompositionGuidance {
    when_to_use: "Use the Register layout template when the screen has BOTH a browseable \
        items pane (TileGrid) and a running-selection pane (SelectionPanel) ...",
    form_state_contract: "A single Form element ...",
    data_attributes: REGISTER_DATA_ATTRIBUTES,
    fill_viewport_requirement: "fill_viewport: true at the Spec level is required ...",
    lint_rules,
    template_helper: "register_template() at ferro-json-ui/...",
};
```

**New assembly block for `execute()`** — add after the `register_composition` block, before the final `GenerationContext { ... }` constructor:

```rust
// ── Live projection surface guidance (D-03) ──────────────────────────────
let live_projection = LiveProjectionGuidance {
    live_fragment: "Use LiveFragment when a page element must reflect a ferro-projection \
        per-key snapshot in real time without a page reload. ...",
    container_contract: "The server wraps the rendered child template in \
        <div data-live-fragment data-channel=\"projection.{name}.{key}\">...</div>. \
        The client runtime subscribes over /_ferro/ws and swaps innerHTML on each \
        `fragment` event. Channel values are HTML-escaped server-side. ...",
    memoize: "Annotate an async fn with #[memoize] (use ferro::memoize) when it is \
        called N times per request by N intents over the same key. ...",
    asset_macro: "asset!(\"path/to/file\") embeds the file at call-site-source-relative \
        path using include_bytes!, registers it once via OnceLock, and returns a \
        content-hashed &'static str URL (e.g. \"/bundles/file.a1b2c3d4.js\"). ...",
    docs: "See docs/src/json-ui/components.md#livefragment, \
        docs/src/json-ui/runtime-primitives.md, docs/src/features/ferro-assets.md, \
        and docs/src/features/projections.md#memoize for usage examples.",
};
```

Then add `live_projection` to the `GenerationContext { ... }` constructor at line 383.

---

#### `test_generation_context_has_all_sections` update (lines 506–556):

```rust
// Verify register composition guidance populated (D-03)
assert!(
    !context.register_composition.when_to_use.is_empty(),
    "register_composition.when_to_use must be non-empty"
);
assert_eq!(
    context.register_composition.lint_rules.len(),
    4,
    "must derive all four register-composition rules from design::rules()"
);
```

**Pattern to add inside the same test** — mirror the style:

```rust
// Verify live projection guidance populated (D-03)
assert!(
    !context.live_projection.live_fragment.is_empty(),
    "live_projection.live_fragment must be non-empty"
);
assert!(
    !context.live_projection.container_contract.is_empty(),
    "live_projection.container_contract must be non-empty"
);
assert!(
    !context.live_projection.memoize.is_empty(),
    "live_projection.memoize must be non-empty"
);
assert!(
    !context.live_projection.asset_macro.is_empty(),
    "live_projection.asset_macro must be non-empty"
);
```

---

#### `register_composition_drift_guard` test — mirror as `live_projection_drift_guard` (lines 559–638):

Full test to copy and adapt (the entire block, condensed here to the structure):

```rust
#[test]
fn register_composition_drift_guard() {
    use std::collections::HashSet;
    let ctx = execute();

    // 1. Component names mentioned in the guidance exist as builtins, AND the
    // guidance prose actually mentions them.
    let builtins: HashSet<String> = ferro_json_ui::global_catalog()
        .components_sorted()
        .map(|c| c.name.clone())
        .collect();
    for name in [
        "TileGrid",
        "SelectionPanel",
        "FilterTabs",
        "QuantityStepper",
        "Numpad",
        "Tile",
    ] {
        assert!(
            builtins.contains(name as &str),
            "register guidance names non-builtin `{name}`"
        );
    }
    let prose = format!(
        "{} {} {}",
        ctx.register_composition.when_to_use,
        ctx.register_composition.form_state_contract,
        ctx.register_composition.fill_viewport_requirement
    );
    for name in ["TileGrid", "SelectionPanel", "FilterTabs", "Numpad", "Tile"] {
        assert!(
            prose.contains(name),
            "register guidance prose no longer mentions `{name}`"
        );
    }

    // 2. Every id the guidance hardcodes exists in the rule registry.
    let rule_ids: HashSet<&str> = ferro_json_ui::design::rules()
        .iter()
        .map(|r| r.id)
        .collect();
    let derived: HashSet<&str> = ctx
        .register_composition
        .lint_rules
        .iter()
        .map(|r| r.id)
        .collect();
    for id in [
        "register-fill-viewport",
        "register-grid-fill",
        "register-selection-present",
        "fill-viewport-layout-unknown",
    ] {
        assert!(rule_ids.contains(id), "registry lost rule `{id}`");
        assert!(derived.contains(id), "guidance failed to derive rule `{id}`");
    }

    // 3. EVERY published attribute appears in the assembled runtime bundle.
    for entry in ctx.register_composition.data_attributes {
        let name = entry
            .split([' ', '='])
            .next()
            .expect("attribute entry is non-empty");
        assert!(
            ferro_json_ui::FERRO_RUNTIME_JS.contains(name),
            "runtime bundle missing `{name}` — register guidance is stale"
        );
    }
}
```

**New test to add** — mirror the same three-assertion structure:

```rust
#[test]
fn live_projection_drift_guard() {
    use std::collections::HashSet;
    let ctx = execute();

    // 1. LiveFragment is a builtin and the guidance prose mentions it.
    let builtins: HashSet<String> = ferro_json_ui::global_catalog()
        .components_sorted()
        .map(|c| c.name.clone())
        .collect();
    assert!(
        builtins.contains("LiveFragment"),
        "live_projection guidance names non-builtin `LiveFragment`"
    );
    let prose = format!(
        "{} {}",
        ctx.live_projection.live_fragment,
        ctx.live_projection.container_contract
    );
    assert!(
        prose.contains("LiveFragment"),
        "live_projection prose no longer mentions `LiveFragment`"
    );

    // 2. Data attributes mentioned in the guidance appear in the assembled runtime bundle.
    for attr in ["data-live-fragment", "data-channel"] {
        assert!(
            ferro_json_ui::FERRO_RUNTIME_JS.contains(attr),
            "runtime bundle missing `{attr}` — live_projection guidance is stale"
        );
        assert!(
            prose.contains(attr),
            "live_projection prose no longer mentions `{attr}`"
        );
    }

    // 3. Macro names mentioned in the guidance exist as framework re-exports.
    // Verified structurally: the prose must name each macro.
    let full_prose = format!(
        "{} {} {}",
        ctx.live_projection.memoize,
        ctx.live_projection.asset_macro,
        ctx.live_projection.docs
    );
    for name in ["memoize", "asset!"] {
        assert!(
            full_prose.contains(name),
            "live_projection prose no longer mentions `{name}`"
        );
    }
}
```

**Key difference from `register_composition_drift_guard`:** no `lint_rules` (LiveFragment has no design-lint rules); instead assertion 2 checks runtime JS attributes directly, and assertion 3 checks macro names in prose. Structure is otherwise identical.

---

### `docs/src/json-ui/components.md` — add `LiveFragment` component section

**Analog:** `### Tile` section (lines 1413–1444) — closest because Tile is also a single-purpose builtin with a props table, behavioral notes, and one usage example. Also reference `### StatCard` (lines 673–696) for the `sse_target` live-behavior note pattern.

**Section format to copy:**

```markdown
### ComponentName

One-sentence description framing the use case.

| Prop | Type | Description |
|------|------|-------------|
| `prop_a` | `type` | Description |
| `prop_b` | `type \| null` | Description |

```json
"element_id": {
  "type": "ComponentName",
  "props": {
    "prop_a": "value",
    "prop_b": "value"
  }
}
```

Optional behavioral notes paragraph.
```

**LiveFragment section placement:** The component overview table (lines 23–37) lists components by category. `LiveFragment` is a new category or fits under a `Live / Real-time` category. Add it to the overview table and add the section after `### StreamText` (which already handles SSE) or before `---` at the end of Extensible Components.

**LiveFragment props table** (from `LiveFragmentProps` in `ferro-json-ui/src/component.rs:753–763` per RESEARCH.md):

| Prop | Type | Description |
|------|------|-------------|
| `projection` | `string` | ferro-projection NAME (`Projection::NAME` const) |
| `key` | `string` | Per-key channel selector; combined with `projection` to form the subscription channel |
| `template` | `object` | Child JSON-UI spec rendered against the snapshot as its data scope |

**Behavioral note to include:** When no snapshot exists for the key at first paint, the container is rendered empty (the child template receives `{}` as data). On each server delta the client runtime swaps `innerHTML` of the container without a page reload.

**Usage example** (from RESEARCH.md Pattern 3, framed as a sample):

```json
"live_stock": {
  "type": "LiveFragment",
  "props": {
    "projection": "inventory",
    "key": "warehouse-a",
    "template": {
      "$schema": "ferro-json-ui/v2",
      "root": "count",
      "elements": {
        "count": { "type": "Text", "props": { "content": { "$data": "/count" } } }
      }
    }
  }
}
```

---

### `docs/src/json-ui/runtime-primitives.md` — add `LiveFragment` client-subscription behavior section

**Analog:** `## data-lazy-hero` (lines 8–62) — exact format match: intro paragraph → Contract table → Selector/Behavior description → Usage HTML → notes.

**Section format to copy:**

```markdown
## `data-lazy-hero`

One-paragraph description of the primitive's purpose and behavior.

### Contract

| Attribute | Required | Default | Description |
|-----------|----------|---------|-------------|
| `data-lazy-hero` | yes | — | Opt-in marker. ... |
| `data-lazy-hero-margin` | no | `200px 0px` | Per-element rootMargin. |
| `data-lazy-hero-promoted` | no (runtime sets it) | absent | Idempotency marker. |

### Selector

Explanation of the CSS selector and its consequences.

### Usage

```html
<!-- code example -->
```

### Browser support / Lifecycle / Notes

...
```

**LiveFragment section to add:**

```markdown
## `data-live-fragment` / `data-channel`

Emitted by the `LiveFragment` builtin on its container `<div>`. The runtime
opens one shared WebSocket to `/_ferro/ws` per page, subscribes to each
declared channel, and swaps the container's `innerHTML` when a `fragment`
event arrives for the matching channel.

### Contract

| Attribute | Set by | Description |
|-----------|--------|-------------|
| `data-live-fragment` | `LiveFragment` renderer | Opt-in marker; selects the container for WebSocket subscription |
| `data-channel` | `LiveFragment` renderer | Subscription key — `"projection.{name}.{key}"` where `name` and `key` are HTML-escaped |

### Channel format

`projection.{projection_name}.{projection_key}` — matches the channel the server publishes on via `ferro-projection`. Both segments are HTML-escaped by the server; channel values are server-controlled and not user-injectable.

### Subscribe + swap

The runtime:

1. Collects all `[data-live-fragment]` containers on `DOMContentLoaded` and builds a `channelMap` keyed by `data-channel`.
2. Opens one shared WebSocket to `/_ferro/ws`.
3. On `open`, sends `{ "type": "subscribe", "channel": "..." }` for each channel.
4. On `message`, matches `{ "type": "event", "event": "fragment", "channel": "...", "data": { "html": "..." } }` and sets `target.innerHTML = msg.data.html`.

No WASM, no client-side reactive state, no `eval`.

### Limitations

- One `LiveFragment` element per unique channel per page (first container wins for duplicate channels).
- No automatic reconnect on WebSocket error (deferred to a future release).
- No list/collection reconciliation — the entire container HTML is replaced on each delta.

Elements inserted after `DOMContentLoaded` are not observed. The `LiveFragment` builtin always renders its container in the initial server HTML.
```

---

### `docs/src/features/ferro-assets.md` — add `asset!()` and `ferro assets fetch` sections

**Analog:** `### HtmlMinify` (lines 80–92) for the section heading + one-paragraph description + code example format. Also reference the `## Quick Start` section (lines 16–38) for multi-line prose + `rust,ignore` code block format.

**Section format to copy:**

```markdown
### FeatureName

One paragraph describing the feature.

```rust,ignore
// usage example
```

Optional notes paragraph.
```

**Placement:** Add two new sections near the end of the file (after `## Error Reference`), or add a top-level `## Compile-time Asset Embedding` section before the pipeline content. The `ferro-assets` crate page documents the pipeline, but `asset!()` is a framework-level macro (`ferro::asset!`), not a pipeline transform — add a top-level section clearly separate from `Pipeline` usage, with a note explaining the distinction.

**`asset!()` section content (from RESEARCH.md Code Examples):**

```markdown
## Compile-time Asset Embedding

`asset!("path")` is a proc-macro that embeds a static file at compile time and
registers it in the content-hashed bundle registry at first use.

```rust,ignore
use ferro::asset;

// Returns a content-hashed URL as &'static str, e.g. "/bundles/app.a1b2c3d4.js"
let url: &'static str = asset!("assets/app.js");
```

The path is resolved relative to the call site's source file (using `include_bytes!`
semantics). The bytes are registered once via `OnceLock` on the first call and never
re-read. The content hash is derived from the bytes at compile time.

**Requirements:**

- The app must mount `ferro::bundle` serving (e.g. `ferro::bundle::serve`) so that
  the hashed URL resolves to a response. Without this, the URL is emitted but requests
  to it return 404.
- The MIME type is inferred from the file extension (`.js` → `text/javascript`,
  `.css` → `text/css`, etc.).

**Return type:** `&'static str` — the content-hashed URL string, valid for the
lifetime of the process.

## `ferro assets fetch`

Downloads third-party assets (icon sets, font files) to a local path at author time,
so they can be embedded with `asset!()` without depending on a CDN at runtime.

```
ferro assets fetch iconify
ferro assets fetch fontsource
```

`ferro assets fetch iconify` downloads the Iconify offline bundle.
`ferro assets fetch fontsource` downloads font files from the Fontsource CDN.

The fetched files are written to a local directory and are NOT auto-wired into
`asset!()` calls or route generation. After fetching, reference the downloaded
paths manually in `asset!()` calls.
```

---

### `docs/src/features/projections.md` — add `#[memoize]` section

**Analog:** `## MCP CRUD Opt-In` (lines 646–702) — additive H2 section at the end of the file, with a brief prose introduction, a code block showing the usage, a prerequisites/behavior table, and notes on what it complements. Also reference the `## Conversational-Text Rendering` section (lines 312–410) for the "separate concern, separate section" section shape.

**Placement:** Add as a new H2 section `## Request-Scoped Render Deduplication` at the end of the file (after `## MCP CRUD Opt-In`), or as a subsection of `## Rendering`. The section is short (the feature is simple) — keep it to one code block and one prose paragraph plus a notes block.

**`#[memoize]` section content (from RESEARCH.md Code Examples):**

```markdown
## Request-Scoped Render Deduplication

When multiple intents or projection renders call the same data-fetch function
with the same arguments in one request, `#[memoize]` coalesces them into a
single in-flight `await` and returns the cached result to every caller.

```rust,ignore
use ferro::memoize;

#[memoize]
async fn fetch_stock(warehouse_id: String) -> Result<StockLevel, AppError> {
    // body executes at most once per (call site, warehouse_id) per request
    db_query(warehouse_id).await
}
```

**Semantics:**

- **Scope:** request-scoped task-local store (`MEMO_STORE`). The cache is dropped
  with the request; there is no cross-request sharing.
- **Coalescing:** concurrent callers waiting on the same key are all resumed when
  the first in-flight future resolves.
- **Error caching:** a transient error is returned to all coalesced callers.
- **Outside request scope:** calling a memoized fn outside a request context (e.g.
  in a background job or a test) is a graceful no-op — the fn runs un-memoized.

**Relationship to `eager_loading` / `BatchLoad`:**

`#[memoize]` and `eager_loading`/`BatchLoad` address different levels of the same
problem. `eager_loading` batches N rows into one query up front. `#[memoize]`
deduplicates N calls to the same fn during a render pass. They complement each other
and can be used together.

`#[memoize]` is not a cross-request cache. For cross-request caching, use `ferro-cache`.
```

---

### `Cargo.toml` — workspace version bump (world-state-dependent)

**Analog:** same file, line 47:

```toml
[workspace.package]
version = "0.2.91"
```

**Action at gate time (D-11):** Read crates.io `ferro-rs` current version via `curl -s https://crates.io/api/v1/crates/ferro-rs`.

- If crates.io < 0.2.91: publish 0.2.91 as-is — no Cargo.toml change needed.
- If crates.io >= 0.2.91: change `version = "0.2.91"` to `version = "0.2.{N+1}"` where N = crates.io max.

Single-field edit. No other Cargo.toml changes (D-13: no new crates, no publish.yml wave changes needed — ferro-bundle Wave 1a was verified pre-satisfied per RESEARCH.md §Open Questions D-13).

---

## Shared Patterns

### Doc comment style (applies to all new `pub` items in `generation_context.rs`)

**Source:** `ferro-mcp/src/tools/generation_context.rs` lines 63–74 (`DesignSystemSummary`) and lines 99–122 (`RegisterCompositionGuidance`)

```rust
/// Design system summary for agent-authoring context (D-06).
#[derive(Debug, Serialize)]
pub struct DesignSystemSummary {
    /// Semantic token vocabulary (30 slots). Each entry: CSS variable name + one-line purpose.
    pub tokens: &'static [TokenInfo],
    // ...
}
```

Every `pub struct` and every `pub` field must have a `///` doc comment — required by `RUSTDOCFLAGS=-Dwarnings` in the CI docs gate (RESEARCH.md Pitfall 3). One-sentence doc is sufficient.

---

### Compact guidance style (D-04)

**Source:** `generation_context.rs` lines 354–381 (the `RegisterCompositionGuidance` field values in `execute()`)

Each prose field is one or two sentences. The last sentence of each field points to `docs/src/` for depth. Example:

```rust
when_to_use: "Use the Register layout template when the screen has BOTH a browseable \
    items pane (TileGrid) and a running-selection pane (SelectionPanel) ...",
```

Apply the same wrapping style (backslash line continuation, 4-space indent on continuation) to the new `LiveProjectionGuidance` field strings.

---

### Registry-derive pattern (D-05)

**Source:** `register_composition_drift_guard` lines 563–573

```rust
let builtins: HashSet<String> = ferro_json_ui::global_catalog()
    .components_sorted()
    .map(|c| c.name.clone())
    .collect();
```

For the `live_projection_drift_guard`, derive the builtin name check from `global_catalog()` — never hardcode a name list that bypasses the registry. Use `ferro_json_ui::FERRO_RUNTIME_JS.contains(attr)` for attribute verification (same pattern as line 633–636).

---

### `execute()` section comment style

**Source:** `generation_context.rs` lines 279, 336

```rust
// ── Design system summary (D-06) ─────────────────────────────────────────
// ── Register composition guidance (D-03) ─────────────────────────────────
```

Add a matching section header for the new block:

```rust
// ── Live projection surface guidance (D-03) ──────────────────────────────
```

---

### mdBook section heading levels

**Source:** `docs/src/json-ui/components.md` uses `###` for component sections inside `##` category headings; `docs/src/json-ui/runtime-primitives.md` uses `##` for each top-level primitive with `###` for sub-sections; `docs/src/features/projections.md` uses `##` for top-level feature sections.

- `components.md` → `### LiveFragment` (inside `## Extensible Components` or a new `## Live / Real-time` category)
- `runtime-primitives.md` → `## data-live-fragment / data-channel` (parallel to `## data-lazy-hero`)
- `projections.md` → `## Request-Scoped Render Deduplication` (parallel to `## MCP CRUD Opt-In`)
- `ferro-assets.md` → `## Compile-time Asset Embedding` + `## ferro assets fetch` (parallel to `## Quick Start`)

---

## No Analog Found

None — all files are modifications of existing files, and each has a direct analog in the same file or its immediate sibling.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `docs/src/json-ui/`, `docs/src/features/`, root `Cargo.toml`
**Files scanned:** 7 source files read directly
**Authoritative contract sources verified:**
- `ferro-json-ui/src/render/containers.rs:1678` — `data-live-fragment` + `data-channel` HTML output
- `ferro-json-ui/src/runtime/live_fragment.rs` — client JS subscribe+swap contract
- RESEARCH.md §Code Examples — `asset!()` expansion, `#[memoize]` usage, framework re-exports
**Pattern extraction date:** 2026-07-26
