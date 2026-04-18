# Phase 116: Flat Element Renderer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-18
**Phase:** 116-flat-element-renderer
**Mode:** `--auto` — all gray areas auto-selected with recommended defaults informed by Vercel json-render, the v1 renderer (8057 LOC reference), and Airbnb/DoorDash/Lyft SDUI practice.
**Areas discussed:** Dispatch architecture, Slot binding, Graceful failure, Visibility evaluation, Plugin fallback + asset collection, Module layout, Testing strategy.

---

## Dispatch architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Big `match el.type_name.as_str()` (~30 arms, plugin fallback in default arm) | Port of v1 `match &node.component { Component::X(_) => render_x(...) }`, string-keyed. Simple, fast, one file shows the full component surface. | ✓ |
| Dispatch table `HashMap<&str, fn(&Value, &Element, &Spec, &Value, usize) -> String>` | Lookup table, flexible but requires props-agnostic fn signature → inner deserialize overhead either way. | |
| Trait-object registry for built-ins (mirror plugin registry) | Maximally symmetric with plugins but loses niche optimization, adds `Box<dyn>` indirection, and obscures the canonical type list. | |

**Selected:** Big match. **Rationale:** Vercel json-render uses the same pattern at 13k★; the switch is the clearest documentation of "what components exist". Trait-object symmetry isn't a real runtime goal — plugins extend; built-ins are closed-set.

---

## Slot binding (multi-slot containers after Phase 115 stripped `Vec<ComponentNode>` fields)

| Option | Description | Selected |
|--------|-------------|----------|
| Slot-specific `Vec<String>` fields re-added to multi-slot Props (`CardProps.footer`, `ModalProps.footer`, `Tab.children`, `KanbanColumnProps.children`, `PageHeaderProps.actions`) | Multi-slot containers carry typed slot IDs in props; single-slot uses `Element.children`. Works within Phase 115's frozen Element shape. | ✓ |
| All children in `Element.children`, `Vec<usize>` indices in props for slot partitioning | Compact, but indices are author-hostile for hand-written and LLM-generated specs. | |
| Add `Element.slots: HashMap<String, Vec<String>>` | Cleanest generic slot model, but mutates Phase 115's frozen Element struct — out of Phase 116 scope. | |
| Convention-based: all children in `Element.children` ordered by slot (first N = body, last M = footer) | Fragile; needs sentinel counts in props; breaks down for Tabs/KanbanBoard. | |

**Selected:** Slot-specific Vec<String> fields. **Rationale:** Minimum change to existing shape, author-friendly (named slots, string IDs), preserves Phase 115's parse-time validator for `Element.children` while accepting that slot-borne IDs are Phase 117 catalog's responsibility.

---

## Graceful failure surface

| Option | Description | Selected |
|--------|-------------|----------|
| HTML comments (`<!-- ferro-json-ui: missing child 'id' -->`) + return empty fragment | Observable in devtools, zero new deps, preserves v1 `-> String` contract, graceful degradation. | ✓ |
| Panic on first authoring error | Matches Rust culture but is wrong for SDUI — production specs may come from AI and must not kill the request. | |
| Silent skip | Bugs become invisible; authors can't diagnose. | |
| `tracing::warn!` + skip | Requires adding `tracing` dep to a library crate; devtools visibility still requires separate instrumentation. | |

**Selected:** HTML comment diagnostics. **Rationale:** Zero-dep, browser-visible, consistent with v1's style (no logging infra), and informs authors inline. If ops experience later shows comments are insufficient, `tracing` can be added in a follow-up.

---

## Props deserialization failure behavior

| Option | Description | Selected |
|--------|-------------|----------|
| `from_value::<TProps>(el.props.clone())` → on Err, emit HTML comment + return empty | Typed, strict, diagnostic-friendly. | ✓ |
| `.unwrap_or_default()` | Hides real authoring bugs behind plausible-looking output. | |
| Panic | Wrong for SDUI (see graceful-failure rationale). | |

**Selected:** Emit diagnostic + empty. **Rationale:** Consistent with D-10 graceful-failure stance; authors always get a signal.

---

## Visibility evaluation

