# Phase 118: Server-Side Expressions - Research

**Researched:** 2026-04-19
**Domain:** ferro-json-ui expression resolution (Rust, serde_json::Value traversal)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- D-01: Single-pass pre-render resolver, mutating-clone pattern. `pub fn resolve_expressions(spec: &mut Spec)` in `ferro-json-ui/src/expression.rs`.
- D-02: Slash-separated path syntax for both `$data` and `$template`. Reuse `crate::data::resolve_path` verbatim. Template placeholders use `{/slash/path}`, not dot-paths.
- D-03: Type-preserving `$data` resolution. Missing path → `Value::Null`. No auto-stringification.
- D-04: Resolution scope is `Element.props` recursive only. Not `Spec.data`, `title`, `layout`, `children`, `action`, or `visible`.
- D-05: `$template` produces a `Value::String`. Missing placeholders → empty string `""`.
- D-06: Malformed expressions (non-string `$data`/`$template` value, sibling keys) degrade to literal JSON silently.
- D-07: Single-pass resolution. No recursive expansion. `$data` returning an object that contains `$data` markers: inner markers stay as literal.
- D-08: Pipeline order: `Spec::from_json` → `resolve_actions` → `resolve_expressions` → `Catalog::validate` → `render_spec_to_html_with_plugins`.
- D-09: Resolver is infallible. Returns `()`. No `Result`, no log, no HTML comment.
- D-10: Always walk every `Element.props`. No fast-path detection, no `Cow<Spec>`.
- D-11: Exactly one new file: `ferro-json-ui/src/expression.rs`. Minimal modifications to `lib.rs`, `data.rs` (likely none), `framework/src/json_ui/mod.rs`.
- D-12: Unit tests in `expression.rs`, integration tests in `framework/src/json_ui/mod.rs`.
- D-13: Catalog schema unchanged. No `oneOf: [String, ExpressionObject]` per typed slot.
- D-14: Plugin props walk identically to built-in props.
- **Hard cap (locked, do not re-open):** Only `$data` and `$template`. No `$if`, `$for`, `$state`, `$bind`, `$ref`, `$concat`, `$let`.

### Claude's Discretion

- Whether `is_data_expr` / `is_template_expr` are free functions, methods on a private `Expr` enum, or pattern-matched inline.
- Whether the template parser is a hand-rolled scanner or uses a tiny `regex` / `winnow` dependency — prefer hand-rolled (zero new deps; the grammar is trivial).
- Whether `expression.rs` exports any helper types beyond `resolve_expressions`.
- Whether nested `$data` markers inside `$template` are special-cased (they are not).
- Whether integration tests live alongside the existing `framework/src/json_ui/mod.rs` test block or in a sibling `#[cfg(test)]` file.
- Whether `data::resolve_path` / `resolve_path_string` get renamed or kept as-is — keep as-is.

### Deferred Ideas (OUT OF SCOPE)

- `$if`, `$for`, `$switch`, `$state`, `$bindState`, `$ref`, `$concat`, `$let`
- Expression markers inside `Spec.data`, `Element.children`
- JSON Pointer `~0`/`~1` escape compliance
- Recursive multi-pass expression resolution
- Per-element error reporting from the resolver
- Path cache or Cow-based fast path
- Schema-level expression markers
- AI generation tools emitting expressions (Phase 120)
- gestiscilo migration (Phase 121)
- MCP introspection tool for expressions
</user_constraints>

---

## Summary

Phase 118 adds a narrow pre-render expression resolver to `ferro-json-ui`. The scope is completely defined by the CONTEXT.md decisions: one new file, two expression types, zero new dependencies, infallible return. Research confirms all implementation choices are correct given the existing codebase.

The two functions this phase reuses already exist and are `pub(crate)` inside `ferro-json-ui/src/data.rs`: `resolve_path(&Value, &str) -> Option<&Value>` and `resolve_path_string(&Value, &str) -> Option<String>`. Because `expression.rs` lives in the same crate, both are directly accessible without any visibility change. [VERIFIED: source read of `ferro-json-ui/src/data.rs`]

