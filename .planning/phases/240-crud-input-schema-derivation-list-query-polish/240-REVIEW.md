---
phase: 240-crud-input-schema-derivation-list-query-polish
reviewed: 2026-06-23T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-projections/src/service.rs
  - ferro-mcp-server/src/schema.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-mcp-server/src/dispatch.rs
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 240: Code Review Report

**Reviewed:** 2026-06-23
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 240 adds: (a) `is_write_excluded_field` and `is_server_injected_field` predicates on `ServiceDef`; (b) `is_range_filter_field` predicate and extended `build_input_schema` (range/ne/in/sort params) in `schema.rs`; (c) `build_create/update/delete_input_schema` builders; (d) CRUD tool emission in `renderer.rs`; (e) an NTI envelope in `write_dispatch.rs` for not-yet-executable CRUD verb calls; (f) `__op`/`sort` read-dispatch extension in `dispatch.rs`.

The security-critical properties — parameterized values, allowlisted keys, tenant predicate always applied, soft-delete always applied — are correctly implemented. No injection vector was found. The `__in` placeholder index advancement is correct. The NTI framing correctly intercepts CRUD verb calls before `find_action`, preventing -32601 fallthrough.

Four warnings and three info items follow. None block correctness on the current read-path or schema-emission path; most affect future write-path robustness.

---

## Warnings

### WR-01: `sort` allowlist uses `is_filter_field` — range-only fields (Money, Quantity) cannot be sorted

**File:** `ferro-mcp-server/src/dispatch.rs:151`
**Issue:** Sort column validation calls `is_filter_field(f)`, which uses the meaning-based allowlist (Identifier, ForeignKey, Status, Category, Boolean, Custom). Fields whose meaning is Money, Quantity, or Percentage are explicitly excluded from `is_filter_field` by design (they are range-only via `is_range_filter_field`). The result is that `total__gt` / `total__lte` work as range filters, but `sort=total` returns `InvalidFilter("unknown or non-sortable field: total")`. Meanwhile, the input schema advertises `sort` as accepting any field string (type: string, no enum). This is an under-documented restriction that will surface as a confusing agent error: "I can range-filter by `total` but I cannot sort by it."

The security boundary is fine — both allowlists prevent injection. The issue is functional completeness: range-sortable fields (ordered numeric/datetime types) should also be sortable.

**Fix:** In `dispatch.rs` around line 151, replace the sort validation branch with a union check:

```rust
Some(f) if is_filter_field(f) || is_range_filter_field(f) => Some((col.to_string(), dir)),
```

Also update the `sort` property description in `build_input_schema` (schema.rs:174) to mention that any filterable or range-filterable field is accepted, so the schema stays in sync with execution.

---

### WR-02: `build_create_input_schema` description is the raw field name, not a human label

**File:** `ferro-mcp-server/src/schema.rs:263`
**Issue:** The `create` and `update` schema builders set the property description to `field.name.clone()` (the snake_case column name), while `build_action_input_schema` uses either `input.description` or falls back to the field name, and `build_input_schema` uses `"Filter by {field.name}"`. Agents use the description to understand what to supply. Receiving `"notes"` as the description of the `notes` property is a no-op: the property name already conveys that. By contrast, the delete schema emits no description at all for the identifier field (line 347 calls `data_type_to_json_schema` without attaching a description).

This is a quality issue in the agent-experience surface; the schema compiles and tests pass because no test asserts on description content.

**Fix:** In `build_create_input_schema` and `build_update_input_schema`, produce a more informative description. Minimal improvement:

```rust
// build_create_input_schema, line ~263
m.insert(
    "description".into(),
    serde_json::Value::String(format!(
        "{} value for the new record",
        field.name
    )),
);

// build_delete_input_schema, line ~347 — also add a description for the id prop
m.insert(
    "description".into(),
    serde_json::Value::String(format!(
        "ID of the {} record to delete",
        service.display_name.as_deref().unwrap_or(&service.name)
    )),
);
```

---

### WR-03: `build_update_input_schema` re-runs `is_write_excluded_field` on the Identifier field, silently double-filtering it

**File:** `ferro-mcp-server/src/schema.rs:310-323`
**Issue:** `build_update_input_schema` first injects the Identifier field explicitly (lines 289-305), then iterates `service.fields` again with `is_write_excluded_field` as the exclusion predicate (line 311). `is_write_excluded_field` returns `true` for Identifier fields (Gate A), so the identifier is correctly excluded from the second loop. The output is correct, but the structure is subtle: a reader must trace through two loops to verify the identifier appears exactly once in `properties`. If Gate A were ever relaxed or the loop order changed, the identifier could be duplicated or omitted silently.