| Option | Description | Selected |
|--------|-------------|----------|
| Inline per-element check inside `render_element` (evaluate then dispatch) | Cheap, no extra pass, children never walk if parent hidden. React-semantics. | ✓ |
| Pre-pass that prunes the spec before render | Allocates extra Spec copy; complicates render API. | |
| CSS `display:none` emission | Wrong for SEO/a11y; hidden content still in DOM. | |

**Selected:** Inline check. **Rationale:** Correct semantics (hidden = not emitted); lowest overhead; matches v1 intent.

---

## Plugin fallback + asset collection

| Option | Description | Selected |
|--------|-------------|----------|
| Dispatch default arm consults plugin registry; separate O(n) pass over `spec.elements` collects plugin type names for asset emission | Symmetric with v1 plugin path; walk is flat so collection is a trivial iterator. | ✓ |
| Built-ins registered as plugins too (one registry, one loop) | Loses compile-time dispatch benefit and obscures the built-in surface. | |
| Asset collection inlined into render walk | Couples render and collection; harder to test independently. | |

**Selected:** Registry fallback in default arm + separate collection pass. **Rationale:** Preserves the simple built-in dispatch, mirrors v1's `collect_plugin_types_node` structure (now flat instead of recursive), keeps collection testable in isolation.

---

## Built-in type identification

| Option | Description | Selected |
|--------|-------------|----------|
| `const BUILTIN_TYPES: &[&str]` next to the dispatch match — one source of truth | Cheap, static, easy to audit. | ✓ |
| Derive from the dispatch match at macro-expansion time | Premature abstraction; adds compile-time complexity. | |
| Look up in the plugin registry with a "plugins cannot shadow built-ins" check per element | Slower, runs registry lock per dispatch. | |

**Selected:** `const BUILTIN_TYPES` constant. **Rationale:** Simple, debuggable, used both by the dispatch match (implicitly via arm exhaustion) and by the asset-collection pass (explicitly via "not in BUILTINS").

---

## Module layout

| Option | Description | Selected |
|--------|-------------|----------|
| `render/` directory with `mod.rs` (API + dispatch), `containers.rs`, `form.rs`, `data.rs`, `atoms.rs` | 4–6 files <2000 LOC each, grouped by responsibility, matches v1 function grouping. | ✓ |
| Single `render.rs` (~8000 LOC port) | Matches v1 exactly but is unwieldy for navigation and review. | |
| One file per component type (~30 files) | Fragmented; too many files for the dispatch to cross-reference. | |

**Selected:** `render/` directory split. **Rationale:** Balances locality (group by concern) with reviewability (no single 8000 LOC file); per-component renderers live near related ones so changes to styling conventions stay local.

---

## Testing strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Port v1 inline tests + add Phase-116-specific walker tests (missing-slot, visibility-hides, plugin-dispatch, action-inlining, cycle-tripwire) + integration at framework level | Full coverage of both contract continuity and new flat-walker semantics. | ✓ |
| Golden-file snapshot tests only | Captures HTML regressions but misses walker correctness and graceful-failure behaviors. | |
| Rely on framework integration tests only | Under-covers the crate's public API; hard to diagnose crate-internal regressions. | |

**Selected:** Ported v1 tests + new walker tests + framework integration updates. **Rationale:** Covers both "v1 HTML stays byte-stable for ported cases" and "v2 semantics (flat walk, diagnostic surface) work as specified".

---

## Claude's Discretion

- Exact file split between `render/containers.rs` and `render/data.rs` — pick whatever balances line counts.
- Whether to introduce a `Walker { spec, data, depth }` struct vs. passing three args explicitly — pick cleaner idiom after port.
- Whether per-component functions are `pub(crate)` in section files or private with module-level re-export — idiomatic Rust choice.
- Diagnostic comment exact wording — consistency matters more than specific phrasing.
- Whether to emit a root-level wrapper `<div class="flex flex-wrap gap-4 ...">` identically to v1 (yes — matches existing gestiscilo styling expectations).

## Deferred Ideas (captured from this discussion)

- `tracing` / `log` dep (might be added post-v1.0 if HTML-comment diagnostics prove insufficient).
- Render-cache / memoization for shared children (post-v1.0 perf pass).
- Streaming / `Write`-based renderer (post-v1.0).
- Full slot-ID graph validation (Phase 117 catalog concern).
- Sandboxing externally-authored specs (not in current threat model).