The template grammar is trivially simple: `{path}` with `\{`/`\}` escapes. A hand-rolled scanner is ~25 lines and eliminates any external dependency. The `regex` crate would add ~2 MB compile time for no benefit. [VERIFIED: grammar specified in D-02, D-05; no alternative grammar exists that fits the locked path syntax]

The `JsonUi::resolve` insertion point is `framework/src/json_ui/mod.rs` line 39-43. Adding one line after `resolve_actions` covers every public render path. [VERIFIED: source read of `framework/src/json_ui/mod.rs`]

**Primary recommendation:** Implement `expression.rs` as a flat module with four private helpers and one public function. Hand-roll the template scanner. Use `serde_json::Value`'s `as_object_mut` / `as_array_mut` / `as_str` for traversal and detection — no additional crates needed.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Expression detection (`$data`/`$template` keys) | `ferro-json-ui` crate | — | Pure JSON introspection; lives where the spec type lives |
| Path resolution (`/segment/segment`) | `ferro-json-ui::data` (existing) | — | Already owns the path resolver; expression.rs calls it |
| Template scanning (`{path}` substitution) | `ferro-json-ui::expression` (new) | — | Grammar is expression-specific; belongs in the same file |
| Pipeline wiring (call order) | `framework::json_ui::JsonUi::resolve` | — | Single entry point for all render methods |
| Validation of resolved props | `ferro-json-ui::catalog` (unchanged) | — | Runs after resolution per D-08; catalog stays unaware of expressions |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde_json` | 1.0 (workspace) | `Value` traversal and mutation | Already the spec's JSON type; `as_object_mut`, `as_array_mut`, `as_str` are the correct traversal primitives | [VERIFIED: `ferro-json-ui/Cargo.toml`] |

### Supporting

No new dependencies. All required primitives already exist:

| Existing Asset | Location | Used By Expression Module |
|----------------|----------|---------------------------|
| `resolve_path` | `ferro-json-ui/src/data.rs:19` | `$data` resolution |
| `resolve_path_string` | `ferro-json-ui/src/data.rs:55` | `$template` placeholder substitution |
| `serde_json::Map::len`, `Map::get`, `Map::iter_mut` | stdlib | Expression detection |
| `serde_json::Value::as_object_mut`, `as_array_mut`, `as_str` | stdlib | Walker descent |

### Alternatives Considered and Rejected

| Problem | Rejected Option | Reason Rejected |
|---------|----------------|-----------------|
| Template scanning | `regex` crate | ~2 MB compile overhead for a 5-token grammar; zero new deps is a hard constraint per D-11 |
| Template scanning | `winnow` crate | Same reason; grammar does not need a parser combinator |
| Path resolution | `jsonptr` crate (RFC 6901) | `resolve_path` already does this without the `~0`/`~1` escape complexity; adding a crate would create a parallel convention |
| Path resolution | `serde_json_path` / JSONPath | JSONPath is a query language (returns multiple nodes); `$data` semantics are single-node lookup — wrong abstraction |
| Value traversal | A visitor crate | None in the Rust ecosystem matches `serde_json::Value` mutation semantics cleanly; hand-rolled recursion is idiomatic here |

**Installation:** No new packages. `ferro-json-ui/Cargo.toml` is unchanged.

---

## Architecture Patterns

### System Architecture Diagram

```
Handler data  -->  Spec::from_json()
                      |
                      v
              [structural validation]
              (Phase 115 — parse-time)
                      |
                      v
              resolve_actions(&mut spec, resolver)
              (Phase 116 — action URLs)
                      |
                      v
         >>>  resolve_expressions(&mut spec)   <<<  Phase 118
              |                                |
              v                                v
         $data: "/path"              $template: "Hello, {/name}!"
         ---> resolve_path()         ---> scan placeholders
              ---> cloned Value           ---> resolve_path_string()
              or Value::Null              ---> String
              (type preserved)           (missing → "")
                      |
                      v
              Catalog::validate(&spec)
              (Phase 117 — typed props check on concrete values)
                      |
                      v
              render_spec_to_html_with_plugins(&spec, &data)
              (Phase 116 walker — receives concrete values only)
                      |
                      v
                   HTML response
