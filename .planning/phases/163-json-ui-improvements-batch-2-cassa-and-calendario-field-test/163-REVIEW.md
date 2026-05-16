---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
reviewed: 2026-05-16T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - ferro-json-ui/src/spec.rs
  - ferro-json-ui/src/resolve.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/tests/directives_e2e.rs
  - framework/src/json_ui/mod.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - ferro-cli/src/commands/json_ui_migrate_v1.rs
  - ferro-cli/src/commands/mod.rs
  - ferro-cli/src/main.rs
  - ferro-cli/tests/json_ui_migrate_v1.rs
  - ferro-cli/tests/fixtures/migrate_v1/in_auth.rs
  - ferro-cli/tests/fixtures/migrate_v1/in_with_runtime_branch.rs
  - ferro-cli/tests/fixtures/migrate_v1/out_auth.rs
  - ferro-cli/tests/fixtures/migrate_v1/out_auth_login_form.json
  - docs/src/json-ui/spec-construction.md
  - docs/src/json-ui/expressions.md
  - docs/src/SUMMARY.md
  - CHANGELOG.md
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 163: Code Review Report

**Reviewed:** 2026-05-16T00:00:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Phase 163 ships `$each` / `$if` directives, `expand_directives`, five new `SpecError` validator gates, the `NestedElement` DSL, the `ferro json-ui:migrate-v1` AST codemod, MCP catalog updates, and documentation. The core directive pipeline is well-structured and the validator gates are thorough. Test coverage for the new paths is comprehensive.

Three warnings are raised:

1. The codemod's output fixture for `in_auth.rs` — and therefore the production codemod logic — produces a spec with orphaned/unreachable elements when a v1 handler has multiple top-level nodes. The form, its inputs, and the submit button are all in `elements` but unreachable from `root`.
2. The `rewrite_handler_body` function matches `fn {handler_name}(` as a plain substring, which will match the wrong function if two handlers share a common name prefix (e.g., `index` matches before `index_admin`).
3. The MCP `BUILDER_API` constant does not document `SpecBuilder::element_nested` or `NestedElement`, even though this is a new public API surface added in Phase 163 for AI-assisted authoring.

Four info items cover: a "cassa" friction-site reference in a test comment, the `gestiscilo` app-identity leak in CHANGELOG, a missing `$each` / `$if` field documentation in the `BUILDER_API` constant's `Element` shape description, and the fact that `IfPathMissing` fires only when `spec.data` is non-null but `$if` also runs at resolve time against per-request data injected via `merge_data` (the validator's best-effort gap is documented but not surfaced anywhere user-facing).

---

## Warnings

### WR-01: Codemod emits orphaned (unreachable-from-root) elements for multi-root v1 views

**File:** `ferro-cli/src/commands/json_ui_migrate_v1.rs:253-286`
**Also:** `ferro-cli/tests/fixtures/migrate_v1/out_auth_login_form.json:5-29`

**Issue:** `try_migrate_handler` sets `root` to `top_ids.first()` (the first element in the top-level `vec![...]` call). When a v1 handler has more than one top-level node — as `login_form` does (`page-title` and `login-form`) — only the first becomes root. The remaining top-level elements and their entire subtrees are inserted into `elements` but are never reachable from `root`. They silently appear in the JSON file, occupy bytes, and are invisible at render time.

The `out_auth_login_form.json` fixture confirms this: `root` is `"page-title"` (a `PageHeader` with no children), while `login-form`, `email`, `password`, and `submit` exist in the map but are unreachable. A user who runs the codemod on this fixture will see a page with only a header and no form.

The spec validator (`validate_structure`) has no unreachable-element check, so the file passes validation cleanly.

**Fix:** Two options, with different tradeoffs:

Option A — wrap multiple top-level nodes in a synthetic `Group` / `Fragment` parent element when more than one top-level node exists:
```rust
// in try_migrate_handler, after flatten_nodes:
if top_ids.len() > 1 {
    // Insert a synthetic wrapper at the root.
    let wrapper_id = format!("{handler_name}-root");
    let mut wrapper = Map::new();
    wrapper.insert("type".to_string(), Value::String("Group".to_string()));
    wrapper.insert(
        "children".to_string(),
        Value::Array(top_ids.iter().cloned().map(Value::String).collect()),
    );
    elements.insert(wrapper_id.clone(), Value::Object(wrapper));
    root = wrapper_id;
} else {
    root = top_ids.into_iter().next().ok_or_else(|| ...)?;
}
```

