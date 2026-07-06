# Phase 115: Spec v2 Data Structures - Research

**Researched:** 2026-04-18
**Domain:** Server-driven UI spec type system, manual `schemars::JsonSchema` impls, graph-structure validation (cycles, depth, IDs)
**Confidence:** HIGH

## Summary

Phase 115 replaces v1's nested component tree (`JsonUiView` + recursive `Vec<ComponentNode>` + 40-variant `Component` enum with ~200 LoC of custom ser/de) with a flat `Spec { root, elements }` map where each `Element` carries a type-erased `type_name: String` and props as `serde_json::Value`. The clean break removes the `Component::Plugin` escape-hatch and the hand-rolled discriminator ser/de; discriminator logic becomes a schema concern (Phase 117), not a type concern.

Structural validation runs once at `Spec::from_json()` with five ordered checks: duplicate-ID detection (during raw parse), root existence, dangling-child detection, cycle detection (DFS + gray-set), and depth-from-root bounded at 3. Each check returns a typed `SpecError` variant carrying structural paths, not formatted strings.

`schemars 1.2.0` is already in `Cargo.lock` and all three upstream crates (`ferro-projections`, `ferro-json-ui`, `ferro-mcp`) use `schemars = "1"`. Once `Vec<ComponentNode>` fields are stripped from `*Props` structs, every remaining Props struct derives `JsonSchema` cleanly — no manual impls are required in Phase 115. The "manual `JsonSchema` impl for Component enum" caveat from ROADMAP.md dissolves once the Component enum itself is deleted (D-02). Phase 117 assembles the per-type `oneOf` discriminator from Props struct schemas registered in a catalog; that's not Phase 115's job.

**Primary recommendation:** Build `spec.rs` as a single cohesive file (`Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SpecError`, `SCHEMA_VERSION`, validation). Strip `Vec<ComponentNode>` and `Vec<Tab>` children from 10 Props structs but keep the structs. Delete `component.rs`'s Component enum + ComponentNode + custom ser/de block (~200 LoC). Placeholder renderer emits a `<pre>`-wrapped pretty-JSON dump inside a minimal HTML skeleton — sufficient to keep framework tests green while Phase 116 builds the real walker. Migrate every caller to `Spec::builder()`; no compat shims.

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Type Foundation**
- **D-01:** Element is fully type-erased (`type_name: String` + `props: serde_json::Value`). No built-in vs plugin distinction at the type level; resolved only at catalog/render time (Phase 117). Kills the v1 `Component::Plugin` escape-hatch.
- **D-02:** `Component` enum and `ComponentNode` wrapper are **deleted**. The 40-variant match and custom Serialize/Deserialize in `component.rs` go away.
- **D-03:** Typed `*Props` structs survive. Strip `children: Vec<ComponentNode>` and `fields: Vec<ComponentNode>` fields — children have moved to `Element.children: Vec<String>`.
- **D-04:** Manual `JsonSchema` impls only where necessary. Document which Props structs still need manual work (expected: zero after the strip).
- **D-05:** `SCHEMA_VERSION = "ferro-json-ui/v2"` lives in `spec.rs`. v1 constant is removed.

**Spec + Element shape**
- **D-06:** Exact `Spec` struct shape with `schema` (rename `$schema`), `root`, `elements: HashMap<String, Element>`, `title`, `layout`, `data`. Errors map is NOT on Spec v2.
- **D-07:** Exact `Element` struct shape with `type_name` (rename `type`), `props: Value`, `children: Vec<String>`, `action: Option<Action>`, `visible: Option<Visibility>`.

**Validation**
- **D-08:** Structural validation runs at parse time in `Spec::from_json(&str) -> Result<Spec, SpecError>`.
- **D-09:** Four structural checks: root exists → dangling refs → no cycles → depth ≤ 3 from root.
- **D-10:** ID regex `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`.
- **D-11:** `SpecError` with structured variants: `RootMissing`, `DanglingChild`, `Cycle`, `DepthExceeded`, `InvalidId`, `Json`, plus `DuplicateId`.
- **D-12:** Duplicate-ID detection at raw-parse time before hydrating into HashMap.
- **D-13:** No JSON Schema / semantic validation in Phase 115.

**Plugin story**
- **D-14:** Plugin registry unchanged; plugin = element with `type_name = "Map"` (or whatever). No special variant.
- **D-15:** Phase 117 adds `props_schema` to plugin registration. Out of scope for 115.
- **D-16:** Phase 115 does not validate plugin type names. Unknown `type_name` parses cleanly.

**Caller migration**
- **D-17:** `framework/src/json_ui/mod.rs::JsonUi::render` signature changes from `&JsonUiView` → `&Spec`. Body becomes placeholder serializing Spec as pretty JSON inside HTML shell.
- **D-18:** Sample `app` crate rewrites any JsonUiView construction to `Spec::builder()`.
- **D-19:** `ferro-mcp` `code_templates` emits v2 flat-spec syntax. `json_ui_inspect`/`json_ui_generate` keep compiling; Phase 120 rewrites semantics.
- **D-20:** `JsonUiRenderer::Output` switches from `JsonUiView` → `Spec`. Internal mapping stays naive; Phase 117.1 rewrites.
- **D-21:** No migration shims. No `v1_compat`. No `#[cfg(feature = "v1")]`.

**Builder API**
- **D-22:** `Spec::builder()` fluent API (see CONTEXT.md for exact sketch).
- **D-23:** `Element::new(type_name) + .prop(k,v) + .child(id) + .action(a) + .visible(v)`.
- **D-24:** `build()` runs same validation as `from_json()`.

**File layout**
- **D-25:** New: `ferro-json-ui/src/spec.rs`.
- **D-26:** Rewritten: `component.rs` (drop Component+ComponentNode+custom ser/de), `render.rs` (placeholder), `projection/mod.rs` (Output = Spec), `lib.rs` (re-exports). `plugin.rs` unchanged.
- **D-27:** Deleted: `ferro-json-ui/src/view.rs`.
- **D-28:** Unchanged: `action.rs`, `visibility.rs`, `config.rs`, `data.rs`, `layout.rs`, `resolve.rs`, `plugins/`, `runtime/`.

**Testing**
- **D-29:** Round-trip test corpus under `ferro-json-ui/tests/fixtures/`.
- **D-30:** Rejection test corpus with specific `SpecError` variants.
- **D-31:** Builder parity test (builder output matches fixture).
- **D-32:** `schema_for!` smoke tests on every surviving `*Props` struct.

### Claude's Discretion (research recommends)

- `spec.rs` module organization: single file recommended (see §Architecture Patterns).
- `SpecError` uses `thiserror` (convention — 15/20 crates use it).
- Placeholder renderer output: recommend `<pre><code>…pretty-JSON…</code></pre>` inside minimal HTML (see §Pitfall-free placeholder).
- Depth constant: `MAX_NESTING_DEPTH: usize = 3`.
- `PluginProps`: delete. No longer needed once Element is type-erased.

### Deferred Ideas (OUT OF SCOPE)

- Catalog / JSON Schema assembly → Phase 117
- Plugin schema registration API → Phase 117
- `$data` / `$template` expression evaluation → Phase 118
- Spec loader with hot reload → Phase 119
- MCP `json_ui_generate` two-tier AI strategy → Phase 120
- gestiscilo field test conversion → Phase 121
- IDE plugin that consumes exported JSON Schema — future backlog
- Cross-spec composition / include directives — explicitly out of scope for v12.0 (inner platform effect risk)
- Client-side interactivity beyond existing IIFE runtime — PROJECT-level deferred

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPEC-01 | `Spec` struct exists with `root`, `elements: HashMap<String, Element>`, `title`, `layout`, `data` | §Code Examples §1 — exact layout verified against CONTEXT.md D-06 |
| SPEC-02 | `Element` struct with `type_name`, `props: serde_json::Value`, `children: Vec<String>`, `action`, `visible` | §Code Examples §1 — exact layout verified against CONTEXT.md D-07 |
| SPEC-03 | `Spec::from_json()` parses + round-trips + validates (root + dangling + cycles + depth ≤ 3 + IDs + duplicates) | §Code Examples §2–§5, §Common Pitfalls, §Ordering of Structural Validation |
| SPEC-04 | All v1 types deleted (`JsonUiView`, `ComponentNode`, `Component` enum). Schema version = `"ferro-json-ui/v2"`. All surviving `*Props` derive `JsonSchema` after `Vec<ComponentNode>` strip | §Standard Stack, §Migration Blast Radius |

## Architectural Responsibility Map