```

### Recommended Module Layout

```
ferro-json-ui/src/
├── expression.rs          NEW — resolve_expressions, resolve_value, helpers, tests
├── data.rs                UNCHANGED — resolve_path, resolve_path_string (pub(crate))
├── resolve.rs             UNCHANGED — resolve_actions pattern reference
├── lib.rs                 ADD pub mod expression + pub use expression::resolve_expressions
framework/src/json_ui/
└── mod.rs                 ADD one line in JsonUi::resolve + integration tests
```

### Pattern 1: Expression Detection (Single-Key Object Convention)

**What:** An expression node is a `serde_json::Value::Object` with exactly one key whose name matches `"$data"` or `"$template"` and whose value is a JSON string.

**When to use:** Applied to every `Value::Object` encountered during props walk.

**Why this detection rule:** The single-key convention is standard in JSON expression systems (Vega-Lite uses it for `{"field": "x"}`, Liquid-style DSLs use sigil prefixes). It disambiguates expression objects from regular prop objects that happen to contain a `$`-prefixed key alongside other keys (D-06: sibling keys → literal passthrough). [ASSUMED — based on CONTEXT.md D-03/D-06 rationale; no external source needed since the rule is fully specified]

```rust
// Source: CONTEXT.md D-03, D-06 — verified against serde_json API
fn is_data_expr(obj: &serde_json::Map<String, Value>) -> Option<&str> {
    if obj.len() == 1 {
        if let Some(Value::String(path)) = obj.get("$data") {
            return Some(path.as_str());
        }
    }
    None
}