Option B (simpler, less surprising) — reject multi-root handlers as `HandlerResult::Unsupported` and emit a TODO marker, since v2 has no native multi-root concept without a container:
```rust
if top_ids.len() != 1 {
    return HandlerResult::Unsupported;
}
```

Option B is safer for correctness (no silent data loss); Option A produces a runnable spec but requires a `Group` component to exist in the catalog. Given Phase 163 D-06 notes that `Fragment`/`Group` was explicitly NOT added, Option B is the correct choice for this phase.

The fixture `out_auth_login_form.json` and the `codemod_one_handler_emits_spec_and_rewrites_controller` test must be updated to match whichever path is chosen.

---

### WR-02: `rewrite_handler_body` substring match is ambiguous when handler names share a prefix

**File:** `ferro-cli/src/commands/json_ui_migrate_v1.rs:682-683`

**Issue:** Both `inject_todo_above_handler` and `rewrite_handler_body` locate handler positions with `src.find(format!("fn {handler_name}("))`. If a controller contains `fn index(` and `fn index_admin(`, and `index` is processed first, `src.find("fn index(")` will match the first occurrence correctly. But if the first handler is `index_admin` and the second is `index`, `src.find("fn index(")` will match `fn index_admin(` — inserting the rewrite at the wrong site. The ambiguity depends on document order, which is non-deterministic across `HashMap` iteration.

The visitor collects handlers in `HashMap` iteration order (via `spec.elements.iter()`, then `visitor.specs`), which is arbitrary.