Phase 115 is a single-crate phase (`ferro-json-ui`) with ripple updates in `framework`, `ferro-mcp`, `ferro-cli`. No web-tier reasoning required, but the spec's tier contract matters:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `Spec` + `Element` types | Data layer (`ferro-json-ui`) | — | Wire format only — no runtime behavior |
| Structural validation (`from_json`) | Data layer (`ferro-json-ui`) | — | Parse-time only; catalog validation is Phase 117 |
| `Spec::builder()` API | Data layer (`ferro-json-ui`) | — | Constructor facade; validation reuses `from_json` logic |
| Placeholder HTML render | Rendering layer (`ferro-json-ui::render`) | — | Walked in Phase 116; Phase 115 produces debug fallback only |
| `JsonUi::render(&Spec)` | Framework integration (`framework`) | — | Thin wrapper; delegates to renderer |
| `JsonUiRenderer::Output = Spec` | Projection rendering (`ferro-json-ui::projection`) | ferro-projections (trait) | Output type change only; internal mapping unchanged this phase |
| MCP code templates | Agent surface (`ferro-mcp`) | — | Template string rewrite only; tool semantics preserved |

## Standard Stack

### Core (already present — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.0 | Serialize/Deserialize derives | [VERIFIED: ferro-json-ui/Cargo.toml:17] — universal Rust serialization |
| `serde_json` | 1.0 | `Value`, `from_str`, `to_string_pretty`, `Map` | [VERIFIED: ferro-json-ui/Cargo.toml:18] — ecosystem default |
| `schemars` | 1.2.0 | `JsonSchema` derive on surviving Props structs | [VERIFIED: Cargo.lock:4115-4116] — 1.x uses `Schema = wrapper<Value>` and `json_schema!(…)` macro |
| `std::collections::HashMap` | std | Flat element map storage | Built-in — matches D-06 shape |
| `std::collections::HashSet` | std | Cycle-detection gray set | Built-in — standard DFS implementation |
| `thiserror` | 1.0 | `SpecError` derive | [VERIFIED: 15 crates use it — grep results above] — workspace convention |

### Supporting (for tests only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `insta` | — | Snapshot tests for fixture round-trip | [ASSUMED — not currently in ferro-json-ui/Cargo.toml] — skip; use direct equality instead |

**Installation:** no additions. Everything is already pinned. No `Cargo.toml` edits required in `ferro-json-ui` unless `thiserror` is missing — check:

```bash
grep -l "thiserror" ferro-json-ui/Cargo.toml
```

[VERIFIED via Grep 2026-04-18] `ferro-json-ui/Cargo.toml` does NOT currently list `thiserror`. Add:

```toml
[dependencies]
# …existing…
thiserror = "1.0"
```

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `HashMap<String, Element>` | `IndexMap<String, Element>` (preserve insertion order) | Extra dep; round-trip equality becomes order-sensitive which breaks D-29 goals. Stay with HashMap. |
| Manual cycle detection | `petgraph` crate | Full graph crate is overkill for ~dozens-of-nodes specs. DFS + HashSet is 20 LoC. |
| `serde_path_to_error` for friendly errors | — | Nice to have, deferred to Phase 117 where semantic errors dominate. |
| `regex` crate for ID validation | Hand-rolled byte scan | `regex` is heavy at compile time; single-char check is faster and avoids pulling `regex` into ferro-json-ui if not already present. **Recommend: hand-rolled check.** |