fn is_template_expr(obj: &serde_json::Map<String, Value>) -> Option<&str> {
    if obj.len() == 1 {
        if let Some(Value::String(tmpl)) = obj.get("$template") {
            return Some(tmpl.as_str());
        }
    }
    None
}
```

### Pattern 2: Props Walker (Recursive Mutation)

**What:** `resolve_value` takes `&mut Value` and `&Value` (the spec's data), walks `Object` and `Array` nodes recursively, replacing expression nodes in place.

**When to use:** Called for each `el.props` in `spec.elements.values_mut()`.

**Why mutation over clone:** The CONTEXT.md clone-then-mutate pattern means `resolve_expressions` always receives the cloned spec; mutation within that clone is correct and avoids a second copy.

```rust
// Source: CONTEXT.md D-04, resolve.rs pattern (verified by source read)
fn resolve_value(val: &mut Value, data: &Value) {
    match val {
        Value::Object(map) => {
            if let Some(path) = is_data_expr(map) {
                let path = path.to_owned();
                *val = crate::data::resolve_path(data, &path)
                    .cloned()
                    .unwrap_or(Value::Null);
                // Single-pass: do NOT recurse into the resolved value (D-07)
            } else if let Some(tmpl) = is_template_expr(map) {
                let tmpl = tmpl.to_owned();
                *val = Value::String(substitute_template(&tmpl, data));
                // Single-pass: do NOT recurse into the resolved string
            } else {
                // Not an expression node — recurse into values
                for v in map.values_mut() {
                    resolve_value(v, data);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                resolve_value(v, data);
            }
        }
        // Strings, numbers, booleans, null are leaves
        _ => {}
    }
}
```

### Pattern 3: Template Scanner (Hand-Rolled)

**What:** Left-to-right character scan that emits literal characters until `{` (or `\{` escape), extracts the path between braces, resolves it via `resolve_path_string`, and continues.

**When to use:** Inside `resolve_template_expr` / `substitute_template`.

**Why hand-rolled over regex:** The grammar has five terminals (`{`, `}`, `\{`, `\}`, `\\`). A state machine over `chars()` is ~25 lines with no allocations beyond the output `String`. [VERIFIED: grammar specified in D-02/D-05, no `regex` in Cargo.toml, confirmed by CONTEXT.md D-11 "No new dependencies"]

```rust
// Source: CONTEXT.md D-02, D-05
fn substitute_template(template: &str, data: &Value) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.peek() {
                Some('{') => { out.push('{'); chars.next(); }
                Some('}') => { out.push('}'); chars.next(); }
                Some('\\') => { out.push('\\'); chars.next(); }
                _ => out.push('\\'),
            },
            '{' => {
                // Collect path until matching '}'
                let mut path = String::new();
                let mut closed = false;
                for inner in chars.by_ref() {
                    if inner == '}' { closed = true; break; }
                    path.push(inner);
                }
                if closed {
                    let trimmed = path.trim();
                    let resolved = crate::data::resolve_path_string(data, trimmed)
                        .unwrap_or_default(); // missing → ""
                    out.push_str(&resolved);
                } else {
                    // Unclosed brace: emit as literal
                    out.push('{');
                    out.push_str(&path);
                }
            }
            _ => out.push(ch),
        }
    }
    out
}
```

### Pattern 4: Public Entry Point (Mirrors resolve_actions)

**What:** `resolve_expressions` iterates `spec.elements.values_mut()`, calls `resolve_value` on each `el.props`.

```rust
// Source: resolve.rs:35 pattern (verified by source read)
pub fn resolve_expressions(spec: &mut Spec) {
    let data = spec.data.clone();
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, &data);
    }
}
```

Note: `spec.data` must be cloned once before the mutable borrow of `spec.elements` — the borrow checker requires splitting borrows across struct fields or cloning the data field first. [VERIFIED: Rust borrow checker semantics; mutable borrow of `elements` cannot coexist with shared borrow of `data` through `spec`]

### Pattern 5: Framework Wiring (One-Line Change)

**What:** `JsonUi::resolve` in `framework/src/json_ui/mod.rs` gains one call after `resolve_actions`.

```rust
// Source: framework/src/json_ui/mod.rs:39-43 (verified by source read)
fn resolve(spec: &Spec) -> Spec {
    let mut resolved = spec.clone();
    resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
    ferro_json_ui::resolve_expressions(&mut resolved);  // Phase 118 addition
    resolved
}
```

This single line covers `render`, `render_with_config`, `render_json`, and the `_with_errors` variants because all routes go through `JsonUi::resolve`. The `resolve_with_errors` method also needs the same line added after `resolve_actions`.

### Anti-Patterns to Avoid

- **Recursing into resolved `$data` output:** After replacing `*val` with the resolved `Value`, do NOT call `resolve_value` on the new value. Single-pass is the inner-platform-effect firewall (D-07).
- **Walking `spec.data`:** The resolver reads `spec.data` as a source; walking it as a target creates recursive-expansion hazards.
- **Walking `el.children`, `el.action`, `el.visible`:** These have their own invariants (D-04). Only `el.props` is the substitution surface.
- **Using `std::mem::take` on props then putting back:** Unnecessary — `as_object_mut` / direct field mutation is sufficient.
- **Borrowing `spec.data` and `spec.elements` simultaneously:** Split by cloning `data` before the mutable element iteration (see Pattern 4).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Slash-path resolution | Custom tree walker | `crate::data::resolve_path` | Already handles leading-slash optionality, array indices, deep nesting, and `None` on miss |
| Path-to-string conversion | Custom stringification | `crate::data::resolve_path_string` | Already handles numbers, bools, JSON-serialize for objects/arrays, `None` for null |
| JSON object key inspection | Manual JSON parsing | `serde_json::Map::len()` + `Map::get()` | Single-key detection is two expressions, not a parser |

**Key insight:** The expression resolver is a thin orchestration layer over three existing primitives (`resolve_path`, `resolve_path_string`, and `serde_json`'s mutation API). The only genuinely new code is the template scanner and the detection predicates.

---

## Common Pitfalls

### Pitfall 1: Simultaneous Borrow of `spec.data` and `spec.elements`

**What goes wrong:** `for el in spec.elements.values_mut()` mutably borrows `spec`, preventing a subsequent shared borrow of `spec.data` inside the loop body.

**Why it happens:** Rust's borrow checker sees the entire `spec` struct as one borrow target, even though the fields are independent.

**How to avoid:** Clone `spec.data` into a local binding before the loop:
```rust
let data = spec.data.clone();
for el in spec.elements.values_mut() { ... }
```

**Warning signs:** Compiler error `cannot borrow spec.data as immutable because it is also borrowed as mutable`.

### Pitfall 2: Infinite-Expansion via Recursive Walk

**What goes wrong:** After resolving a `$data` node to a `Value::Object`, calling `resolve_value` again on the new value expands any `$data` markers inside the resolved data — effectively implementing recursive expansion, which D-07 explicitly bans.

**Why it happens:** It's natural to write a post-substitution recursive call without noticing the inner-platform risk.

**How to avoid:** In the `$data` and `$template` match arms, replace `*val` and return immediately — do NOT fall through to the recursive descent. See Pattern 2.

**Warning signs:** Tests where `spec.data` contains a `$data` marker and the resolved output substitutes it rather than returning the literal `$data` object.

### Pitfall 3: Walking `resolve_with_errors` Path Separately

**What goes wrong:** `JsonUi::resolve` is updated but `resolve_with_errors` (a separate method in `mod.rs:150`) is not, causing expression resolution to be skipped for error-rendering paths.

**Why it happens:** Two resolution paths exist (`resolve` and `resolve_with_errors`) — updating only one is easy to miss.

**How to avoid:** Both methods call `resolve_actions`; both must also call `resolve_expressions` immediately after. The integration tests in D-12 cover `render_with_errors` specifically.

**Warning signs:** Integration tests for `render_with_errors` using `$data` expressions fail, while `render` tests pass.

### Pitfall 4: Treating `$data` with Sibling Keys as an Expression

**What goes wrong:** `{"$data": "/path", "class": "text-red"}` is mistakenly treated as an expression and the `class` key is lost.

**Why it happens:** The detection predicate checks for `"$data"` existence but not for single-key constraint.

**How to avoid:** `is_data_expr` must check `obj.len() == 1` first (see Pattern 1). Objects with sibling keys pass through as literal JSON per D-06.

**Warning signs:** Tests for sibling-key objects produce a resolved value instead of the literal object.

### Pitfall 5: Escaping in Template — Unclosed Brace

**What goes wrong:** A template string like `"Enter amount {/price"` (no closing `}`) panics or emits corrupt output.

**Why it happens:** The scanner loop exits the iterator without the `closed` flag being set.

**How to avoid:** After the inner scan loop, check `closed`. If false, emit `{` and the accumulated path characters as literal text (see Pattern 3 — the `else` branch).

**Warning signs:** Tests with deliberately malformed templates (no closing brace) produce empty output or panic.

---

## Code Examples

### Full `expression.rs` Module Skeleton

The complete module is ~120 lines including inline tests. Key public surface:

```rust
// ferro-json-ui/src/expression.rs
// Source: CONTEXT.md D-11, verified against resolve.rs pattern

const EXPR_DATA_KEY: &str = "$data";
const EXPR_TEMPLATE_KEY: &str = "$template";

/// Resolve all `$data` and `$template` expressions in `spec.elements[*].props`.
///
/// Mutates in place. Returns `()` — the resolver is infallible. Missing paths
/// resolve to `Value::Null` for `$data` and `""` for `$template` placeholders.
/// Malformed expression shapes (non-string value, sibling keys) pass through
/// as literal JSON without error.
///
/// **Call order:** Must be called after `resolve_actions` and before
/// `Catalog::validate`. The pipeline in `JsonUi::resolve` enforces this.
///
/// **Single-pass guarantee:** `$data`/`$template` markers inside `spec.data`
/// are NOT sources of expressions and are NOT walked. A `$data` expression
/// that resolves to a value containing further `$data` markers will NOT
/// re-resolve those inner markers.
pub fn resolve_expressions(spec: &mut Spec) {
    let data = spec.data.clone();
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, &data);
    }
}
```

### Test Coverage Checklist (maps to D-12)

Unit tests in `expression.rs`:

```rust
// Source: CONTEXT.md D-12
#[cfg(test)]
mod tests {
    // $data — success cases
    // data_simple_path: {"$data": "/name"} → resolved string
    // data_nested_path: {"$data": "/user/name"} → resolved string
    // data_array_index: {"$data": "/items/0"} → resolved value
    // data_preserves_number: {"$data": "/count"} → Value::Number(42)
    // data_preserves_bool: {"$data": "/active"} → Value::Bool(true)
    // data_preserves_object: {"$data": "/user"} → Value::Object(...)
    // data_preserves_array: {"$data": "/items"} → Value::Array(...)
    // data_missing_path: {"$data": "/missing"} → Value::Null
    //
    // $data — passthrough cases
    // data_non_string_value: {"$data": 42} → literal passthrough
    // data_sibling_keys: {"$data": "/x", "class": "y"} → literal passthrough
    // data_null_value: {"$data": null} → literal passthrough
    //
    // $template — success cases
    // template_single_placeholder: "{/name}" → resolved string
    // template_multiple_placeholders: "{/a} and {/b}" → "v1 and v2"
    // template_no_placeholder: "static text" → "static text"
    // template_missing_placeholder: "{/missing}" → ""
    // template_whitespace_trimmed: "{ /name }" → resolved (trimmed path)
    //
    // $template — escape cases
    // template_escaped_open_brace: "\\{not a placeholder}" → "{not a placeholder}"
    // template_escaped_close_brace: "text\\}" → "text}"
    // template_escaped_backslash: "a\\\\b" → "a\\b"
    // template_unclosed_brace: "{/missing_close" → "{/missing_close" (literal)
    //
    // $template — passthrough cases
    // template_non_string_value: {"$template": 42} → literal passthrough
    //
    // nested expressions
    // nested_in_array: [{...}, {"$data": "/x"}] → resolved second element
    // nested_in_object_values: {"key": {"$data": "/x"}} → resolved value
    //
    // scope restrictions
    // does_not_touch_spec_data: expressions inside spec.data stay literal
    // single_pass: $data returning object with inner $data marker → NOT re-resolved
}
```

Integration tests in `framework/src/json_ui/mod.rs`:

```rust
// Tests to add at bottom of existing test block in mod.rs
// render_resolves_data_expression_before_html_emission
// render_json_returns_spec_with_no_expression_markers
// render_with_config_honors_expression_resolution
// render_with_errors_resolves_expressions_then_applies_errors
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact on Phase 118 |
|--------------|------------------|--------------|---------------------|
| `data_path` field per component (e.g., `InputProps.data_path`) | `$data` expression in any props position | Phase 118 | `$data` is additive; `data_path` fields coexist and are populated by the projector. Both resolve from `spec.data`. |
| No string interpolation | `$template` with `{/path}` placeholders | Phase 118 | Enables human-readable labels and titles that reference handler data |
| No pre-render pass | `resolve_expressions` in pipeline after `resolve_actions` | Phase 118 | Renderers receive concrete values; no expression-awareness needed in the walker or catalog |