**Fix:** Anchor the match to the line boundary or use a word-boundary suffix to rule out prefix collisions:
```rust
// Require that the character immediately after handler_name(
// is either '(' to prevent "index" matching "index_admin".
// A tighter approach: require `fn {name}` at the start of a word.
let needle = format!("fn {handler_name}(");
// After find(needle), verify the character before `(` is exactly `)` from the name,
// not a letter — i.e. check src[pos + needle.len() - 1] == '('.
// Simpler: scan for `\bfn {handler_name}(` — or use a regex on small source files.
```

Alternatively, use the `syn` span offset directly instead of a text search — the visitor has the `ItemFn` from the AST and could use `item.sig.ident.span()` with `proc_macro2::Span::start()` to get the byte offset:
```rust
// Preferred — avoids the substring problem entirely:
// Store byte offset from ItemFn.sig.ident.span().start() in MigrationVisitor,
// then use it in finalize() to replace text at the correct position.
```

---

### WR-03: MCP `BUILDER_API` constant omits `element_nested` / `NestedElement` — new public API invisible to agents

**File:** `ferro-mcp/src/tools/json_ui_catalog.rs:236-259`

**Issue:** `BUILDER_API` is the agent-facing documentation for spec construction. Phase 163 adds `SpecBuilder::element_nested` and `NestedElement` as a new first-class construction path — explicitly described in `docs/src/json-ui/spec-construction.md` as the fourth quadrant of the decision rubric. Neither appears in `BUILDER_API`. An agent reading the MCP catalog has no way to discover this API.

The Element shape description also omits the new `$each` and `$if` fields from the inline type-comment, even though those are now first-class `Element` fields with wire-format names.

**Fix:**
```rust
const BUILDER_API: &str = "\
Spec::builder() -> SpecBuilder
  .title(impl Into<String>) -> Self
  .layout(impl Into<String>) -> Self
  .data(serde_json::Value) -> Self
  .element(id, Element) -> Self          // flat construction; explicit child IDs
  .element_nested(id, NestedElement) -> Self  // tree construction; auto-IDs {id}-0, {id}-1, ...
  .build() -> Result<Spec, SpecError>

NestedElement::new(type_name: impl Into<String>) -> NestedElement
  .prop(key, value) -> Self
  .child(NestedElement) -> Self          // children auto-assigned {parent}-{idx} IDs
  .action(Action) -> Self
  .visible(Visibility) -> Self

Element::new(type_name: impl Into<String>) -> ElementBuilder
  .prop(key, value) -> Self
  .child(id: impl Into<String>) -> Self
  .action(Action) -> Self
  .visible(Visibility) -> Self

Spec { $schema, root, elements: HashMap<String, Element>, title?, layout?, data? }
Element { type, props, children: Vec<String>, action?, visible?, $each?, $if? }
  - $each: EachDirective { path, as } — expand N clones at resolve time
  - $if: Visibility — remove element when predicate is false";
```

---

## Info

### IN-01: Test comment references friction-site application name "cassa"

**File:** `ferro-json-ui/tests/directives_e2e.rs:23`

**Issue:** The doc comment for `e2e_orders_kanban_each_produces_n_cards` reads "Mirrors the cassa orders-kanban friction site." `cassa` is a consumer application name. Per project conventions, `ferro-*` crates must be project-agnostic; test commentary in a library crate should not reference specific application identities.

**Fix:** Replace with a neutral description:
```rust
/// Test 1: full kanban fixture with $each over /orders produces one rendered
/// output per row. Exercises the canonical orders-kanban use case.
```

---

### IN-02: CHANGELOG references "gestiscilo" application name (two entries)

**File:** `CHANGELOG.md:15,32`

**Issue:** Two CHANGELOG entries name `gestiscilo` as a concrete application:
- Line 15: "Two consumer sites in gestiscilo documenti templates unblocked"
- Line 32: "gestiscilo audit confirmed all auth specs already declare Card roots"

CHANGELOG is a committed repository artifact. Per project conventions, application-specific identities belong in local memory files. The context these entries carry (which consumer sites were unblocked, which app was audited) is internal rationale, not public-facing changelog content.

**Fix:** Neutralize both entries:
- Line 15: "Two consumer sites unblocked (162-04, D-18)."
- Line 32: "Breaking for any spec that relied on the implicit wrapper; all surveyed auth specs already declare Card roots."

---

### IN-03: `$if` best-effort validation gap is undocumented at the API level

**File:** `ferro-json-ui/src/spec.rs:826-831`

**Issue:** `validate_directives` checks `$if` paths only when `spec.data` is non-null. However, `merge_data` can inject per-request data after `from_json` runs, so a spec that validates cleanly with null data can still have a `$if` path that references a missing key at expand time. `expand_directives` / `remove_if_falsy` calls `predicate.evaluate(data)` — if the path is absent, the `Visibility` evaluator returns false (the element is silently removed). This is not a crash, but a spec author who omitted a required data key gets silent element removal rather than an error.

The gap is correct behavior by design (best-effort), but it is documented only in the inline comment and not surfaced in `SpecError::IfPathMissing`'s doc comment or the `expressions.md` validation section.

**Fix:** Add a sentence to `SpecError::IfPathMissing`'s doc comment making the best-effort scope explicit:
```rust
#[error("element '{element_id}' has `$if.path = \"{path}\"` referencing a key absent from spec.data")]
/// Note: this check fires only when `spec.data` is non-null at parse time.
/// Per-request data injected via `merge_data` is not validated; missing paths
/// silently evaluate to false (element removed) at expand time.
IfPathMissing { element_id: String, path: String },
```

---

### IN-04: `out_auth_login_form.json` passes spec validation despite containing unreachable elements (no validator coverage)

**File:** `ferro-json-ui/src/spec.rs:647-658`

**Issue:** The spec validator (`validate_structure`) has no reachability check. Elements present in `elements` but not reachable from `root` via the children graph are silently accepted. This means a malformed spec (including the one produced by the codemod's multi-root bug in WR-01) passes `Spec::from_json` cleanly. Users and tooling have no indication that part of the element tree is dead.

This is a separate, pre-existing gap from WR-01 (the codemod bug) — adding a reachability check would catch codemod output errors at load time rather than requiring the user to notice missing rendered content.

**Fix (optional, scope may exceed Phase 163):** Add a `validate_reachable` pass after `check_depth`:
```rust
fn validate_reachable(elements: &HashMap<String, Element>, root: &str) {
    // DFS from root; collect visited set; compare to elements.keys().
    // For any id in elements.keys() - visited: emit eprintln! warning.
    // (Non-fatal per precedent; hard error would require a new SpecError variant.)
}
```

This is marked Info because it is a pre-existing condition not introduced by Phase 163, and making it an error would be a breaking change for any spec that deliberately carries unused elements (e.g., template libraries).

---

_Reviewed: 2026-05-16T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
