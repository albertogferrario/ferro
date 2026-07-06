---
phase: 120-cli-and-mcp-updates
reviewed: 2026-04-21T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - ferro-cli/src/ai.rs
  - ferro-cli/src/commands/make_json_view.rs
  - ferro-cli/src/templates/make.rs
  - ferro-mcp/src/tools/code_templates.rs
  - ferro-mcp/src/tools/generation_context.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - ferro-mcp/src/tools/json_ui_generate.rs
  - ferro-mcp/src/tools/json_ui_inspect.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 120: Code Review Report

**Reviewed:** 2026-04-21
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Eight files reviewed covering the CLI `make:json-view` command and its AI generation layer (`ai.rs`), the static template library (`make.rs`), and five MCP tools (`code_templates`, `generation_context`, `json_ui_catalog`, `json_ui_generate`, `json_ui_inspect`).

The code is generally clean and well-structured. Three issues worth fixing were found: a latent panic on non-ASCII source files (duplicated in two scan-models functions), a dead parameter that silently discards the caller's work, and a semantic inconsistency in `inspect_component` for unknown component names. Three lower-priority items round out the review.

---

## Warnings

### WR-01: Char index used as byte index in struct-body slicing — latent panic on non-ASCII source

**Files:**
- `ferro-cli/src/ai.rs:338-353`
- `ferro-mcp/src/tools/json_ui_generate.rs:173-188`

**Issue:** Both `scan_models` implementations use `rest.chars().enumerate()` where `i` is a *character* index (number of Unicode scalar values consumed), then slice `&rest[..struct_end]` using that value as a *byte* index. Rust `str` slicing panics with `byte index N is not a char boundary` whenever the byte index does not land on a UTF-8 character boundary. For source files that contain only ASCII this is safe (each char is one byte), but any `.rs` file with a multi-byte character before or inside the struct (e.g. a Unicode identifier, an accented letter in a doc-comment or string literal) will cause a panic at runtime.

**Fix:** Use `char_indices()` instead of `chars().enumerate()` so the index is always a byte offset:

```rust
// Before
for (i, ch) in rest.chars().enumerate() {
    match ch {
        '{' => depth += 1,
        '}' => {
            depth -= 1;
            if depth == 0 {
                struct_end = i;
                break;
            }
        }
        _ => {}
    }
}

// After
for (byte_idx, ch) in rest.char_indices() {
    match ch {
        '{' => depth += 1,
        '}' => {
            depth -= 1;
            if depth == 0 {
                struct_end = byte_idx;
                break;
            }
        }
        _ => {}
    }
}
```

The fix needs to be applied identically in both files since the logic is duplicated.

---

### WR-02: Dead parameter `_schema` in `build_json_view_pass2` silently discards caller work

**File:** `ferro-cli/src/ai.rs:199`

**Issue:** `build_json_view_pass2(pass1_result: &str, _schema: &serde_json::Value)` accepts a schema argument that is never used (the leading `_` suppresses the compiler warning). The caller in `make_json_view.rs` line 132-133 clones `global_catalog().json_schema()` to produce this value, then passes it, and the function ignores it. The actual schema used by Pass 2 comes from the `call_anthropic_structured` call at line 134, where the caller passes a *second* clone of the same value. The dead parameter creates a misleading API: callers believe the schema is wired through `build_json_view_pass2` when it is not, and the intermediate clone is pure waste.

**Fix:** Remove the `_schema` parameter from `build_json_view_pass2` and update the single call site:

```rust
// ai.rs — remove second parameter
pub fn build_json_view_pass2(pass1_result: &str) -> (String, String) { … }

// make_json_view.rs — remove the schema argument
let (sys2, usr2) = ai::build_json_view_pass2(&pass1_result);
```

---

### WR-03: `inspect_component` classifies unknown components as `is_plugin: true`

**File:** `ferro-mcp/src/tools/json_ui_inspect.rs:76-85`