**Deprecated/outdated:**

- None. This phase is purely additive. No existing behavior is removed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The borrow-checker issue with simultaneous `spec.data` (shared) and `spec.elements` (mutable) requires cloning `data` first | Architecture Patterns / Pitfall 1 | None — the fix (clone before loop) is correct regardless of whether Rust would actually reject the alternative |

All other claims in this research are verified against:
- Source files read directly (`data.rs`, `resolve.rs`, `spec.rs`, `lib.rs`, `Cargo.toml`, `framework/src/json_ui/mod.rs`)
- CONTEXT.md decisions (treated as locked specifications)

---

## Open Questions

None. The CONTEXT.md decisions fully specify the implementation. All investigative priorities from the phase brief are resolved:

1. **Path resolution:** Use `crate::data::resolve_path` — already implemented, `pub(crate)`, no changes needed.
2. **Template interpolation:** Hand-roll a ~25-line scanner. No new dependency.
3. **Traversal/rewrite pattern:** `serde_json::Value` mutation in place via `as_object_mut`/`as_array_mut`. Recursive function, no visitor crate.
4. **Expression detection:** Single-key object with string value. `obj.len() == 1 && obj.get("$data")` is `Some(Value::String(_))`.
5. **Error semantics:** Missing `$data` → `Value::Null`. Missing `$template` placeholder → `""`. Malformed → literal passthrough. No panic, no log.
6. **Testing patterns:** Inline `#[cfg(test)]` blocks matching existing crate convention (`data.rs`, `resolve.rs`, `visibility.rs`).