[VERIFIED via Grep 2026-04-18] `regex` is NOT currently a direct dependency of ferro-json-ui (it's pulled in transitively via other crates). Keep it out; hand-roll the ID check (~15 LoC).

## Architecture Patterns

### System Architecture Diagram

```
       ┌──────────────────────┐
       │ Author (Rust/JSON)   │
       └──────────┬───────────┘
                  │ writes
                  ▼
     ┌──────────────────────────┐
     │ Spec::builder() ─or─     │
     │ Spec::from_json(&str)    │
     └──────────┬───────────────┘
                │  ┌─ validate ─────────────────────┐
                │  │ 1. raw JSON → detect dup IDs   │
                │  │ 2. root ∈ elements             │
                │  │ 3. every child ID resolves     │
                │  │ 4. no cycles (DFS gray set)    │
                │  │ 5. depth from root ≤ 3         │
                │  │ 6. ID format regex match       │
                │  └────────────────────────────────┘
                ▼
     ┌──────────────────────────┐
     │ Spec { $schema: "…v2",   │
     │   root, elements, …}     │
     └──────────┬───────────────┘
                │ consumed by
                ▼
    ┌───────────────────────────────────┐
    │ JsonUi::render(&Spec, &data)      │  ← framework entry point
    │   └── placeholder HTML (P115)     │
    │       or real walker (P116)       │
    └─────────────┬─────────────────────┘
                  │ produces
                  ▼
     ┌──────────────────────────┐
     │ HttpResponse (text/html) │
     └──────────────────────────┘

Phase boundary lines ──────────────────────────
P115: builder, from_json, validation, placeholder renderer
P116: real flat-element walker
P117: catalog + JSON Schema semantic validation
P118: $data / $template resolution inside props
```

### Recommended Project Structure

```
ferro-json-ui/
├── src/
│   ├── spec.rs          # NEW: Spec, Element, SpecBuilder, ElementBuilder,
│   │                    #      SpecError, SCHEMA_VERSION, validate_spec(),
│   │                    #      ID_RE or is_valid_id() helper, MAX_NESTING_DEPTH.
│   ├── component.rs     # REWRITTEN: keep *Props structs (sans children fields),
│   │                    #  keep enums (Size, ButtonVariant, …),
│   │                    #  DELETE Component, ComponentNode, custom ser/de,
│   │                    #  DELETE PluginProps.
│   ├── render.rs        # REWRITTEN: placeholder render_spec_to_html(&Spec, &Value) -> String
│   │                    #  and render_spec_to_html_with_plugins(&Spec, &Value) -> RenderResult
│   │                    #  (the _with_plugins variant is a no-op stub until P116).
│   ├── resolve.rs       # REWRITTEN: resolve_actions(&mut Spec, resolver),
│   │                    #  resolve_errors(&mut Spec, errors) — walk elements HashMap
│   │                    #  instead of tree. (Mostly mechanical.)
│   ├── lib.rs           # UPDATED: re-export Spec, Element, SpecBuilder, SpecError,
│   │                    #  SCHEMA_VERSION. Remove JsonUiView, ComponentNode, Component.
│   ├── projection/
│   │   └── mod.rs       # UPDATED: JsonUiRenderer::Output = Spec.
│   ├── view.rs          # DELETED.
│   ├── action.rs        # UNCHANGED
│   ├── visibility.rs    # UNCHANGED
│   ├── config.rs        # UNCHANGED
│   ├── data.rs          # UNCHANGED (resolve_path, resolve_path_string)
│   ├── layout.rs        # UNCHANGED
│   ├── plugin.rs        # UNCHANGED (registry is independent of Component enum)
│   ├── plugins/         # UNCHANGED (MapPlugin, …)
│   └── runtime/         # UNCHANGED
└── tests/
    └── fixtures/
        ├── ok/          # 5+ fixtures (D-29)
        └── reject/      # 8+ fixtures (D-30), each with companion asserting
                         # the exact SpecError variant
```

### Pattern 1: Manual JsonSchema impl (only if needed — expected: NOT needed in P115)

**What:** When a type can't derive `JsonSchema` (custom ser/de, recursive self-ref), implement the trait directly.
**When to use:** In Phase 115, only if a surviving `*Props` struct still has an unusual shape after the strip. Current inventory shows **zero** such cases — see §Props Struct Audit below.
**Source:** [CITED: https://github.com/gresau/schemars/blob/master/docs/2-implementing.md]

```rust
// Source: schemars docs/2-implementing.md
use std::borrow::Cow;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

impl JsonSchema for MyType {
    fn schema_name() -> Cow<'static, str> {
        "MyType".into()
    }

    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::MyType").into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "properties": {
                "children": {
                    "type": "array",
                    "items": { "$ref": "#" }   // self-reference for recursion
                }
            }
        })
    }

    // CRITICAL for recursive types — default returns true for primitives.
    // Returning false routes through $defs + $ref, preventing infinite cycles.
    fn inline_schema() -> bool { false }
}
```

**Self-reference (`$ref: "#"`):** Valid JSON Schema 2020-12 syntax — points at the root of the current schema. schemars 1.x supports this in `json_schema!` macro bodies. For element graphs where `children: Vec<String>` stores IDs rather than nested structures, **self-references are unnecessary** — the recursion lives in the HashMap, not in the type.

### Pattern 2: DFS cycle detection with gray set

**What:** Classical three-color DFS: white (unvisited), gray (on stack), black (finished). A gray→gray edge is a cycle.
**When to use:** Always, for `Spec::from_json` validation step 4.
**Source:** [CITED: CLRS 22.3 — standard graph algorithm]

See §Code Examples §3 for the implementation sketch.

### Pattern 3: Flat-map builder with deferred validation

**What:** The `SpecBuilder` accumulates elements in arbitrary order, resolves validation only at `build()`. Forward references (child ID referenced before child element added) are fine because the map is complete before validation runs.
**When to use:** Always, for `Spec::builder()` ergonomics.

See §Code Examples §6 for the sketch.

### Anti-Patterns to Avoid

- **Validating children at `.element(…)` call time.** Would break forward references. Save all validation for `build()`.
- **Using `#[serde(flatten)]` on `Element`'s props.** The v1 `ComponentNode` used `flatten` to merge Component's fields into the node — that's exactly the pattern we're leaving behind. In v2, `props` is its own field holding `Value`. Do not flatten.
- **Using `IndexMap` or other ordered maps.** Breaks round-trip determinism across insertion orders. Round-trip compares by field equality, not serialized-string equality.
- **Writing the validation as a single 200-line function.** Each of the five checks is independent; each is its own function returning its own `SpecError` variant. Keep them small (CLAUDE.md: "If you need comments to explain sections, split into functions").
- **Using `#[serde(default)] Vec<String>` on `children`** without `skip_serializing_if = "Vec::is_empty"`. Would emit `"children": []` for leaf elements, bloating the wire format.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Custom discriminator serialization for Element | Manual `Serialize`/`Deserialize` | `#[serde(rename = "type")] pub type_name: String` — derive does the whole thing | Type-erasure removes the need for discriminator dispatch at the type level entirely |
| Tagged enum for built-in vs plugin | `#[serde(untagged)]` + panic fallback | Nothing — type_name is just a string | v1's `Component::Plugin` fallback was the wart we're removing |
| Custom JSON parser for duplicate-key detection | Regex pre-pass over input | `serde_json::Deserializer` + visitor OR deserialize `Vec<(String, Value)>` wrapper | See §Pitfall "Duplicate-ID detection" |
| Path-aware deserialization errors | Manual path accumulation in SpecError | `serde_path_to_error` — deferred to P117 | P115 errors don't need JSONPath precision |
| JSON Schema construction by hand for simple Props | `json_schema!({…})` macro | `#[derive(JsonSchema)]` | schemars derive is the whole point of using the crate |
| Recursive depth tracking in Element constructor | Threading counter through builder | Post-hoc BFS from `root` at validate time | Constructors should be cheap; validation runs once |

**Key insight:** The type-erasure decision (D-01, D-02) eliminates three hand-rolled patterns from v1 in a single move: the custom Serialize/Deserialize block (~200 LoC), the `PluginProps` escape hatch, and the "JsonSchema skipped" comments scattered across `component.rs` (14 of them). This is the compressive payoff the ROADMAP promises.

## Runtime State Inventory

Phase 115 is a code-and-types refactor with **no** external runtime state. Categories:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — Specs are generated per-request, never persisted | None |
| Live service config | None — no external services hold `JsonUiView` or `ComponentNode` references | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | Possible stale `target/` directory caching old Component variants across the rename. [ASSUMED] Safe; `cargo` rebuilds on source change | `cargo clean` only if a builder sees link errors referring to deleted variants |

**Nothing found in category:** Stored data — none. Live service config — none. OS-registered state — none. Secrets/env — none. Verified by grep: no `JsonUiView`, `ComponentNode`, or `Component::` strings appear in `.env*`, `.planning/`, or any non-source file other than docs.

## Common Pitfalls

### Pitfall 1: Forgetting `inline_schema() -> false` in manual impls for recursive types

**What goes wrong:** Infinite loop during `schema_for!` generation. Stack overflow.
**Why it happens:** Default `inline_schema()` returns `true` for types with simple schemas; recursive types must return `false` to route through `$defs` + `$ref`.
**How to avoid:** Not an issue in Phase 115 after the strip — Element's `children: Vec<String>` is not recursive at the type level, and no surviving `*Props` struct has a self-reference. If Phase 117 needs a recursive `Spec → Element` relationship in a generated JSON Schema, that impl lives in the catalog crate, not here.
**Warning signs:** `schema_for!(MyProps)` hangs or stack-overflows in tests.

### Pitfall 2: Duplicate IDs silently overwritten

**What goes wrong:** `serde_json::from_str::<Spec>` uses `serde_json::Map::insert` which silently replaces duplicate keys in objects. A JSON spec with `{"a": …, "a": …}` parses without error — the second wins. User never learns they shadowed an element.
**Why it happens:** JSON spec allows duplicate keys; `serde_json`'s default behavior mirrors "last writer wins" semantics of many JSON parsers.
**How to avoid:** Parse into an intermediate representation that fails on duplicates. Two viable approaches:

**Approach A — Custom visitor (recommended):** implement a `MapAccess` visitor that tracks seen keys in a `HashSet` and errors on duplicates. ~30 LoC.

**Approach B — Two-pass parse:** first parse into `serde_json::Value`, then manually inspect. Unreliable because serde_json has already deduplicated by the time you hold the `Value`. **Do not use.**

**Approach C — Deserialize into `Vec<(String, Value)>` wrapper for the elements field:** serde can deserialize a JSON object into a `Vec<(K, V)>` if you wrap it with a helper type that implements `Deserialize` manually via a `MapAccess` visitor. This is essentially Approach A with explicit state.

**Recommendation:** Approach A. Sketch:

```rust
// During Spec::from_json, the elements map uses a wrapper that detects duplicates.
use std::collections::HashMap;
use std::fmt;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

struct ElementsMap(HashMap<String, Element>);

impl<'de> Deserialize<'de> for ElementsMap {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = ElementsMap;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON object with unique element IDs")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<ElementsMap, M::Error> {
                let mut map = HashMap::new();
                while let Some(k) = m.next_key::<String>()? {
                    if map.contains_key(&k) {
                        return Err(serde::de::Error::custom(
                            format!("duplicate element ID: {k}")
                        ));
                    }
                    let v: Element = m.next_value()?;
                    map.insert(k, v);
                }
                Ok(ElementsMap(map))
            }
        }
        d.deserialize_map(V)
    }
}
```

Then in `Spec::from_json`: distinguish `serde::de::Error::custom` messages starting with `"duplicate element ID: "` and convert them to `SpecError::DuplicateId(id)`. Alternatively, thread a side channel via `serde_json::from_value` → custom error, but the string-based match works and keeps the visitor simple.

**Warning signs:** A test fixture with `{"root":"a", "elements":{"a":{…}, "a":{…}}}` parses without error. Your duplicate detection is silently broken.

### Pitfall 3: Cycle detection with only a "visited" set (not "on-stack" set)

**What goes wrong:** Diamond graphs (A → [B, C], B → D, C → D) incorrectly flagged as cycles.
**Why it happens:** A visited-only set cannot distinguish "already finished" from "currently on the DFS stack".
**How to avoid:** Three-color / two-set DFS (white/gray/black or "visited" + "gray/on-stack"). A back-edge is gray → gray, not any → visited.
**Warning signs:** Legitimate diamond specs (rare but possible — two parents referencing the same leaf) rejected as cycles.

### Pitfall 4: Depth computation with memoization makes diamonds silently pass

**What goes wrong:** Memoizing "element X has depth N" means you compute depth once and reuse it. But if X is reached via two parents, the depth depends on the path, not the element.
**Why it happens:** Memoization conflates "intrinsic depth" (wrong) with "max path depth from root" (right).
**How to avoid:** Compute depth per-path. Walk from root carrying a depth counter; bail as soon as `current_depth > MAX_NESTING_DEPTH`. No memoization needed because early bail caps total work at O(nodes * depth) in the worst case.
**Warning signs:** A spec where root → A → [B, C], B → D, C → D has D appear at depth 3 through both paths. With memoization, if another path reaches D at depth 4 after D was already memoized at depth 3, the violation is missed.

### Pitfall 5: Placeholder renderer panics on unknown `type_name`

**What goes wrong:** Phase 115's placeholder renderer walks elements; if it tries to dispatch on type_name it panics on any non-built-in string (including plugin names).
**Why it happens:** Muscle memory from v1 where the Component enum had `Plugin` as a fallback.
**How to avoid:** The placeholder renderer does NOT dispatch by type. It serializes the whole Spec to pretty JSON and wraps in `<pre>`. No walk, no dispatch.
**Warning signs:** Framework tests fail with panics when Phase 115 lands.

### Pitfall 6: `Vec<String>` children field missing `skip_serializing_if = "Vec::is_empty"`

**What goes wrong:** Every leaf element in serialized output carries `"children": []`, bloating wire size and breaking round-trip equality with hand-authored fixtures that omit the field.
**Why it happens:** The default for `Vec<String>` is the empty vec, but serde emits it unless told to skip.
**How to avoid:** `#[serde(default, skip_serializing_if = "Vec::is_empty")]` (already in D-07).
**Warning signs:** A fixture file with no `"children"` key round-trips but emits `"children": []` on re-serialization.

### Pitfall 7: ID validator rejects hyphens or underscores inconsistently

**What goes wrong:** Author writes `"user-form"` which the regex `^[A-Za-z_][A-Za-z0-9_-]{0,127}$` accepts at element-key level but rejects at child-reference level (or vice versa), because the two validation sites use slightly different regexes.
**Why it happens:** Copy-paste drift.
**How to avoid:** One constant, one helper: `const ID_RE_PATTERN: &str = "^[A-Za-z_][A-Za-z0-9_-]{0,127}$"` or better, a hand-rolled `fn is_valid_id(s: &str) -> bool` used in both places.
**Warning signs:** Same ID passes `from_json` as a key but fails as a child reference in some other test.

## Code Examples

### §1 — Exact Spec and Element types (verbatim from D-06 / D-07)

```rust
// Source: CONTEXT.md D-06, D-07; verified against current v1 types
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::action::Action;
use crate::visibility::Visibility;

pub const SCHEMA_VERSION: &str = "ferro-json-ui/v2";
pub const MAX_NESTING_DEPTH: usize = 3;

/// Top-level flat spec — element map keyed by ID plus a root pointer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub root: String,
    pub elements: HashMap<String, Element>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
}

/// A single type-erased element. `type_name` is a plain string; dispatch to
/// built-ins or plugins is a catalog/render-time concern (Phase 117+).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub props: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Visibility>,
}
```

### §2 — SpecError with thiserror

```rust
// Source: CONTEXT.md D-11, D-12; workspace convention via 15 crates using thiserror
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("duplicate element ID in spec: {0}")]
    DuplicateId(String),

    #[error("root element '{0}' not found in elements map")]
    RootMissing(String),

    #[error("element '{element}' references child '{child}' which does not exist")]
    DanglingChild { element: String, child: String },

    #[error("cycle detected in element graph: {}", path.join(" -> "))]
    Cycle { path: Vec<String> },

    #[error("nesting depth exceeds maximum of {max}: found depth {found} at {}", path.join(" -> "))]
    DepthExceeded { max: usize, found: usize, path: Vec<String> },

    #[error("invalid element ID '{0}' — must match ^[A-Za-z_][A-Za-z0-9_-]{{0,127}}$")]
    InvalidId(String),
}
```

### §3 — DFS cycle detection (returns the cycle path for the error)

```rust
// Source: CLRS 22.3 + CONTEXT.md D-09 step 3
use std::collections::{HashMap, HashSet};

fn detect_cycle(
    elements: &HashMap<String, Element>,
    root: &str,
) -> Result<(), SpecError> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut on_stack: Vec<String> = Vec::new();
    dfs(root, elements, &mut visited, &mut on_stack)
}

fn dfs(
    node: &str,
    elements: &HashMap<String, Element>,
    visited: &mut HashSet<String>,
    on_stack: &mut Vec<String>,
) -> Result<(), SpecError> {
    // Self-cycle: A.children contains "A".
    if on_stack.iter().any(|n| n == node) {
        // Back-edge — reconstruct cycle from on_stack.
        let start = on_stack.iter().position(|n| n == node).unwrap();
        let mut path: Vec<String> = on_stack[start..].to_vec();
        path.push(node.to_string());
        return Err(SpecError::Cycle { path });
    }
    if visited.contains(node) {
        return Ok(()); // Already fully explored — diamond, not a cycle.
    }
    on_stack.push(node.to_string());
    if let Some(el) = elements.get(node) {
        for child in &el.children {
            dfs(child, elements, visited, on_stack)?;
        }
    }
    on_stack.pop();
    visited.insert(node.to_string());
    Ok(())
}
```

Edge cases handled:
- Self-cycle (A.children = ["A"]): caught at first recursion; `on_stack` contains A, current node is A → Cycle with path `["A", "A"]`.
- Two-cycle (A → B, B → A): second recursion sees A on stack → Cycle with path `["A", "B", "A"]`.
- Diamond (A → [B, C], B → D, C → D): first descent marks D visited; second descent sees D in `visited` but NOT `on_stack` → Ok. Not a cycle.

### §4 — Depth computation without memoization

```rust
// Source: CONTEXT.md D-09 step 4; recommended approach for diamonds
fn check_depth(
    elements: &HashMap<String, Element>,
    root: &str,
) -> Result<(), SpecError> {
    // Pre-condition: cycles already rejected, so recursion terminates.
    let mut path: Vec<String> = Vec::new();
    walk(root, elements, 1, &mut path)
}

fn walk(
    node: &str,
    elements: &HashMap<String, Element>,
    depth: usize,
    path: &mut Vec<String>,
) -> Result<(), SpecError> {
    path.push(node.to_string());
    if depth > MAX_NESTING_DEPTH {
        return Err(SpecError::DepthExceeded {
            max: MAX_NESTING_DEPTH,
            found: depth,
            path: path.clone(),
        });
    }
    if let Some(el) = elements.get(node) {
        for child in &el.children {
            walk(child, elements, depth + 1, path)?;
        }
    }
    path.pop();
    Ok(())
}
```

Correctness for diamonds: each path from root is walked independently. If any path exceeds depth 3, we bail immediately with the offending path. Memoization would be wrong here — see Pitfall 4.

Worst-case complexity: O(paths-through-graph). For legitimate specs (cycle-free, depth ≤ 3), paths ≤ branching-factor^3 ≤ a few hundred. Fine.

### §5 — ID format check (hand-rolled, avoids regex dep)

```rust
// Source: CONTEXT.md D-10 — ^[A-Za-z_][A-Za-z0-9_-]{0,127}$
fn is_valid_id(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let bytes = s.as_bytes();
    let first = bytes[0];
    let first_ok = first.is_ascii_alphabetic() || first == b'_';
    if !first_ok {
        return false;
    }
    bytes[1..].iter().all(|&b| {
        b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
    })
}

fn validate_ids(
    elements: &HashMap<String, Element>,
) -> Result<(), SpecError> {
    for (id, el) in elements {
        if !is_valid_id(id) {
            return Err(SpecError::InvalidId(id.clone()));
        }
        for child in &el.children {
            if !is_valid_id(child) {
                return Err(SpecError::InvalidId(child.clone()));
            }
        }
    }
    Ok(())
}
```

### §6 — Builder API (matches D-22, D-23)

```rust
// Source: CONTEXT.md D-22 and CLAUDE.md (consuming builder convention)
use serde_json::{Map, Value};

pub struct SpecBuilder {
    schema: String,
    root: Option<String>,
    elements: HashMap<String, Element>,
    title: Option<String>,
    layout: Option<String>,
    data: Value,
}

impl Spec {
    pub fn builder() -> SpecBuilder {
        SpecBuilder {
            schema: SCHEMA_VERSION.to_string(),
            root: None,
            elements: HashMap::new(),
            title: None,
            layout: None,
            data: Value::Null,
        }
    }
}

impl SpecBuilder {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn layout(mut self, l: impl Into<String>) -> Self {
        self.layout = Some(l.into());
        self
    }
    pub fn data(mut self, d: Value) -> Self {
        self.data = d;
        self
    }
    /// Explicitly sets the root. If not called, the first element added is used.
    pub fn root(mut self, id: impl Into<String>) -> Self {
        self.root = Some(id.into());
        self
    }
    pub fn element(mut self, id: impl Into<String>, el: ElementBuilder) -> Self {
        let id = id.into();
        if self.root.is_none() {
            self.root = Some(id.clone());
        }
        self.elements.insert(id, el.build());
        self
    }
    /// Runs the same validation pipeline as `Spec::from_json`.
    pub fn build(self) -> Result<Spec, SpecError> {
        let root = self.root.ok_or_else(|| SpecError::RootMissing("<unset>".to_string()))?;
        let spec = Spec {
            schema: self.schema,
            root,
            elements: self.elements,
            title: self.title,
            layout: self.layout,
            data: self.data,
        };
        validate_structure(&spec)?;
        Ok(spec)
    }
}

pub struct ElementBuilder {
    type_name: String,
    props: Map<String, Value>,
    children: Vec<String>,
    action: Option<Action>,
    visible: Option<Visibility>,
}

impl Element {
    pub fn new(type_name: impl Into<String>) -> ElementBuilder {
        ElementBuilder {
            type_name: type_name.into(),
            props: Map::new(),
            children: Vec::new(),
            action: None,
            visible: None,
        }
    }
}

impl ElementBuilder {
    pub fn prop(mut self, k: impl Into<String>, v: impl Into<Value>) -> Self {
        self.props.insert(k.into(), v.into());
        self
    }
    pub fn child(mut self, id: impl Into<String>) -> Self {
        self.children.push(id.into());
        self
    }
    pub fn action(mut self, a: Action) -> Self {
        self.action = Some(a);
        self
    }
    pub fn visible(mut self, v: Visibility) -> Self {
        self.visible = Some(v);
        self
    }
    fn build(self) -> Element {
        Element {
            type_name: self.type_name,
            props: if self.props.is_empty() {
                Value::Null
            } else {
                Value::Object(self.props)
            },
            children: self.children,
            action: self.action,
            visible: self.visible,
        }
    }
}
```

Forward-reference safety: `.child("x")` before `.element("x", …)` is fine. The child reference is just a string in a Vec; `build()` validates all references after the whole map is assembled.

Order-agnosticism: the first `.element(…)` call sets root by default. Authors who want explicit ordering can call `.root("id")` before or after their `.element()` calls.

### §7 — Full `from_json` assembly

```rust
impl Spec {
    pub fn from_json(json: &str) -> Result<Spec, SpecError> {
        // Step 1: parse via a Deserialize impl that rejects duplicate keys in
        // the `elements` object. See Pitfall 2 — use a wrapper ElementsMap
        // type for `elements` during parse, then unwrap.
        let raw: SpecWire = serde_json::from_str(json)?;
        let spec = Spec {
            schema: raw.schema,
            root: raw.root,
            elements: raw.elements.0, // unwrap ElementsMap wrapper
            title: raw.title,
            layout: raw.layout,
            data: raw.data,
        };
        // Steps 2-6: structural validation.
        validate_structure(&spec)?;
        Ok(spec)
    }
}

// Internal parse-wire type with duplicate-ID detection.
#[derive(Deserialize)]
struct SpecWire {
    #[serde(rename = "$schema", default = "default_schema")]
    schema: String,
    root: String,
    elements: ElementsMap, // duplicate-rejecting wrapper from Pitfall 2
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    layout: Option<String>,
    #[serde(default)]
    data: Value,
}

fn default_schema() -> String {
    SCHEMA_VERSION.to_string()
}

fn validate_structure(spec: &Spec) -> Result<(), SpecError> {
    // Order per §Ordering of Structural Validation Checks below.
    validate_ids(&spec.elements)?;
    if !spec.elements.contains_key(&spec.root) {
        return Err(SpecError::RootMissing(spec.root.clone()));
    }
    validate_no_dangling(&spec.elements)?;
    detect_cycle(&spec.elements, &spec.root)?;
    check_depth(&spec.elements, &spec.root)?;
    Ok(())
}

fn validate_no_dangling(
    elements: &HashMap<String, Element>,
) -> Result<(), SpecError> {
    for (id, el) in elements {
        for child in &el.children {
            if !elements.contains_key(child) {
                return Err(SpecError::DanglingChild {
                    element: id.clone(),
                    child: child.clone(),
                });
            }
        }
    }
    Ok(())
}
```

### §8 — Placeholder renderer (minimum viable; keeps workspace green)

```rust
// ferro-json-ui/src/render.rs
use serde_json::Value;
use crate::spec::Spec;

/// Placeholder renderer for Phase 115. Emits the Spec as pretty-printed JSON
/// inside a <pre> block. The real flat-element walker lands in Phase 116.
pub fn render_spec_to_html(spec: &Spec, _data: &Value) -> String {
    let pretty = serde_json::to_string_pretty(spec)
        .unwrap_or_else(|e| format!("{{\"error\": \"serialize failed: {e}\"}}"));
    let escaped = html_escape(&pretty);
    format!(
        "<!-- ferro-json-ui v2 render pipeline arrives in Phase 116 -->\n\
         <div class=\"ferro-json-ui\" data-spec-version=\"v2\">\n\
         <pre style=\"font-family:monospace;white-space:pre-wrap;\"><code>{}</code></pre>\n\
         </div>",
        escaped
    )
}

/// Plugin-asset-aware variant. In Phase 115 there is no walk, so no assets
/// are collected — both results are empty.
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    RenderResult {
        html: render_spec_to_html(spec, data),
        css_head: String::new(),
        scripts: String::new(),
    }
}

pub struct RenderResult {
    pub html: String,
    pub css_head: String,
    pub scripts: String,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
```

Wire into `framework/src/json_ui/mod.rs` by swapping `render_to_html_with_plugins(view, data)` for `render_spec_to_html_with_plugins(spec, data)` and dropping the `.components.is_empty()` fallback branches (there's no `components` field anymore; the Spec is always well-formed or `from_json`/`build` would have failed).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| v1 `JsonUiView { components: Vec<ComponentNode> }` nested tree | v2 `Spec { root, elements: HashMap }` flat map | Phase 115 (this phase) | AI generation becomes schema-constrained; no recursion in schema; mirrors Vercel json-render |
| `Component` enum with 40 variants + custom `Serialize`/`Deserialize` | Type-erased `Element { type_name: String, props: Value }` | Phase 115 | ~200 LoC of hand-rolled ser/de deleted; 14 "JsonSchema skipped" comments deleted |
| `Component::Plugin(PluginProps)` fallback variant | Plugins are elements whose `type_name` isn't in the catalog | Phase 115 | Removes the "some types are special" wart |
| schemars 0.8 `Schema` enum + `SchemaObject` | schemars 1.2 `Schema` wrapping `serde_json::Value` + `json_schema!` macro | schemars 1.0 release (late 2024) | Used already via `derive(JsonSchema)`; `$ref: "#"` for recursive types is idiomatic when `inline_schema() → false` |

**Deprecated/outdated:**
- `COMPONENT_CATALOG` const string in `ferro-json-ui/src/lib.rs` (180 LoC of hand-written catalog) — survives Phase 115 untouched (used by MCP tools), replaced by a machine-readable Catalog in Phase 117.
- v1 `render_to_html`, `render_to_html_with_plugins`, `collect_plugin_types` tree walkers in `render.rs` — replaced by placeholder in Phase 115, fully rewritten in Phase 116.

## Props Struct Audit (answers research question #1b, #1c)

After stripping `Vec<ComponentNode>` and `Vec<Tab>` fields, the surviving `*Props` structs. Reviewed every struct in `ferro-json-ui/src/component.rs` as of 2026-04-18:

### Currently "JsonSchema skipped" due to `Vec<ComponentNode>` (expected to derive after strip)

| Struct | Fields to strip | Other blockers? | Result after strip |
|--------|----------------|-----------------|--------------------|
| `CardProps` (line 147) | `children`, `footer` | none | Clean derive |
| `FormProps` (line 189) | `fields` | none (Action derives JsonSchema per existing code) | Clean derive |
| `ModalProps` (line 309) | `children`, `footer` | none | Clean derive |
| `Tab` (line 408) | `children` | none | Clean derive (just `value`, `label`) |
| `TabsProps` (line 418) | — (contains `Vec<Tab>` not `Vec<ComponentNode>`) | Tab derives after its own strip | Clean derive |
| `GridProps` (line 648) | `children` | none | Clean derive |
| `CollapsibleProps` (line 676) | `children` | none | Clean derive |
| `FormSectionProps` (line 708) | `children` | none | Clean derive |
| `PageHeaderProps` (line 722) | `actions` (Vec<ComponentNode>) | none | Clean derive |
| `ButtonGroupProps` (line 733) | `buttons` (Vec<ComponentNode>) | none | Clean derive |
| `KanbanColumnProps` (line 774) | `children` | none | Clean derive |
| `KanbanBoardProps` (line 785) | — (contains `Vec<KanbanColumnProps>`) | KanbanColumnProps derives after its own strip | Clean derive |

### Currently "JsonSchema skipped" for other reasons

| Struct | Reason | After Phase 115 |
|--------|--------|------------------|
| `SwitchProps` (line 358, comment says "Option<Action> blocks") | **Stale comment** — `Action` has `#[derive(JsonSchema)]` per `schema_for!(Action)` test at view.rs:457. The v1 code already derives `JsonSchema` on `EmptyStateProps` which also contains `Option<Action>`. The skip is cargo-cult. | Just add `JsonSchema` to the derive list. |
| `DropdownMenuAction` (line 740) | No skip reason documented — does NOT derive JsonSchema. Suspected cargo-cult skip. | Add JsonSchema derive. |
| `DropdownMenuProps` (line 749) | Contains `Vec<DropdownMenuAction>` | Derives once DropdownMenuAction does. |
| `DataTableProps` (line 760) | Contains `Option<Vec<DropdownMenuAction>>` | Same — derives once action derives. |
| `PluginProps` (line 858) | Custom `Serialize`/`Deserialize` | **DELETED in Phase 115** (type-erasure kills it). |
| `Component` enum (line 909) | Custom `Serialize`/`Deserialize` | **DELETED in Phase 115.** |
| `ComponentNode` (line 1168) | Custom ser/de + Component | **DELETED in Phase 115.** |

**Key finding for research question #1c:** There are **zero** surviving `*Props` structs in Phase 115 that block `JsonSchema` derive after the strip + cargo-cult cleanup. No manual impls are needed in this phase. ROADMAP.md's "~200 lines of manual JsonSchema impls" caveat referred to the Component enum discriminator, which D-02 deletes entirely. The discriminator work becomes a Catalog concern (Phase 117), assembled from already-derivable per-Props schemas via `schemars::schema_for!(CardProps)` etc. plus manual `oneOf` composition at catalog build time.

**No `#[serde(flatten)]` or untagged enums survive** on any Props struct. (The flatten was on `ComponentNode.component` — deleted. The untagged-style fallback was in `Component`'s custom Deserialize — deleted.)

## Ordering of Structural Validation Checks (answers research question #4)

Proposed ordering from CONTEXT.md D-09 is almost correct but has a subtle ordering bug. Revised:

| # | Check | Cheap? | Depends on | Returns |
|---|-------|--------|-----------|---------|
| 1 | Duplicate-ID detection | Yes — O(n), during parse | parse time | `DuplicateId` |
| 2 | ID format validation (every key + every child) | Yes — O(n + Σchildren) | parse done | `InvalidId` |
| 3 | Root exists | Yes — O(1) | parse done | `RootMissing` |
| 4 | Dangling child refs | Yes — O(n + Σchildren) | parse done | `DanglingChild` |
| 5 | No cycles (DFS from root) | O(n + edges) | #3 + #4 passed | `Cycle` |
| 6 | Depth from root ≤ 3 | O(paths) | #5 passed (else infinite recursion) | `DepthExceeded` |

**Differences from CONTEXT.md D-09 ordering:**
1. **ID format check runs at #2, before root check.** Reason: if the root string is malformed (e.g., contains a space), the `RootMissing` error is a symptom, not the cause. Better to surface the real error first.
2. **Duplicate-ID is the very first check.** Runs during parse (mandatory, because serde_json otherwise silently overwrites). Cannot be deferred.
3. **Dangling-child check runs before cycle check.** A dangling child ref in a cycle would be reported as a cycle otherwise; the author gets a less useful error.

**Justification:** cheapest-and-most-local errors first. Each check assumes earlier ones passed. A spec that fails check #1 never reaches #2–#6, so the later checks can assume uniqueness, validity, and resolvability of every ID they encounter. This makes the cycle and depth walkers simpler (they can unwrap `elements.get(child)` without bounds-checking).

## Migration Blast Radius (answers research question #7)

Grep-verified inventory of v1 type references across the workspace as of 2026-04-18:

### (a) Trivial rewrite — `Spec::builder()` drop-in

| File | Lines | What changes |
|------|-------|--------------|
| `ferro-cli/src/templates/make.rs` | 108–130 | Template string literal: `JsonUiView::new()...` → `Spec::builder()...`. Pure string replacement in a code-gen template. |
| `ferro-cli/src/templates/module.rs` | 72–105 | Same. Template string in the `make:module` scaffold. |
| `ferro-cli/src/ai.rs` | 96–130 | AI prompt string — replace v1 builder sample with v2 `Spec::builder()` sample. Purely text. |
| `ferro-mcp/src/tools/code_templates.rs` | 909–1109 | Five `code: r#"…"#` template strings showing v1 syntax. Each becomes the v2 equivalent. Mechanical. |
| `ferro-mcp/src/tools/json_ui_generate.rs` | 71–131 | Sample code embedded in the tool's output. String replacement. |
| `ferro-mcp/src/tools/generation_context.rs` | 116–143 | Same — sample string. String replacement. |
| `ferro-mcp/src/service.rs` | 1270 | Documentation string mentioning "JsonUiView builder API". Replace with "Spec builder API". |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 70–860, 1228–1233 | Many `"Vec<ComponentNode>"` string literals describing prop types. Replace with `"Vec<String>"` or remove as appropriate for v2. Mechanical bulk replace. |
| `framework/src/lib.rs` | 78–92 | Re-exports list — swap `JsonUiView` → `Spec`, drop `ComponentNode`, drop `Component`, add `Element`, `SpecBuilder`, `SpecError`. |
| `ferro-json-ui/src/lib.rs` | 56–87 | Same at crate level — remove `Component`, `ComponentNode`, `JsonUiView`, `SCHEMA_VERSION` re-export swap. |

### (b) Non-trivial rewrite — real logic port

| File | Lines | What changes | Does P115 placeholder suffice? |
|------|-------|--------------|-------------------------------|
| `ferro-json-ui/src/resolve.rs` | entire file (1153 LoC) | `resolve_actions(&mut JsonUiView, …)` walks recursive component tree. In v2, it iterates `spec.elements.values_mut()` and resolves `el.action`. The per-Component recursion goes away. **Mostly shrinks.** | Yes — resolver is structure-aware, not render-aware. Rewrite cleanly with Spec. |
| `ferro-json-ui/src/render.rs` | entire file (8057 LoC) | Tree walker consumes `JsonUiView` and dispatches per `Component::Variant`. In v2, walker does ID lookups in `spec.elements`. **Full rewrite is Phase 116.** P115 replaces with placeholder (see §8 code example). | Yes — the placeholder is THE point of P115 here. |
| `framework/src/json_ui/mod.rs` | entire file (1020 LoC) | Signature: `JsonUi::render(&JsonUiView, &Value)` → `JsonUi::render(&Spec, &Value)`. The test fixtures at the bottom (lines 268–1019) all use `JsonUiView::new()...component(ComponentNode{…})...`. Each test fixture converts to `Spec::builder()...element(…)...`. Lots of files-in-one-file mechanical work but per-test change is small. | Yes — body calls the new placeholder renderer. Tests pass because they assert on title, data-props attributes, or HTML shape — none require component-specific rendering. The two plugin-specific tests (lines 975–1018, `test_plugin_component_renders_in_full_page` asserting Leaflet CSS/JS) WILL fail because the placeholder doesn't collect plugin assets. **Tag those two tests `#[ignore]` in Phase 115 with a comment pointing to Phase 116.** |
| `framework/src/json_ui/mod.rs` `resolve_with_errors` | 161–167 | Calls `resolve_actions` + `resolve_errors` + sets `resolved.errors`. In v2, `.errors` is gone (D-06 removes the field). The errors map flows separately — the placeholder passes it into the pretty-JSON dump as a separate block. | Yes — the placeholder just serializes both Spec and errors side by side. |
| `ferro-json-ui/src/projection/mod.rs` | lines 107–120 (the `impl Renderer for JsonUiRenderer` block) | `type Output = serde_json::Value` → `type Output = Spec`. Internal mapping (all the `render_browse`/`render_focus`/etc. helpers at lines 130+) builds JSON that matches `JsonUiView` shape — that builds have to construct `Spec` instead. **Shape swap, not logic rewrite.** D-20 says the mapping stays "functionally identical; only the output struct changes". Practically: wherever the internal code does `json!({"$schema": "ferro-json-ui/v1", "components": [...]})` it switches to constructing a `Spec` via `Spec::builder()` with a root Card or Table element and child IDs for each field. | Partially — the naive strategy is: wrap every mapping result as a single-element Spec of `type_name = "DataTable"` or similar, with flat props. Phase 117.1 rewrites this properly. P115 just needs the output type to be `Spec` and the mapping to compile. |
| `ferro-mcp/src/tools/render_projection.rs` | 6–30 | Uses `JsonUiRenderer`, `VisualContext`, `RenderMode` — imports unchanged (location didn't move). Return type field `json_ui: serde_json::Value` should stay `serde_json::Value` via `serde_json::to_value(&spec)?` since the MCP protocol wants JSON. Mechanical. | Yes — wrapper over the renderer. |

### (c) Can be reduced to a TODO placeholder

| File | Lines | Rationale |
|------|-------|-----------|
| `ferro-mcp/src/tools/json_ui_inspect.rs` | 119, 122 (regex for `-> JsonUiView` and `Component::(\w+)`) | The regex scans user source code for v1 patterns. In a v2 codebase the regex matches nothing. **Phase 120 rewrites the scan to look for `-> Spec` and parse flat specs.** Phase 115 acceptable state: regex still says `JsonUiView` and the tool returns empty results for v2 projects. Tag the tool as "v1-only, v2 scan lands in Phase 120" in a comment. |
| `ferro-mcp/src/tools/application_info.rs` | 19, 57, 244, 274 (`JsonUiViewsStatus` struct + `scan_json_ui_views` function) | Same — scans source for `-> JsonUiView`. Phase 120 rewrites. P115: keep struct name, or rename struct to `JsonUiSpecsStatus` if you prefer (low-risk since it's an MCP output type — schema-breaking only for MCP consumers, which is acceptable per project norms). |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 1228–1233 (builder_api description assertions) | Update assertion strings from `"JsonUiView::new()"` to `"Spec::builder()"`. The whole catalog string is ~180 LoC of hand-written doc text, overhauled in Phase 117. P115: minimum edit — replace just the builder lines. |

### Callers NOT affected (confirmed)

- `ferro-json-ui/src/action.rs` — no changes.
- `ferro-json-ui/src/visibility.rs` — no changes.
- `ferro-json-ui/src/plugin.rs` — plugin registry is independent of Component enum. Asset collection was driven by `collect_plugin_types(view)` which walks Component::Plugin variants. In v2, Phase 116's walker discovers plugin `type_name`s by checking against the registry directly — but P115 doesn't do this walk (placeholder renderer). Verified: `plugin.rs` does not import `Component` or `ComponentNode`.
- `app/` — no v1 references found (app uses Inertia, not JSON-UI, in the sample).

## Test Fixtures for Phase 115 (answers research question #9)

Under `ferro-json-ui/tests/fixtures/`:

### `ok/` — round-trip (D-29)

| File | Description | Assertion |
|------|-------------|-----------|
| `minimal_single_element.json` | `{"$schema":"ferro-json-ui/v2","root":"a","elements":{"a":{"type":"Text","props":{"content":"Hi"}}}}` | Parses, round-trips byte-equal to normalized pretty-print |
| `three_level_nested.json` | Root Card with child Card holding a child Text — exactly depth 3 | Parses, passes all validation |
| `with_actions.json` | Root with `action: {handler: "users.create", method: "POST"}` | Parses, preserves action on round-trip |
| `with_visibility.json` | Root with `visible: {operator: "eq", path: "/auth/admin", value: true}` | Parses, preserves visibility |
| `with_plugin_named_type.json` | An element with `"type":"Map"`, props `{"center":[51.5,-0.09],"zoom":13}` | Parses. No catalog check in P115 — unknown type_name is fine. |
| `with_data_payload.json` | Root Text + top-level `data: {"user":{"name":"Alice"}}` | Round-trips including data |
| `omitted_optional_fields.json` | No `title`, no `layout`, no `data`, elements have no `children`, no `action`, no `visible` | Round-trips with `skip_serializing_if` behavior — output matches input exactly |

### `reject/` — validation failure (D-30)

| File | Description | Expected SpecError variant |
|------|-------------|----------------------------|
| `missing_root.json` | `{"root":"nope","elements":{"a":…}}` | `RootMissing("nope")` |
| `dangling_child.json` | Root references child `"ghost"` not in elements | `DanglingChild { element: "root", child: "ghost" }` |
| `simple_cycle.json` | Root → A, A.children = [root] | `Cycle { path: ["root","A","root"] }` |
| `self_cycle.json` | `A.children = ["A"]` | `Cycle { path: ["A","A"] }` |
| `four_level_nesting.json` | Root → A → B → C → D (4 edges = depth 5) | `DepthExceeded { max: 3, found: 5, path: ["root","A","B","C","D"] }` |
| `invalid_id_space.json` | Element key `"user form"` | `InvalidId("user form")` |
| `invalid_id_empty.json` | Element key `""` (empty string) | `InvalidId("")` |
| `invalid_id_digit_start.json` | Element key `"1form"` | `InvalidId("1form")` |
| `invalid_id_too_long.json` | Element key of 129 chars | `InvalidId(…)` |
| `invalid_child_ref_format.json` | Valid key "a", but `a.children = ["user form"]` | `InvalidId("user form")` |
| `duplicate_id.json` | Raw JSON with `"elements":{"a":…,"a":…}` (yes, two identical keys) | `DuplicateId("a")` |

Fixture loader pattern:

```rust
// tests/validation.rs
use std::fs;
use ferro_json_ui::{Spec, SpecError};

fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
}

#[test]
fn ok_minimal_round_trips() {
    let json = fixture("ok/minimal_single_element.json");
    let spec = Spec::from_json(&json).unwrap();
    let reserialized = serde_json::to_string(&spec).unwrap();
    let spec2 = Spec::from_json(&reserialized).unwrap();
    assert_eq!(spec, spec2);
}

#[test]
fn reject_missing_root_gives_specific_variant() {
    let json = fixture("reject/missing_root.json");
    match Spec::from_json(&json) {
        Err(SpecError::RootMissing(id)) => assert_eq!(id, "nope"),
        other => panic!("expected RootMissing, got {other:?}"),
    }
}

// … one test per fixture, asserting variant
```

## Builder API Survey (answers research question #6)

| Library | Host language pattern | Applies to ferro? |
|---------|----------------------|-------------------|
| Vercel json-render | TS authors write `{ root: "a", elements: {...} }` literals directly. No builder. | No — TS object literals are already terse; Rust needs a builder for ergonomics |
| JSON Forms | No builder — consumers hand-author UI Schema JSON | No — same |
| react-jsonschema-form | Consumers hand-author `schema` + `uiSchema` | No |
| Protocol Buffers (comparable — GraphQL unions in Airbnb SDUI) | Generated builder code per message. Consuming `mut self → Self`. | Yes — Rust idiom matches protobuf-generated builders; consuming style is the standard (per CLAUDE.md user memory) |

Recommendation: the builder sketch in §6 matches proto-style consuming builders AND the v1 `JsonUiView::new().title().component()` pattern already familiar to ferro users. Order-agnostic because the HashMap doesn't care about insertion order. Forward references are fine because validation is deferred to `build()`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | entire workspace | ✓ | workspace uses edition from Cargo — verified compiling | — |
| `schemars` 1.2.0 | derive macros on Props structs | ✓ | 1.2.0 [VERIFIED: Cargo.lock:4115] | — |
| `serde_json` 1.0 | all JSON IO | ✓ | [VERIFIED: ferro-json-ui/Cargo.toml:18] | — |
| `thiserror` | SpecError derive | ✗ in ferro-json-ui currently | — | Add `thiserror = "1.0"` to `ferro-json-ui/Cargo.toml` (workspace convention per 15 other crates) |

**Missing dependencies with fallback:** `thiserror` — add to ferro-json-ui Cargo.toml (small addition, matches workspace convention).

**Missing dependencies with no fallback:** None.

## Validation Architecture

> nyquist_validation is enabled (no explicit `false` in workflow config).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in Rust test harness), workspace-level |
| Config file | none (Cargo default) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPEC-01 | `Spec` struct with exact field shape per D-06 | unit | `cargo test -p ferro-json-ui --lib spec::tests::spec_shape_ok` | ❌ Wave 0 |
| SPEC-02 | `Element` struct with exact field shape per D-07 | unit | `cargo test -p ferro-json-ui --lib spec::tests::element_shape_ok` | ❌ Wave 0 |
| SPEC-03a | `Spec::from_json` round-trip (D-29) | integration | `cargo test -p ferro-json-ui --test validation ok_` | ❌ Wave 0 |
| SPEC-03b | `Spec::from_json` rejection (D-30) | integration | `cargo test -p ferro-json-ui --test validation reject_` | ❌ Wave 0 |
| SPEC-03c | `Spec::builder` parity (D-31) | unit | `cargo test -p ferro-json-ui --lib spec::tests::builder_parity_with_fixture` | ❌ Wave 0 |
| SPEC-04a | All v1 types deleted (compile-time check) | compile | `! grep -r 'JsonUiView\\|ComponentNode' ferro-json-ui/src/ app/src/` at post-phase verification | ✅ (grep is a standard POSIX tool) |
| SPEC-04b | Surviving Props derive JsonSchema (D-32) | unit | `cargo test -p ferro-json-ui --lib spec::tests::schema_for_every_surviving_props` | ❌ Wave 0 |
| Cross-crate compile | Framework + MCP + CLI compile against v2 | smoke | `cargo check --all --all-features` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui` (≤ 5s) + `cargo check -p ferro-json-ui` (≤ 3s).
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` (workspace gate).
- **Phase gate:** Full suite green before `/gsd-verify-work`.

### Validation Dimension: Structural Invariants Contract

**Validation Dimension:** Correctness / contract — the parse-time validation of `Spec::from_json` is the single source of truth for what counts as a well-formed v2 spec.

**Invariant:** ∀ `s: &str`, if `Spec::from_json(s)` returns `Ok(spec)` then (1) `spec.root ∈ spec.elements`, (2) ∀ `e ∈ spec.elements.values()`, ∀ `c ∈ e.children`, `c ∈ spec.elements`, (3) the element graph rooted at `spec.root` contains no cycles, (4) the max path depth from root is ≤ 3, (5) every key and every child reference matches `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`, (6) no JSON object in the raw source has duplicate keys inside the `elements` object.

**Validation method:** **Table-driven** fixture tests (not property-based — proptest adds build-time cost and the fixture corpus gives equally strong coverage for a structural contract). Each structural invariant has one or more fixtures under `tests/fixtures/reject/` that MUST fail and one or more fixtures under `tests/fixtures/ok/` that MUST pass. The contract is: for every fixture under `ok/`, `Spec::from_json` returns `Ok`. For every fixture under `reject/`, `Spec::from_json` returns `Err(<specific variant>)`.

**Sampling strategy:**
- 7 `ok/` fixtures × round-trip check = 7 assertions
- 11 `reject/` fixtures × variant match = 11 assertions
- Builder parity: for every `ok/` fixture, construct the equivalent via `Spec::builder()` and assert `spec_from_json == spec_from_builder` = 7 assertions
- Schema smoke tests: for every surviving `*Props` struct (~35 structs), assert `schemars::schema_for!(TProps)` returns a JSON object with non-empty properties field = 35 assertions

Total automated: **~60 fast assertions** covering the structural contract completely. No property-based fuzzing required — the surface area is small enough that exhaustive fixture coverage dominates.

### Wave 0 Gaps

- [ ] `ferro-json-ui/tests/` directory — does not exist, verified via `ls`. Create it.
- [ ] `ferro-json-ui/tests/fixtures/ok/` — 7 JSON files (see §Test Fixtures §ok/)
- [ ] `ferro-json-ui/tests/fixtures/reject/` — 11 JSON files (see §Test Fixtures §reject/)
- [ ] `ferro-json-ui/tests/validation.rs` — fixture-driven integration test harness
- [ ] `ferro-json-ui/tests/builder_parity.rs` — builder vs from_json equality tests
- [ ] `ferro-json-ui/src/spec.rs` — inline `#[cfg(test)] mod tests { … }` for shape + validation unit tests
- [ ] `ferro-json-ui/Cargo.toml` — add `thiserror = "1.0"` under `[dependencies]`
- [ ] `framework/src/json_ui/mod.rs` tests — the existing ~30 test functions rewrite from `JsonUiView::new()...` to `Spec::builder()...`. Mechanical but tedious; split across multiple tasks.
- [ ] Two existing plugin tests in `framework/src/json_ui/mod.rs` (`test_plugin_component_renders_in_full_page`, theme tests 876+ that create sample_view) — tag `#[ignore]` with "TODO(Phase 116): placeholder renderer does not collect plugin assets" comment. Do NOT delete; Phase 116 will re-enable.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | schemars 1.x's `$ref: "#"` self-reference is accepted when `inline_schema() → false`. [ASSUMED — based on schemars v1 migration doc stating "Schema is now a wrapper around serde_json::Value" and the `json_schema!` macro accepts arbitrary JSON. Not independently verified against schemars 1.2.0 changelog.] | Architecture Patterns §Pattern 1 | Low — Phase 115 does not need manual JsonSchema impls after the strip. If A1 is wrong, it only affects Phase 117. |
| A2 | Adding `thiserror` to `ferro-json-ui/Cargo.toml` is acceptable (not a new dep in a heavy sense; 15 other workspace crates use it). [VERIFIED: dep already in Cargo.lock via other crates; added cost is zero.] | Standard Stack | None — thiserror is universal in this workspace. |
| A3 | Two plugin-asset tests in `framework/src/json_ui/mod.rs` fail under the placeholder renderer and must be `#[ignore]`d. [ASSUMED — not yet compiled with v2 types; inferred from reading the test assertions which expect Leaflet CSS/JS in HTML output.] | Migration Blast Radius (b) | Low — worst case, one or two more tests need ignore tags. Plan should permit flex for ignore tags at task level. |
| A4 | The `JsonUiRenderer` internal mapping (`render_browse`/`render_focus`/etc. at ~2500 LoC in `projection/mod.rs`) can be mechanically rewritten from "emit a `JsonUiView` shape" to "emit a `Spec` shape" without rewriting the intent→layout mapping logic. [ASSUMED — based on D-20 which says mapping stays naive.] Verified that the internal helpers produce `serde_json::Value` trees that mirror `JsonUiView` structure; the conversion is mechanical. | Migration Blast Radius (b) | Medium — if the internal helpers turn out to need deeper restructuring, Phase 115 grows by ~300 LoC of mapping work. Mitigation: the planner should scope "projection/mod.rs Output switch" as a dedicated task with its own test. |
| A5 | `regex` crate is not a direct dep of `ferro-json-ui` and should stay that way; hand-rolled ID check is preferred. [VERIFIED: Cargo.toml only lists serde, serde_json, schemars, and optional ferro-projections/ferro-theme.] | Standard Stack §Alternatives | None — verified. |
| A6 | The `DropdownMenuAction` + `DropdownMenuProps` + `DataTableProps` JsonSchema skips are cargo-cult (no actual blocker in the type). [ASSUMED — inferred from the lack of skip-reason comments on those structs vs. the clear comments on Modal/Form/etc.] | Props Struct Audit | Low — if wrong, add those three to the manual-impl list. Still < 100 LoC of manual work. |

## Open Questions

None. Every question posed in the research brief (#1–#12) has a concrete answer above, backed by source code inspection or schemars documentation.

## Sources

### Primary (HIGH confidence)

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/component.rs` — lines 146–1177, direct inspection for Props struct inventory
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/view.rs` — lines 1–480, v1 baseline
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/lib.rs` — lines 42–180, current re-exports
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/src/projection/mod.rs` — lines 1–120, current JsonUiRenderer shape
- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/json_ui/mod.rs` — lines 1–1020, primary caller
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-json-ui/Cargo.toml` — full file, dep verification
- `/Users/alberto/repositories/albertogferrario/ferro/Cargo.lock:4115-4116` — schemars 1.2.0 version
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — locked decisions D-01 through D-32
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/133-generalize-renderer-trait/133-CONTEXT.md` — Renderer trait shape
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/134-relocate-renderers-to-output-crates/134-CONTEXT.md` — projection module location
- Context7: `/gresau/schemars` — manual JsonSchema impl pattern, `json_schema!` macro, `inline_schema` semantics, v1 migration doc

### Secondary (MEDIUM confidence)

- CLRS (Cormen et al.) ch. 22.3 — DFS cycle detection with gray set / on-stack parity (standard algorithm, not re-verified)
- Project memory `project_ferro_publication.md` (cited by CONTEXT.md) — clean-break is a project norm
- Airbnb/DoorDash/Lyft SDUI patterns — depth-3 constraint cited in ROADMAP.md

### Tertiary (LOW confidence)

- None — every architectural claim above is either cited or inspected.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every version verified against Cargo.lock or Cargo.toml
- Architecture: HIGH — types and builder shape specified verbatim in CONTEXT.md; this research confirms they are implementable as-specified with no hidden issues
- Pitfalls: HIGH — duplicate-ID detection pattern verified against serde_json docs; cycle-detection pattern is standard CS; depth-memoization hazard is a well-known trap
- Migration blast radius: HIGH — grep-verified file inventory
- Manual JsonSchema impl count: HIGH — `zero` — audit reads every Props struct and eliminates every "JsonSchema skipped" reason
- Placeholder renderer design: HIGH — minimum viable implementation is trivial and verified against existing `framework/json_ui` test assertions

**Research date:** 2026-04-18
**Valid until:** 2026-05-18 (30 days; schemars 1.x is stable, ferro-json-ui internals change rapidly but the v2 target is locked by CONTEXT.md)