**Issue:** When `component_type` is neither a built-in nor a registered plugin, `inspect_component` returns `ComponentSchemaInfo { is_plugin: true, props_schema: None, catalog_entry: None }`. The `is_plugin: true` flag is factually wrong for a component that does not exist. MCP consumers reading this response may interpret the result as "this is a plugin we just don't have the schema for" rather than "this component does not exist at all." The test at line 354 asserts this current (incorrect) behavior rather than the correct one.

**Fix:** Distinguish the three cases explicitly:

```rust
pub fn inspect_component(component_type: &str) -> ComponentSchemaInfo {
    use ferro_json_ui::global_catalog;
    let cat = global_catalog();

    // Built-in
    if cat.components_sorted().any(|s| s.name.eq_ignore_ascii_case(component_type)) {
        let catalog = super::json_ui_catalog::execute(Some(component_type));
        return ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: false,
            props_schema: None,
            catalog_entry: catalog.components.into_iter().next(),
        };
    }

    // Plugin
    let catalog = super::json_ui_catalog::execute(Some(component_type));
    let plugin_entry = catalog.plugin_components.into_iter().next();
    if plugin_entry.is_some() {
        let schema = ferro_json_ui::with_plugin(component_type, |p| p.props_schema());
        return ComponentSchemaInfo {
            name: component_type.to_string(),
            is_plugin: true,
            props_schema: schema,
            catalog_entry: plugin_entry,
        };
    }

    // Unknown — no matching built-in or plugin
    ComponentSchemaInfo {
        name: component_type.to_string(),
        is_plugin: false,   // not a plugin — simply unknown
        props_schema: None,
        catalog_entry: None,
    }
}
```

Update the `test_inspect_unknown_component` assertion to expect `is_plugin: false`.

---

## Info

### IN-01: `generate_json_view` in `ai.rs` is dead code — its logic is duplicated in `make_json_view.rs`

**File:** `ferro-cli/src/ai.rs:222-299`

**Issue:** `generate_json_view` implements the full two-pass AI flow, but `make_json_view.rs::generate_with_ai` (line 115) re-implements the same flow inline and is the code actually called. `generate_json_view` is never invoked. Both functions embed Pass 1 system prompts independently and call `call_anthropic_plain` / `call_anthropic_structured` directly. If the module docstring's `generate_json_view` entry in the `Provides:` list is meant to be authoritative, the dead function should be removed and `generate_with_ai` should be promoted to `pub` and renamed, or vice versa.

**Fix:** Delete `generate_json_view` and `scan_models` / `scan_routes` from `ai.rs` (they are already in `json_ui_generate.rs`), or consolidate by making `generate_with_ai` call `generate_json_view`.

---

### IN-02: Route scanning is implemented twice with different strategies

**Files:**
- `ferro-cli/src/ai.rs:375-411` — uses `generate_routes::parse_routes_file`
- `ferro-mcp/src/tools/json_ui_generate.rs:212-237` — uses its own regex

**Issue:** Two different implementations of route scanning exist in the codebase. The CLI version delegates to the authoritative `parse_routes_file` parser; the MCP version has a hand-rolled regex that will miss route definitions that use macros or multiline syntax. The MCP tool will silently return fewer routes than the CLI tool for the same project.

**Fix:** Extract route scanning into a shared utility (in `ferro-cli` or a common crate) and have both callers use it, or have `json_ui_generate.rs` import and call `generate_routes::parse_routes_file` directly if the crate dependency is acceptable.

---

### IN-03: `to_snake_case` in `make_json_view.rs` does not handle consecutive uppercase letters

**File:** `ferro-cli/src/commands/make_json_view.rs:192-204`

**Issue:** The conversion inserts a separator before every uppercase letter. `"JSONView"` becomes `"j_s_o_n_view"` instead of `"json_view"`. This is a common edge case with simple uppercase-per-character scanners and affects any view name using an acronym. The existing tests only cover `"UserList"` → `"user_list"` and `"dashboard"` → `"dashboard"`.

**Fix:** Treat a run of uppercase letters as a single word boundary:

```rust
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = chars.get(i + 1).map_or(false, |n| n.is_lowercase());
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_uppercase())) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
```

---

_Reviewed: 2026-04-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