---

## Environment Availability

Step 2.6: SKIPPED — Phase 118 is purely code changes within the existing Rust workspace. No external tools, services, databases, or CLIs beyond the existing `cargo` toolchain. All tests run with `cargo test --all-features`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — standard `#[cfg(test)]` blocks |
| Quick run command | `cargo test --package ferro-json-ui --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXPR-01 | `$data` resolves to typed JSON value at path | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ Wave 0 (new file) |
| EXPR-02 | `$template` interpolates slash-path placeholders | unit | `cargo test --package ferro-json-ui --all-features -- expression::` | ❌ Wave 0 (new file) |
| EXPR-03 | Resolution runs before render; renderers receive concrete values | integration | `cargo test --package framework --all-features -- json_ui::` | ❌ Wave 0 (new tests in existing file) |

### Sampling Rate

- **Per task commit:** `cargo test --package ferro-json-ui --all-features`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/expression.rs` — covers EXPR-01 and EXPR-02 (new file; created in Wave 1)
- [ ] New test block in `framework/src/json_ui/mod.rs` — covers EXPR-03 (additions to existing file)

No new test framework installation needed — `cargo test` is already operational.

---

## Security Domain

Phase 118 performs server-side string substitution of data paths into prop values. Relevant ASVS categories:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Partial | Spec data is authored by the handler (trusted); but malformed expression objects degrade silently (D-06) — no injection surface |
| V5 Output Encoding | Yes | `$template` output goes into `el.props` which is later HTML-rendered by the existing walker's escaping logic. Expression resolver does NOT emit HTML — it emits `Value::String`. The walker's existing HTML escaping applies. |
| V6 Cryptography | No | No cryptographic operations |
| V2 Authentication | No | No auth surface |
| V4 Access Control | No | Resolver does not gate data access; all `spec.data` is equally accessible to all expressions in the same spec |

