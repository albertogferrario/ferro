# Phase 148: ImageProps inline-SVG source — extend Image, don't add HtmlEmbed — Context

**Gathered:** 2026-04-24
**Reworked:** 2026-04-24 (scope changed from `Component::HtmlEmbed` → extend `Component::Image`; see DISCUSSION-LOG.md)
**Status:** Ready for re-planning
**Mode:** `--auto` (single-pass, recommended defaults selected for all gray areas)

<domain>
## Phase Boundary

Extend `ImageProps` so it can carry **either** an external URL (current `src` field)
**or** a server-constructed inline SVG string. The renderer picks the right output
shape based on which source variant is set. This unblocks gestiscilo.it's v6.1
Statistiche revenue-trend bar chart (Phase 117) — a server-generated SVG that the
JSON-UI view tree today has no clean way to carry.

No new `Component` variant. No new resolver arm. No new MCP exhaustive-list entry.
The `Image` conceptual slot ("a bounded visual asset rendered into a box") naturally
covers both cases. `alt: String` stays required — every inline-SVG caller is
compile-forced to write accessibility text (a chart reader's win).

**Primary files touched:**
- `ferro-json-ui/src/component.rs` — introduce `ImageSource` enum; refactor `ImageProps`
  to carry `ImageSource` in a `#[serde(flatten)]` field (preserving the wire format
  `{"src": "..."}` for the URL variant and gaining `{"svg": "..."}` for the inline
  variant); update the existing `ComponentNode::image` factory call-sites and the
  `image_round_trips` test to use the new source shape
- `ferro-json-ui/src/render.rs` — `render_image` gains one branch: when
  `source = InlineSvg { svg }`, emit `<div role="img" aria-label="{escaped alt}">{svg verbatim}</div>`
  wrapped in the existing aspect-ratio container; the URL branch keeps the existing
  `<img src alt>` shape with `html_escape` on both attributes (existing
  `image_xss_src_escaped` test stays green)
- `ferro-json-ui/src/lib.rs` — add `### Image` section to `COMPONENT_CATALOG`
  (currently missing — pre-existing gap we close here) documenting both source
  variants with a one-line safety note for the SVG variant
- `ferro-mcp/src/tools/json_ui_catalog.rs` — update the existing `Image` catalog
  entry to describe both source variants (no new `CatalogComponent`, no
  exhaustive-list bump)
- `docs/src/json-ui/components.md` — add `### Image` section (currently missing —
  pre-existing gap) with props table covering both source variants, Rust + JSON
  examples for each, and a safety callout on the inline-SVG branch

**Out of scope:**
- Generic `Component::HtmlEmbed` / raw-HTML escape hatch (rejected; see rationale
  in `<decisions>` D-00)
- Markdown rendering (future: a dedicated `MarkdownEmbed` if a real call-site
  appears — not this phase)
- `class`, `id`, or `style` props on the SVG wrapper `<div>` beyond what
  `aspect_ratio` + `placeholder_label` already provide
- Data-binding for the SVG string (`data_path`) — the SVG is always constructed
  by Rust code at render-plan time; no runtime lookup
- Configurable wrapper element (`<span>`, `<figure>`, …) — `<div role="img">`
  is the single output shape; if a real semantic need appears, extend later
- Client-side sanitization — callers construct SVG themselves (typically from
  typed data via a `bar_chart_svg(...) -> String` helper); if sanitization is
  needed, callers run it before setting `InlineSvg { svg }`
- Fixing the pre-existing Avatar props-table drift in `docs/src/json-ui/components.md`
  (line 629 claims `src: Option<String>` — out of scope here, handle in a later
  docs-cleanup phase)

</domain>

<decisions>
## Implementation Decisions

### Core shape

- **D-00 (scope rejection):** We explicitly reject the earlier `Component::HtmlEmbed`
  shape (see `archive-htmlembed/` and DISCUSSION-LOG.md). The named harms:
  (a) a `HtmlEmbed` name invites use for escaping user input ("I have some HTML,
  I'll just embed it"); (b) it weakens the projection/intent story by adding a
  pixel-level escape hatch disjoint from any structural concept; (c) its
  `<deferred>` list (class/id/wrapper_tag/data_path/sanitize/iframe/MarkdownEmbed)
  is the roadmap of a swiss army knife; (d) it duplicates the "bounded visual
  asset" concept that `Image` already owns. Extending `Image` is the right call.

- **D-01:** Introduce `ImageSource` as a serde-untagged enum with exactly two
  variants, both struct variants (so the JSON discriminator is the field name):

  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
  #[serde(untagged)]
  pub enum ImageSource {
      Url { src: String },
      InlineSvg { svg: String },
  }
  ```

  Rationale: structurally guarantees exactly-one-of-(src, svg). No nullable
  `src: Option<String> + svg: Option<String>` smell. The untagged shape keeps
  the wire format readable and — critically — keeps the URL case fully
  backward-compatible with existing callers (`{"src":"…","alt":"…"}` still
  deserializes).

- **D-02:** Refactor `ImageProps` to flatten the source:

  ```rust
  pub struct ImageProps {
      #[serde(flatten)]
      pub source: ImageSource,
      pub alt: String,
      #[serde(default, skip_serializing_if = "Option::is_none")]
      pub aspect_ratio: Option<String>,
      #[serde(default, skip_serializing_if = "Option::is_none")]
      pub placeholder_label: Option<String>,
  }
  ```

  `alt` stays required — compile-enforced a11y for both source variants. The
  flatten attribute preserves the JSON shape: no wrapping `"source"` key in the
  wire format, discriminator is the presence of `src` vs `svg` at the top level.

- **D-03:** Add `ImageProps::url(src: impl Into<String>, alt: impl Into<String>) -> Self`
  and `ImageProps::inline_svg(svg: impl Into<String>, alt: impl Into<String>) -> Self`
  convenience constructors. Keeps call-sites compact:
  `ImageProps::url("/logo.png", "Logo")` and
  `ImageProps::inline_svg(bar_chart_svg(&data, 800, 300), "Incassi settimanali")`.
  Both default `aspect_ratio = None` and `placeholder_label = None`.

- **D-04:** Existing call-sites (there are some — grep `ImageProps {` across the
  workspace) get rewritten to either use `ImageProps::url(...)` or spell the
  struct literal with `source: ImageSource::Url { src: ... }`. No deprecation
  shim, no keep-the-old-field compat. This is pre-1.0; a direct refactor is
  correct per `CLAUDE.md` ("This is always a feature branch").

### Rendering

- **D-05:** `render_image` takes `&ImageProps` (unchanged signature) and branches
  on `props.source`:
  - `ImageSource::Url { src }` → existing code path: `<img src="{escaped}" alt="{escaped}" …>`
    inside the aspect-ratio container and placeholder. Existing
    `image_xss_src_escaped` test stays as written.
  - `ImageSource::InlineSvg { svg }` → `<div role="img" aria-label="{escaped alt}">{svg verbatim}</div>`
    inside the same aspect-ratio container. The placeholder is NOT rendered for
    the SVG variant (the SVG itself is the content; if callers want a
    no-data placeholder, they construct an empty-state view upstream — matches
    Phase 117's EmptyState-when-no-data pattern in gestiscilo).

- **D-06:** `render_image` carries an inline comment on the `InlineSvg` branch
  flagging the deliberate `html_escape` omission on the svg string. This is the
  **one** place in `ferro-json-ui` that emits a caller-supplied string unescaped;
  the comment documents that asymmetry so future audits don't "fix" it. The
  `alt` text IS escaped in both branches — a11y text is still attacker-controllable
  in principle (e.g. pass-through from form data), so escape discipline applies.

- **D-07:** No change to `render_component` dispatch — `Component::Image(props) =>
  render_image(props)` is unchanged.

- **D-08:** No change to `collect_plugin_types_node` — Image is already in the
  leaf group; that list stays at 41 components, no exhaustive-list bump anywhere.

- **D-09:** No new resolver arms. All three resolver passes already handle
  `Component::Image(_)` in their leaf OR-chains.

### Serde

- **D-10:** The untagged enum means serde round-trip handles both wire shapes
  automatically:
  - `{"type":"Image","src":"/logo.png","alt":"Logo"}` → `ImageSource::Url { src: "/logo.png" }`
  - `{"type":"Image","svg":"<svg>…</svg>","alt":"Chart"}` → `ImageSource::InlineSvg { svg: "<svg>…</svg>" }`

  Ambiguous input (both `src` and `svg` set, or neither) is rejected by serde's
  untagged-enum discriminator; one test MUST assert this failure path so a
  future refactor can't silently weaken it.

- **D-11:** The existing `Component::Serialize` / `Component::Deserialize` arms
  for `"Image"` stay as-is; serde handles the enum discrimination for us via
  `#[serde(flatten)]` + `#[serde(untagged)]` on `ImageSource`.

### Safety framing — load-bearing

- **D-12:** Rustdoc on `ImageSource::InlineSvg` (and on `ImageProps::inline_svg`
  constructor) must lead with a scoped safety note. Exact text TBD at write
  time; required substance:
  - "SVG is emitted verbatim without escaping."
  - "Intended for server-constructed SVG (charts, icons), not user input."
  - "Callers that incorporate user-supplied data into SVG output are responsible
    for sanitization."
  - Contrast the `Url` variant ("src is URL-escaped as an attribute value").

- **D-13:** `COMPONENT_CATALOG` entry for `### Image` (new — currently absent)
  documents both source variants with a one-line safety note on the SVG branch.
  Mirror the density of the existing `### Separator` / `### Avatar` entries.

- **D-14:** MCP `CatalogComponent` entry for `Image` (currently exists, needs
  updating) surfaces both variants and the safety note. The `props` list grows
  to reflect the two source fields, but the catalog count stays at 41.

### Docs

- **D-15:** Add `### Image` section to `docs/src/json-ui/components.md`
  (currently missing). Section includes:
  - Opening paragraph: "Renders a bounded visual asset — either an external
    image via URL, or a server-constructed inline SVG."
  - Props table covering the flattened shape: `src` OR `svg` exactly-one-of,
    `alt` (required), `aspect_ratio` (optional), `placeholder_label` (optional).
  - **Safety callout** styled as a blockquote, scoped to the `svg` variant:
    "The `svg` value is emitted verbatim. Intended for server-constructed SVG
    (charts, icons). Not suitable for user input."
  - Rust examples for both constructors (`ImageProps::url`, `ImageProps::inline_svg`).
  - JSON output examples for both variants.
  - Use-case list for inline SVG: revenue trend charts, sparklines, diagrams,
    decorative vector assets.
  - Explicit pointer: "For rendering HTML (not SVG), no generic escape hatch
    exists; author a narrower component."

### TDD wave structure

- **D-16:** Plans decompose (proposed — planner may consolidate):
  - **Wave 0 RED tests** (no deps):
    - `component.rs` tests: serde round-trip for both variants; ambiguous-input
      rejection (both `src` + `svg`, and neither); `ImageProps::url` +
      `::inline_svg` constructors exist; round-trip fixture
      (`all_known_types_round_trip`) updated to cover both variants.
    - `render.rs` tests: URL variant renders escaped `<img src>` (existing test
      preserved); InlineSvg variant emits `<div role="img" aria-label="…">…SVG
      verbatim…</div>`; inline SVG with `<script>` tag passes through UNESCAPED
      (load-bearing: proves the deliberate bypass is working and documents
      intent via a test); `alt` text IS escaped on both variants.
  - **Wave 1 impl** (depends on Wave 0):
    - `component.rs`: `ImageSource` enum + `ImageProps` refactor with
      `#[serde(flatten)]` + constructors + rustdoc safety note.
    - `render.rs`: `render_image` gains the `InlineSvg` branch with inline
      safety-contract comment.
    - Fix any in-tree call-sites of the old `ImageProps { src, alt, ... }`
      struct literal.
  - **Wave 2 surface updates** (depends on Wave 1):
    - `ferro-json-ui/src/lib.rs`: add `### Image` section to `COMPONENT_CATALOG`.
    - `ferro-mcp/src/tools/json_ui_catalog.rs`: update the `Image`
      `CatalogComponent` entry to describe both variants.
    - `docs/src/json-ui/components.md`: add `### Image` section with safety
      callout.
    - CI gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets
      -- -D warnings && cargo test --all-features`.

- **D-17:** Expect 3–4 plans total (not 5). Wave 2 is small enough that the
  planner may consolidate catalog + docs into a single plan.

### Claude's Discretion

- Exact rustdoc wording for `ImageSource::InlineSvg` and the two constructors —
  must satisfy D-12's content requirements but prose style follows existing
  `ImageProps` / `AvatarProps` doc patterns.
- Exact description text in the MCP catalog and `COMPONENT_CATALOG` — must
  satisfy D-13 / D-14 (both variants described, SVG branch safety-flagged).
- Whether to add a `ComponentNode::image_svg(key, svg, alt)` factory as a
  sibling to the existing `ComponentNode::image`. Recommended: yes, it's trivial
  and keeps the call-site compact at the `ComponentNode` construction point.
- Whether the ambiguous-input rejection test is a serde round-trip failure
  assertion or a panic assertion — recommended: serde `from_value` returning
  `Err` is the cleaner shape; use `.expect_err("ambiguous source")`.
- Whether the `alt`-escape test for the InlineSvg branch uses a literal
  injection string (`" onload="alert(1)`) or a more neutral value. Recommended:
  use the injection string (same pattern as `image_xss_src_escaped`) for
  symmetry.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Current Image surface (what we extend)
- `ferro-json-ui/src/component.rs:601-614` — `ImageProps` struct as it exists
  today (`src: String`, `alt: String`, `aspect_ratio: Option<String>`,
  `placeholder_label: Option<String>`)
- `ferro-json-ui/src/component.rs:1094` — `Component::Image(ImageProps)` variant
  (unchanged by this phase)
- `ferro-json-ui/src/component.rs:1162` — `Component::Serialize` arm for Image
  (unchanged — serde handles the flatten/untagged discrimination)
- `ferro-json-ui/src/component.rs:1295-1298` — `Component::Deserialize` arm
  for `"Image"` (unchanged)
- `ferro-json-ui/src/component.rs:1719-1722` — `ComponentNode::image` factory
  (needs one-line refactor if we add `image_svg` sibling; otherwise unchanged)
- `ferro-json-ui/src/component.rs:3696-3715` — `image_round_trips` test
  (extended — add one assertion per variant)
- `ferro-json-ui/src/component.rs:2173-` — `all_known_types_round_trip` fixture
  (extend the Image entry to cover both variants, OR add a sibling `InlineSvg`
  fixture entry)

### Current render.rs surface (what we extend)
- `ferro-json-ui/src/render.rs:2420-2447` — `render_image` as it stands today
  (URL path, with placeholder, aspect-ratio container, `html_escape` on src/alt)
- `ferro-json-ui/src/render.rs:354` — dispatch arm `Component::Image(props) =>
  render_image(props)` (unchanged)
- `ferro-json-ui/src/render.rs:193` — `collect_plugin_types_node` leaf list
  (Image already present, unchanged)
- `ferro-json-ui/src/render.rs:274` — aspect_ratio container style branch
  (unchanged — applies to both variants)
- `ferro-json-ui/src/render.rs:3755-3809` — existing Image render tests
  (`image_with_aspect_ratio`, `image_without_aspect_ratio_omits_style`,
  `image_xss_src_escaped`) — all stay green; we add parallel tests for the
  InlineSvg branch

### Resolver (no changes expected)
- `ferro-json-ui/src/resolve.rs:30-162` — `resolve_component_node` leaf chain
  (Image already covered)
- `ferro-json-ui/src/resolve.rs:211-339` — `collect_unresolved_node` leaf chain
  (Image already covered)
- `ferro-json-ui/src/resolve.rs:389-493` — `resolve_errors_node` leaf chain
  (Image already covered)

### MCP catalog (one entry to update)
- `ferro-mcp/src/tools/json_ui_catalog.rs` — find the existing `CatalogComponent
  { name: "Image", ... }` entry (grep `"Image"` in this file) and widen its
  `props` list + description to cover both source variants; catalog total stays
  41, exhaustive-list assertion unchanged

### COMPONENT_CATALOG runtime string (one section to add)
- `ferro-json-ui/src/lib.rs:103+` — `pub const COMPONENT_CATALOG: &str`. Image
  is currently **absent** from this catalog (a pre-existing gap we close). Add
  an `### Image` section near `### Separator` / `### Avatar` density; include
  both source shapes and the SVG safety note.

### Documentation
- `docs/src/json-ui/components.md:623-654` — `### Avatar` section: density to
  mirror for the new `### Image` section (but Image needs the dual-source
  treatment + safety callout).
- `docs/src/json-ui/components.md:795-843` — `### StatCard` section: shape of
  the `icon` prop documentation (StatCard already accepts raw SVG in `icon` —
  useful precedent for "SVG-injection is not new to this crate").

### Existing SVG-injection precedents in the crate
- `ferro-json-ui/src/render.rs:2475-2479` — `BREADCRUMB_SEP` const (an inline
  SVG literal inlined directly into rendered markup — precedent for
  Rust-constructed SVG being fine).
- `StatCardProps.icon: Option<String>` and `ActionCardProps.icon: Option<String>`
  — existing props that accept raw SVG strings unescaped. Phase 148 generalizes
  the pattern from "a prop on two specific components" to "a first-class source
  variant on the generic Image component."

### Real call-site that drove this phase
- `../../gestiscilo-it/app/.planning/research/v6.1-ANALYTICS-ARCHITECTURE.md`
  — the revenue-trend bar chart use case that pulled this capability into
  existence. That doc currently references `Component::HtmlEmbed`; it needs a
  follow-up update to reference `ImageProps::inline_svg(svg, alt)` once this
  phase ships. (Tracked as a cross-repo TODO; not in this phase's scope.)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/ROADMAP.md` Phases
  115-117 — the consuming feature; Phase 117 is the actual chart-wiring phase
  and must migrate from the old `Component::HtmlEmbed` reference.

### Adjacent-phase precedent
- `.planning/phases/147-detailform-component-for-inline-edit-ferro-json-ui/`
  — phase 147 is the most recent component-change phase; its wave structure
  (RED tests → impl → surface updates) is the shape to emulate here even
  though we're extending not adding.

### Project principles
- `.planning/VISION.md` — agent-first surface; the MCP catalog and
  COMPONENT_CATALOG must communicate the dual-source shape and the SVG safety
  scope clearly; the updated Image entry is load-bearing, not cosmetic.
- `.planning/PROJECT.md` §"Beauty as a design criterion" — extending Image is
  the conceptually coherent move; a new `HtmlEmbed` would have added a
  projection-incoherent component that exists purely as an escape hatch.
- `/Users/alberto/.claude/CLAUDE.md` §"Architecture Principles" — "This is
  always a feature branch" — we refactor `ImageProps` to the new shape
  directly; no field-level deprecation, no dual-field compat, no migration
  shim. Existing in-tree `ImageProps { src, alt, ... }` literals get rewritten
  in the same commit that introduces `ImageSource`.
- `/Users/alberto/.claude/CLAUDE.md` §"Testing & Linting" — CI gate
  (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D
  warnings && cargo test --all-features`) is the Wave 2 completion criterion.

### Requirements (rewrite in ROADMAP)
- `.planning/ROADMAP.md` Phase 148 entry — needs rewrite:
  - Old: EMBED-01..EMBED-05 (HtmlEmbed variant, renderer bypass, resolver
    passes, MCP catalog, CI gate)
  - New: **IMG-SRC-01..IMG-SRC-05**:
    - IMG-SRC-01: `ImageSource` enum introduced (`Url {src}` / `InlineSvg {svg}`)
      with serde untagged discrimination
    - IMG-SRC-02: `ImageProps` refactored to flatten `source: ImageSource`;
      `alt` stays required
    - IMG-SRC-03: `render_image` branches on source; URL path unchanged
      (existing XSS escape test stays green); InlineSvg path emits `<div
      role="img" aria-label="{escaped alt}">{svg verbatim}</div>`
    - IMG-SRC-04: COMPONENT_CATALOG `### Image` section added; MCP
      `CatalogComponent` entry for Image describes both variants; docs
      `### Image` section added with safety callout on the SVG branch
    - IMG-SRC-05: CI gate green

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `html_escape(&str) -> String` (existing helper in `render.rs`) — already used
  in the URL branch; used on the `alt` attribute for the InlineSvg branch too.
- `ComponentNode::image` factory (`component.rs:1719-1722`) — keep as-is; takes
  `props: ImageProps` so the constructor refactor covers both variants.
- The existing `image_xss_src_escaped` test (`render.rs:3795`) — documents the
  URL branch's escape discipline; stays verbatim.
- `#[serde(flatten)]` + `#[serde(untagged)]` idioms — standard serde pattern,
  no new dependencies.

### Established Patterns
- **Raw SVG is already injected** via `StatCardProps.icon` and
  `ActionCardProps.icon`. Phase 148 promotes that pattern from "an icon prop on
  two components" to "a first-class source variant on Image".
- **`alt`-required for visual assets** — `ImageProps.alt: String` (not
  Option) since inception. This is what makes extending Image preferable to
  a new component: we get compile-enforced a11y text for free on the SVG
  variant too.
- **`html_escape`-by-default on dynamic strings** — upheld on the `src` and
  `alt` attributes in both source branches. The one deliberate bypass is the
  `svg` body of the InlineSvg variant — documented inline with a safety
  comment.

### Integration Points
- `ImageProps` in `component.rs:601-614` — refactored shape
- `render_image` in `render.rs:2420-2447` — branches on source
- `COMPONENT_CATALOG` in `lib.rs:103+` — `### Image` section added (new)
- `CatalogComponent` for Image in `ferro-mcp/src/tools/json_ui_catalog.rs` —
  widened
- `docs/src/json-ui/components.md` — `### Image` section added (new)
- All in-tree `ImageProps { src, alt, ... }` literals — rewrite to new shape

### Creative Options
- The load-bearing test asserting inline SVG with a `<script>` tag passes
  through verbatim serves as both a regression guard AND executable
  documentation of the intended-bypass contract. This is now scoped to the
  SVG variant only (not the general component), which makes it less of a
  general-HTML-XSS-passthrough statement.
- The `ambiguous source` rejection test (both `src` and `svg` set in JSON
  input) exercises the serde untagged discriminator and guards future
  refactors.

</code_context>

<specifics>
## Specific Ideas

- The coherence win: gestiscilo.it's `bar_chart_svg(data, w, h) -> String` can
  be dropped directly into a JSON-UI view as
  `Component::Image(ImageProps::inline_svg(bar_chart_svg(&data, 800, 300),
  "Incassi settimanali: 150€ lun, 320€ mar, …"))`. Alt text is required —
  a11y is structurally enforced.
- The asymmetry is scoped: every other `Image` render path escapes; the
  InlineSvg branch is the one documented exception. The inline comment in
  `render_image` makes this visible to future readers.
- The `<deferred>` pressure list from the HtmlEmbed scope largely evaporates:
  - `class/id/style` → handled by `aspect_ratio` + surrounding components
  - `wrapper_tag` → `<div role="img">` is the correct a11y shape; no variance
    needed
  - `data_path` → SVG is always Rust-constructed at plan time; not applicable
  - `sanitize` → if callers want sanitization, they run it before constructing
    `InlineSvg`; no in-component opt-in needed
  - `MarkdownEmbed` → orthogonal; a future narrower component, not a feature
    of Image
- Target content for `InlineSvg`: charts, sparklines, diagrams, decorative
  vector assets, server-rendered icon sets. Explicit non-target: user input.

</specifics>

<deferred>
## Deferred Ideas

- **`Component::Chart` with structured data** — a proper projection-aligned
  chart component (`bars: Vec<BarDatum>`, `width`, `height`, …) that owns
  SVG generation inside ferro. Ambitious and bigger than one phase; worth
  doing later, but the Image extension unblocks gestiscilo immediately.
- **`Component::MarkdownEmbed { source: String }`** — a narrower rendered-markdown
  component layered atop a Rust markdown renderer. If a real call-site ever
  needs this; not today.
- **SVG sanitization helper in the framework** — a `sanitize_svg(&str) -> String`
  utility that callers may opt into before constructing `InlineSvg`. Ambitious;
  ships only if a concrete threat model demands it.
- **`figure` / `figcaption` wrapper variant** — if a real caller needs caption
  text integrated with the visual, add a `caption: Option<String>` prop and
  switch to `<figure>` wrapper when present. Not today.
- **Cross-repo automation** — a script that, when ferro ships a surface
  change, updates consuming repos' planning docs automatically. gestiscilo's
  v6.1 research + ROADMAP need manual updates today; a future chore.

</deferred>

---

*Phase: 148-image-inline-svg-source*
*Context reworked: 2026-04-24 (prior HtmlEmbed scope archived in `archive-htmlembed/`)*