**Fix:** Add an explicit guard comment at the top of the second loop, or skip the identifier explicitly by name. A minimal, clear form:

```rust
for field in service.fields.iter().filter(|f| {
    // Skip the identifier — already injected above as the required param.
    !matches!(f.meaning, FieldMeaning::Identifier)
        && !service.is_write_excluded_field(f, exclude_sm_status)
}) {
```

This makes the invariant explicit and removes reliance on Gate A coincidentally covering the already-injected identifier.

---

### WR-04: NTI detection in `write_dispatch.rs` matches on service name equality, not the CRUD flag

**File:** `ferro-mcp-server/src/write_dispatch.rs:158-168`
**Issue:** The NTI guard block checks:
```rust
if services.iter().any(|s| s.mcp_exposed && s.name == svc_name) {
```
This matches any mcp-exposed service with the matching name, regardless of whether `creatable`/`updatable`/`deletable` is actually set. Consider a service `order` that is `mcp_exposed(true)` but has `creatable(false)` (the default). An agent that somehow discovers and calls `create_order` (perhaps from a stale tool cache) would receive the NTI envelope rather than a -32601 "Method not found" error.

The NTI path is intentional for Phase 240 — the argument is that `create_order` is only *listed* when `creatable=true`, so a well-behaved agent would not call it when it was never listed. But the defense-in-depth argument for checking the flag here is stronger: the NTI envelope should only be emitted for tools that were actually emitted by `render_exposed_tools`.

**Fix:** Check the relevant flag for the matching prefix:

```rust
for (prefix, flag_check) in &[
    ("create_", Box::new(|s: &ServiceDef| s.creatable) as Box<dyn Fn(&ServiceDef) -> bool>),
    ("update_", Box::new(|s: &ServiceDef| s.updatable)),
    ("delete_", Box::new(|s: &ServiceDef| s.deletable)),
] {
    if let Some(svc_name) = tool_name.strip_prefix(prefix) {
        if services.iter().any(|s| s.mcp_exposed && s.name == svc_name && flag_check(s)) {
            // ... NTI envelope
        }
    }
}
```

Or, simpler without closures, by using three separate if-let blocks that check the respective flag.

---

## Info

### IN-01: `generate_confirmation_token` uses `rand::thread_rng` — will break when `rand` 0.9 stabilises

**File:** `ferro-mcp-server/src/write_dispatch.rs:87`
**Issue:** `rand::thread_rng()` is the `rand 0.8` API. `rand 0.9` replaced `thread_rng()` with `rng()`. This is not a current bug but will produce a compile error on the next `rand` major bump. The function is `#[cfg(feature = "confirmation")]` so it is not exercised in the default feature set, making the breakage easy to miss.

**Fix:** No change required now, but when upgrading `rand` to 0.9, replace `rand::thread_rng()` with `rand::rng()` and `rng.gen_range(...)` with `rng.random_range(...)`.

---

### IN-02: `build_input_schema` `sort` property has no `enum` constraint — schema does not self-document accepted values

**File:** `ferro-mcp-server/src/schema.rs:171-178`
**Issue:** The `sort` property is advertised as `type: string` with a description mentioning the `-` prefix convention. The set of valid sort columns is derivable from `service.fields` — specifically the union of `is_filter_field` and `is_range_filter_field` fields. Not advertising an `enum` means an agent cannot know valid values without inference; it also means the schema and execution allowlist can drift (WR-01 above is an instance of that drift).

**Fix (optional, Phase 241 or later):** Populate an `enum` array in the sort property. This is an enhancement, not a blocking issue.

```rust
let sort_values: Vec<serde_json::Value> = service
    .fields
    .iter()
    .filter(|f| is_filter_field(f) || is_range_filter_field(f))
    .flat_map(|f| {
        vec![
            serde_json::Value::String(f.name.clone()),
            serde_json::Value::String(format!("-{}", f.name)),
        ]
    })
    .collect();
// Insert sort_values into the sort property as "enum": sort_values
```

---

### IN-03: `write_tool_error_result` extracts `message` from the payload but stores the full payload in `structuredContent` — message key must always be present

**File:** `ferro-mcp-server/src/write_dispatch.rs:59-70`
**Issue:** `write_tool_error_result` falls back to `"error"` if the `message` key is absent. All current call sites supply a `message` key in the payload JSON, so the fallback never fires. However, the function's contract is implicit: callers must include `"message"` in `payload` or the content block will say `"error"` with no detail. There are no tests exercising the fallback path.

**Fix:** Document the contract explicitly in the function's doc comment, or assert `payload["message"].is_string()` in debug builds. Not a correctness issue in current call sites.

---

_Reviewed: 2026-06-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