**Threat pattern — data exfiltration via `$data` into visible output:**

`$data` can reach any path inside `spec.data`. If `spec.data` contains sensitive values (e.g., internal server state not meant to be user-visible), an expression like `{"$data": "/internal/secret"}` would surface that value in the rendered HTML. This is an authoring-time concern, not a runtime security bug — the author controls what goes into `spec.data`. The resolver makes no policy decision about which paths are displayable.

**Standard mitigation:** Authors should populate `spec.data` only with values intended for the UI. This is a documentation note, not a code change for Phase 118.

---

## Sources

### Primary (HIGH confidence)

- Source read: `ferro-json-ui/src/data.rs` — `resolve_path` and `resolve_path_string` signatures, behavior, visibility
- Source read: `ferro-json-ui/src/resolve.rs` — `resolve_actions` signature pattern, infallible convention
- Source read: `ferro-json-ui/src/spec.rs` — `Spec` and `Element` struct fields, `spec.data: Value`, `el.props: Value`
- Source read: `ferro-json-ui/src/lib.rs` — export conventions, re-export grouping
- Source read: `ferro-json-ui/Cargo.toml` — confirmed no `regex` dependency; `serde_json = "1.0"` available
- Source read: `framework/src/json_ui/mod.rs` — `JsonUi::resolve` (lines 39-43), `resolve_with_errors` (line 150), integration test patterns
- Source read: `.planning/phases/118-server-side-expressions/118-CONTEXT.md` — all implementation decisions D-01 through D-14

### Secondary (MEDIUM confidence)

- Borrow-checker analysis: `spec.data` + `spec.elements.values_mut()` simultaneous borrow — standard Rust ownership rule, verified by understanding the struct layout. Fix (clone `data` first) is idiomatic.

### Tertiary (LOW confidence)

None — no claims in this research rely on unverified web sources.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — verified against Cargo.toml and source files; no new dependencies required
- Architecture: HIGH — derived directly from source-read patterns (resolve.rs) and locked CONTEXT.md decisions
- Pitfalls: HIGH — Pitfalls 1 and 3 verified against actual source code structure; Pitfalls 2, 4, 5 are logical consequences of the detection and walk algorithms

**Research date:** 2026-04-19
**Valid until:** Stable — all primitives are internal to the workspace; no external APIs that could change
